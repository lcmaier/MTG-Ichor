use std::collections::{BTreeMap, HashSet};

use crate::events::event::{GameEvent, LossReason};
use crate::oracle::characteristics::{
    get_effective_controller, get_effective_name, get_effective_toughness, has_keyword,
    has_subtype, has_supertype, has_type, is_creature,
};
use crate::types::keywords::KeywordFlag;
use crate::state::game_state::GameState;
use crate::types::card_types::{ArtifactType, CardType, EnchantmentType, Subtype, Supertype};
use crate::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use crate::engine::resolve::ResolvedTarget;
use crate::types::effects::CounterType;
use crate::types::ids::ObjectId;
use crate::types::zones::Zone;
use crate::ui::ask::ask_choose_legend_to_keep;
use crate::ui::decision::DecisionProvider;

/// State-Based Actions (rule 704)
///
/// SBAs are checked whenever a player would receive priority. They don't use
/// the stack — they just happen. If any SBA is performed, they're all checked
/// again before a player actually gets priority.

/// One permanent leaving the battlefield because of a state-based action.
///
/// Gathered against the pre-check game state and performed in one batch, per
/// CR 704.3. The `display` event is sugar for the log — the `ZoneChange` and
/// its LKI frame are what a trigger matcher reads.
struct SbaZoneChange {
    /// Kept out of `action` so the CR 704.7 dedupe does not have to match on a
    /// `GameAction` variant to find the object.
    object: ObjectId,
    action: GameAction,
    display: GameEvent,
}

impl GameState {
    /// Build the state-based zone change that `cause` calls for on `id`.
    ///
    /// Reads the owner *before* the batch runs, which is the point of gathering:
    /// every field of the proposal describes the board as the check saw it.
    fn sba_death(&self, id: ObjectId, cause: ZoneChangeCause) -> SbaZoneChange {
        let owner = self.objects.get(&id).map(|o| o.owner).unwrap_or(0);
        let display = match cause {
            ZoneChangeCause::ZeroToughness | ZoneChangeCause::DestroyedBySba => {
                GameEvent::CreatureDied { creature_id: id, owner }
            }
            ZoneChangeCause::ZeroLoyalty => GameEvent::PlaneswalkerDied { object_id: id, owner },
            ZoneChangeCause::LegendRule => {
                GameEvent::LegendRuleSacrificed { object_id: id, owner }
            }
            ZoneChangeCause::AuraSba => GameEvent::AuraDied { object_id: id, owner },
            other => unreachable!("{:?} is not a state-based zone change", other),
        };
        SbaZoneChange {
            object: id,
            action: GameAction::ZoneChange {
                object: id,
                from: Zone::Battlefield,
                to: Zone::Graveyard,
                cause,
            },
            display,
        }
    }
}

