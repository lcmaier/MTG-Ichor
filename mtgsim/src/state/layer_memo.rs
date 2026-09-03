//! The cross-call memo of the layer walk — critical-path item 7a
//! (`layers-architecture.md` §12).
//!
//! A priority sweep asks `compute_characteristics` the same question about
//! the same unchanged board once per permanent: 96% of walks recomputed an
//! object nothing had touched. This is the store that answers the repeats.
//!
//! The key is coarse on purpose — one epoch for the whole game, bumped at
//! every write to any walk input (`GameState::layer_epoch`). A coarse key has
//! no inputs to enumerate, so the only way it can be wrong is a write that
//! skipped its bump, and `compute.rs`'s debug audit turns that into a panic
//! rather than a stale answer.
//!
//! **Never iterated.** `HashMap` order is unobservable only while nothing
//! reads it in order; a memo that fed a choice, a log or a count would be a
//! determinism hole (CLAUDE.md, "Determinism at the decision boundary").

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::types::ids::ObjectId;

/// Per-object frames, each stamped with the epoch it was computed at.
///
/// `RefCell` because the walk fills it through `&GameState` — the same reason
/// `EngineCounters` is `Cell`s — and a frame is served as a shared `Arc`, so a
/// hit never clones the ability list. Cloned with the state: a fork inherits
/// the frames, which are exactly as valid for the clone as for the original,
/// and the two epochs then move independently.
///
/// A stale entry is ignored, not evicted: the epoch comparison on read is the
/// whole invalidation, and the next miss overwrites it. Size is bounded by the
/// objects ever queried — a few hundred per game.
#[derive(Debug, Clone, Default)]
pub struct LayerMemo {
    frames: RefCell<HashMap<ObjectId, (u64, Arc<EffectiveCharacteristics>)>>,
}

impl LayerMemo {
    /// The frame for `id`, if one was stored at exactly `epoch`.
    pub(crate) fn get(&self, id: ObjectId, epoch: u64) -> Option<Arc<EffectiveCharacteristics>> {
        self.frames
            .borrow()
            .get(&id)
            .filter(|(stored, _)| *stored == epoch)
            .map(|(_, frame)| Arc::clone(frame))
    }

    /// Store `frame` for `id` as of `epoch`, replacing whatever was there.
    pub(crate) fn insert(&self, id: ObjectId, epoch: u64, frame: Arc<EffectiveCharacteristics>) {
        self.frames.borrow_mut().insert(id, (epoch, frame));
    }
}

