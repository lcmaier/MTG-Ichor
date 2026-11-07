//! CR 613.7m: objects that receive a timestamp at once, in APNAP order, each
//! player's in the order of that player's choice
//! (`choices-architecture.md#decision-6--when-the-engine-asks-an-order-the-cr-hands-a-player`).
//!
//! **Settled once the batch has performed.** The moves stamp in batch order
//! and a token was stamped as it was created, so the batch's stamps are
//! dealt back out here, by which object gets which, and that is the whole of
//! what the order does. No record, zone or graveyard order moves. It is also
//! when the question can be exact: every object has arrived, so the rows it
//! registered are the continuous effects it generates.
//!
//! **Asked where a timestamp is read.** CR 613.7a reads one for the order of
//! effects a static ability generates, and an ability gained or copied later
//! takes the later effect's. So the order matters only among objects that
//! generate continuous effects from static abilities, and not among objects
//! each of whose effects reach only that object, or among objects alike. The
//! Role rule's "most recent" (CR 303.7a) reads Roles, and every printed Role
//! generates such an effect. `audit_entry_orders` checks every order where
//! this says none can matter: unlike CR 601.2h's, this one stays open for as
//! long as the objects stay, so the audit reads the board as they arrive.

use std::collections::HashMap;

use crate::engine::actions::ActionContext;
use crate::engine::audit::{assert_frames_agree, orders_to_audit, permanents_as_they_are, Frames};
use crate::engine::layers::types::ContinuousEffect;
use crate::events::unasked::{Elision, Unasked, UnaskedAnswer};
use crate::oracle::characteristics::controller_or_owner;
use crate::state::game_state::GameState;
use crate::types::effects::ObjectSet;
use crate::types::ids::{IdSet, ObjectId, PlayerId, Timestamp};
use crate::ui::ask::ask_order_simultaneous_entries;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption};

impl GameState {
    /// CR 613.7m for `stamped`, the objects a batch just moved or created, in
    /// batch order. Most batches stamp one object.
    pub(crate) fn settle_simultaneous_timestamps(&mut self, stamped: Vec<ObjectId>, ctx: &ActionContext) {
        if stamped.len() < 2 || self.result.is_some() {
            return;
        }
        let mut seats: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
        for id in stamped.iter().copied().filter(|id| self.objects.contains_key(id)) {
            let Some(player) = controller_or_owner(self, id) else { continue };
            match seats.iter_mut().find(|(seat, _)| *seat == player) {
                Some((_, theirs)) => theirs.push(id),
                None => seats.push((player, vec![id])),
            }
        }
        seats.sort_by_key(|(player, _)| self.apnap_index(*player));

        let generating = self.rows_generated_by(&stamped);
        let mut order: Vec<ObjectId> = Vec::with_capacity(stamped.len());
        for (player, theirs) in seats {
            order.extend(self.in_the_order_of_their_choice(player, theirs, &generating, ctx));
        }
        self.deal_timestamps(&order);
    }

    /// Each of `stamped` that generates a continuous effect from a static
    /// ability, with the rows it registered.
    fn rows_generated_by(&self, stamped: &[ObjectId]) -> HashMap<ObjectId, Vec<&ContinuousEffect>> {
        let stamped: IdSet<ObjectId> = stamped.iter().copied().collect();
        let mut generating: HashMap<ObjectId, Vec<&ContinuousEffect>> = HashMap::new();
        for row in self.continuous_effects.iter() {
            if row.generated_by_a_static_ability_of(row.source) && stamped.contains(&row.source) {
                generating.entry(row.source).or_default().push(row);
            }
        }
        generating
    }