impl GameState {
    /// Check and perform all state-based actions.
    /// Returns true if any SBA was performed (caller should re-check).
    pub fn check_state_based_actions(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<bool, String> {
        // CR 704 sweeps are turn-based, not part of any resolution.
        let actx = ActionContext::new(decisions);
        let mut any_performed = false;

        // 704.5a — Player with 0 or less life loses the game
        for i in 0..self.players.len() {
            if self.players[i].life_total <= 0 && !self.player_lost[i] {
                self.player_lost[i] = true;
                self.events.emit(GameEvent::PlayerLost {
                    player_id: i,
                    reason: LossReason::LifeReachedZero,
                });
                any_performed = true;
            }
        }

        // 704.5b — Player who attempted to draw from empty library loses
        for i in 0..self.players.len() {
            if self.players[i].has_drawn_from_empty_library && !self.player_lost[i] {
                self.player_lost[i] = true;
                self.events.emit(GameEvent::PlayerLost {
                    player_id: i,
                    reason: LossReason::DrawnFromEmptyLibrary,
                });
                any_performed = true;
            }
        }

        // 704.5c — Player with 10 or more poison counters loses the game
        for i in 0..self.players.len() {
            if self.players[i].poison_counters >= 10 && !self.player_lost[i] {
                self.player_lost[i] = true;
                self.events.emit(GameEvent::PlayerLost {
                    player_id: i,
                    reason: LossReason::PoisonCounters,
                });
                any_performed = true;
            }
        }

        // 704.5 — Player who has been dealt 21 or more combat damage by a single
        // commander loses the game (Commander variant rule)
        for i in 0..self.players.len() {
            if !self.player_lost[i] {
                let lost = self.players[i].commander_damage_taken.values().any(|&dmg| dmg >= 21);
                if lost {
                    self.player_lost[i] = true;
                    self.events.emit(GameEvent::PlayerLost {
                        player_id: i,
                        reason: LossReason::CommanderDamage,
                    });
                        any_performed = true;
                }
            }
        }

        // --- CR 704.3: one check, one event -------------------------------
        //
        // "The game checks for any of the listed conditions for state-based
        // actions, then performs all applicable state-based actions
        // simultaneously as a single event."
        //
        // Every condition below is evaluated against *this* game state, before
        // any of the resulting moves is performed. That is the whole difference
        // from the sweep this replaces, which performed 704.5f's moves before it
        // asked 704.5g's question, and so could never produce the simultaneity
        // CR 704.7 and CR 616.1 are written against.
        //
        // Ordered sweeps throughout: the batch order is the order a CR 616.1
        // prompt would be offered in, and the graveyard is an ordered zone.
        let mut deaths: Vec<SbaZoneChange> = Vec::new();

        // 704.5f — Creature with toughness 0 or less is put into owner's graveyard
        for id in self.battlefield_ids_ordered() {
            if !is_creature(self, id) {
                continue;
            }
            if get_effective_toughness(self, id).unwrap_or(0) <= 0 {
                deaths.push(self.sba_death(id, ZoneChangeCause::ZeroToughness));
            }
        }

        // 704.5g — Creature with lethal damage is destroyed
        // Also handles deathtouch (rule 702.2b): any nonzero damage from a
        // deathtouch source is lethal.
        for id in self.battlefield_ids_ordered() {
            if !is_creature(self, id) {
                continue;
            }
            let effective_t = get_effective_toughness(self, id).unwrap_or(0);
            if effective_t <= 0 {
                continue; // handled by 704.5f
            }
            // Indestructible creatures are not destroyed by lethal damage (rule 702.12b)
            if has_keyword(self, id, KeywordFlag::Indestructible) {
                continue;
            }
            let entry = self.battlefield.get(&id).unwrap();
            // TODO: check for regeneration
            let lethal = entry.damage_marked >= effective_t as u32
                || (entry.damage_marked > 0 && entry.damaged_by_deathtouch);
            if lethal {
                deaths.push(self.sba_death(id, ZoneChangeCause::DestroyedBySba));
            }
        }

        // 704.5i — Planeswalker with 0 loyalty is put into owner's graveyard
        for id in self.battlefield_ids_ordered() {
            if !self.objects.contains_key(&id) || !has_type(self, id, CardType::Planeswalker) {
                continue;
            }
            if self.battlefield[&id].counter_count(CounterType::Loyalty) == 0 {
                deaths.push(self.sba_death(id, ZoneChangeCause::ZeroLoyalty));
            }
        }

        // 704.5j — Legend rule: if a player controls two or more legendary
        // permanents with the same name, they choose one to keep and the
        // rest are put into their owners' graveyards.
        {
            // Group legendary permanents by (controller, effective_name).
            //
            // `BTreeMap` over an ordered sweep, both deliberate: the controller
            // is prompted once per group, so group order is decision order, and
            // `ids` is the option list they pick from by index.
            let mut legend_groups: BTreeMap<(usize, String), Vec<ObjectId>> = BTreeMap::new();
            for (id, _entry) in self.battlefield_ordered() {
                if self.objects.contains_key(&id) {
                    if has_supertype(self, id, Supertype::Legendary) {
                        let name = get_effective_name(self, id);
                        // CR 704.5j groups by controller, so it has to be the
                        // effective one: taking an opponent's copy of a legend
                        // you already control is what creates the conflict.
                        let Some(controller) = get_effective_controller(self, id) else {
                            continue;
                        };
                        legend_groups.entry((controller, name)).or_default().push(id);
                    }
                }
            }

            // For each group with more than one, the controller chooses one to keep.
            //
            // Note this now runs against a board that still contains creatures
            // dying elsewhere in the same check — CR 704.3 is explicit that every
            // condition is read before anything is performed — so a player with
            // two Isamarus, one of them dead to lethal damage, is genuinely asked
            // which to keep. The old sequential sweep skipped that prompt by
            // having already removed the dead one.
            let mut chosen: Vec<ObjectId> = Vec::new();
            for ((controller, name), ids) in &legend_groups {
                if ids.len() > 1 {
                    // The group key, not a re-read: the key is what put these
                    // permanents together, so asking anyone else would prompt a
                    // player who does not control them.
                    let keep = ask_choose_legend_to_keep(decisions, self, *controller, name, ids);
                    for &id in ids {
                        if id != keep {
                            chosen.push(id);
                        }
                    }
                }
            }
            for id in chosen {
                deaths.push(self.sba_death(id, ZoneChangeCause::LegendRule));
            }
        }

        // 704.5k — World rule
        // (future SBAs added here as needed)

        // 704.5m — Aura not attached to anything -> owner's graveyard
        // 704.5n — Aura attached to an illegal object -> owner's graveyard
        //
        // Collect aura IDs in a single pass to avoid borrow-checker issues:
        // we need &self.objects for subtype checks but &mut self for move_object.
        let auras_to_graveyard: Vec<ObjectId> = self.battlefield_ordered()
            .into_iter()
            .filter_map(|(id, entry)| {
                let obj = self.objects.get(&id)?;
                if !has_subtype(self, id, &Subtype::Enchantment(EnchantmentType::Aura)) {
                    return None;
                }
                match entry.attached_to {
                    // 704.5m: Aura not attached to anything
                    None => Some(id),
                    Some(host_id) => {
                        // 704.5n: host no longer on the battlefield
                        if !self.battlefield.contains_key(&host_id) {
                            return Some(id);
                        }
                        // 704.5n: host doesn't match enchant filter
                        // The enchant restriction is text on the Aura, so
                        // CR 109.5 makes its "you" the Aura's controller — not
                        // the enchanted creature's, which CR 303.4e keeps
                        // separate.
                        if let Some(filter) = &obj.card_data.enchant_filter {
                            let candidate = ResolvedTarget::Object(host_id);
                            let you = get_effective_controller(self, id)?;
                            if self.validate_selection(filter, &candidate, you).is_err() {
                                return Some(id);
                            }
                        }
                        None
                    }
                }
            })
            .collect();

        for id in auras_to_graveyard {
            deaths.push(self.sba_death(id, ZoneChangeCause::AuraSba));
        }

        // --- Perform the gathered zone changes as one event (CR 704.3) ------
        //
        // CR 704.7's same-result collapse is the dedupe: two state-based
        // actions that would put the same permanent into the same graveyard at
        // the same time have the same *result*, so they are one event with one
        // applied set, not two. The first condition in CR order names the cause
        // — a creature that is both a duplicate legend and dead to lethal damage
        // was destroyed (704.5g), not put away by the legend rule.
        let mut seen: HashSet<ObjectId> = HashSet::new();
        deaths.retain(|d| seen.insert(d.object));

        if !deaths.is_empty() {
            let batch = deaths.iter().map(|d| d.action.clone()).collect();
            self.execute_actions(batch, &actx)?;
            // Display sugar, emitted after the event it describes. A trigger
            // matcher keys on the `ZoneChange` and its LKI frame — see the doc
            // comments on `GameEvent::CreatureDied` and its two siblings.
            for death in deaths {
                self.events.emit(death.display);
            }
            any_performed = true;
        }

        // 704.5p — Equipment/Fortification attached to non-creature → unattach
        // Equipment stays on the battlefield; only the attachment is broken.
        let equip_bad_host: Vec<(ObjectId, ObjectId)> = self.battlefield_ordered()
            .into_iter()
            .filter_map(|(id, entry)| {
                self.objects.get(&id)?;
                let has_equip = has_subtype(self, id, &Subtype::Artifact(ArtifactType::Equipment));
                let has_fort = has_subtype(self, id, &Subtype::Artifact(ArtifactType::Fortification));
                if !has_equip && !has_fort { return None; }
                let host_id = entry.attached_to?;
                if !is_creature(self, host_id) {
                    Some((id, host_id))
                } else {
                    None
                }
            })
            .collect();

        for (equip_id, host_id) in equip_bad_host {
            if let Some(entry) = self.battlefield.get_mut(&equip_id) {
                entry.detach();
            }
            if let Some(host) = self.battlefield.get_mut(&host_id) {
                host.attached_by.retain(|&aid| aid != equip_id);
            }
            self.events.emit(GameEvent::EquipmentDetached { equipment_id: equip_id, former_host: host_id });
            any_performed = true;
        }

        // 704.5q (attachment catch-all) — If a permanent that's neither an Aura,
        // Equipment, nor Fortification is attached to another permanent, it becomes
        // unattached. This catches illegal attachment state that may arise from
        // type-changing effects.
        let illegal_attachments: Vec<(ObjectId, ObjectId)> = self.battlefield_ordered()
            .into_iter()
            .filter_map(|(id, entry)| {
                self.objects.get(&id)?;
                let is_aura = has_subtype(self, id, &Subtype::Enchantment(EnchantmentType::Aura));
                let is_equip = has_subtype(self, id, &Subtype::Artifact(ArtifactType::Equipment));
                let is_fort = has_subtype(self, id, &Subtype::Artifact(ArtifactType::Fortification));
                if is_aura || is_equip || is_fort {
                    return None;
                }
                let host_id = entry.attached_to?;
                Some((id, host_id))
            })
            .collect();

        for (att_id, host_id) in illegal_attachments {
            if let Some(entry) = self.battlefield.get_mut(&att_id) {
                entry.detach();
            }
            if let Some(host) = self.battlefield.get_mut(&host_id) {
                host.attached_by.retain(|&aid| aid != att_id);
            }
            self.events.emit(GameEvent::EquipmentDetached { equipment_id: att_id, former_host: host_id });
            any_performed = true;
        }

        // TODO: 704.5p — An Aura that is also a creature can't enchant anything.
        // If this occurs, the Aura becomes unattached and remains on the battlefield as a creature. 
        // Relevant when L4 type-changing effects (e.g., a hypothetical
        // non-Aura-excluding Opalescence variant) add Creature to an Aura. Bestow
        // (702.103) avoids this by being only an aura if cast for a Bestow cost, switching over
        // to creature if it becomes unattached for any reason. Implement when L4 type-changing + Aura cards coexist.

        // 704.5q — +1/+1 and -1/-1 counter annihilation
        // If a permanent has both +1/+1 and -1/-1 counters, remove pairs
        // until only one type remains.
        let annihilation_targets: Vec<(ObjectId, u32)> = self.battlefield_ordered()
            .into_iter()
            .filter_map(|(id, entry)| {
                let plus = entry.counter_count(CounterType::PlusOnePlusOne);
                let minus = entry.counter_count(CounterType::MinusOneMinusOne);
                if plus > 0 && minus > 0 {
                    Some((id, plus.min(minus)))
                } else {
                    None
                }
            })
            .collect();

        for (id, pairs) in annihilation_targets {
            if let Some(entry) = self.battlefield.get_mut(&id) {
                entry.remove_counters(CounterType::PlusOnePlusOne, pairs);
                entry.remove_counters(CounterType::MinusOneMinusOne, pairs);
            }
            self.events.emit(GameEvent::CountersAnnihilated { object_id: id, pairs_removed: pairs });
            any_performed = true;
        }

        // 704.5d — Token in a non-battlefield zone ceases to exist
        // Tokens cease to exist — they are removed from the game entirely.
        // This is NOT a zone change (no death trigger, no ZoneChange event).
        let tokens_to_remove: Vec<(ObjectId, Zone)> = self.objects.iter()
            .filter(|(_, obj)| obj.is_token && obj.zone != Zone::Battlefield)
            .map(|(&id, obj)| (id, obj.zone))
            .collect();

        for (id, zone) in tokens_to_remove {
            // Remove from zone collection (reuse the centralized helper;
            // stack_entries cleanup is handled internally)
            self.remove_from_zone_collection(id, zone)?;
            // Remove from central object store
            self.objects.remove(&id);
            self.events.emit(GameEvent::TokenCeasedToExist { object_id: id });
            any_performed = true;
        }

        // CR 704.3 — one check, one event. This used to be emitted once per
        // performed action (and not at all for the two creature-death sweeps),
        // which announced a simultaneity the rule denies.
        if any_performed {
            self.events.emit(GameEvent::StateBasedActionPerformed);
        }

        Ok(any_performed)
    }

    /// Repeatedly check SBAs until none are performed (rule 704.3)
    pub fn check_state_based_actions_loop(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        loop {
            if !self.check_state_based_actions(decisions)? {
                break;
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::engine::actions::ZoneChangeCause;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::colors::Color;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::ScriptedDecisionProvider;

    #[test]
    fn test_sba_lethal_damage_destroys_creature() {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Green], 1))
            .color(Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let bears_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(bears_id, 0).damage_marked = 2; // lethal for a 2/2

        // SBA should destroy the creature
        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&bears_id));
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(game.get_object(bears_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_deathtouch_damage_destroys_creature() {
        let mut game = GameState::new(2, 20);

        // 4/5 creature with 1 damage from deathtouch source
        let data = CardDataBuilder::new("Earth Elemental")
            .card_type(CardType::Creature)
            .power_toughness(4, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let bf = game.place_on_battlefield(id, 0);
        bf.damage_marked = 1; // only 1 damage
        bf.damaged_by_deathtouch = true; // but from deathtouch

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&id));
        assert_eq!(game.get_object(id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_deathtouch_zero_damage_no_destroy() {
        let mut game = GameState::new(2, 20);

        // Creature with deathtouch flag but 0 damage (shouldn't happen normally,
        // but verify the guard)
        let data = CardDataBuilder::new("Earth Elemental")
            .card_type(CardType::Creature)
            .power_toughness(4, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let bf = game.place_on_battlefield(id, 0);
        bf.damage_marked = 0;
        bf.damaged_by_deathtouch = true;

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id));
    }

    #[test]
    fn test_sba_counter_annihilation() {
        // Permanent with 3 +1/+1 and 2 -1/-1 → ends with 1 +1/+1 and 0 -1/-1
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);
        game.add_counters(id, crate::types::effects::CounterType::PlusOnePlusOne, 3);
        game.add_counters(id, crate::types::effects::CounterType::MinusOneMinusOne, 2);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        let entry = game.battlefield.get(&id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::PlusOnePlusOne), 1);
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::MinusOneMinusOne), 0);
    }

