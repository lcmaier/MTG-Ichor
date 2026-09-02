use std::collections::HashMap;

use crate::engine::actions::{ActionContext, ZoneChangeCause};
use crate::engine::costs::assemble_total_cost;
use crate::events::event::GameEvent;
use crate::objects::card_data::AbilityType;
use crate::types::costs::Cost;
use crate::objects::object::GameObject;
use crate::state::game_state::{GameState, PhaseType, StackEntry};
use crate::types::card_types::CardType;
use crate::types::effects::EffectRecipient;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;
use crate::types::zones::Zone;
use crate::oracle::legality::enumerate_legal_selections;
use crate::oracle::mana_helpers::{
    enumerate_activatable_mana_abilities, remaining_cost_after_pool,
};
use crate::ui::ask::{
    ask_activate_mana_ability,
    ask_choose_alternative_cost, ask_choose_additional_costs,
    ask_choose_x_value, ask_select_recipients, ask_choose_generic_mana_allocation,
};
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// Cast a spell from hand onto the stack (rule 601.2).
    ///
    /// Steps follow CR 601.2a–i:
    /// 1. Pre-proposal legality check (rule 601.3)
    /// 2. Move to stack (601.2a) — silently; the move is announced at 601.2i
    /// 3. Choose alternative cost, additional costs, X value (601.2b)
    /// 4. Choose targets (601.2c)
    /// 5. Distribution placeholder (601.2d — T18c)
    /// 6. Post-proposal legality check with rollback (601.2e)
    /// 7. Assemble total cost (601.2f)
    /// 8. Mana ability window placeholder (601.2g)
    /// 9. Pay total cost (601.2h)
    /// 10. Announce the 601.2a move and emit SpellCast (601.2i)
    pub fn cast_spell(
        &mut self,
        player_id: PlayerId,
        card_id: ObjectId,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // --- Pre-proposal legality check (rule 601.3) ---
        self.check_cast_legality(player_id, card_id)?;

        // Casting is not itself a resolution (CR 601), so no resolution stamp.
        let actx = ActionContext::new(decisions);

        // Snapshot data we need before moving the card
        let card_data = self.get_object(card_id)?.card_data.clone();

        // PRE-LAYER ZONE: reads printed abilities and types on purpose. The card
        // is still in hand here, so it is not a permanent and the layer system has
        // nothing to contribute -- see "Before Layers" in plans/codebase-state.md.
        //
        // Find the spell ability on the card.
        // Permanent spells (creatures, enchantments, artifacts, planeswalkers)
        // may not have a spell ability — they resolve by entering the
        // battlefield. Use an empty Sequence as a no-op effect.
        let (effect, recipient) = if let Some(spell_ability) = card_data.abilities.iter()
            .find(|a| a.ability_type == AbilityType::Spell)
        {
            let effect = spell_ability.effect.clone();
            let recipient = match &effect {
                crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
                crate::types::effects::Effect::Sequence(effects) => {
                    // For sequence effects, use the target spec from the first atom
                    effects.iter().find_map(|e| {
                        if let crate::types::effects::Effect::Atom(_, ts) = e {
                            Some(ts.clone())
                        } else {
                            None
                        }
                    }).unwrap_or(EffectRecipient::Implicit)
                }
                _ => EffectRecipient::Implicit,
            };
            (effect, recipient)
        } else if card_data.types.iter().any(|t| t.is_permanent()) {
            // Permanent spell with no spell ability — resolves by ETB alone
            (crate::types::effects::Effect::Sequence(Vec::new()), EffectRecipient::Implicit)
        } else {
            return Err(format!("Card '{}' has no spell ability", card_data.name));
        };

        // --- 601.2a: Move to stack ---
        // Capture the origin first: once the card is on the stack its `zone`
        // field says Stack, and CR 903.8 / "cast from exile" both need to know
        // where it came from. See `StackEntry::cast_from`.
        let cast_from = self.get_object(card_id)?.zone;
        // CAST-ROLLBACK: silent in both directions. Nothing in the CR replaces
        // a card being put onto the stack — a "can't cast" is CR 601.3's
        // question, asked of the player ahead of this step — so the move is
        // not a proposal; and the spell is not cast until 601.2i, so a cast
        // that rewinds (CR 732.1) must leave no trace of it. Announced at
        // 601.2i below, once the spell is cast. `rollback_cast_to_hand` is the
        // other direction.
        self.move_object(card_id, Zone::Stack)?;

        // --- 601.2b: Choose alternative cost, additional costs, X value ---
        let chosen_alt_cost_idx = if !card_data.alternative_costs.is_empty() {
            ask_choose_alternative_cost(decisions, self, player_id, &card_data.alternative_costs)
        } else {
            None
        };

        // Validate alt cost index is in range
        if let Some(idx) = chosen_alt_cost_idx {
            if idx >= card_data.alternative_costs.len() {
                self.rollback_cast_to_hand(card_id)?;
                return Err(format!(
                    "Alternative cost index {} out of range (card has {})",
                    idx, card_data.alternative_costs.len()
                ));
            }
        }

        let chosen_additional_cost_indices = if !card_data.additional_costs.is_empty() {
            ask_choose_additional_costs(decisions, self, player_id, &card_data.additional_costs)
        } else {
            Vec::new()
        };

        // Validate additional cost indices are in range
        for &idx in &chosen_additional_cost_indices {
            if idx >= card_data.additional_costs.len() {
                self.rollback_cast_to_hand(card_id)?;
                return Err(format!(
                    "Additional cost index {} out of range (card has {})",
                    idx, card_data.additional_costs.len()
                ));
            }
        }

        // Choose X value if the cost has X symbols (rule 107.3a)
        let base_mana_cost = card_data.mana_cost.clone()
            .unwrap_or_else(ManaCost::zero);
        let x_count = base_mana_cost.x_count();
        let x_value = if x_count > 0 {
            ask_choose_x_value(decisions, self, player_id, card_id, x_count as u64)
        } else {
            0
        };

        // --- 601.2c: Choose targets ---
        let targets = if let EffectRecipient::Target(filter, count) | EffectRecipient::Choose(filter, count) = &recipient {
            let legal = enumerate_legal_selections(self, filter, Some(card_id), player_id);
            let (min_sel, max_sel) = match count {
                crate::types::effects::TargetCount::Exactly(n) => (*n as usize, *n as usize),
                crate::types::effects::TargetCount::UpTo(n) => (0, *n as usize),
            };
            let chosen = ask_select_recipients(
                decisions, self, player_id, &recipient, card_id,
                &legal, min_sel, max_sel,
            );
            if let Err(e) = self.validate_targets(&recipient, &chosen, player_id) {
                self.rollback_cast_to_hand(card_id)?;
                return Err(e);
            }
            chosen
        } else {
            Vec::new()
        };

        // --- 601.2d: Distribution placeholder (T18c) ---

        // --- Create StackEntry with all proposal data ---
        let chosen_alt = chosen_alt_cost_idx.map(|idx| card_data.alternative_costs[idx].clone());
        let chosen_additional: Vec<_> = chosen_additional_cost_indices.iter()
            .map(|&idx| card_data.additional_costs[idx].clone())
            .collect();

        let entry = StackEntry {
            object_id: card_id,
            controller: player_id,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: if x_count > 0 { Some(x_value) } else { None },
            effect,
            is_spell: true,
            chosen_alternative_cost: chosen_alt.clone(),
            additional_costs_paid: chosen_additional.clone(),
            cast_from: Some(cast_from),
            ability_identity: None,
        };
        self.stack_entries.insert(card_id, entry);

        // --- 601.2e: Post-proposal legality check ---
        // At this point the only mutations are: card moved to stack + StackEntry created.
        // No costs paid yet. If the proposal is illegal, rollback via move_object(Hand)
        // which also cleans up the StackEntry.
        //
        // Currently a no-op (the pre-proposal check is sufficient for the cards we
        // support). Future: validate that chosen targets are still legal after all
        // proposal choices are made, and that the assembled cost is payable.

        // --- 601.2f: Assemble total cost ---
        let additional_refs: Vec<_> = chosen_additional.iter().collect();
        let total_costs = assemble_total_cost(
            &base_mana_cost,
            chosen_alt.as_ref(),
            &additional_refs,
            x_value,
        );

        // --- 601.2g: Mana ability window ---
        // Rule 601.2g / 605.1a: the player activates mana abilities to pay
        // the cost. Each activation is a player decision — the engine does
        // not auto-tap. This is the rules-correct implementation point for
        // "tap lands before casting": instead of priority-level mana
        // abilities (which the candidate list does not include), the engine
        // prompts the casting player here, one ability at a time, until the
        // pool covers the cost or the player declines.
        //
        // Loop termination: (a) pool covers cost — break, proceed to 601.2h;
        // (b) DP declines (empty pick) — break, 601.2h will fail, rollback;
        // (c) no abilities remain — break, 601.2h will fail, rollback;
        // (d) loop guard trips — defensive bound to prevent infinite loops
        //     from buggy DPs or stale enumeration.
        self.run_mana_ability_window(player_id, card_id, &total_costs, decisions);

        // --- 601.2h: Pay total cost ---
        // Pre-check: can we pay? If not, roll back.
        if let Err(e) = self.can_pay_costs(&total_costs, player_id, card_id) {
            // CR 601.2 rewind, not a zone change — see rollback_cast_to_hand.
            self.rollback_cast_to_hand(card_id)?;
            return Err(e);
        }

        // Find the mana cost component for generic allocation
        let mana_cost_for_alloc = total_costs.iter().find_map(|c| {
            if let Cost::Mana(mc) = c { Some(mc.clone()) } else { None }
        }).unwrap_or_else(ManaCost::zero);

        let generic_allocation = if mana_cost_for_alloc.generic_count() > 0 {
            let mut available: Vec<(crate::types::mana::ManaType, u64)> = self.players[player_id]
                .mana_pool.available().iter()
                .filter(|(_, amt)| **amt > 0)
                .map(|(mt, amt)| (*mt, *amt))
                .collect();
            available.sort_by_key(|(mt, _)| *mt as u8);
            ask_choose_generic_mana_allocation(
                decisions, self, player_id, &mana_cost_for_alloc,
                &available, mana_cost_for_alloc.generic_count() as u64,
            )
        } else {
            HashMap::new()
        };

        self.pay_costs(&total_costs, player_id, card_id, &generic_allocation, &actx)?;

        // --- 601.2i: the spell becomes cast ---
        // The move 601.2a made silently is an event from this moment, and it is
        // announced ahead of `SpellCast` so the log reads as the CR does: put
        // onto the stack, then cast. No LKI: nothing is cast from the
        // battlefield. The mana abilities activated at 601.2g are already in
        // the log, where CR 732.1 leaves them even when a cast rewinds.
        self.announce_zone_change(card_id, cast_from, Zone::Stack, ZoneChangeCause::Cast, None)?;
        self.events.emit(GameEvent::SpellCast {
            spell_id: card_id,
            caster: player_id,
        });

        Ok(())
    }

    /// Activate a non-mana activated ability and put it on the stack (rule 602.2).
    ///
    /// Creates a new stack object representing the ability. The source permanent
    /// remains where it is. Mana abilities are handled separately in engine/mana.rs.
    ///
    /// # Future extensibility
    /// Currently assumes the source is on the battlefield. This will need to
    /// become zone-aware once we implement:
    /// - **Cycling** (activated from hand, rule 702.29)
    /// - **Unearth** (activated from graveyard, rule 702.84)
    /// - **Channel** (activated from hand, rule 702.47)
    /// - Various graveyard-activated abilities (e.g. Reassembling Skeleton's self-recursion)
    ///
    /// Planned approach: each AbilityDef gains an `activation_zone: Option<Zone>`
    /// field (None = battlefield, the default). This function would check the
    /// source is in the ability's declared activation zone.
    pub fn activate_ability(
        &mut self,
        player_id: PlayerId,
        source_id: ObjectId,
        ability_index: usize,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Verify the source is on the battlefield and controlled by this player
        // (see doc comment for future zone-aware activation plan)
        if !self.battlefield.contains_key(&source_id) {
            return Err(format!("Permanent {} not on battlefield", source_id));
        }
        // CR 602.1a's *default*, not a universal rule: an activated ability is
        // activated by the object's controller unless the ability says
        // otherwise, and 41 printed cards say otherwise with "Any player may
        // activate this ability" (Aether Storm, Excavation, Feral Hydra). That
        // permission is unmodeled — `AbilityDef` has nowhere to record it — so
        // this rejects an activation those cards would allow. Deferred
        // Migrations, "Before card breadth".
        //
        // Effective controller, not the battlefield field, so a stolen
        // permanent answers to whoever stole it (CR 613.1b).
        if !crate::oracle::characteristics::controls(self, source_id, player_id) {
            return Err(
                "Only this permanent's controller can activate its abilities                  (CR 602.1a; \"any player may activate\" is not yet modeled)"
                    .to_string(),
            );
        }

        let card_data = self.get_object(source_id)?.card_data.clone();
        // `ability_index` indexes the EFFECTIVE ability list — see the matching
        // comment in `oracle::mana_helpers::activatable_abilities`.
        let abilities = crate::oracle::characteristics::get_effective_abilities(self, source_id);
        let ability = abilities.get(ability_index)
            .ok_or_else(|| format!("Ability index {} out of range", ability_index))?;

        if ability.ability_type == AbilityType::Mana {
            return Err("Use activate_mana_ability for mana abilities".to_string());
        }
        if ability.ability_type != AbilityType::Activated {
            return Err(format!("Ability at index {} is not an activated ability", ability_index));
        }

        let effect = ability.effect.clone();
        let ability_costs = ability.costs.clone();
        let identity = crate::state::game_state::AbilityIdentity {
            source: source_id,
            ability: ability.id,
        };
        let recipient = match &effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            _ => EffectRecipient::Implicit,
        };

        // Create a new object on the stack representing the ability (rule 602.2a)
        // Abilities on the stack are not cards — they have no CardData.
        // We create a minimal GameObject to track it.
        let ability_obj = GameObject::new(card_data.clone(), player_id, Zone::Stack);
        let ability_obj_id = ability_obj.id;
        self.objects.insert(ability_obj_id, ability_obj);
        self.stack.push(ability_obj_id);

        // From here on, any Err path must call `rollback_ability_activation`
        // to keep game state clean (required by the priority-retry loop in
        // `run_priority_round` — see D26 / SPECIAL-2).

        // Choose targets
        let targets = if let EffectRecipient::Target(filter, count) | EffectRecipient::Choose(filter, count) = &recipient {
            let legal = enumerate_legal_selections(self, filter, Some(ability_obj_id), player_id);
            let (min_sel, max_sel) = match count {
                crate::types::effects::TargetCount::Exactly(n) => (*n as usize, *n as usize),
                crate::types::effects::TargetCount::UpTo(n) => (0, *n as usize),
            };
            let chosen = ask_select_recipients(
                decisions, self, player_id, &recipient, ability_obj_id,
                &legal, min_sel, max_sel,
            );
            if let Err(e) = self.validate_targets(&recipient, &chosen, player_id) {
                self.rollback_ability_activation(ability_obj_id);
                return Err(e);
            }
            chosen
        } else {
            Vec::new()
        };

        // Create StackEntry
        let stack_entry = StackEntry {
            object_id: ability_obj_id,
            controller: player_id,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: None,
            effect,
            is_spell: false,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            // An activated ability is not cast from anywhere (CR 602.2a gives
            // it a source, which is a different fact). See `cast_from`.
            cast_from: None,
            ability_identity: Some(identity),
        };
        self.stack_entries.insert(ability_obj_id, stack_entry);

        // CR 602.2a — the ability is on the stack. Identified durably: the
        // ephemeral `ability_obj_id` is deleted at resolution.
        self.events.emit(GameEvent::AbilityActivated { identity, controller: player_id });

        // --- 602.1b: Mana ability window ---
        // Same rules-correct model as 601.2g for spells. The player activates
        // mana abilities as needed to pay the activated-ability cost. Pool is
        // filled in-place; caller's pay_costs below detects insufficiency and
        // triggers rollback.
        self.run_mana_ability_window(player_id, source_id, &ability_costs, decisions);

        // Pay ability costs. Activation is CR 602, not a resolution.
        let actx = ActionContext::new(decisions);
        let generic_allocation = HashMap::new();
        if let Err(e) = self.pay_costs(&ability_costs, player_id, source_id, &generic_allocation, &actx) {
            self.rollback_ability_activation(ability_obj_id);
            return Err(e);
        }

        Ok(())
    }

    /// Run the 601.2g / 602.1b mana-ability window for a pending spell or
    /// activated ability.
    ///
    /// Prompts the player to activate mana abilities one at a time until
    /// `total_costs` can be paid from the player's mana pool, the player
    /// declines, or no activatable abilities remain. Does not roll back on
    /// failure — the caller's post-window `can_pay_costs` / `pay_costs` step
    /// handles rollback if the pool still doesn't cover the cost.
    ///
    /// # Mana-cost extraction
    /// Only `Cost::Mana` is relevant to this window: rule 601.2g explicitly
    /// restricts the activation-during-cost-payment window to *mana abilities*.
    /// Non-mana costs (Cost::Tap, Cost::SacrificeSelf, Cost::PayLife, …) are
    /// paid in 601.2h, which has no activation window. We extract the mana
    /// component to build the `remaining_cost` context the DP sees.
    ///
    /// # Termination
    /// Termination is a DP-correctness property, not an engine invariant. The
    /// CR places no cap on how many mana abilities a player may activate
    /// during 601.2g. The loop terminates when one of the following holds:
    ///
    /// 1. `can_pay_costs` succeeds (cost covered) → return.
    /// 2. `ask_activate_mana_ability` returns `None` (DP declines) → return.
    /// 3. `enumerate_activatable_mana_abilities` returns empty after filtering
    ///    the failure blacklist → return.
    ///
    /// The **failure blacklist** guards against enumeration over-approximation
    /// or TOCTOU bugs: if `activate_mana_ability` fails after enumeration said
    /// the ability was legal, we blacklist `(perm_id, ability_id)` for the
    /// remainder of this window so the DP can't pick it again. The blacklist
    /// is bounded by `|initial_legal|`, so it cannot loop forever on failure.
    ///
    /// The only remaining infinite-loop risk is a buggy DP that keeps
    /// successfully activating abilities forever (e.g., cycling mana-filter
    /// abilities). That is a DP-correctness concern — `RandomDecisionProvider`
    /// caps itself with an internal per-window counter; a future `AutoPayDP`
    /// will use a mana-bootstrap solver; a human CLI user self-polices.
    fn run_mana_ability_window(
        &mut self,
        player_id: PlayerId,
        spell_or_ability_id: ObjectId,
        total_costs: &[Cost],
        decisions: &dyn DecisionProvider,
    ) {
        let mana_cost_for_window = total_costs
            .iter()
            .find_map(|c| if let Cost::Mana(mc) = c { Some(mc.clone()) } else { None })
            .unwrap_or_else(ManaCost::zero);

        let mut failed: std::collections::HashSet<(ObjectId, AbilityId)> =
            std::collections::HashSet::new();

        loop {
            if self.can_pay_costs(total_costs, player_id, spell_or_ability_id).is_ok() {
                return;
            }

            let legal: Vec<(ObjectId, AbilityId)> =
                enumerate_activatable_mana_abilities(self, player_id)
                    .into_iter()
                    .filter(|k| !failed.contains(k))
                    .collect();
            if legal.is_empty() {
                return; // caller's pay_costs will fail and roll back
            }

            let pool = &self.players[player_id].mana_pool;
            let remaining = remaining_cost_after_pool(&mana_cost_for_window, pool);

            match ask_activate_mana_ability(
                decisions, self, player_id, spell_or_ability_id, &remaining, &legal,
            ) {
                Some((perm_id, ability_id)) => {
                    let actx = ActionContext::new(decisions);
                    if let Err(e) = self.activate_mana_ability(player_id, perm_id, ability_id, &actx) {
                        // Enumeration said this was legal but activation
                        // failed — likely staleness or a `can_pay_ability_costs`
                        // over-approximation bug. Blacklist so we can't loop
                        // on it; the set is bounded by |initial legal|.
                        eprintln!(
                            "WARN: mana-ability activation failed in 601.2g window \
                             (perm={}, ab={}): {}",
                            perm_id, ability_id, e
                        );
                        failed.insert((perm_id, ability_id));
                    }
                }
                None => {
                    // DP declined; caller's pay_costs determines whether the
                    // current pool suffices.
                    return;
                }
            }
        }
    }

    /// Put a card back in its owner's hand after a failed `cast_spell`.
    ///
    /// **This is not a zone change, and it deliberately bypasses the
    /// chokepoint.** CR 601.2 rewinds the entire casting process when a step
    /// cannot be completed: the card was never legally on the stack, so nothing
    /// in the game may observe it moving back. Routing this through
    /// `change_zone` would offer a replacement effect an event that did not
    /// happen — and CR 903.9b redirecting a commander mid-rollback is exactly
    /// the wrong answer.
    ///
    /// It is also what the no-catchall rule predicts. `ZoneChangeCause` names
    /// why the engine moved an object, every mover names its reason, and a
    /// rewind has no honest reason to give (`replacement-architecture.md` §11).
    /// A site with nothing to say is a site that does not belong on the
    /// chokepoint, not a site that needs an `Other` variant.
    ///
    /// `move_object` does the full teardown — `remove_from_zone_collection(Stack)`
    /// clears the `StackEntry` — and announces nothing, like the 601.2a move it
    /// reverses: neither direction is an event, and the forward move is
    /// announced at 601.2i only once the spell is cast.
    fn rollback_cast_to_hand(&mut self, card_id: ObjectId) -> Result<(), String> {
        // CAST-ROLLBACK: see the doc comment. Do not "fix" this into change_zone.
        self.move_object(card_id, Zone::Hand)
    }

    /// Remove an ability object that was pushed onto the stack by a failed
    /// `activate_ability` call. Used to keep state clean when target
    /// validation or cost payment fails mid-activation (see D26 / SPECIAL-2).
    fn rollback_ability_activation(&mut self, ability_obj_id: ObjectId) {
        self.stack.retain(|&id| id != ability_obj_id);
        self.stack_entries.remove(&ability_obj_id);
        self.objects.remove(&ability_obj_id);
    }

    /// Check whether a player can legally begin casting a spell (rule 601.3).
    ///
    /// # Future extensibility
    /// Currently hard-codes Zone::Hand as the only legal cast zone. This will
    /// need to become a query against "cast permissions" once we implement:
    /// - **Flashback** (cast from graveyard, rule 702.33)
    /// - **Cascade / Impulse draw** (cast from exile)
    /// - **Cycling-adjacent** cast-from-zone effects
    ///
    /// The planned approach: introduce a `CastPermission` enum or trait that
    /// cards/effects register on the GameState (e.g. "player X may cast card Y
    /// from zone Z this turn"). `check_cast_legality` would then check the
    /// card's current zone against any active permissions, defaulting to Hand.
    fn check_cast_legality(
        &self,
        player_id: PlayerId,
        card_id: ObjectId,
    ) -> Result<(), String> {
        let obj = self.get_object(card_id)?;

        // Card must be in hand (see doc comment for future zone-casting plan)
        if obj.zone != Zone::Hand {
            return Err(format!("Card is in {:?}, not in hand", obj.zone));
        }

        // Card must belong to (or be controlled by) this player
        if obj.owner != player_id {
            return Err("Cannot cast another player's spell".to_string());
        }

        // Timing check (rule 117.1a):
        // - Instants and spells with flash: anytime you have priority
        // - Everything else: main phase, stack empty, active player only
        // PRE-LAYER ZONE: printed types and keywords, for the same reason -- this
        // decides whether a card in hand may be cast now.
        let is_instant = obj.card_data.types.contains(&CardType::Instant);
        let has_flash = obj.card_data.keyword_flags.contains(&KeywordFlag::Flash);

        if !is_instant && !has_flash {
            // Sorcery-speed timing
            if player_id != self.active_player {
                return Err("Only the active player can cast sorcery-speed spells".to_string());
            }
            match self.phase.phase_type {
                PhaseType::Precombat | PhaseType::Postcombat => {}
                _ => return Err("Sorcery-speed spells can only be cast during a main phase".to_string()),
            }
            // Since RC-1 the resolving object is still on the stack (CR 608.2),
            // so this reads "not empty" throughout a resolution. Unreachable
            // today — CR 608.2g forbids casting during one at all — but the
            // "unless an effect instructs" half of 608.2g is what an RC-era card
            // brings, and then this site has to say which it means: the
            // instruction overriding timing outright, or the resolving object not
            // counting against its own instruction. Not decided here.
            if !self.stack.is_empty() {
                return Err("Sorcery-speed spells can only be cast when the stack is empty".to_string());
            }
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::resolve::ResolvedTarget;
    use crate::objects::card_data::{AbilityDef, CardDataBuilder};
    use crate::types::card_types::*;
    use crate::types::effects::{AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::ScriptedDecisionProvider;
    use crate::test_support::lightning_bolt as make_bolt;

    fn setup_for_casting() -> (GameState, ObjectId, ScriptedDecisionProvider) {
        let mut game = GameState::new(2, 20);
        // Give player 0 a bolt in hand and red mana
        let bolt = make_bolt();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        // Set to precombat main phase so sorcery-speed works too
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        // SelectionFilter::Any → [Player(0), Player(1)] — Player(1) is at index 1
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: card_id,
        }, vec![1]);

        (game, card_id, decisions)
    }

    #[test]
    fn test_cast_records_origin_zone() {
        let (mut game, card_id, decisions) = setup_for_casting();
        game.cast_spell(0, card_id, &decisions).unwrap();

        // CR 601.2a's origin, captured before the card moved. Unrecoverable
        // afterward — the object's own `zone` now says Stack, which is the
        // whole reason the field exists (CR 903.8 commander tax counts casts
        // *from the command zone*).
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.cast_from, Some(Zone::Hand));
        assert_eq!(game.get_object(card_id).unwrap().zone, Zone::Stack);
    }

    #[test]
    fn test_ability_activation_and_resolution_carry_a_durable_identity() {
        use crate::events::event::GameEvent;
        let mut game = crate::test_support::setup_two_player_game();
        let thaum = crate::test_support::put_on_battlefield(
            &mut game,
            crate::cards::utility_creatures::merfolk_thaumaturgist(),
            0,
        );
        game.battlefield.get_mut(&thaum).unwrap().controller_since_turn = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(
                SelectionFilter::Creature,
                TargetCount::Exactly(1),
            ),
            spell_id: thaum,
        }, vec![0]);

        let abilities = crate::oracle::characteristics::get_effective_abilities(&game, thaum);
        let idx = abilities.iter()
            .position(|a| a.ability_type == AbilityType::Activated)
            .unwrap();
        let ability_id = abilities[idx].id;

        game.activate_ability(0, thaum, idx, &decisions).unwrap();
        game.resolve_top_of_stack(&decisions).unwrap();

        let activated: Vec<_> = game.events.events().filter_map(|e| match e {
            GameEvent::AbilityActivated { identity, .. } => Some(*identity),
            _ => None,
        }).collect();
        let resolved: Vec<_> = game.events.events().filter_map(|e| match e {
            GameEvent::AbilityResolved { identity, .. } => Some(*identity),
            _ => None,
        }).collect();

        // The point of the pair: both name (source, ability), not the ephemeral
        // stack object — which by now has been deleted (CR 608.2n). CR 603.7h
        // counts resolutions of *this ability of this permanent*, and neither
        // half survives in the ephemeral id.
        assert_eq!(activated.len(), 1);
        assert_eq!(resolved.len(), 1);
        assert_eq!(activated[0].source, thaum);
        assert_eq!(activated[0].ability, ability_id);
        assert_eq!(resolved[0], activated[0], "same identity across activation and resolution");

        // And the ephemeral really is gone, so nothing could have used it.
        let ephemeral_still_present = game.objects.values()
            .any(|o| o.zone == Zone::Stack);
        assert!(!ephemeral_still_present);
    }

    #[test]
    fn test_activated_ability_has_no_origin_zone() {
        // The `cast_from.is_some() == is_spell` invariant, from the other side.
        // An activated ability is not cast from anywhere; CR 602.2a gives it a
        // *source*, which is a different fact and must not collapse into this
        // field.
        let mut game = crate::test_support::setup_two_player_game();
        let thaum = crate::test_support::put_on_battlefield(
            &mut game,
            crate::cards::utility_creatures::merfolk_thaumaturgist(),
            0,
        );
        // Summoning sickness would block the {T} cost (CR 302.6). 0 = pregame,
        // the convention has_summoning_sickness reads.
        game.battlefield.get_mut(&thaum).unwrap().controller_since_turn = 0;

        let decisions = ScriptedDecisionProvider::new();
        let ability_id = crate::oracle::characteristics::get_effective_abilities(&game, thaum)
            .iter()
            .find(|a| a.ability_type == AbilityType::Activated)
            .expect("Merfolk Thaumaturgist has an activated ability")
            .id;
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(
                SelectionFilter::Creature,
                TargetCount::Exactly(1),
            ),
            spell_id: thaum,
        }, vec![0]);

        let idx = crate::oracle::characteristics::get_effective_abilities(&game, thaum)
            .iter()
            .position(|a| a.id == ability_id)
            .unwrap();
        game.activate_ability(0, thaum, idx, &decisions).unwrap();

        // There is an ability on the stack, and it records no origin zone.
        let ability_entries: Vec<_> = game.stack_entries.values()
            .filter(|e| !e.is_spell)
            .collect();
        assert_eq!(ability_entries.len(), 1, "the ability should be on the stack");
        assert_eq!(ability_entries[0].cast_from, None);

        for entry in game.stack_entries.values() {
            assert_eq!(
                entry.cast_from.is_some(),
                entry.is_spell,
                "cast_from must be Some exactly when the entry is a spell",
            );
        }
    }

    #[test]
    fn test_cast_instant_spell() {
        let (mut game, card_id, decisions) = setup_for_casting();
        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card should be on the stack
        assert!(game.stack.contains(&card_id));
        assert!(game.stack_entries.contains_key(&card_id));
        assert_eq!(game.get_object(card_id).unwrap().zone, Zone::Stack);

        // Mana should be spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);

        // Hand should be empty
        assert!(game.players[0].hand.is_empty());

        // StackEntry should have correct targets
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.chosen_targets, vec![ResolvedTarget::Player(1)]);
        assert!(entry.is_spell);
    }

    #[test]
    fn test_cast_from_wrong_zone() {
        let mut game = GameState::new(2, 20);
        let bolt = make_bolt();
        let obj = GameObject::new(bolt, 0, Zone::Graveyard);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].graveyard.push(card_id);

        let decisions = ScriptedDecisionProvider::new();
        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_not_enough_mana() {
        let (mut game, card_id, decisions) = setup_for_casting();
        // Drain the mana pool
        let _ = game.players[0].mana_pool.remove(ManaType::Red, 1);

        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_sorcery_timing_wrong_phase() {
        let mut game = GameState::new(2, 20);
        let sorcery_data = CardDataBuilder::new("Lava Axe")
            .card_type(CardType::Sorcery)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 4))
            .ability(AbilityDef {
                is_characteristic_defining: false,
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(5)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(sorcery_data, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Set to combat phase — sorceries can't be cast here
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);

        let decisions = ScriptedDecisionProvider::new();
        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_instant_during_combat() {
        let (mut game, card_id, decisions) = setup_for_casting();
        // Instants can be cast during any phase
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);
        game.cast_spell(0, card_id, &decisions).unwrap();
        assert!(game.stack.contains(&card_id));
    }

    // --- T18a: X value, alternative cost, additional cost, rollback tests ---

    fn make_x_spell() -> std::sync::Arc<crate::objects::card_data::CardData> {
        use crate::types::mana::ManaSymbol;
        // Blaze: {X}{R} — deal X damage to any target
        CardDataBuilder::new("Blaze")
            .card_type(CardType::Sorcery)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
            .ability(AbilityDef {
                is_characteristic_defining: false,
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Variable),
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build()
    }

    #[test]
    fn test_cast_x_spell_x_equals_3() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Need {R} + 3 generic = 4 total mana
        game.players[0].mana_pool.add(ManaType::Red, 4);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.expect_number(ChoiceKind::ChooseXValue { spell_id: card_id, x_count: 1 }, 3);
        // SelectionFilter::Any → [Player(0), Player(1)] — Player(1) is at index 1
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: card_id,
        }, vec![1]);
        // 3 generic from Red pool → [3]
        decisions.expect_allocation(
            ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::zero() },
            vec![3],
        );

        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card on stack
        assert!(game.stack.contains(&card_id));
        // X value stored in StackEntry
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, Some(3));
        // Mana spent: 1 Red + 3 generic (from Red pool) = 4 Red total
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_x_spell_x_equals_0() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Only need {R} for X=0
        game.players[0].mana_pool.add(ManaType::Red, 1);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.expect_number(ChoiceKind::ChooseXValue { spell_id: card_id, x_count: 1 }, 0);
        // SelectionFilter::Any → [Player(0), Player(1)] — Player(1) is at index 1
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: card_id,
        }, vec![1]);

        game.cast_spell(0, card_id, &decisions).unwrap();

        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, Some(0));
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_x_spell_insufficient_mana_rollback() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Only 2 Red, but X=3 needs 4 total
        game.players[0].mana_pool.add(ManaType::Red, 2);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.expect_number(ChoiceKind::ChooseXValue { spell_id: card_id, x_count: 1 }, 3);
        // SelectionFilter::Any → [Player(0), Player(1)] — Player(1) is at index 1
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: card_id,
        }, vec![1]);

        let result = game.cast_spell(0, card_id, &decisions);
        assert!(result.is_err());

        // Card should be back in hand (rollback)
        assert_eq!(game.get_object(card_id).unwrap().zone, Zone::Hand);
        assert!(game.players[0].hand.contains(&card_id));
        assert!(!game.stack.contains(&card_id));
        assert!(!game.stack_entries.contains_key(&card_id));
        // Mana should not have been spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 2);
    }

    #[test]
    fn test_cast_with_alternative_cost() {
        use crate::types::costs::AlternativeCost;

        // Card with alt cost: "Pay 3 life instead of mana cost"
        let card = CardDataBuilder::new("Force Spike Variant")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Blue)
            .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
            .ability(AbilityDef {
                is_characteristic_defining: false,
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Sequence(Vec::new()),
            })
            .alternative_cost(AlternativeCost::Custom(
                "Pay 3 life".to_string(),
                vec![Cost::PayLife(3)],
            ))
            .build();

        let mut game = GameState::new(2, 20);
        let obj = GameObject::new(card, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // No mana needed — paying life instead
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        // Options: [NormalCost, AlternativeCost(Custom(...))] — index 1 = first alt cost
        decisions.expect_pick_n(ChoiceKind::ChooseAlternativeCost, vec![1]);

        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card on stack
        assert!(game.stack.contains(&card_id));
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert!(entry.chosen_alternative_cost.is_some());
        // Life paid
        assert_eq!(game.players[0].life_total, 17);
    }

    #[test]
    fn test_cast_with_kicker_additional_cost() {
        use crate::types::costs::AdditionalCost;

        // Card: {1}{R} with kicker {R}
        let card = CardDataBuilder::new("Goblin Bushwhacker")
            .card_type(CardType::Creature)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 1))
            .power_toughness(1, 1)
            .additional_cost(AdditionalCost::Kicker(vec![
                Cost::Mana(ManaCost::build(&[ManaType::Red], 0)),
            ]))
            .build();

        let mut game = GameState::new(2, 20);
        let obj = GameObject::new(card, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Need {1}{R} (base) + {R} (kicker) = 3 red total
        game.players[0].mana_pool.add(ManaType::Red, 3);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        // Options: [AdditionalCost::Kicker(...)] — index 0 = first (only) additional cost
        decisions.expect_pick_n(ChoiceKind::ChooseAdditionalCosts, vec![0]);
        // 1 generic from Red pool → [1]
        decisions.expect_allocation(
            ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::zero() },
            vec![1],
        );

        game.cast_spell(0, card_id, &decisions).unwrap();

        assert!(game.stack.contains(&card_id));
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.additional_costs_paid.len(), 1);
        assert!(matches!(&entry.additional_costs_paid[0], AdditionalCost::Kicker(_)));
        // All 3 red mana spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_normal_cost_no_x_no_alt() {
        // Verify the normal path still sets x_value=None and no alt/additional
        let (mut game, card_id, decisions) = setup_for_casting();
        game.cast_spell(0, card_id, &decisions).unwrap();

        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, None);
        assert!(entry.chosen_alternative_cost.is_none());
        assert!(entry.additional_costs_paid.is_empty());
    }
}
