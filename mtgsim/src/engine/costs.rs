use std::collections::HashMap;

use crate::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use crate::types::costs::Cost;
use crate::oracle::characteristics::has_summoning_sickness;
use crate::state::game_state::GameState;
use crate::types::effects::ObjectFilter;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};
use crate::ui::ask::{ask_choose_generic_mana_allocation, ask_choose_sacrifice_for_cost};
use crate::types::zones::Zone;

/// Shared cost payment logic — CR 601.2h and 602.2b.
///
/// All spells and ability types (mana, activated, spell casting) that need to pay costs
/// funnel through this module. This avoids duplicating the cost payment
/// pattern across mana_abilities.rs, activated.rs, etc. *Determining* a
/// spell's total cost (CR 601.2f) is `engine::cost_determination`'s.
///
/// **Deciding is separated from performing.** [`GameState::plan_payment`]
/// takes every choice CR 601.2h's payment needs — the generic mana split, and
/// which permanents pay a `Cost::Sacrifice` — against the board as it stands
/// before anything is paid; [`GameState::pay_costs`] then performs the plan
/// and asks nobody anything. **No payment prompt is asked after a payment has
/// been performed**, which is what lets a client stage a payment and let the
/// player take it back until they confirm: the engine never holds a
/// half-performed one. It is also the shape the replacement pipeline already
/// uses for CR 704.3 (`replacement-architecture.md` §4.1).

/// The decisions CR 601.2h's payment needs, taken against one board.
///
/// **Not a CR object.** The rules have no "payment plan"; this is the engine's
/// record of choices that 601.2h leaves to the player — the generic mana split
/// and which permanents pay each `Cost::Sacrifice` — separated out so that
/// deciding and performing do not interleave (see the module doc).
#[derive(Debug, Clone)]
pub struct PaymentPlan {
    /// The costs in the order they will be paid ([`payment_order_rank`]).
    ordered: Vec<Cost>,
    /// How the mana component's generic part is split across the pool.
    generic_allocation: HashMap<ManaType, u64>,
    /// Which permanents pay each `Cost::Sacrifice`, keyed by index into
    /// `ordered`.
    sacrifices: HashMap<usize, Vec<ObjectId>>,
}

impl PaymentPlan {
    /// The costs in the order they will be paid. For tests and for a UI that
    /// wants to show the player what it is about to spend.
    pub fn ordered_costs(&self) -> &[Cost] {
        &self.ordered
    }

    /// The permanents this plan will sacrifice, in payment order.
    pub fn planned_sacrifices(&self) -> Vec<ObjectId> {
        let mut keys: Vec<&usize> = self.sacrifices.keys().collect();
        keys.sort();
        keys.into_iter().flat_map(|k| self.sacrifices[k].iter().copied()).collect()
    }
}

/// CR 601.2h's payment order, as ranks. See [`payment_order_rank`].
const RANK_MANA: u8 = 0;
const RANK_MUTATES: u8 = 1;
const RANK_MOVES_AN_OBJECT: u8 = 2;

