//! The cross-call memo of the layer walk — critical-path item 7a
//! (`layers-architecture.md` §12).
//!
//! A priority sweep asks `compute_characteristics` the same question about
//! the same unchanged board once per permanent: 96% of walks recomputed an
//! object nothing had touched (measured 2026-09-03). This is the store that
//! answers the repeats.
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

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use crate::engine::layers::board::RowNote;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::oracle::mana_supply::ManaProductionWatchers;
use crate::state::history_log::RowKind;
use crate::state::shared_stores::ObjectTable;
use crate::types::ids::ObjectId;

/// Per-object frames, each stamped with the epoch it was computed at.
///
/// `RefCell` because the walk fills it through `&GameState` — the same reason
/// `Diagnostics` is `Cell`s — and a frame is served as a shared `Arc`, so a
/// hit never clones the ability list. Cloned with the state: a fork inherits
/// the frames, which are exactly as valid for the clone as for the original,
/// and the two epochs then move independently.
///
/// A stale entry is ignored, not evicted: the epoch comparison on read is the
/// whole invalidation, and the next miss overwrites it. An object taken out of
/// the store takes its frame with it ([`Self::forget`]): its id is never
/// reused, so the frame could never be served, and the memo is bounded by the
/// store rather than by every object the game ever walked. Kept in an
/// [`ObjectTable`], so a clone copies a pointer per chunk of frames and a fork's
/// walk copies the chunks it stores into (`clone-architecture.md` decision 2).
#[derive(Debug, Clone, Default)]
pub struct LayerMemo {
    frames: RefCell<ObjectTable<StoredFrame>>,
    /// The notes of the pass that filled `frames`, stamped with its epoch as
    /// each frame is: what that pass noted about each row reaching a zone it
    /// leaves out, which a card there is walked with (`layers-architecture.md`
    /// §13e). `None` until a pass leaves a zone out. One shared slice, so a
    /// fork bumps a count and copies nothing; never written once stored, so a
    /// fork's own walks go into its own `frames` and leave the original's as
    /// they were.
    notes: RefCell<Option<(u64, Arc<[RowNote]>)>>,
    /// The board's watchers of a mana production, stamped with the epoch they
    /// were found at (`oracle::mana_supply`, `mana-architecture.md` §3.4):
    /// one scan per epoch for every player's inventory. Shared, so a fork
    /// copies nothing for it.
    mana_production_watchers: RefCell<Option<(u64, Arc<ManaProductionWatchers>)>>,
    /// What the frames stored at the latest epoch read of the gated inputs:
    /// what a write to one asks before it moves the epoch.
    gated_reads: Cell<GatedReadsAt>,
    /// The debug audits' pause (`GameState::pause_layer_audit`).
    #[cfg(debug_assertions)]
    audit_paused: std::cell::Cell<bool>,
    /// The frames the memo audit has checked inside
    /// `GameState::audit_each_frame_once`.
    #[cfg(debug_assertions)]
    audit_scope: AuditScope,
}

/// One object's frame in the memo, and the epoch it was computed at: it is
/// served only at that epoch.
#[derive(Debug, Clone)]
pub(crate) struct StoredFrame {
    epoch: u64,
    frame: Arc<EffectiveCharacteristics>,
}

/// A walk input whose write moves the layer epoch only when a frame stored at
/// the current epoch read it. Writes to these are frequent and nearly every
/// walk ignores them: a tap for mana, a life change, an attack, a record
/// counted. A bump at each would throw away every stored frame each time,
/// for statics few boards have. So a walk keeps what it read with the frames
/// it stores, and a write asks (`GameState::bump_layer_epoch_if_read`): when
/// no stored frame read the input, no stored answer depends on it, and the
/// next walk reads the new value anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GatedWalkInput {
    /// Each player's life total (CR 119): "as long as you have 30 or more
    /// life".
    LifeTotals,
    /// A permanent's tapped status (CR 110.5): "as long as this artifact is
    /// untapped".
    TappedStatus,
    /// What each creature is attacking or blocking (CR 506): "defending
    /// player".
    Combat,
    /// The turn number: "entered this turn", and every "this turn" over the
    /// history log.
    TurnNumber,
    /// Which combat of the turn is the latest: "this combat".
    CombatCount,
    /// The history log's rows of one kind (`state::history_log`), noted by
    /// the lookup that scans them: "creatures that attacked this turn".
    Rows(RowKind),
}