    /// `theirs`, `player`'s objects, with the ones a timestamp is read for in
    /// the order `player` chooses, each in a place one of them held, and the
    /// rest where they were.
    fn in_the_order_of_their_choice(
        &self,
        player: PlayerId,
        theirs: Vec<ObjectId>,
        generating: &HashMap<ObjectId, Vec<&ContinuousEffect>>,
        ctx: &ActionContext,
    ) -> Vec<ObjectId> {
        let rows: Vec<&Vec<&ContinuousEffect>> = theirs.iter().filter_map(|id| generating.get(id)).collect();
        let read: Vec<ObjectId> = theirs.iter().copied().filter(|id| generating.contains_key(id)).collect();
        if read.len() < 2 {
            return theirs;
        }
        let reach_others = rows.iter().any(|rows| rows.iter().any(|row| !matches!(row.affected_objects, ObjectSet::SourceOnly)));
        let alike = rows.windows(2).all(|pair| rows_functionally_identical(pair[0], pair[1]));
        if !reach_others || alike {
            self.note_unasked(player, Unasked::OneGame(Elision::EntryOrder), || {
                let objects = read.iter().map(|id| ChoiceOption::Object(*id)).collect();
                (ChoiceKind::OrderSimultaneousEntries { player }, UnaskedAnswer::Order(objects))
            });
            if self.audit_on() {
                self.audit_entry_orders(&read);
            }
            return theirs;
        }
        let answer = ask_order_simultaneous_entries(ctx.dp, self, player, &read);
        let mut chosen = answer.into_iter().map(|at| read[at]);
        theirs.into_iter().map(|id| if generating.contains_key(&id) { chosen.next().unwrap_or(id) } else { id }).collect()
    }

    /// Hand `order`'s objects the timestamps they already hold among them,
    /// earliest first, each one's rows with it (CR 613.7a).
    fn deal_timestamps(&mut self, order: &[ObjectId]) {
        let mut stamps: Vec<Timestamp> = order.iter().map(|id| self.object_timestamp(*id)).collect();
        stamps.sort_unstable();
        let mut moved = false;
        for (&id, stamp) in order.iter().zip(stamps) {
            if self.object_timestamp(id) != stamp {
                // NOT REPLACEABLE: which of a batch's stamps an object holds,
                // no event, and no replacement effect can watch for it.
                self.set_object_timestamp(id, stamp);
                self.continuous_effects.retime_static_rows(id, stamp);
                moved = true;
            }
        }
        if moved {
            self.bump_layer_epoch();
        }
    }

    /// The shortcut's check with the shortcut off: `objects` dealt their
    /// timestamps in other orders on a clone, and every permanent read
    /// after, whole. A difference is a timestamp read where this module says
    /// none is.
    fn audit_entry_orders(&self, objects: &[ObjectId]) {
        let orders: Vec<Vec<ObjectId>> =
            orders_to_audit(objects.len()).into_iter().map(|order| order.into_iter().map(|at| objects[at]).collect()).collect();
        let (dealt, others) = self.unobserved(|| {
            let others: Vec<_> = orders.iter().map(|order| (order, self.permanents_after_dealing(order))).collect();
            (self.permanents_after_dealing(objects), others)
        });
        for (order, frames) in others {
            assert_frames_agree("CR 613.7m's order", || format!("{order:?}"), &dealt, &frames);
        }
    }

    /// Every permanent, on a clone with `order`'s objects dealt their stamps.
    fn permanents_after_dealing(&self, order: &[ObjectId]) -> Frames {
        let mut fork = self.clone();
        fork.deal_timestamps(order);
        permanents_as_they_are(&fork)
    }
}

/// Whether two objects' effects are one effect on each other's board: the
/// same rows, in the same layers, reaching the same objects.
fn rows_functionally_identical(a: &[&ContinuousEffect], b: &[&ContinuousEffect]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| a.layer == b.layer && a.modification == b.modification && a.affected_objects == b.affected_objects)
}

#[cfg(test)]
mod tests {
    use crate::cards::fixtures::{godhead_of_awe_fixture, kudo_king_among_bears_fixture};
    use crate::cards::printings::savannah_lions;
    use crate::test_support::{put_on_battlefield, setup_two_player_game};

    /// The audit bites: two effects that do conflict, handed to it as though
    /// nothing read their timestamps, and it finds the Lions two sizes.
    #[test]
    #[should_panic(expected = "CR 613.7m's order")]
    fn the_audit_finds_a_timestamp_read_where_none_was_claimed() {
        let mut game = setup_two_player_game();
        let kudo = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
        let godhead = put_on_battlefield(&mut game, godhead_of_awe_fixture(), 0);
        put_on_battlefield(&mut game, savannah_lions(), 0);
        game.audit_entry_orders(&[kudo, godhead]);
    }
}