/// Where a cost sits in the order the engine pays a total cost.
///
/// **CR 601.2h gives the order to the player** — "they pay all costs that
/// don't involve random elements or moving objects from the library to a
/// public zone, in any order" — and the engine picks one of them. (That
/// sentence's own two groups are not modelled: no `Cost` arm is random and
/// none moves a card out of a library, so the second group is empty for every
/// cost that exists, and an arm nothing can reach is worse than a missing
/// one. `ATOM-601.2h-003` is the atom for handing the order to the player.)
///
/// The engine picks **the order in which no payment can fail after one that
/// cannot be taken back**, which is what makes CR 732.1's "any payments
/// already made are canceled" unreachable rather than unimplemented:
///
/// - rank 0, `Mana` — the only cost whose payment can fail on a choice the
///   player made (a generic split that spends a color a pip still needs), so
///   it is paid while nothing else has been. `cost_determination::total`
///   already merges every mana entry into one component ahead of this
///   (`cost-architecture.md` §3.3); this extends the same argument to an
///   ability's cost list, which is its *printed* one and never merged.
/// - rank 1 — mutates, moves nothing: reads the source's tapped state, its
///   summoning sickness or the player's life, none of which a rank-0 payment
///   changes.
/// - rank 2 — moves an object. Its own payment cannot fail, because
///   `plan_payment` enumerated the candidates and `validate_pick_n` bounds
///   the answer; and nothing after it can fail, because nothing after it
///   exists that a removal could invalidate.
///
/// **This is 601.2h and 602.2b only.** A resolving spell's instructions are
/// CR 608.2c's — "in the order written" — and reach `resolve_effect`, never
/// this function; a resolution-time payment routed through `pay_costs` would
/// be silently reordered by it. 601.2h's *own* two groups are not modelled
/// because the second is empty for every `Cost` arm; `codebase-state.md`
/// item 80 owns the day it stops being.
///
/// Matched exhaustively so a new arm has to decide where it sits.
fn payment_order_rank(cost: &Cost) -> u8 {
    match cost {
        Cost::Mana(_) => RANK_MANA,
        Cost::Tap
        | Cost::Untap
        | Cost::PayLife(_)
        | Cost::RemoveCounters(_, _)
        | Cost::AddCounters(_, _) => RANK_MUTATES,
        Cost::Sacrifice(_, _)
        | Cost::SacrificeSelf
        | Cost::Discard(_, _)
        | Cost::ExileFromGraveyard(_, _) => RANK_MOVES_AN_OBJECT,
    }
}

/// `costs` in [`payment_order_rank`] order. Stable, so costs of one rank keep
/// the order the card printed them in.
fn ordered_for_payment(costs: &[Cost]) -> Vec<&Cost> {
    let mut ordered: Vec<&Cost> = costs.iter().collect();
    ordered.sort_by_key(|c| payment_order_rank(c));
    ordered
}

impl GameState {
    /// Read-only check: can all costs be paid right now?
    ///
    /// Checks both resource availability AND cost restrictions.
    /// Cost restrictions (Phase 5) start as a no-op — the `check_cost_restrictions`
    /// call is a placeholder for when continuous effects populate
    /// `GameState::cost_restrictions`.
    pub fn can_pay_costs(
        &self,
        costs: &[Cost],
        player_id: PlayerId,
        source_id: ObjectId,
    ) -> Result<(), String> {
        for cost in ordered_for_payment(costs) {
            self.check_cost_resource(cost, player_id, source_id)?;
            // Phase 5: self.check_cost_restrictions(cost, player_id, source_id)?;
        }
        Ok(())
    }