impl GatedWalkInput {
    /// Its bit in a [`GatedInputs`].
    const fn bit(self) -> u32 {
        let place = match self {
            GatedWalkInput::LifeTotals => 0,
            GatedWalkInput::TappedStatus => 1,
            GatedWalkInput::Combat => 2,
            GatedWalkInput::TurnNumber => 3,
            GatedWalkInput::CombatCount => 4,
            GatedWalkInput::Rows(kind) => 5 + kind as u32,
        };
        1 << place
    }
}

const _: () = assert!(5 + RowKind::COUNT <= u32::BITS as usize);

/// A set of [`GatedWalkInput`]s: what one walk read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GatedInputs(u32);

impl GatedInputs {
    pub(crate) const NONE: GatedInputs = GatedInputs(0);

    /// This set and `input`.
    pub(crate) fn with(self, input: GatedWalkInput) -> GatedInputs {
        GatedInputs(self.0 | input.bit())
    }

    fn union(self, other: GatedInputs) -> GatedInputs {
        GatedInputs(self.0 | other.0)
    }

    fn contains(self, input: GatedWalkInput) -> bool {
        self.0 & input.bit() != 0
    }
}

/// What the walks stored at `epoch` read. A later epoch starts it over, since
/// every frame stored before it is stale.
#[derive(Debug, Clone, Copy, Default)]
struct GatedReadsAt {
    epoch: u64,
    read: GatedInputs,
}

impl GatedReadsAt {
    /// These reads, and a walk's stored at `epoch` that read `read`.
    fn and(self, epoch: u64, read: GatedInputs) -> GatedReadsAt {
        let kept = if self.epoch == epoch { self.read } else { GatedInputs::NONE };
        GatedReadsAt { epoch, read: kept.union(read) }
    }

    fn read_at(self, epoch: u64, input: GatedWalkInput) -> bool {
        self.epoch == epoch && self.read.contains(input)
    }
}

/// What the open `GameState::audit_each_frame_once` has checked: every
/// member's frame once its one pass has run, and each other object's by id.
/// `None` outside one. A clone starts outside any scope, since a clone can be
/// written.
#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct AuditScope(RefCell<Option<Checked>>);

#[cfg(debug_assertions)]
impl Clone for AuditScope {
    fn clone(&self) -> AuditScope {
        AuditScope::default()
    }
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct Checked {
    members: bool,
    others: crate::types::ids::IdSet<ObjectId>,
}

/// What the memo audit owes a hit (`compute::audit_memo_hit`).
#[cfg(debug_assertions)]
pub(crate) enum AuditOwed {
    /// The frame served, against a fresh walk: every hit outside a scope, and
    /// inside one the first on each object the pass does not hold.
    ThisFrame,
    /// Every member's stored frame, against one pass: inside a scope, the
    /// first hit on a member.
    EveryMember,
    /// Nothing: the scope has checked this frame.
    Nothing,
}

/// Closes the scope [`LayerMemo::open_audit_scope`] opened when it drops,
/// a read that panics included.
#[cfg(debug_assertions)]
pub(crate) struct OpenAuditScope<'m>(&'m LayerMemo);

#[cfg(debug_assertions)]
impl Drop for OpenAuditScope<'_> {
    fn drop(&mut self) {
        *self.0.audit_scope.0.borrow_mut() = None;
    }
}

impl LayerMemo {
    /// Put back what `saved`, a clone of this memo taken earlier, held: an
    /// observer's read leaves the memo as it found it (`GameState::unobserved`).
    /// The audit scope goes back to none, as a clone's does.
    pub(crate) fn restore(&self, saved: LayerMemo) {
        let LayerMemo {
            frames,
            notes,
            mana_production_watchers,
            gated_reads,
            #[cfg(debug_assertions)]
            audit_paused,
            #[cfg(debug_assertions)]
            audit_scope,
        } = saved;
        self.frames.replace(frames.into_inner());
        self.notes.replace(notes.into_inner());
        self.mana_production_watchers.replace(mana_production_watchers.into_inner());
        self.gated_reads.set(gated_reads.get());
        #[cfg(debug_assertions)]
        {
            self.audit_paused.set(audit_paused.get());
            self.audit_scope.0.replace(audit_scope.0.into_inner());
        }
    }

    /// Whether the debug audits check what this memo serves.
    #[cfg(debug_assertions)]
    pub(crate) fn audited(&self) -> bool {
        !self.audit_paused.get()
    }