    #[test]
    fn test_sba_counter_annihilation_equal() {
        // Equal counts → both zeroed
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);
        game.add_counters(id, crate::types::effects::CounterType::PlusOnePlusOne, 4);
        game.add_counters(id, crate::types::effects::CounterType::MinusOneMinusOne, 4);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        let entry = game.battlefield.get(&id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::PlusOnePlusOne), 0);
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::MinusOneMinusOne), 0);
    }

    #[test]
    fn test_sba_token_ceases_to_exist_in_graveyard() {
        // Token in graveyard is removed from the game entirely
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Goblin Token")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let mut obj = GameObject::new(data, 0, Zone::Graveyard);
        obj.is_token = true;
        let id = obj.id;
        game.add_object(obj);
        game.players[0].graveyard.push(id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        // Token should be completely removed from the game
        assert!(game.objects.get(&id).is_none());
        assert!(!game.players[0].graveyard.contains(&id));

        // Should have emitted TokenCeasedToExist event
        let has_event = game.events.events().any(|e| {
            matches!(e, crate::events::event::GameEvent::TokenCeasedToExist { object_id } if *object_id == id)
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_token_on_battlefield_stays() {
        // Token on battlefield should NOT be removed
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Goblin Token")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let mut obj = GameObject::new(data, 0, Zone::Battlefield);
        obj.is_token = true;
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);

        // Token should still exist
        assert!(game.objects.get(&id).is_some());
        assert!(game.battlefield.contains_key(&id));
    }

    #[test]
    fn test_sba_no_action_when_healthy() {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let bears_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(bears_id, 0);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&bears_id));
    }

    // -----------------------------------------------------------------------
    // CR 704.3 — one check, one event (RA-3 ticket 9)
    // -----------------------------------------------------------------------

    /// Two legendary creatures with the same name, both controlled by player 0.
    fn two_isamarus(
        game: &mut GameState,
    ) -> (crate::types::ids::ObjectId, crate::types::ids::ObjectId) {
        let place = || {
            let data = CardDataBuilder::new("Isamaru, Hound of Konda")
                .card_type(CardType::Creature)
                .supertype(Supertype::Legendary)
                .power_toughness(2, 2)
                .build();
            let obj = GameObject::new(data, 0, Zone::Battlefield);
            let id = obj.id;
            (obj, id)
        };
        let (o1, id1) = place();
        game.add_object(o1);
        game.place_on_battlefield(id1, 0);
        let (o2, id2) = place();
        game.add_object(o2);
        game.place_on_battlefield(id2, 0);
        (id1, id2)
    }

    /// The `(object, cause, batch)` of every zone change in the log, in order.
    fn zone_changes(
        game: &GameState,
    ) -> Vec<(
        crate::types::ids::ObjectId,
        ZoneChangeCause,
        Option<crate::events::event::BatchId>,
    )> {
        game.events
            .records()
            .iter()
            .filter_map(|r| match &r.event {
                crate::events::event::GameEvent::ZoneChange { object_id, cause, .. } => {
                    Some((*object_id, *cause, r.batch()))
                }
                _ => None,
            })
            .collect()
    }

    // COVERS: ATOM-704.3-001
    #[test]
    fn test_lethal_damage_deaths_are_one_event() {
        // The atom's board: a 2/2 with 2 damage and a 1/1 with 1 damage.
        let mut game = GameState::new(2, 20);
        let big = crate::test_support::place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        let small = crate::test_support::place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        game.battlefield.get_mut(&big).unwrap().damage_marked = 2;
        game.battlefield.get_mut(&small).unwrap().damage_marked = 1;

        let dp = ScriptedDecisionProvider::new();
        assert!(game.check_state_based_actions(&dp).unwrap());

        // CR 704.3: "performs all applicable state-based actions simultaneously
        // as a single event." One batch id across both deaths is what makes
        // "whenever one or more creatures die" fire once rather than twice.
        let moves = zone_changes(&game);
        assert_eq!(moves.len(), 2, "both creatures died");
        let batch = moves[0].2.expect("a state-based action is performed in a batch");
        assert_eq!(moves[1].2, Some(batch), "one check, one event");

        // "The check repeats — no more SBAs found, so priority is granted."
        assert!(!game.check_state_based_actions(&dp).unwrap());
    }

    #[test]
    fn test_a_dying_legend_is_still_a_legend_when_the_rule_is_checked() {
        // CR 704.3 reads every condition against the same game state, so a
        // creature dead to lethal damage has not left the battlefield yet when
        // 704.5j asks who controls two legends with one name. Sequencing the
        // sweeps — perform 704.5g, then look — skips the prompt entirely and
        // silently lets the survivor live.
        let mut game = GameState::new(2, 20);
        let (doomed, healthy) = two_isamarus(&mut game);
        game.battlefield.get_mut(&doomed).unwrap().damage_marked = 2;

        // The player is asked, and keeps the one that is about to die anyway.
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(
            ChoiceKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
            vec![0],
        );
        let keep_first = game.battlefield_ids_ordered()[0];
        assert_eq!(keep_first, doomed, "fixture assumes the damaged one sorts first");

        assert!(game.check_state_based_actions(&dp).unwrap());

        assert!(!game.battlefield.contains_key(&doomed), "704.5g destroyed it");
        assert!(
            !game.battlefield.contains_key(&healthy),
            "704.5j applied: keeping the damaged one puts the healthy one away",
        );
        // Which rule claimed which permanent, read off the zone change itself.
        assert_eq!(
            zone_changes(&game).into_iter().map(|(id, c, _)| (id, c)).collect::<Vec<_>>(),
            vec![
                (doomed, ZoneChangeCause::DestroyedBySba),
                (healthy, ZoneChangeCause::LegendRule),
            ],
        );
    }

    #[test]
    fn test_two_state_based_actions_with_the_same_result_are_one_event() {
        // CR 704.7 — "if multiple state-based actions would have the same result
        // at the same time, a single replacement effect will replace all of
        // them." Same result means one event: the doomed legend is gathered by
        // both 704.5g and 704.5j and must move once, under the cause that came
        // first in CR order.
        let mut game = GameState::new(2, 20);
        let (doomed, healthy) = two_isamarus(&mut game);
        game.battlefield.get_mut(&doomed).unwrap().damage_marked = 2;

        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(
            ChoiceKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
            vec![1],
        );
        assert_eq!(game.battlefield_ids_ordered()[1], healthy, "fixture assumes this order");

        assert!(game.check_state_based_actions(&dp).unwrap());

        let moves = zone_changes(&game);
        assert_eq!(
            moves.iter().filter(|(id, _, _)| *id == doomed).count(),
            1,
            "one permanent, one zone change — not one per rule that wanted it",
        );
        assert_eq!(
            moves[0].1,
            ZoneChangeCause::DestroyedBySba,
            "704.5g comes first in CR order, so it is what claims the permanent",
        );
        let deaths = game.events.events().filter(|e| matches!(
            e, crate::events::event::GameEvent::CreatureDied { creature_id, .. }
                if *creature_id == doomed
        )).count();
        let legend_sacs = game.events.events().filter(|e| matches!(
            e, crate::events::event::GameEvent::LegendRuleSacrificed { object_id, .. }
                if *object_id == doomed
        )).count();
        assert_eq!((deaths, legend_sacs), (1, 0), "one event, announced once");
    }

    #[test]
    fn test_one_check_announces_one_state_based_action_performed() {
        let mut game = GameState::new(2, 20);
        let a = crate::test_support::place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        let b = crate::test_support::place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        game.battlefield.get_mut(&a).unwrap().damage_marked = 2;
        game.battlefield.get_mut(&b).unwrap().damage_marked = 2;

        let dp = ScriptedDecisionProvider::new();
        assert!(game.check_state_based_actions(&dp).unwrap());

        let announced = game.events.events()
            .filter(|e| matches!(e, crate::events::event::GameEvent::StateBasedActionPerformed))
            .count();
        assert_eq!(announced, 1, "CR 704.3 performs one event per check");
    }

    // -----------------------------------------------------------------------
    // T14: Legend rule tests (704.5j)
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_legend_rule_two_same_name() {
        // Two legendary permanents with the same name controlled by the same player.
        // SBA should remove one (the default keeps the first).
        let mut game = GameState::new(2, 20);

        let legend1_data = CardDataBuilder::new("Isamaru, Hound of Konda")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 2)
            .build();
        let legend2_data = CardDataBuilder::new("Isamaru, Hound of Konda")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 2)
            .build();

        let obj1 = GameObject::new(legend1_data, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0);

        let obj2 = GameObject::new(legend2_data, 0, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 0);

        // Both on the battlefield
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));

        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::LegendRule {
            legend_name: "Isamaru, Hound of Konda".to_string(),
        }, vec![0]);
        let performed = game.check_state_based_actions(&dp).unwrap();
        assert!(performed);

        // Exactly one should remain, one should be in graveyard
        let on_bf = game.battlefield.contains_key(&id1) as usize
            + game.battlefield.contains_key(&id2) as usize;
        assert_eq!(on_bf, 1);
        assert_eq!(game.players[0].graveyard.len(), 1);
    }

    #[test]
    fn test_sba_legend_rule_different_names_ok() {
        // Two legendary permanents with DIFFERENT names — no SBA.
        let mut game = GameState::new(2, 20);

        let legend1 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();
        let legend2 = CardDataBuilder::new("Isamaru, Hound of Konda")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();

        let obj1 = GameObject::new(legend1, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0);

        let obj2 = GameObject::new(legend2, 0, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 0);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));
    }

    #[test]
    fn test_sba_legend_rule_different_controllers_ok() {
        // Two legendary permanents with the SAME name but different controllers — no SBA.
        let mut game = GameState::new(2, 20);

        let data1 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();
        let data2 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();

        let obj1 = GameObject::new(data1, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0); // controller = player 0

        let obj2 = GameObject::new(data2, 1, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 1); // controller = player 1

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));
    }

    // -----------------------------------------------------------------------
    // T14: Planeswalker loyalty tests (704.5i)
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_planeswalker_zero_loyalty_dies() {
        // A planeswalker with 0 loyalty counters should be put into graveyard by SBA.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Jace, the Mind Sculptor")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        // Verify ETB set loyalty counters
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            3
        );

        // Remove all loyalty counters to simulate damage
        game.battlefield.get_mut(&pw_id).unwrap()
            .remove_counters(crate::types::effects::CounterType::Loyalty, 3);
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            0
        );

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&pw_id));
        assert_eq!(game.get_object(pw_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_planeswalker_with_loyalty_stays() {
        // A planeswalker with positive loyalty should NOT be affected by SBA.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Jace, the Mind Sculptor")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&pw_id));
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            3
        );
    }

    #[test]
    fn test_planeswalker_etb_sets_loyalty_counters() {
        // When a planeswalker enters the battlefield, it should get loyalty counters
        // equal to its printed loyalty (rule 306.5b / ATOM-209.1-001).
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Liliana of the Veil")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        let entry = game.battlefield.get(&pw_id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::Loyalty), 3);
    }

    #[test]
    fn test_planeswalker_zero_printed_loyalty_dies_immediately() {
        // A planeswalker with 0 printed loyalty enters with 0 loyalty counters.
        // The SBA should immediately put it into the graveyard.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Tibalt, the Zero")
            .card_type(CardType::Planeswalker)
            .loyalty(0)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        // Should have 0 loyalty counters (loyalty(0) → guard skips adding)
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            0
        );

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&pw_id));
        assert_eq!(game.get_object(pw_id).unwrap().zone, Zone::Graveyard);
    }

    // -----------------------------------------------------------------------
    // T15: Aura/Equipment legality SBAs (704.5m, 704.5n, 704.5p)
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_unattached_aura_dies() {
        // 704.5m — An Aura on the battlefield not attached to anything goes to graveyard.
        let mut game = GameState::new(2, 20);

        let aura_data = CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .build();
        let obj = GameObject::new(aura_data, 0, Zone::Battlefield);
        let aura_id = obj.id;
        game.add_object(obj);
        // Place on battlefield with no attached_to (simulates losing its host)
        game.place_on_battlefield(aura_id, 0);
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, None);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&aura_id));
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Graveyard);

        // Verify AuraDied event was emitted
        let has_event = game.events.events().any(|e| {
            matches!(e, crate::events::event::GameEvent::AuraDied { object_id, .. } if *object_id == aura_id)
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_aura_host_left_battlefield() {
        // 704.5n — Aura attached to an object no longer on the battlefield goes to graveyard.
        let mut game = GameState::new(2, 20);

        // Create a host creature
        let host_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let host_obj = GameObject::new(host_data, 0, Zone::Battlefield);
        let host_id = host_obj.id;
        game.add_object(host_obj);
        game.place_on_battlefield(host_id, 0);

        // Create an aura attached to the host
        let aura_data = CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .build();
        let aura_obj = GameObject::new(aura_data, 0, Zone::Battlefield);
        let aura_id = aura_obj.id;
        game.add_object(aura_obj);
        game.place_on_battlefield(aura_id, 0);

        // Wire up attachment
        game.battlefield.get_mut(&aura_id).unwrap().attach_to(host_id);
        game.battlefield.get_mut(&host_id).unwrap().attached_by.push(aura_id);

        // Verify setup
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, Some(host_id));

        // No SBA triggered yet — aura is legally attached
        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);

        // Now remove the host (simulating destruction)
        game.move_object(host_id, Zone::Graveyard).unwrap();
        assert!(!game.battlefield.contains_key(&host_id));

        // cleanup_zone_state should have cleared the aura's attached_to
        // since the host left. But 704.5m catches unattached auras anyway.
        // Either way, the SBA should put the aura in the graveyard.
        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&aura_id));
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_equipment_on_noncreature_unattaches() {
        // 704.5p — Equipment attached to a non-creature unattaches but stays on battlefield.
        let mut game = GameState::new(2, 20);

        // Create a non-creature permanent (a land)
        let land_data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let land_obj = GameObject::new(land_data, 0, Zone::Battlefield);
        let land_id = land_obj.id;
        game.add_object(land_obj);
        game.place_on_battlefield(land_id, 0);

        // Create an equipment
        let equip_data = CardDataBuilder::new("Bonesplitter")
            .card_type(CardType::Artifact)
            .subtype(Subtype::Artifact(ArtifactType::Equipment))
            .build();
        let equip_obj = GameObject::new(equip_data, 0, Zone::Battlefield);
        let equip_id = equip_obj.id;
        game.add_object(equip_obj);
        game.place_on_battlefield(equip_id, 0);

        // Illegally attach equipment to the land
        game.battlefield.get_mut(&equip_id).unwrap().attach_to(land_id);
        game.battlefield.get_mut(&land_id).unwrap().attached_by.push(equip_id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        // Equipment should be unattached but still on the battlefield
        assert!(game.battlefield.contains_key(&equip_id));
        assert_eq!(game.battlefield.get(&equip_id).unwrap().attached_to, None);

        // Land should no longer have equipment in attached_by
        assert!(game.battlefield.get(&land_id).unwrap().attached_by.is_empty());

        // Verify EquipmentDetached event
        let has_event = game.events.events().any(|e| {
            matches!(e, crate::events::event::GameEvent::EquipmentDetached { equipment_id, former_host }
                if *equipment_id == equip_id && *former_host == land_id)
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_equipment_on_creature_stays() {
        // Equipment legally attached to a creature — no SBA.
        let mut game = GameState::new(2, 20);

        let creature_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let creature_obj = GameObject::new(creature_data, 0, Zone::Battlefield);
        let creature_id = creature_obj.id;
        game.add_object(creature_obj);
        game.place_on_battlefield(creature_id, 0);

        let equip_data = CardDataBuilder::new("Bonesplitter")
            .card_type(CardType::Artifact)
            .subtype(Subtype::Artifact(ArtifactType::Equipment))
            .build();
        let equip_obj = GameObject::new(equip_data, 0, Zone::Battlefield);
        let equip_id = equip_obj.id;
        game.add_object(equip_obj);
        game.place_on_battlefield(equip_id, 0);

        // Legally attach
        game.battlefield.get_mut(&equip_id).unwrap().attach_to(creature_id);
        game.battlefield.get_mut(&creature_id).unwrap().attached_by.push(equip_id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&equip_id));
        assert_eq!(game.battlefield.get(&equip_id).unwrap().attached_to, Some(creature_id));
    }

    #[test]
    fn test_sba_illegal_attachment_catchall() {
        // 704.5q catch-all — A permanent that is not an Aura, Equipment, or
        // Fortification but is somehow attached to another permanent gets unattached.
        let mut game = GameState::new(2, 20);

        let host_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let host_obj = GameObject::new(host_data, 0, Zone::Battlefield);
        let host_id = host_obj.id;
        game.add_object(host_obj);
        game.place_on_battlefield(host_id, 0);

        // A plain creature illegally attached to the host
        let att_data = CardDataBuilder::new("Hill Giant")
            .card_type(CardType::Creature)
            .power_toughness(3, 3)
            .build();
        let att_obj = GameObject::new(att_data, 0, Zone::Battlefield);
        let att_id = att_obj.id;
        game.add_object(att_obj);
        game.place_on_battlefield(att_id, 0);

        // Wire up illegal attachment
        game.battlefield.get_mut(&att_id).unwrap().attach_to(host_id);
        game.battlefield.get_mut(&host_id).unwrap().attached_by.push(att_id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        // Attachment should be broken, both permanents stay on battlefield
        assert!(game.battlefield.contains_key(&att_id));
        assert!(game.battlefield.contains_key(&host_id));
        assert_eq!(game.battlefield.get(&att_id).unwrap().attached_to, None);
        assert!(game.battlefield.get(&host_id).unwrap().attached_by.is_empty());
    }

    #[test]
    fn test_sba_aura_attached_stays() {
        // An Aura properly attached to a permanent — no SBA.
        let mut game = GameState::new(2, 20);

        let host_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let host_obj = GameObject::new(host_data, 0, Zone::Battlefield);
        let host_id = host_obj.id;
        game.add_object(host_obj);
        game.place_on_battlefield(host_id, 0);

        let aura_data = CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .build();
        let aura_obj = GameObject::new(aura_data, 0, Zone::Battlefield);
        let aura_id = aura_obj.id;
        game.add_object(aura_obj);
        game.place_on_battlefield(aura_id, 0);

        // Attach
        game.battlefield.get_mut(&aura_id).unwrap().attach_to(host_id);
        game.battlefield.get_mut(&host_id).unwrap().attached_by.push(aura_id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&aura_id));
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, Some(host_id));
    }

    #[test]
    fn test_aura_etb_no_legal_target_dies() {
        // An Aura enters the battlefield with no legal host (attached_to = None).
        // SBA 704.5m should send it to the graveyard.
        let mut game = GameState::new(2, 20);

        let aura_data = CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .enchant_filter(crate::types::effects::SelectionFilter::Creature)
            .build();
        let aura_obj = GameObject::new(aura_data, 0, Zone::Battlefield);
        let aura_id = aura_obj.id;
        game.add_object(aura_obj);
        game.place_on_battlefield(aura_id, 0);

        // Aura is on the battlefield but not attached to anything
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, None);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&aura_id));
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Graveyard);
    }

    // -----------------------------------------------------------------------
    // T16: Poison, commander damage, indestructible SBA tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_poison_10_loses() {
        // 704.5c — A player with 10 or more poison counters loses the game.
        let mut game = GameState::new(2, 20);
        game.players[0].poison_counters = 10;

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(game.player_lost[0]);
        assert!(!game.player_lost[1]);

        // Verify correct LossReason
        let has_event = game.events.events().any(|e| {
            matches!(e, crate::events::event::GameEvent::PlayerLost {
                player_id: 0,
                reason: crate::events::event::LossReason::PoisonCounters,
            })
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_poison_9_survives() {
        // 704.5c — A player with 9 poison counters does NOT lose.
        let mut game = GameState::new(2, 20);
        game.players[0].poison_counters = 9;

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(!performed);
        assert!(!game.player_lost[0]);
    }

    #[test]
    fn test_sba_commander_damage_21_loses() {
        // Commander variant: 21+ combat damage from a single commander → lose.
        let mut game = GameState::new(2, 20);

        // Create a fake commander object ID
        let commander_id = crate::types::ids::new_object_id();
        game.players[1].commander_damage_taken.insert(commander_id, 21);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);
        assert!(game.player_lost[1]);

        let has_event = game.events.events().any(|e| {
            matches!(e, crate::events::event::GameEvent::PlayerLost {
                player_id: 1,
                reason: crate::events::event::LossReason::CommanderDamage,
            })
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_indestructible_survives_lethal_damage() {
        // 702.12b — Indestructible creatures are NOT destroyed by lethal damage.
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Darksteel Colossus")
            .card_type(CardType::Creature)
            .power_toughness(11, 11)
            .keyword(crate::types::keywords::KeywordFlag::Indestructible)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0).damage_marked = 11; // lethal for an 11/11

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        // No SBA should destroy it
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id));
    }

    #[test]
    fn test_enchant_filter_creature_only() {
        // Aura with SelectionFilter::Creature attached to a non-creature (a land).
        // SBA 704.5n should detect the illegal attachment and send the Aura to the graveyard.
        let mut game = GameState::new(2, 20);

        // Create a land (non-creature) on the battlefield
        let land_data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let land_obj = GameObject::new(land_data, 0, Zone::Battlefield);
        let land_id = land_obj.id;
        game.add_object(land_obj);
        game.place_on_battlefield(land_id, 0);

        // Create an Aura with "Enchant creature" attached to the land
        let aura_data = CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .enchant_filter(crate::types::effects::SelectionFilter::Creature)
            .build();
        let aura_obj = GameObject::new(aura_data, 0, Zone::Battlefield);
        let aura_id = aura_obj.id;
        game.add_object(aura_obj);
        game.place_on_battlefield(aura_id, 0);

        // Wire up illegal attachment (Aura enchanting a land with "Enchant creature")
        game.battlefield.get_mut(&aura_id).unwrap().attach_to(land_id);
        game.battlefield.get_mut(&land_id).unwrap().attached_by.push(aura_id);

        let performed = game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
        assert!(performed);

        // Aura should be in the graveyard
        assert!(!game.battlefield.contains_key(&aura_id));
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Graveyard);

        // Land should still be on the battlefield, with no attachments
        assert!(game.battlefield.contains_key(&land_id));
        assert!(game.battlefield.get(&land_id).unwrap().attached_by.is_empty());
    }
}