/// One test per writer class in `layers-architecture.md` §12's census, each of
/// the same shape: query, write through the funnel, and the next query is a
/// miss with the new answer. `resolving`'s two writes in
/// `resolve_top_of_stack` have no funnel to call from here and are covered by
/// the debug audit under `fuzz_games` instead.
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::engine::layers::compute_characteristics;
    use crate::engine::layers::types::{EffectModification, EffectiveCharacteristics, Layer, PtValue};
    use crate::objects::object::GameObject;
    use crate::state::game_state::{GameState, StackEntry};
    use crate::test_support::{put_in_hand, put_on_battlefield, registered, test_ctx, vanilla_creature};
    use crate::types::effects::{CounterType, Effect};
    use crate::types::ids::ObjectId;
    use crate::types::replacement::EnterMods;
    use crate::types::zones::{Zone, ZoneChangeCause};

    /// Query `id`, and say whether the memo answered.
    fn query(game: &GameState, id: ObjectId) -> (Option<Arc<EffectiveCharacteristics>>, bool) {
        let hits = game.counters.memo_hits();
        let walks = game.counters.layer_walks();
        let frame = compute_characteristics(game, id);
        let hit = game.counters.memo_hits() == hits + 1;
        let miss = game.counters.layer_walks() == walks + 1;
        assert!(hit != miss, "a query is exactly one of a hit and a walk");
        (frame, hit)
    }

    fn power(game: &GameState, id: ObjectId) -> (Option<i32>, bool) {
        let (frame, hit) = query(game, id);
        (frame.and_then(|f| f.power), hit)
    }

    fn controller(game: &GameState, id: ObjectId) -> (Option<usize>, bool) {
        let (frame, hit) = query(game, id);
        (frame.map(|f| f.controller), hit)
    }

    fn pump(id: ObjectId, timestamp: u64) -> crate::engine::layers::types::ContinuousEffect {
        registered(
            id,
            Layer::Layer7cModifyPT,
            timestamp,
            EffectModification::ModifyPowerToughness {
                power: PtValue::Fixed(3),
                toughness: PtValue::Fixed(0),
            },
        )
    }

    #[test]
    fn a_repeated_query_is_a_hit_and_shares_the_frame() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

        let (first, hit) = query(&game, bears);
        assert!(!hit, "the first query walks");
        let (second, hit) = query(&game, bears);
        assert!(hit, "the second is served from the memo");
        assert!(Arc::ptr_eq(first.as_ref().unwrap(), second.as_ref().unwrap()));
        assert_eq!(game.counters.layer_walks(), 1, "one walk for two queries");
    }

    #[test]
    fn a_missing_object_is_never_stored() {
        let game = GameState::new(2, 20);
        let ghost = crate::types::ids::new_object_id();
        assert!(query(&game, ghost).0.is_none());
        let (frame, hit) = query(&game, ghost);
        assert!(frame.is_none() && !hit, "a `None` is walked again, never memoized");
    }

    #[test]
    fn registry_add_and_remove_bump() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(power(&game, bears), (Some(2), false));
        assert_eq!(power(&game, bears), (Some(2), true));

        let row = game.continuous_effects.add(pump(bears, 10));
        assert_eq!(power(&game, bears), (Some(5), false), "the add bumped");
        assert_eq!(power(&game, bears), (Some(5), true));

        game.continuous_effects.remove(row);
        assert_eq!(power(&game, bears), (Some(2), false), "the remove bumped");
    }

    #[test]
    fn registry_expiry_bumps_only_when_a_row_leaves() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        game.continuous_effects.add(pump(bears, 10));
        assert_eq!(power(&game, bears), (Some(5), false));

        // CR 514.2 — the until-end-of-turn row leaves at cleanup.
        assert_eq!(game.continuous_effects.remove_expired_at_cleanup(0, 1).len(), 1);
        assert_eq!(power(&game, bears), (Some(2), false), "a row left, so the epoch moved");

        // The next cleanup finds nothing to expire, and the memo survives it:
        // an expiry pass that removes nothing is not a write.
        assert!(game.continuous_effects.remove_expired_at_cleanup(0, 2).is_empty());
        assert_eq!(power(&game, bears), (Some(2), true));
    }

    #[test]
    fn placing_on_the_battlefield_bumps() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        // In a hand a card has no controller; the frame reports its owner.
        assert_eq!(controller(&game, card), (Some(0), false));
        assert_eq!(controller(&game, card), (Some(0), true));

        game.move_object(card, Zone::Battlefield).unwrap();
        game.place_on_battlefield(card, 1, &EnterMods::NONE);
        assert_eq!(controller(&game, card), (Some(1), false), "the entity is the seed now");
    }

    #[test]
    fn a_zone_change_bumps() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
        assert_eq!(power(&game, bears), (Some(3), false));
        assert_eq!(power(&game, bears), (Some(3), true));

        game.change_zone(bears, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
        assert_eq!(power(&game, bears), (Some(2), false), "no entity, so no counter");
    }

    /// The `// CAST-ROLLBACK:` shape: a move that is not an event still writes
    /// the zone, and `move_object` bumps for it.
    #[test]
    fn a_silent_move_bumps() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert!(!query(&game, card).1);
        assert!(query(&game, card).1);

        game.move_object(card, Zone::Stack).unwrap();
        assert!(!query(&game, card).1, "onto the stack: a miss");
        game.move_object(card, Zone::Hand).unwrap();
        assert!(!query(&game, card).1, "and the rewind: a miss");
        assert!(query(&game, card).1);
    }

    #[test]
    fn counters_bump_when_they_change() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(power(&game, bears), (Some(2), false));

        game.add_counters(bears, CounterType::PlusOnePlusOne, 2);
        assert_eq!(power(&game, bears), (Some(4), false));
        assert_eq!(game.remove_counters(bears, CounterType::PlusOnePlusOne, 1), 1);
        assert_eq!(power(&game, bears), (Some(3), false));
        assert_eq!(power(&game, bears), (Some(3), true));

        // Removing a kind the permanent does not carry writes nothing.
        assert_eq!(game.remove_counters(bears, CounterType::MinusOneMinusOne, 1), 0);
        assert_eq!(power(&game, bears), (Some(3), true));
    }

    #[test]
    fn the_object_store_bumps() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert!(query(&game, card).0.is_some());
        assert!(query(&game, card).1);

        assert!(game.remove_object(card).is_some());
        let (frame, hit) = query(&game, card);
        assert!(frame.is_none() && !hit, "gone from the store, and the memo did not say otherwise");

        let obj = GameObject::new(vanilla_creature(1, 1, &[]), 0, Zone::Hand);
        let other = obj.id;
        game.add_object(obj);
        assert_eq!(power(&game, other), (Some(1), false));
    }

    #[test]
    fn stack_entries_bump() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        game.move_object(card, Zone::Stack).unwrap();
        assert_eq!(controller(&game, card), (Some(0), false), "no entry: the owner");
        assert_eq!(controller(&game, card), (Some(0), true));

        // CR 108.4 — a spell's controller is read off its entry, here a
        // player who does not own the card.
        game.set_stack_entry(StackEntry {
            object_id: card,
            controller: 1,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Sequence(Vec::new()),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            cast_from: Some(Zone::Hand),
            ability_identity: None,
        });
        assert_eq!(controller(&game, card), (Some(1), false));
        assert_eq!(controller(&game, card), (Some(1), true));

        assert!(game.take_stack_entry(card).is_some());
        assert_eq!(controller(&game, card), (Some(0), false));
        assert!(game.take_stack_entry(card).is_none());
        assert_eq!(controller(&game, card), (Some(0), true), "taking nothing writes nothing");
    }

    #[test]
    fn a_clone_keeps_its_own_frames() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(power(&game, bears), (Some(2), false));

        let fork = game.clone();
        assert_eq!(power(&fork, bears), (Some(2), true), "the fork inherits the frame");

        game.continuous_effects.add(pump(bears, 10));
        assert_eq!(power(&game, bears), (Some(5), false));
        assert_eq!(power(&fork, bears), (Some(2), true), "and the fork's epoch did not move");
    }

    /// The debug mode: a write to a walk input that skips its bump is caught
    /// on the next hit rather than served. Direct field writes are how a
    /// bump gets skipped, so the test does exactly that.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "stale frame")]
    fn a_skipped_bump_is_caught_by_the_audit() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(controller(&game, bears), (Some(0), false));

        game.battlefield.get_mut(&bears).unwrap().controller = 1;
        let _ = compute_characteristics(&game, bears);
    }
}