    #[cfg(debug_assertions)]
    pub(crate) fn pause_audit(&self, paused: bool) {
        self.audit_paused.set(paused);
    }

    /// Opens `GameState::audit_each_frame_once`'s scope, unless one is open.
    #[cfg(debug_assertions)]
    pub(crate) fn open_audit_scope(&self) -> Option<OpenAuditScope<'_>> {
        let mut scope = self.audit_scope.0.borrow_mut();
        if scope.is_some() {
            return None;
        }
        *scope = Some(Checked::default());
        Some(OpenAuditScope(self))
    }

    /// What the audit owes a hit on `id`, marked as checked. `is_member`, the
    /// pass's membership, is asked only inside a scope.
    #[cfg(debug_assertions)]
    pub(crate) fn audit_owed(&self, id: ObjectId, is_member: impl FnOnce() -> bool) -> AuditOwed {
        if self.audit_scope.0.borrow().is_none() {
            return AuditOwed::ThisFrame;
        }
        // Asked with no borrow held: the membership may read a frame.
        let member = is_member();
        let mut scope = self.audit_scope.0.borrow_mut();
        let Some(checked) = scope.as_mut() else { return AuditOwed::ThisFrame };
        match member {
            true if std::mem::replace(&mut checked.members, true) => AuditOwed::Nothing,
            true => AuditOwed::EveryMember,
            false if checked.others.insert(id) => AuditOwed::ThisFrame,
            false => AuditOwed::Nothing,
        }
    }

    /// The frame for `id`, if one was stored at exactly `epoch`.
    pub(crate) fn get(&self, id: ObjectId, epoch: u64) -> Option<Arc<EffectiveCharacteristics>> {
        self.frames
            .borrow()
            .get(&id)
            .filter(|stored| stored.epoch == epoch)
            .map(|stored| Arc::clone(&stored.frame))
    }

    /// Store `frame` for `id` as of `epoch`, replacing whatever was there.
    pub(crate) fn insert(&self, id: ObjectId, epoch: u64, frame: Arc<EffectiveCharacteristics>) {
        self.frames.borrow_mut().insert(id, StoredFrame { epoch, frame });
    }

    /// Drop `id`'s frame: the store took the object out, and no later read
    /// can name it.
    pub(crate) fn forget(&self, id: ObjectId) {
        self.frames.borrow_mut().remove(&id);
    }

    /// The notes stored at exactly `epoch`, if the pass that filled the memo
    /// then left a zone out.
    pub(crate) fn notes(&self, epoch: u64) -> Option<Arc<[RowNote]>> {
        self.notes
            .borrow()
            .as_ref()
            .filter(|(stored, _)| *stored == epoch)
            .map(|(_, notes)| Arc::clone(notes))
    }

    /// Store a pass's notes as of `epoch`, replacing whatever was there.
    pub(crate) fn insert_notes(&self, epoch: u64, notes: Arc<[RowNote]>) {
        *self.notes.borrow_mut() = Some((epoch, notes));
    }

    /// The production watchers found at exactly `epoch`.
    pub(crate) fn mana_production_watchers(&self, epoch: u64) -> Option<Arc<ManaProductionWatchers>> {
        self.mana_production_watchers
            .borrow()
            .as_ref()
            .filter(|(stored, _)| *stored == epoch)
            .map(|(_, found)| Arc::clone(found))
    }

    pub(crate) fn insert_mana_production_watchers(&self, epoch: u64, found: Arc<ManaProductionWatchers>) {
        *self.mana_production_watchers.borrow_mut() = Some((epoch, found));
    }

    /// Keep what a walk whose frames are stored at `epoch` read, beside them.
    pub(crate) fn insert_gated_reads(&self, epoch: u64, read: GatedInputs) {
        if read != GatedInputs::NONE {
            self.gated_reads.set(self.gated_reads.get().and(epoch, read));
        }
    }

    /// Whether a frame stored at `epoch` read `input`.
    pub(crate) fn gated_read_at(&self, input: GatedWalkInput, epoch: u64) -> bool {
        self.gated_reads.get().read_at(epoch, input)
    }
}

/// One test per writer class in `layers-architecture.md` §12's census, each of
/// the same shape: query, write through the funnel, and the next query is a
/// miss with the new answer. `resolving`'s two writes in
/// `resolve_top_of_stack` have no funnel to call from here and are covered by
/// the debug audit under `fuzz_games` instead.
#[cfg(test)]
mod tests {
    use crate::test_support::place_on_battlefield;
    use std::sync::Arc;