    /// Resource check: does the player have the resources to pay this cost?
    fn check_cost_resource(
        &self,
        cost: &Cost,
        player_id: PlayerId,
        source_id: ObjectId,
    ) -> Result<(), String> {
        match cost {
            Cost::Tap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if entry.tapped {
                    return Err("Permanent is already tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // tapping, unless they have haste. `has_summoning_sickness` is
                // false for a noncreature permanent, so it needs no type gate here.
                if has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                Ok(())
            }
            Cost::Untap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if !entry.tapped {
                    return Err("Permanent is not tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // paying {Q} (untap symbol), unless they have haste.
                if has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                Ok(())
            }
            Cost::Mana(mana_cost) => {
                let player = self.get_player(player_id)?;
                if !player.mana_pool.can_pay(mana_cost) {
                    return Err("Not enough mana".to_string());
                }
                Ok(())
            }
            Cost::PayLife(amount) => {
                let player = self.get_player(player_id)?;
                if player.life_total < *amount as i64 {
                    return Err(format!(
                        "Cannot pay {} life, only {} available",
                        amount, player.life_total
                    ));
                }
                Ok(())
            }
            Cost::SacrificeSelf => {
                if !self.battlefield.contains_key(&source_id) {
                    return Err(format!("Permanent {} not on battlefield", source_id));
                }
                Ok(())
            }
            Cost::Sacrifice(filter, n) => {
                let available = self.sacrifice_candidates(filter, player_id).len();
                if available < *n as usize {
                    return Err(format!(
                        "Cannot sacrifice {} permanent(s): only {} match",
                        n, available
                    ));
                }
                Ok(())
            }
            Cost::Discard(_, _)
            | Cost::ExileFromGraveyard(_, _)
            | Cost::RemoveCounters(_, _)
            | Cost::AddCounters(_, _) => {
                Err(format!("Cost {:?} validation not yet implemented", cost))
            }
        }
    }

    /// Every permanent the player may sacrifice to pay a cost with `filter`
    /// (CR 701.21a), in a process-independent order.
    ///
    /// **"You control" is the rule's, not the filter's.** CR 701.21a — "a
    /// player can't sacrifice ... something that's a permanent they don't
    /// control" — holds whatever the card prints, so Altar's Reap's filter is
    /// `ByType(Creature)` and this supplies the rest. The source of the spell
    /// or ability is not excluded: Krark-Clan Ironworks paying its own
    /// "Sacrifice an artifact" is the filter matching normally.
    fn sacrifice_candidates(&self, filter: &ObjectFilter, player_id: PlayerId) -> Vec<ObjectId> {
        self.battlefield_ids_ordered()
            .into_iter()
            .filter(|&id| crate::oracle::characteristics::controls(self, id, player_id))
            .filter(|&id| self.object_matches_filter(id, filter, player_id).unwrap_or(false))
            .collect()
    }

    /// Take every choice CR 601.2h's payment needs, against one board.
    ///
    /// Reads only — nothing is paid here. `can_pay_costs` must already have
    /// passed; this asks *how* to pay, not *whether*, and the one error it
    /// returns is a cost it cannot plan at all.
    ///
    /// Every prompt a payment asks is asked here, which is the property
    /// [`Self::pay_costs`] leans on: once the first permanent moves, no
    /// decision is outstanding. See the module doc.
    pub fn plan_payment(
        &self,
        costs: &[Cost],
        player_id: PlayerId,
        source_id: ObjectId,
        ctx: &ActionContext,
    ) -> Result<PaymentPlan, String> {
        let ordered: Vec<Cost> = ordered_for_payment(costs).into_iter().cloned().collect();

        // The mana component's generic part, split across the pool. Merged into
        // one component at 601.2f (`cost_determination::total`), so `find` is
        // exact for a spell; an ability's cost list is its printed one and has
        // at most one mana entry.
        let mana_cost = ordered
            .iter()
            .find_map(|c| if let Cost::Mana(mc) = c { Some(mc.clone()) } else { None })
            .unwrap_or_else(ManaCost::zero);
        let generic_allocation = if mana_cost.generic_count() == 0 {
            HashMap::new()
        } else {
            let mut available: Vec<(ManaType, u64)> = self.players[player_id]
                .mana_pool
                .available()
                .iter()
                .filter(|(_, amt)| **amt > 0)
                .map(|(mt, amt)| (*mt, *amt))
                .collect();
            available.sort_by_key(|(mt, _)| *mt as u8);
            ask_choose_generic_mana_allocation(
                ctx.dp, self, player_id, &mana_cost, &available,
                mana_cost.generic_count() as u64,
            )
        };

        let mut sacrifices: HashMap<usize, Vec<ObjectId>> = HashMap::new();
        for (idx, cost) in ordered.iter().enumerate() {
            if let Cost::Sacrifice(filter, count) = cost {
                let candidates = self.sacrifice_candidates(filter, player_id);
                let n = *count as usize;
                if candidates.len() < n {
                    return Err(format!(
                        "Cannot sacrifice {} permanent(s): only {} match",
                        n, candidates.len()
                    ));
                }
                // CR 102.2 / `CLAUDE.md`: as many candidates as the cost takes
                // is not a choice, and prompting for it would be a prompt with
                // one answer.
                let chosen = if candidates.len() == n {
                    candidates
                } else {
                    ask_choose_sacrifice_for_cost(
                        ctx.dp, self, player_id, source_id, *count, &candidates,
                    )
                };
                sacrifices.insert(idx, chosen);
            }
        }

        Ok(PaymentPlan { ordered, generic_allocation, sacrifices })
    }

    /// Pay a planned total cost (CR 601.2h).
    ///
    /// Performs `plan.ordered` in order and asks nothing. If a cost can't be
    /// paid this returns an error and **costs already paid are not rolled
    /// back**: CR 732.1 would cancel them, and the engine does not build that
    /// cancellation because [`payment_order_rank`] makes it unreachable — see
    /// the debug assertion below, which is where that claim is enforced.
    pub fn pay_costs(
        &mut self,
        plan: &PaymentPlan,
        player_id: PlayerId,
        source_id: ObjectId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let mut moved_an_object = false;
        for (idx, cost) in plan.ordered.iter().enumerate() {
            let result = self.pay_single_cost(
                cost, player_id, source_id, plan, idx, ctx,
            );
            if let Err(e) = result {
                // CR 732.1: "the entire action is reversed and any payments
                // already made are canceled." The engine builds no such
                // cancellation, and this is the assertion that says it never
                // needs one: nothing that can fail is paid after something
                // that cannot be taken back. A new `Cost` arm that trips this
                // has to pick a different `payment_order_rank`, or the rule
                // it needs is a rollback facility rather than an arm.
                debug_assert!(
                    !moved_an_object,
                    "CR 732.1: {:?} failed after an irreversible payment ({})",
                    cost, e,
                );
                return Err(e);
            }
            if payment_order_rank(cost) == RANK_MOVES_AN_OBJECT {
                moved_an_object = true;
            }
        }
        Ok(())
    }

    /// Pay a single cost from the plan. Internal helper.
    ///
    /// `idx` is this cost's position in `plan.ordered`, which is how a
    /// `Cost::Sacrifice` finds the permanents `plan_payment` chose for it.
    fn pay_single_cost(
        &mut self,
        cost: &Cost,
        player_id: PlayerId,
        source_id: ObjectId,
        plan: &PaymentPlan,
        idx: usize,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        match cost {
            Cost::Tap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if entry.tapped {
                    return Err("Permanent is already tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // tapping, unless they have haste.
                if has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                // Through the chokepoint: CR 603.2e "becomes tapped" watchers
                // and CR 122.1d stun counters both act on this.
                self.execute_action(GameAction::Tap { object: source_id }, ctx)
            }
            Cost::Untap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if !entry.tapped {
                    return Err("Permanent is not tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // paying {Q} (untap symbol), unless they have haste.
                if has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                // {Q} — same chokepoint as Cost::Tap. CR 122.1d makes Untap
                // replaceable, which is why this cannot stay a direct write.
                self.execute_action(GameAction::Untap { object: source_id }, ctx)
            }
            Cost::Mana(mana_cost) => {
                let player = self.get_player_mut(player_id)?;
                if mana_cost.generic_count() == 0 {
                    player.mana_pool.pay_specific_only(mana_cost)
                } else {
                    player.mana_pool.pay(mana_cost, &plan.generic_allocation)
                }
            }
            Cost::PayLife(amount) => {
                // CR 119.4 gates the payment on the player's *current* life
                // total, before any replacement gets to touch the loss.
                let player = self.get_player(player_id)?;
                if player.life_total < *amount as i64 {
                    return Err(format!(
                        "Cannot pay {} life, only {} available",
                        amount, player.life_total
                    ));
                }
                // "the player loses that much life" — CR 119.4's second
                // sentence, which is why this is a proposal and not a
                // subtraction. Bloodletter of Aclazotz doubles paid life
                // precisely because it is a loss.
                self.execute_action(
                    GameAction::LoseLife { player: player_id, amount: *amount },
                    ctx,
                )
            }
            Cost::SacrificeSelf => {
                self.change_zone(source_id, crate::types::zones::Zone::Graveyard, ZoneChangeCause::Sacrificed, ctx)
            }
            Cost::Sacrifice(_, n) => {
                let chosen = plan.sacrifices.get(&idx).ok_or_else(|| {
                    format!("No sacrifice planned for cost {} ({:?})", idx, cost)
                })?;
                if chosen.len() != *n as usize {
                    return Err(format!(
                        "Planned {} sacrifice(s) for a cost of {}",
                        chosen.len(), n
                    ));
                }
                // One batch, not a loop: the permanents paying one cost leave
                // the battlefield together, so a "whenever one or more
                // creatures die" trigger sees one event. `execute_actions` is
                // the only thing that can say that (`CLAUDE.md`).
                let batch: Vec<GameAction> = chosen
                    .iter()
                    .map(|&object| GameAction::ZoneChange {
                        object,
                        from: Zone::Battlefield,
                        to: Zone::Graveyard,
                        cause: ZoneChangeCause::Sacrificed,
                    })
                    .collect();
                self.execute_actions(batch, ctx).map(|_| ())
            }
            Cost::Discard(_, _)
            | Cost::ExileFromGraveyard(_, _)
            | Cost::RemoveCounters(_, _)
            | Cost::AddCounters(_, _) => {
                Err(format!("Cost {:?} payment not yet implemented", cost))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_ctx;
    use crate::objects::card_data::CardDataBuilder;
    use crate::types::costs::Cost;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    /// Plan and pay in one step, for a board where the plan asks nothing.
    ///
    /// Production code never gets to collapse these two — a plan is taken
    /// against one board and performed against it — but a test whose costs
    /// hold no choice has nothing to script between them.
    fn plan_and_pay(
        game: &mut GameState,
        costs: &[Cost],
        player: crate::types::ids::PlayerId,
        source: crate::types::ids::ObjectId,
        ctx: &crate::engine::actions::ActionContext,
    ) -> Result<(), String> {
        let plan = game.plan_payment(costs, player, source, ctx)?;
        game.pay_costs(&plan, player, source, ctx)
    }

    fn setup_with_forest() -> (GameState, crate::types::ids::ObjectId) {
        let mut game = GameState::new(2, 20);
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0, 0);
        game.battlefield.insert(id, entry);
        (game, id)
    }

    #[test]
    fn test_pay_tap_cost() {
        let (mut game, forest_id) = setup_with_forest();
        plan_and_pay(&mut game, &[Cost::Tap], 0, forest_id, &test_ctx()).unwrap();
        assert!(game.battlefield.get(&forest_id).unwrap().tapped);
    }

    #[test]
    fn test_pay_tap_cost_already_tapped() {
        let (mut game, forest_id) = setup_with_forest();
        plan_and_pay(&mut game, &[Cost::Tap], 0, forest_id, &test_ctx()).unwrap();
        assert!(plan_and_pay(&mut game, &[Cost::Tap], 0, forest_id, &test_ctx()).is_err());
    }

    #[test]
    fn test_pay_mana_cost_specific() {
        let (mut game, _) = setup_with_forest();
        game.players[0].mana_pool.add(ManaType::Green, 2);
        let cost = ManaCost::build(&[ManaType::Green], 0);
        plan_and_pay(&mut game, &[Cost::Mana(cost)], 0, crate::types::ids::new_object_id(), &test_ctx()).unwrap();
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
    }

    #[test]
    fn test_pay_mana_cost_with_generic() {
        let (mut game, _) = setup_with_forest();
        game.players[0].mana_pool.add(ManaType::Green, 2);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        // Cost: {1}{G} — the player chooses to spend Red for the generic, and
        // the choice is the plan's, not the payment's.
        let cost = ManaCost::build(&[ManaType::Green], 1);
        let dp = crate::ui::decision::ScriptedDecisionProvider::new();
        dp.expect_allocation(
            crate::ui::choice_types::ChoiceKind::GenericManaAllocation { mana_cost: cost.clone() },
            // Buckets are the pool's types sorted by discriminant — Red, then
            // Green — so this spends the Red on the generic.
            vec![1, 0],
        );
        let ctx = crate::engine::actions::ActionContext::new(&dp);
        let source = crate::types::ids::new_object_id();
        let costs = [Cost::Mana(cost)];
        let plan = game.plan_payment(&costs, 0, source, &ctx).unwrap();
        game.pay_costs(&plan, 0, source, &ctx).unwrap();
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_pay_life_cost() {
        let (mut game, forest_id) = setup_with_forest();
        plan_and_pay(&mut game, &[Cost::PayLife(3)], 0, forest_id, &test_ctx()).unwrap();
        assert_eq!(game.players[0].life_total, 17);
    }

    #[test]
    fn test_paying_life_is_a_life_loss() {
        let (mut game, forest_id) = setup_with_forest();
        let before = game.events.len();

        plan_and_pay(&mut game, &[Cost::PayLife(3)], 0, forest_id, &test_ctx()).unwrap();

        // CR 119.4: "the player loses that much life". Paying life used to be a
        // silent subtraction — no event at all — so nothing watching life loss
        // could see it. Bloodletter of Aclazotz doubles paid life precisely
        // because it *is* a loss, and it can only do that if this is proposed.
        let changes: Vec<(i64, i64)> = game.events.records_from(before).iter()
            .filter_map(|r| match &r.event {
                crate::events::event::GameEvent::LifeChanged { player_id: 0, old, new, .. }
                    => Some((*old, *new)),
                _ => None,
            })
            .collect();
        assert_eq!(changes, vec![(20, 17)]);
    }

    #[test]
    fn test_pay_life_cost_insufficient() {
        let (mut game, forest_id) = setup_with_forest();
        assert!(plan_and_pay(&mut game, &[Cost::PayLife(21)], 0, forest_id, &test_ctx()).is_err());
    }

    // --- Cost::Untap ({Q}) summoning sickness tests (T10 / E13) ---

    fn setup_creature_on_turn(turn: u32, keywords: Vec<crate::types::keywords::KeywordFlag>) -> (GameState, crate::types::ids::ObjectId) {
        let mut game = GameState::new(2, 20);
        game.begin_turn(turn, 0);
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .power_toughness(2, 2);
        for kw in keywords {
            builder = builder.keyword_flag(kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0, turn);
        // Start tapped so {Q} (untap) is payable resource-wise
        entry.tapped = true;
        game.battlefield.insert(id, entry);
        (game, id)
    }

    #[test]
    fn test_untap_cost_blocked_by_summoning_sickness() {
        // Creature enters on turn 1, game is on turn 1 → summoning sick → can't pay {Q}
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![]);
        let result = plan_and_pay(&mut game, &[Cost::Untap], 0, creature_id, &test_ctx());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("summoning sickness"));
    }

    #[test]
    fn test_untap_cost_allowed_with_haste() {
        // Creature with haste enters on turn 1, game is on turn 1 → haste bypasses sickness
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![crate::types::keywords::KeywordFlag::Haste]);
        plan_and_pay(&mut game, &[Cost::Untap], 0, creature_id, &test_ctx()).unwrap();
        assert!(!game.battlefield.get(&creature_id).unwrap().tapped);
    }

    #[test]
    fn test_untap_cost_allowed_on_noncreature() {
        // Artifact (non-creature) with {Q} cost — no summoning sickness restriction
        let mut game = GameState::new(2, 20);
        game.begin_turn(1, 0);
        let data = CardDataBuilder::new("Test Artifact")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let mut entry = PermanentState::new(id, 0, 0, 1);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        plan_and_pay(&mut game, &[Cost::Untap], 0, id, &test_ctx()).unwrap();
        assert!(!game.battlefield.get(&id).unwrap().tapped);
    }

    #[test]
    fn test_untap_cost_blocked_by_control_change() {
        // Creature entered on turn 1, control changes on turn 3 → sick again on turn 3
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![]);
        // Advance to turn 3 — P0's next turn, so the creature is no longer sick
        game.begin_turn(3, 0);
        // Simulate control change on turn 3
        game.battlefield.get_mut(&creature_id).unwrap().controller_since_turn = 3;
        game.battlefield.get_mut(&creature_id).unwrap().tapped = true;

        let result = plan_and_pay(&mut game, &[Cost::Untap], 0, creature_id, &test_ctx());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("summoning sickness"));
    }

    #[test]
    fn test_untap_cost_allowed_control_change_haste() {
        // Creature with haste, control changes on turn 3 → haste bypasses
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![crate::types::keywords::KeywordFlag::Haste]);
        game.begin_turn(3, 0);
        game.battlefield.get_mut(&creature_id).unwrap().controller_since_turn = 3;
        game.battlefield.get_mut(&creature_id).unwrap().tapped = true;

        plan_and_pay(&mut game, &[Cost::Untap], 0, creature_id, &test_ctx()).unwrap();
        assert!(!game.battlefield.get(&creature_id).unwrap().tapped);
    }

    // --- CR 601.2h payment order, and the CR 732.1 claim it buys (CM-3) ---

    /// A creature `player` controls, on the battlefield.
    fn add_creature(game: &mut GameState, player: crate::types::ids::PlayerId, name: &str)
        -> crate::types::ids::ObjectId
    {
        let data = CardDataBuilder::new(name)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let obj = GameObject::new(data, player, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        game.battlefield.insert(id, PermanentState::new(id, player, ts, 0));
        id
    }

    fn creature_filter() -> crate::types::effects::ObjectFilter {
        crate::types::effects::ObjectFilter::ByType(CardType::Creature)
    }

    #[test]
    fn test_mana_is_paid_first_and_object_moves_last() {
        // CR 601.2h leaves the order to the player and the engine picks one.
        // It picks this one: the only cost that can fail on a player's choice
        // is the mana split, and it is paid while nothing has moved.
        let costs = [
            Cost::Sacrifice(creature_filter(), 1),
            Cost::Tap,
            Cost::Mana(ManaCost::build(&[ManaType::Red], 0)),
        ];
        let ordered: Vec<&Cost> = ordered_for_payment(&costs);
        assert!(matches!(ordered[0], Cost::Mana(_)));
        assert!(matches!(ordered[1], Cost::Tap));
        assert!(matches!(ordered[2], Cost::Sacrifice(_, _)));
    }

    #[test]
    fn test_payment_order_is_stable_within_a_rank() {
        // Two costs of one rank keep the order the card printed them in —
        // the engine reorders only what the theorem needs it to.
        let costs = [Cost::Tap, Cost::PayLife(1), Cost::Untap];
        let ordered = ordered_for_payment(&costs);
        assert!(matches!(ordered[0], Cost::Tap));
        assert!(matches!(ordered[1], Cost::PayLife(_)));
        assert!(matches!(ordered[2], Cost::Untap));
    }

    #[test]
    fn test_every_object_moving_cost_ranks_last() {
        // The gate the 732.1 argument rests on: a cost that takes an object
        // out of its zone is never followed by one that could fail. Listed
        // rather than derived, so adding an arm to `Cost` fails here first.
        for cost in [
            Cost::SacrificeSelf,
            Cost::Sacrifice(creature_filter(), 1),
            Cost::Discard(crate::types::effects::CardFilter::All, 1),
            Cost::ExileFromGraveyard(crate::types::effects::CardFilter::All, 1),
        ] {
            assert_eq!(
                payment_order_rank(&cost), RANK_MOVES_AN_OBJECT,
                "{:?} moves an object and must be paid last", cost,
            );
        }
    }

    #[test]
    fn test_sacrifice_candidates_are_only_permanents_you_control() {
        // CR 701.21a: "a player can't sacrifice ... a permanent they don't
        // control" — a rule, not something the card's filter has to say.
        let mut game = GameState::new(2, 20);
        let mine = add_creature(&mut game, 0, "Mine");
        let theirs = add_creature(&mut game, 1, "Theirs");

        let candidates = game.sacrifice_candidates(&creature_filter(), 0);
        assert_eq!(candidates, vec![mine]);
        assert!(!candidates.contains(&theirs));
    }

    #[test]
    fn test_sacrifice_with_exactly_enough_candidates_asks_nothing() {
        // `CLAUDE.md`: never prompt with fewer than two candidates. One
        // creature for "sacrifice a creature" is a forced payment, and a
        // ScriptedDecisionProvider with an empty queue panics if asked.
        let mut game = GameState::new(2, 20);
        let victim = add_creature(&mut game, 0, "Only Creature");
        let costs = [Cost::Sacrifice(creature_filter(), 1)];

        plan_and_pay(&mut game, &costs, 0, victim, &test_ctx()).unwrap();

        assert!(!game.battlefield.contains_key(&victim));
        assert!(game.players[0].graveyard.contains(&victim));
    }

    #[test]
    fn test_sacrifice_with_a_choice_asks_and_honors_the_pick() {
        let mut game = GameState::new(2, 20);
        let first = add_creature(&mut game, 0, "First");
        let second = add_creature(&mut game, 0, "Second");

        let dp = crate::ui::decision::ScriptedDecisionProvider::new();
        dp.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::ChooseSacrificeForCost {
                spell_or_ability_id: first,
                count: 1,
            },
            vec![1],
        );
        let ctx = crate::engine::actions::ActionContext::new(&dp);
        let costs = [Cost::Sacrifice(creature_filter(), 1)];

        let plan = game.plan_payment(&costs, 0, first, &ctx).unwrap();
        assert_eq!(plan.planned_sacrifices(), vec![second]);
        game.pay_costs(&plan, 0, first, &ctx).unwrap();

        assert!(game.battlefield.contains_key(&first));
        assert!(!game.battlefield.contains_key(&second));
    }

    #[test]
    fn test_the_source_pays_its_own_sacrifice_cost() {
        // Krark-Clan Ironworks' shape at the unit level: the filter matches
        // the permanent whose cost it is, and nothing excludes it.
        let mut game = GameState::new(2, 20);
        let source = add_creature(&mut game, 0, "Self-Eater");
        let costs = [Cost::Sacrifice(creature_filter(), 1)];

        let plan = game.plan_payment(&costs, 0, source, &test_ctx()).unwrap();
        assert_eq!(plan.planned_sacrifices(), vec![source]);
        game.pay_costs(&plan, 0, source, &test_ctx()).unwrap();
        assert!(!game.battlefield.contains_key(&source));
    }

    #[test]
    fn test_two_permanents_for_one_cost_leave_together() {
        // One cost, one event: the permanents paying `Sacrifice(f, 2)` go
        // through a single `execute_actions` batch, so a "whenever one or
        // more creatures die" trigger will see one event and not two.
        let mut game = GameState::new(2, 20);
        let a = add_creature(&mut game, 0, "A");
        let b = add_creature(&mut game, 0, "B");
        let costs = [Cost::Sacrifice(creature_filter(), 2)];
        let before = game.events.len();

        plan_and_pay(&mut game, &costs, 0, a, &test_ctx()).unwrap();

        assert!(!game.battlefield.contains_key(&a));
        assert!(!game.battlefield.contains_key(&b));
        let batches: std::collections::HashSet<_> = game.events.records_from(before)
            .iter()
            .filter(|r| matches!(
                r.event,
                crate::events::event::GameEvent::ZoneChange { .. },
            ))
            .map(|r| r.batch())
            .collect();
        assert_eq!(batches.len(), 1, "two sacrifices for one cost are one event");
    }

    #[test]
    fn test_sacrifice_is_unpayable_without_enough_candidates() {
        let mut game = GameState::new(2, 20);
        let lone = add_creature(&mut game, 0, "Lone");
        let costs = [Cost::Sacrifice(creature_filter(), 2)];

        assert!(game.can_pay_costs(&costs, 0, lone).is_err());
        assert!(game.plan_payment(&costs, 0, lone, &test_ctx()).is_err());
    }
}
