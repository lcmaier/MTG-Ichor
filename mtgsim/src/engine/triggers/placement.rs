//! Placement — the stub, drained (`triggers-architecture.md` §5; CR 603.3).
//!
//! CR 117.5 and 704.3 written out: state-based actions until none, then
//! triggers, then again until nothing is placed. Every caller of
//! `perform_sba_and_triggers` is a CR moment — the top of a priority round,
//! after a resolution, after a cast or a non-mana activation, and the
//! cleanup step's CR 514.3a probe — so CR 502.4's untap-step triggers are
//! held by construction (nobody receives priority there) and go on the stack
//! at the upkeep's first grant (503.1a).

use std::sync::Arc;

use crate::engine::trace_records;
use crate::engine::triggers::bound_reads::BoundReads;
use crate::objects::object::GameObject;
use crate::state::game_state::{GameState, StackEntry};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::{PendingTrigger, TriggerOrigin, TriggerSeq, TriggerTier};
use crate::types::mana::ManaSpent;
use crate::types::zones::Zone;
use crate::ui::ask::ask_order_triggers;
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// CR 603.3b: two tiers, each player in APNAP order, each player's own
    /// triggers in the order they choose. Returns whether anything was
    /// placed — the CR 117.5 loop's step 3.
    ///
    /// **The drain removes each entry as it places it**, so the queue is the
    /// placement's progress record: a `GameState` cloned at the ordering
    /// prompt resumes by running this again (item 40).
    pub fn place_pending_triggers(&mut self, dp: &dyn DecisionProvider) -> Result<bool, String> {
        if self.pending_triggers.is_empty() {
            return Ok(false);
        }

        // CR 800.4d's second sentence — a trigger a departed player would
        // control is not put onto the stack. One read, at the head, before
        // any tier or order is asked.
        let departed: Vec<PendingTrigger> = {
            let (gone, stay): (Vec<_>, Vec<_>) =
                std::mem::take(&mut self.pending_triggers).into_iter().partition(|t| !self.in_game(t.controller));
            self.pending_triggers = stay;
            gone
        };
        for t in &departed {
            self.trace(|| trace_records::pending(self, t, None, Some("departed"), &[]));
        }

        let mut placed = 0usize;
        let seats: Vec<PlayerId> = {
            let mut seats: Vec<PlayerId> = (0..self.num_players()).filter(|&p| self.in_game(p)).collect();
            seats.sort_by_key(|&p| self.apnap_index(p));
            seats
        };
        for tier in [TriggerTier::First, TriggerTier::Second] {
            for &player in &seats {
                let mine: Vec<TriggerSeq> = self
                    .pending_triggers
                    .iter()
                    .filter(|t| t.tier() == tier && t.controller == player)
                    .map(|t| t.seq)
                    .collect();
                if mine.is_empty() {
                    continue;
                }
                let order: Vec<usize> = if mine.len() >= 2 && !self.trigger_order_cannot_change_outcome(&mine) {
                    let sources: Vec<ObjectId> = mine
                        .iter()
                        .filter_map(|seq| self.pending_triggers.iter().find(|t| t.seq == *seq))
                        .map(|t| t.origin.source())
                        .collect();
                    ask_order_triggers(dp, self, player, tier, &sources)
                } else {
                    (0..mine.len()).collect()
                };
                for i in order {
                    let seq = mine[i];
                    let Some(at) = self.pending_triggers.iter().position(|t| t.seq == seq) else { continue };
                    let pending = self.pending_triggers.remove(at);
                    if self.place_one(pending, dp)? {
                        placed += 1;
                    }
                }
            }
        }
        Ok(placed > 0)
    }

    /// Whether one player's entries give the same game in whatever order
    /// they go on the stack, so the ordering prompt is not asked.
    /// `triggers-architecture.md` §5.2 gives the reason for each condition.
    /// Compared by def rather than by id, since two grants of one ability
    /// carry two ids.
    fn trigger_order_cannot_change_outcome(&self, seqs: &[TriggerSeq]) -> bool {
        let entries: Vec<&PendingTrigger> = seqs
            .iter()
            .filter_map(|seq| self.pending_triggers.iter().find(|t| t.seq == *seq))
            .collect();
        let Some(first) = entries.first() else { return true };
        let reads = first.binding.def.bound_reads();
        entries.iter().all(|t| {
            t.tier() == TriggerTier::First
                && t.instances.is_empty()
                && t.binding.def == first.binding.def
                && self.entries_agree_on(reads, first, t)
        })
    }

    /// Whether two entries of one def agree on every fact it reads (item 163).
    fn entries_agree_on(&self, reads: BoundReads, a: &PendingTrigger, b: &PendingTrigger) -> bool {
        let (x, y) = (&a.binding, &b.binding);
        (!reads.subject || x.subject == y.subject)
            && (!reads.player || self.bound_player(x) == self.bound_player(y))
            && (!reads.amount || self.bound_amount(x) == self.bound_amount(y))
            && (!reads.characteristics || (x.subject == y.subject && x.records.first() == y.records.first()))
            && (!reads.source || a.origin == b.origin)
            && (!reads.ability || self.ability_state(a) == self.ability_state(b))
    }

    /// "This ability"'s state: whether its CR 603.2h action is taken this
    /// turn, and how many times it has resolved (CR 603.7h).
    fn ability_state(&self, entry: &PendingTrigger) -> (bool, u32) {
        let TriggerOrigin::Object(identity) = entry.origin;
        (
            self.action_taken_this_turn.contains(&(identity, entry.controller)),
            self.resolutions_this_turn_of(identity),
        )
    }

    /// One trigger onto the stack (CR 603.3, 603.3d): `activate_ability`'s
    /// twin. The object is created under the controller locked at dispatch
    /// (603.3a), is not cast from anywhere, has no cost, and is `is_spell:
    /// false`, which is what keeps "counter target spell" off it (A4o).
    ///
    /// CR 603.3d's removal — "if a choice is required ... but no legal
    /// choices can be made ... the ability is simply removed from the stack"
    /// — is the rule's own order, taken literally: the object is created,
    /// pushed onto the stack, announced against, and both the push and the
    /// object are undone when the announcement fails. It has to exist to be
    /// announced against, because targeting legality reads the source.
    ///
    /// **Why that is the same game as never creating it.** Nothing between
    /// the creation and the removal is proposed or emitted — no replacement
    /// sees it, no trigger matches it, the log does not carry it — and the
    /// writes are an object id, a timestamp and two layer-epoch bumps, none
    /// of which reaches an outcome. The two facts a removal could have
    /// disturbed were fixed earlier: CR 603.3a's controller at dispatch, and
    /// CR 603.3b's order before targets in the CR's own sequence, so a
    /// removed trigger consumed its slot in the order exactly as it does on
    /// paper. Nothing is announced on the way out either, because a trigger
    /// removed this way is not countered (CR 701.6a).
    fn place_one(&mut self, pending: PendingTrigger, dp: &dyn DecisionProvider) -> Result<bool, String> {
        let controller = pending.controller;
        let object = GameObject::new(Arc::clone(&pending.source_card), controller, Zone::Stack);
        let id = self.add_object(object);
        self.stack.push(id);

        let targets = match self.announce_targets(controller, id, &pending.instances, dp) {
            Ok(targets) => targets,
            Err(_) => {
                self.stack.retain(|&x| x != id);
                self.remove_object(id);
                self.trace(|| trace_records::pending(self, &pending, None, Some("no_legal_choices"), &[]));
                return Ok(false);
            }
        };

        let TriggerOrigin::Object(identity) = pending.origin;
        let entry = StackEntry {
            object_id: id,
            controller,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: None,
            effect: pending.binding.def.effect.clone(),
            is_spell: false,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            mana_spent: ManaSpent::NONE,
            cast_from: None,
            ability_identity: Some(identity),
            trigger: Some(pending.binding.clone()),
            departed: pending.departed.clone(),
        };
        let chosen = entry.chosen_targets.clone();
        self.set_stack_entry(entry);
        self.diagnostics.record_trigger_placed();
        self.trace(|| trace_records::pending(self, &pending, Some(id), None, &chosen));
        Ok(true)
    }
}