    use crate::engine::layers::compute_characteristics;
    use crate::engine::layers::types::{EffectModification, EffectiveCharacteristics, Layer, PtValue};
    use crate::objects::object::GameObject;
    use crate::state::game_state::{GameState, StackEntry};
    use crate::engine::actions::GameAction;
    use crate::objects::card_data::CardDataBuilder;
    use crate::test_support::{
        put_in_hand, put_on_battlefield, registered, setup_game, static_ability, test_ctx, vanilla_creature,
    };
    use crate::types::card_types::CardType;
    use crate::types::effects::{
        AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, PlayerFact, PlayerSet, Primitive,
    };
    use crate::types::ids::ObjectId;
    use crate::types::mana::ManaSpent;
    use crate::types::replacement::EnterMods;
    use crate::types::zones::{Zone, ZoneChangeAct};

    /// Query `id`, and say whether the memo answered.
    fn query(game: &GameState, id: ObjectId) -> (Option<Arc<EffectiveCharacteristics>>, bool) {
        let hits = game.diagnostics.memo_hits();
        let walks = game.diagnostics.layer_walks();
        let frame = compute_characteristics(game, id);
        let hit = game.diagnostics.memo_hits() == hits + 1;
        let miss = game.diagnostics.layer_walks() == walks + 1;
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

    fn pump(id: ObjectId, timestamp: crate::types::ids::Timestamp) -> crate::engine::layers::types::ContinuousEffect {
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
        // The entry's history row read the frame already; count from here.
        let walks = game.diagnostics.layer_walks();

        let (first, _) = query(&game, bears);
        let (second, hit) = query(&game, bears);
        assert!(hit, "the second is served from the memo");
        assert!(Arc::ptr_eq(first.as_ref().unwrap(), second.as_ref().unwrap()));
        assert!(game.diagnostics.layer_walks() - walks <= 1, "at most one walk for two queries");
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

    /// A static over "creatures that attacked this turn" read the log's
    /// attack rows, so its frames stand through a damage row and walk again
    /// once an attack row is written (`triggers-architecture.md` §12, TR-5h).
    #[test]
    fn a_history_read_is_invalidated_by_its_kind_of_row_only() {
        use crate::events::cause::{EventCause, GameRule};
        use crate::events::event::GameEvent;
        use crate::state::battlefield::AttackTarget;
        use crate::types::effects::{ObjectFilter, ObjectSet};
        use crate::types::history::ObjectFact;
        use crate::types::zones::ZoneSet;

        let mut game = GameState::new(2, 20);
        game.begin_turn(1, 0);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 1);
        let mut attackers_pumped = pump(bears, 10);
        attackers_pumped.affected_objects =
            ObjectSet::Filter { filter: ObjectFilter::ThisTurn(ObjectFact::Attacked), zones: ZoneSet::of(Zone::Battlefield) };
        game.continuous_effects.add(attackers_pumped);
        assert_eq!(power(&game, bears), (Some(2), false));
        assert_eq!(power(&game, bears), (Some(2), true));

        let cause = EventCause::GameRule(GameRule::StepBegins);
        let damage = GameEvent::DamageDealt {
            source: game.object_ref(bears).unwrap(),
            target: crate::types::triggers::ObjectOrPlayer::Object(game.object_ref(wall).unwrap()),
            amount: 1,
            excess: 0,
            is_combat: false,
            source_lki: None,
        };
        game.emit_event(damage, cause).unwrap();
        assert!(game.history.log.rows_newest_first().any(|row| row.fact.kind() == crate::state::history_log::RowKind::DealtDamage));
        assert_eq!(power(&game, bears), (Some(2), true), "a damage row is not what the static read");

        game.emit_event(GameEvent::AttackersDeclared { attackers: vec![(bears, AttackTarget::Player(1))] }, cause).unwrap();
        assert_eq!(power(&game, bears), (Some(5), false), "an attack row is, and the bears attacked");
    }

    /// A static's filter that asks a creature's defending player (CR 508.5)
    /// reads what it is attacking, so its frames walk again once it attacks.
    /// The read is noted inside the lookup, for a layer row's filter too.
    #[test]
    fn a_defending_player_read_is_invalidated_by_an_attack() {
        use crate::state::battlefield::{AttackTarget, AttackingInfo};
        use crate::state::layer_memo::GatedWalkInput;
        use crate::types::effects::{Antecedent, ObjectFilter, ObjectSet, PlayerRef};
        use crate::types::zones::ZoneSet;

        let mut game = GameState::new(2, 20);
        game.begin_turn(1, 0);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 1);
        // "Creatures defending player controls get +3/+0", from the bears.
        let mut pumped = pump(bears, 10);
        pumped.affected_objects = ObjectSet::Filter {
            filter: ObjectFilter::ByController(PlayerRef::DefendingPlayer(Antecedent::ThisObject)),
            zones: ZoneSet::of(Zone::Battlefield),
        };
        game.continuous_effects.add(pumped);
        assert_eq!(power(&game, wall).0, Some(0), "the bears attack no one");

        game.battlefield.get_mut(&bears).unwrap().attacking =
            Some(AttackingInfo { target: AttackTarget::Player(1), is_blocked: false, blocked_by: Vec::new() });
        game.bump_layer_epoch_if_read(GatedWalkInput::Combat);
        assert_eq!(power(&game, wall).0, Some(3), "player 1 is the defending player now");
    }

