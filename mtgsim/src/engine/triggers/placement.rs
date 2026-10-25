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

use crate::engine::layers::condition::ConditionReferents;
use super::binding::TriggeringEventView;
use crate::engine::trace_records;
use crate::events::cause::{EventCause, GameRule};
use crate::events::unasked::{Elision, Unasked, UnaskedAnswer};
use crate::engine::triggers::binding_reads::TriggerBindingReads;
use crate::objects::object::GameObject;
use crate::state::game_state::{GameState, StackEntry};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{EventRole, PendingTrigger, TriggerBinding, TriggerId, TriggerTier};
use crate::types::mana::ManaSpent;
use crate::types::zones::Zone;
use crate::ui::ask::ask_order_triggers;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption};
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
        // Read first, and rebuild the queue only when a player has gone: a
        // rebuild copies every waiting trigger a search's fork shares
        // (`state::shared_stores`), most of them for nothing.
        let departed: Vec<PendingTrigger> = if self.pending_triggers.iter().any(|t| !self.in_game(t.controller)) {
            let (gone, stay): (Vec<_>, Vec<_>) =
                std::mem::take(&mut self.pending_triggers).into_iter().partition(|t| !self.in_game(t.controller));
            self.pending_triggers = stay.into_iter().collect();
            gone
        } else {
            Vec::new()
        };
        for t in &departed {
            self.trace(|| trace_records::pending(self, t, None, Some("departed"), &[]));
        }

        let mut placed = 0usize;
        let seats = self.players_in_game_apnap();
        for tier in [TriggerTier::First, TriggerTier::Second] {
            for &player in &seats {
                let entries: Vec<&PendingTrigger> =
                    self.pending_triggers.iter().filter(|t| t.is_placed_with(tier, player)).collect();
                if entries.is_empty() {
                    continue;
                }
                let mine: Vec<TriggerId> = entries.iter().map(|t| t.id).collect();
                let order: Vec<usize> = if entries.len() < 2 {
                    (0..mine.len()).collect()
                } else if self.trigger_order_cannot_change_outcome(&entries) {
                    self.note_unasked(player, Unasked::OneGame(Elision::TriggerOrder), || {
                        let sources = entries.iter().map(|t| ChoiceOption::Object(t.origin.source())).collect();
                        (ChoiceKind::OrderTriggers { player, tier }, UnaskedAnswer::Order(sources))
                    });
                    (0..mine.len()).collect()
                } else {
                    let sources: Vec<ObjectId> = entries.iter().map(|t| t.origin.source()).collect();
                    ask_order_triggers(dp, self, player, tier, &sources)
                };
                self.queue_last_placed_first(tier, player, &order);
                for i in order {
                    let id = mine[i];
                    let Some(from_end) = self.pending_triggers.iter().rev().position(|t| t.id == id) else { continue };
                    let Some(pending) = self.pending_triggers.remove(self.pending_triggers.len() - 1 - from_end) else { continue };
                    if self.place_one(pending, dp)? {
                        placed += 1;
                    }
                }
            }
        }
        Ok(placed > 0)
    }

    /// Move `player`'s entries of `tier`, about to be placed in `order`, to
    /// the queue's end with the last to be placed first. Each then leaves the
    /// queue from behind only the entries that triggered while it was placed,
    /// where taking each from among the others shifts the rest of the queue
    /// once per entry, thousands of times over after a board wipe. The other
    /// entries keep their order, and so did these until their ordering
    /// prompt, which has been asked.
    fn queue_last_placed_first(&mut self, tier: TriggerTier, player: PlayerId, order: &[usize]) {
        let (block, rest): (Vec<PendingTrigger>, Vec<PendingTrigger>) =
            std::mem::take(&mut self.pending_triggers).into_iter().partition(|t| t.is_placed_with(tier, player));
        self.pending_triggers = rest.into_iter().collect();
        let mut slots: Vec<Option<PendingTrigger>> = block.into_iter().map(Some).collect();
        let last_placed_first: Vec<PendingTrigger> = order.iter().rev().filter_map(|&i| slots.get_mut(i)?.take()).collect();
        self.pending_triggers.extend(slots.into_iter().flatten());
        self.pending_triggers.extend(last_placed_first);
    }

    /// Whether one player's entries give the same game in whatever order
    /// they go on the stack, so the ordering prompt is not asked.
    /// `triggers-architecture.md` §5.2 gives the reason for each condition.
    /// Compared by def rather than by id, since two grants of one ability
    /// carry two ids. A target or a mode, chosen as each goes on the stack
    /// (CR 603.3c, 603.3d), is no condition: the entries are interchangeable.
    fn trigger_order_cannot_change_outcome(&self, entries: &[&PendingTrigger]) -> bool {
        let Some(first) = entries.first() else { return true };
        let reads = first.binding.def.trigger_binding_reads();
        entries.iter().all(|t| {
            t.tier() == TriggerTier::First && t.binding.def == first.binding.def && self.entries_agree_on(reads, first, t)
        })
    }

    /// Whether two entries of one def agree on every fact it reads (item 163).
    pub(super) fn entries_agree_on(&self, reads: TriggerBindingReads, a: &PendingTrigger, b: &PendingTrigger) -> bool {
        let (x, y) = (&a.binding, &b.binding);
        let unread = |fact| !reads.contains(fact);
        // The event roles the def names, each triggering event's: one it
        // does not name may differ, as each Wargear's own attachment does.
        // Each as its own entry's arm holds it, since "that creature" of
        // "blocks or becomes blocked by" is a different side on each arm.
        let objects = |binding: &TriggerBinding| -> Vec<(EventRole, ObjectRef)> {
            let named: Vec<EventRole> = reads.event_roles().filter_map(|role| Some(binding.event()?.role_as_recorded(role))).collect();
            binding
                .triggering_events
                .iter()
                .flat_map(|e| e.objects.objects().filter(|(p, _)| named.contains(p)).collect::<Vec<_>>())
                .collect()
        };
        // The source by identity, not the origin: a delayed trigger's origin
        // is its own registry number, which no two share. Each reads the
        // source's last known information as its own records frame it, else
        // as the LKI store keeps it, the one for that existence (CR 113.7a).
        let source = a.origin.source_ref();
        let same_source = source == b.origin.source_ref()
            && self.lki_as_read_by(Some(x), source) == self.lki_as_read_by(Some(y), source);
        (unread(TriggerBindingReads::TRIGGERING_OBJECTS) || objects(x) == objects(y))
            && (unread(TriggerBindingReads::PLAYER) || self.bound_player(x) == self.bound_player(y))
            && (unread(TriggerBindingReads::AMOUNT) || self.bound_amount(x) == self.bound_amount(y))
            && (unread(TriggerBindingReads::CHARACTERISTICS) || (objects(x) == objects(y) && x.first_record() == y.first_record()))
            && (unread(TriggerBindingReads::SOURCE) || same_source)
            && (unread(TriggerBindingReads::X) || a.x_value == b.x_value)
            && (unread(TriggerBindingReads::ABILITY) || self.ability_state(a) == self.ability_state(b))
            && (unread(TriggerBindingReads::REFERRED) || x.referred == y.referred)
    }

    /// "This ability"'s state: whether its CR 603.2h action is taken this
    /// turn, and how many times it has resolved (CR 603.7h).
    fn ability_state(&self, entry: &PendingTrigger) -> (bool, u32) {
        let identity = entry.origin.identity();
        (
            self.history.action_taken_this_turn.contains(&(identity, entry.controller)),
            self.resolutions_this_turn_of(identity),
        )
    }

    /// One trigger onto the stack (CR 603.3, 603.3d): `activate_ability`'s
    /// twin. The object is created under the controller locked at dispatch
    /// (603.3a), is not cast from anywhere, has no cost, and is `is_spell:
    /// false`, which is what keeps "counter target spell" off it.
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

        let identity = pending.origin.identity();
        // A target's "defending player" reads the trigger's attacker off its
        // binding, as it will at resolution.
        let named = ConditionReferents {
            source_lki: None,
            triggering_event: TriggeringEventView::Resolving { game: self, binding: &pending.binding },
        };
        let targets = match self.announce_targets(controller, id, identity.source.id, &pending.instances, named, pending.x_value, dp) {
            Ok(targets) => targets,
            Err(_) => {
                self.ability_leaves_the_stack(id);
                self.trace(|| trace_records::pending(self, &pending, None, Some("no_legal_choices"), &[]));
                return Ok(false);
            }
        };

        let entry = StackEntry {
            object_id: id,
            controller,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: pending.x_value,
            effect: Arc::new(pending.binding.def.effect.clone()),
            is_spell: false,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            mana_spent: ManaSpent::NONE,
            cast_from: None,
            ability_identity: Some(identity),
            trigger: Some(pending.binding.clone()),
        };
        let chosen = entry.chosen_targets.clone();
        self.set_stack_entry(entry);
        self.diagnostics.record_trigger_placed();
        self.trace(|| trace_records::pending(self, &pending, Some(id), None, &chosen));
        self.announce_targeted_records(id, EventCause::GameRule(GameRule::TriggeredAbilityPutOnStack), dp)?;
        Ok(true)
    }
}