    #[test]
    fn placing_on_the_battlefield_bumps() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        // In a hand a card has no controller; the frame reports its owner.
        assert_eq!(controller(&game, card), (Some(0), false));
        assert_eq!(controller(&game, card), (Some(0), true));

        game.move_object(card, Zone::Battlefield).unwrap();
        place_on_battlefield(&mut game, card, 1, &EnterMods::NONE);
        // A frame kept from the hand would still say its owner. The entry's
        // history row may have walked it fresh already, so the hit flag is not
        // the witness.
        assert_eq!(controller(&game, card).0, Some(1), "the entity is the seed now");
    }

    #[test]
    fn a_zone_change_bumps() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
        assert_eq!(power(&game, bears), (Some(3), false));
        assert_eq!(power(&game, bears), (Some(3), true));

        game.change_zone(bears, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
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

    /// CR 800.4a: a player who has left the game is no opponent, and a
    /// layer condition names only players still in it. A loss moves nothing
    /// of a player who owns nothing, so it is a walk input of its own.
    #[test]
    fn a_loss_bumps() {
        let mut game = setup_game(4);
        let while_an_opponent_is_low = CardDataBuilder::new("Fixture: +1/+2 while an opponent has 10 or less life")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .ability(static_ability(Effect::Conditional(
                Condition::Player { players: PlayerSet::Opponents, fact: PlayerFact::LifeAtMost(AmountExpr::Fixed(10)) },
                Box::new(Effect::Atom(
                    Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(2), Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                )),
            )))
            .build();
        let creature = put_on_battlefield(&mut game, while_an_opponent_is_low, 0);
        game.players[2].life_total = 5;
        assert_eq!(power(&game, creature), (Some(2), false));
        assert_eq!(power(&game, creature), (Some(2), true));

        game.execute_action(GameAction::PlayerLoses { player: 2 }, &test_ctx())
            .unwrap();
        assert_eq!(power(&game, creature), (Some(1), false), "the one opponent at 10 or less has left");
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
        let other = game.add_object(obj);
        assert_eq!(power(&game, other), (Some(1), false));
    }

    /// An object out of the store takes its frame with it: its id is never
    /// reused, so a memo that kept it would grow with every token the game
    /// ever walked (`clone-architecture.md` §2.5).
    #[test]
    fn a_removed_objects_frame_is_dropped() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert!(query(&game, card).0.is_some());
        assert!(game.layer_memo.frames.borrow().get(&card).is_some(), "the walk stored its frame");

        game.remove_object(card);
        assert!(game.layer_memo.frames.borrow().get(&card).is_none());
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
            effect: Arc::new(Effect::Sequence(Vec::new())),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            mana_spent: ManaSpent::NONE,
            cast_from: Some(Zone::Hand),
            ability_identity: None,
            trigger: None,
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
    /// Paused for a replay within a session, the audit lets a stale frame
    /// through unchecked, and catches it again once resumed.
    #[cfg(debug_assertions)]
    #[test]
    fn a_paused_audit_checks_nothing_until_it_resumes() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(controller(&game, bears), (Some(0), false));
        game.battlefield.get_mut(&bears).unwrap().controller = 1;
        game.pause_layer_audit();
        assert_eq!(controller(&game, bears), (Some(0), true), "served stale and unchecked");
        game.resume_layer_audit();
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compute_characteristics(&game, bears)));
        assert!(caught.is_err(), "resumed, the audit catches the skipped bump");
    }

    /// `GameState::audit_each_frame_once` checks each frame once, and loses
    /// nothing the audit at every hit catches: a frame gone stale before the
    /// read is caught at its first hit inside, a member's by the one pass.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "stale frame")]
    fn a_frame_audited_once_still_catches_a_skipped_bump() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(controller(&game, bears), (Some(0), false));
        game.battlefield.get_mut(&bears).unwrap().controller = 1;
        game.audit_each_frame_once(|| compute_characteristics(&game, bears));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "stale frame")]
    fn a_card_no_pass_holds_is_audited_once_and_still_caught() {
        let mut game = GameState::new(2, 20);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(controller(&game, card), (Some(0), false));
        game.objects.get_mut(&card).unwrap().owner = 1;
        game.audit_each_frame_once(|| compute_characteristics(&game, card));
    }

    /// The one pass checks every member's frame, not only the one asked for.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "stale frame")]
    fn the_first_hit_on_a_member_checks_every_members_frame() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let other = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
        assert_eq!(controller(&game, bears), (Some(0), false));
        assert_eq!(controller(&game, other), (Some(0), true));
        game.battlefield.get_mut(&other).unwrap().controller = 1;
        game.audit_each_frame_once(|| compute_characteristics(&game, bears));
    }

    /// Once its read returns, or panics, the scope is closed, and every hit
    /// is checked again.
    #[cfg(debug_assertions)]
    #[test]
    fn a_closed_scope_leaves_every_hit_audited() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
        game.audit_each_frame_once(|| (compute_characteristics(&game, bears), compute_characteristics(&game, card)));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.audit_each_frame_once(|| {
                compute_characteristics(&game, bears);
                panic!("the read fails");
            })
        }));
        assert!(failed.is_err());
        game.battlefield.get_mut(&bears).unwrap().controller = 1;
        game.objects.get_mut(&card).unwrap().owner = 1;
        for id in [bears, card] {
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compute_characteristics(&game, id)));
            assert!(caught.is_err(), "the scope closed, so the hit on {id} is audited");
        }
    }

    /// A clone can be written, so one taken inside a scope starts outside it.
    #[cfg(debug_assertions)]
    #[test]
    fn a_clone_taken_inside_a_scope_is_audited_at_every_hit() {
        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert_eq!(controller(&game, bears), (Some(0), false));
        let mut fork = game.audit_each_frame_once(|| {
            // A hit: the scope has checked every member's frame.
            assert_eq!(controller(&game, bears), (Some(0), true));
            game.clone()
        });
        fork.battlefield.get_mut(&bears).unwrap().controller = 1;
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compute_characteristics(&fork, bears)));
        assert!(caught.is_err(), "the fork's hit is audited");
    }

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

    /// A fork that reads a card the pass leaves out stores the card's frame in
    /// its own memo and leaves the original's as it was: the notes the two
    /// share are never written once stored (`layers-architecture.md` §13e
    /// decision 3). The clone shares the notes themselves, allocating nothing
    /// for them.
    #[test]
    fn a_fork_reading_a_left_out_card_leaves_the_originals_memo_as_it_was() {
        use crate::cards::phase_lj_cards::lattice_colorless_clause;
        use crate::test_support::put_in_library;

        let mut game = crate::test_support::setup_two_player_game();
        let lattice = put_on_battlefield(&mut game, lattice_colorless_clause(), 0);
        let card = put_in_library(&mut game, vanilla_creature(2, 2, &[]), 0);
        // The pass fills the memo, and its notes beside the frames.
        compute_characteristics(&game, lattice);
        let epoch = game.layer_epoch();
        let notes = game.layer_memo.notes(epoch).expect("the pass left the library out");

        let fork = game.clone();
        assert!(compute_characteristics(&fork, card).is_some_and(|frame| frame.colors.is_empty()));
        assert!(fork.layer_memo.get(card, epoch).is_some(), "the fork stored the card's frame");
        assert!(game.layer_memo.get(card, epoch).is_none(), "the original stored nothing");
        let forked = fork.layer_memo.notes(epoch).expect("the fork shares the notes");
        assert!(Arc::ptr_eq(&notes, &forked), "one set of notes, not a copy");
    }
}
