use std::collections::HashMap;
use std::sync::Arc;
use crate::engine::layers::types::{EffectiveCharacteristics, Timestamp};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::effects::CounterType;

/// The counters of one kind on a permanent, with the timestamp CR 613.7c gives
/// them.
///
/// > Each counter receives a timestamp as it's put on an object or player. If
/// > that object or player already has a counter of that kind on it, each
/// > counter of that kind receives a new timestamp identical to that of the new
/// > counter.
///
/// The second sentence is what makes a single timestamp per kind *exact*: two
/// counters of the same kind can never carry different timestamps, so there is
/// nothing a per-counter list would express that this does not.
///
/// Read by the layer system: keyword counters (CR 122.1b) are layer 6 effects
/// and interleave with that layer's registry rows by timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterStack {
    pub count: u32,
    pub timestamp: Timestamp,
}

/// A permanent's counters (CR 122), kind by kind, read anywhere as the map.
///
/// **Changed only inside `state/`**, by `GameState::add_counters`,
/// `remove_counters` and an entry's own counters: each bumps the layer epoch
/// (a counter is a layer-walk input) and keeps
/// `GameState::replacement_counter_holders`, which the gather reads in place
/// of every permanent's counters. Outside `state/` no method changes the map
/// and none makes one, so no counter can be put on or taken off there. What
/// still compiles is replacing a permanent's whole field with a clone of
/// another's; nothing does, and a debug build's check at every gather
/// (`replacement_counter_holders_cover_the_board`) would catch the shield it
/// left unindexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counters(HashMap<CounterType, CounterStack>);

impl Counters {
    /// A permanent with no counters, as every permanent enters before its
    /// entry's own are put on.
    pub(in crate::state) fn none() -> Counters {
        Counters(HashMap::new())
    }
}

impl std::ops::Deref for Counters {
    type Target = HashMap<CounterType, CounterStack>;
    fn deref(&self) -> &HashMap<CounterType, CounterStack> {
        &self.0
    }
}

impl<'a> IntoIterator for &'a Counters {
    type Item = (&'a CounterType, &'a CounterStack);
    type IntoIter = std::collections::hash_map::Iter<'a, CounterType, CounterStack>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Battlefield-specific state for a permanent.
///
/// This is stored separately from the GameObject itself — the object just knows
/// it's on the battlefield (via its `zone` field), and the engine looks up its
/// PermanentState here for mutable state like tapped/damage/counters.
#[derive(Debug, Clone)]
pub struct PermanentState {
    pub object_id: ObjectId,
    pub controller: PlayerId,

    /// **The battlefield order key — a copy of `GameObject::timestamp`, kept
    /// here because the ordered sweeps cannot afford to look it up.**
    ///
    /// CR 613.7's timestamp lives on the object, because CR 613.7d
    /// gives an object one for every zone it enters and a card in a graveyard
    /// needed one (`layers-architecture.md` §13d decision 2). That is the
    /// value the *rules* read — `GameState::static_effect_timestamp` reads the
    /// object, never this.
    ///
    /// This copy exists for one reader: `battlefield_ordered` and
    /// `battlefield_ids_ordered`, which run ~5,700 times a game over ~16
    /// permanents each (`codebase-state.md` item 77). Reading the object
    /// instead costs a `HashMap` hop per permanent per call, and that is
    /// **+16.5% of total game time, measured 2026-09-14 against an arm whose
    /// counters are identical** — not an estimate and not within the sitting's
    /// spread. With the copy the same tree measures +1.2%.
    ///
    /// **It cannot drift, and not because anything checks.**
    /// `GameState::set_object_timestamp` is the only writer of a timestamp on
    /// an object that has an entry, and it writes both; the only other write
    /// is `insert_battlefield_entity` filling this field in from the object as
    /// the entry is inserted, where there is nothing yet to disagree with.
    ///
    /// `codebase-state.md` item 77 is what deletes it: a *maintained* order
    /// vector needs no timestamp here at all, a PR of its own when its ~5% is
    /// worth one.
    pub timestamp: Timestamp,

    // Permanent state
    pub tapped: bool,
    pub flipped: bool,
    pub face_down: bool,
    pub phased_out: bool,
    /// The turn number when this permanent entered the battlefield, and so
    /// the turn its controller took control as it entered: CR 302.6's clock,
    /// seeded into `EffectiveCharacteristics::control_since_turn`, which a
    /// Layer 2 change advances. Convention: 0 = pregame (rule 103.6 Leylines).
    pub entered_battlefield_turn: u32,

    // Creature-specific (only meaningful if the permanent is a creature)
    pub damage_marked: u32,
    /// Set when this creature is dealt damage by a source with deathtouch,
    /// and read and cleared by the next state-based action check, which
    /// destroys a creature dealt such damage since the last (CR 704.5h).
    pub damaged_by_deathtouch: bool,

    // Combat state (transient, cleared at end of combat)
    pub attacking: Option<AttackingInfo>,
    pub blocking: Option<BlockingInfo>,

    /// Its counters (CR 122): read anywhere, written only inside `state/`
    /// ([`Counters`]).
    pub counters: Counters,

    /// The value of X chosen when this permanent was cast (rule 107.3f).
    /// Carried from StackEntry on resolution. None for non-X spells.
    pub x_value: Option<u64>,

    // Attachment tracking (rule 301.5, 303.4). Written only by
    // `GameState::attach` / `detach`: `attached_to` is a layer-walk input.
    /// The permanent this is attached to (for Auras, Equipment, Fortifications).
    pub attached_to: Option<ObjectId>,
    /// Permanents attached to this one (Auras, Equipment, Fortifications targeting this).
    pub attached_by: Vec<ObjectId>,

    /// CR 400.7d — who cast this permanent and from which zone, kept because
    /// the spell that became it is gone: `EntersBattlefield { was_cast }`, "if
    /// you cast it", Coal Stoker's "from your hand" read it. `None` for a
    /// permanent that was not cast — a land drop, a token, an effect's
    /// entry, the token a copy of a spell becomes. Written once, by the entry
    /// performer, off the stack entry the proposal's zone change came from
    /// (`codebase-state.md` main item 9).
    pub cast: Option<CastFacts>,

    /// CR 707.10's cost decisions for the spell this permanent was: kicked,
    /// bargained, evoked. Beside `cast` rather than in it, because a copy of
    /// a spell isn't cast and keeps them anyway: the token a copy of a kicked
    /// spell becomes is kicked (Archangel of Wrath's ruling). Empty for a
    /// permanent that was never a spell.
    pub cost_choices: CostChoices,

    /// CR 614.1c — what this permanent entered as: the copy and the
    /// characteristic edits its entry fixed.
    ///
    /// **State, not a registry row**, for the reason `tapped` and the counters
    /// are: it is how the permanent arrived, and it leaves with the permanent
    /// (CR 400.7) whichever permanent's ability made it. A row sourced at
    /// Essence of the Wild would end every copy when Essence left, which its
    /// rulings say does not happen, and a row sourced at Master Biomancer would
    /// take the Mutant away with it. The board pass applies each at this
    /// permanent's timestamp at its own layer (`layers::board`), and
    /// `register_static_effects` files the copy's abilities as the ones the
    /// permanent has.
    pub entered_as: EnteredAs,
    /// What the event stream last announced of its control: the controller
    /// it entered under, until `announce_control_changes` announces a change
    /// (CR 603.10d). Control is the layer walk's output, so this is what a
    /// change is a change from.
    pub announced_control: AnnouncedControl,
}

/// A permanent's control as the last control sweep left it, one value
/// whichever way it was read: a change of its controller is a change from
/// this controller, and looks back to this frame when there is one.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnouncedControl {
    /// Read with no control-changing effect in the game, so no frame was kept.
    Controller(PlayerId),
    /// Its characteristics as last read while a control-changing effect
    /// existed, whose controller is the one announced: what a change of its
    /// controller looks back to (CR 603.10d).
    Frame(Arc<EffectiveCharacteristics>),
}

impl AnnouncedControl {
    pub fn controller(&self) -> PlayerId {
        match self {
            AnnouncedControl::Controller(player) => *player,
            AnnouncedControl::Frame(frame) => frame.controller,
        }
    }

    pub fn frame(&self) -> Option<&Arc<EffectiveCharacteristics>> {
        match self {
            AnnouncedControl::Controller(_) => None,
            AnnouncedControl::Frame(frame) => Some(frame),
        }
    }
}

/// What a permanent entered as (CR 614.1c), once it has entered: the proposal
/// carries the same two members as `EnterMods::copy` and `EnterMods::edits`.
/// Each is shared, so a fork of the state clones a count, never the values.
#[derive(Debug, Clone, Default)]
pub struct EnteredAs {
    /// CR 707.5 — the copiable values, at layer 1a: Clone's choice, or Essence
    /// of the Wild's own values.
    pub copy: Option<std::sync::Arc<crate::engine::layers::copy::CopiableValues>>,
    /// CR 614.1c — each edit at its own layer, not copiable (CR 707.2's last
    /// sentence).
    pub edits: Option<std::sync::Arc<[crate::types::effects::CharacteristicEdit]>>,
}

impl EnteredAs {
    /// Did the entry fix nothing about the permanent's characteristics?
    pub fn is_empty(&self) -> bool {
        self.copy.is_none() && self.edits.is_none()
    }
}

/// CR 400.7d's facts about a permanent that was cast: who cast it, from
/// where, and the mana spent to pay its costs. A copy of a spell has none of
/// them — it isn't cast, and mana is not an object (CR 707.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastFacts {
    pub by: PlayerId,
    pub from: crate::types::zones::Zone,
    pub mana_spent: crate::types::mana::ManaSpent,
}

/// The decisions about a spell's costs that CR 707.10 copies with it, the way
/// it copies X: the additional costs paid and the alternative cost chosen.
#[derive(Debug, Clone, PartialEq)]
pub struct CostChoices {
    /// Kicked, bargained (CR 702.33d, 702.166b) — every additional cost paid,
    /// in printed order.
    pub additional: Vec<crate::types::costs::AdditionalCost>,
    /// Evoked, prowled, dashed (CR 118.9).
    pub alternative: Option<crate::types::costs::AlternativeCost>,
}

impl CostChoices {
    pub const NONE: CostChoices = CostChoices { additional: Vec::new(), alternative: None };
}

#[derive(Debug, Clone)]
pub struct AttackingInfo {
    pub target: AttackTarget,
    pub is_blocked: bool,
    pub blocked_by: Vec<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct BlockingInfo {
    pub blocking: Vec<ObjectId>,
}

/// What a creature attacks: CR 506.3, "only a player, a planeswalker, or a
/// battle can be attacked".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackTarget {
    Player(PlayerId),
    Planeswalker(ObjectId),
    Battle(ObjectId),
}

impl AttackTarget {
    /// The permanent attacked; `None` for a player.
    pub fn permanent(self) -> Option<ObjectId> {
        match self {
            AttackTarget::Player(_) => None,
            AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => Some(id),
        }
    }
}

impl PermanentState {
    pub fn new(object_id: ObjectId, controller: PlayerId, current_turn: u32) -> Self {
        PermanentState {
            object_id,
            controller,
            // Filled in by `Self::entering`: the object's own, where CR 613.7d
            // put it, for the performer, and the would-be value for the
            // look-ahead. A bare `PermanentState` is neither.
            timestamp: 0,
            tapped: false,
            flipped: false,
            face_down: false,
            phased_out: false,
            entered_battlefield_turn: current_turn,
            damage_marked: 0,
            damaged_by_deathtouch: false,
            attacking: None,
            blocking: None,
            counters: Counters::none(),
            x_value: None,
            attached_to: None,
            attached_by: Vec::new(),
            cast: None,
            cost_choices: CostChoices::NONE,
            entered_as: EnteredAs::default(),
            announced_control: AnnouncedControl::Controller(controller),
        }
    }

    /// CR 614.1c — the permanent `object` becomes, entering under
    /// `controller` with `mods`.
    ///
    /// **The one function that turns a proposal into an entity**, called by
    /// the performer (`GameState::perform_entry`) and by the CR 614.12
    /// look-ahead (`layers::Lookahead`), so the frame can never read a
    /// permanent the performer would not build. The two differ only in the
    /// timestamps they pass: the performer's allocated, the frame's predicted.
    /// Each counter row gets its own, from `first_counter` up (CR 613.7c).
    ///
    /// CR 400.7d's facts come off `GameState::resolving` when `object` is the
    /// resolving spell; a permanent that arrives any other way was never one.
    /// `arrived_on` starts CR 302.6's clock: the current turn in play, and a
    /// scenario's stated turn at construction.
    pub(crate) fn entering(
        game: &crate::state::game_state::GameState,
        object: ObjectId,
        controller: PlayerId,
        mods: &crate::types::replacement::EnterMods,
        timestamp: Timestamp,
        first_counter: Timestamp,
        arrived_on: u32,
    ) -> PermanentState {
        let mut entity = PermanentState::new(object, controller, arrived_on);
        entity.timestamp = timestamp;
        entity.tapped = mods.enters_tapped();
        entity.entered_as = EnteredAs {
            copy: mods.copy.as_ref().map(|copy| std::sync::Arc::clone(&copy.values)),
            edits: mods.edits.clone(),
        };
        for (stamp, row) in (first_counter..).zip(&mods.counters) {
            entity.add_counters(row.counter, row.n, stamp);
        }
        if let Some(r) = game.resolving.as_ref().filter(|r| r.id == object) {
            entity.cast = r.cast;
            entity.cost_choices = r.cost_choices.clone();
            entity.x_value = r.x_value;
        }
        entity
    }

    /// Add `n` counters of the given type, timestamped per CR 613.7c.
    ///
    /// `GameState::add_counters` is the route, which allocates the timestamp;
    /// this is the low-level form, for `state/`'s callers that already hold one.
    ///
    /// CR 613.7c's second sentence — "if that object already has a counter of
    /// that kind on it, each counter of that kind receives a new timestamp
    /// identical to that of the new counter" — is the `entry.timestamp =`
    /// below, and it is also why one timestamp per *kind* is exact rather than
    /// a simplification: counters of one kind can never disagree.
    pub(in crate::state) fn add_counters(&mut self, counter_type: CounterType, n: u32, timestamp: Timestamp) {
        let entry = self
            .counters
            .0
            .entry(counter_type)
            .or_insert(CounterStack { count: 0, timestamp });
        // No rule caps a count, but the type does.
        entry.count = entry.count.saturating_add(n);
        entry.timestamp = timestamp;
    }

    /// Remove up to `n` counters of the given type. Returns the number actually removed.
    pub(in crate::state) fn remove_counters(&mut self, counter_type: CounterType, n: u32) -> u32 {
        let Some(entry) = self.counters.0.get_mut(&counter_type) else {
            return 0;
        };
        let removed = entry.count.min(n);
        entry.count -= removed;
        if entry.count == 0 {
            self.counters.0.remove(&counter_type);
        }
        removed
    }

    /// Whether it carries a counter that generates a replacement effect (CR
    /// 122.1c/d/h), which is what files it in
    /// `GameState::replacement_counter_holders`.
    pub fn holds_a_replacement_counter(&self) -> bool {
        // ORDER-FREE: whether any does.
        self.counters.keys().any(|kind| kind.makes_a_replacement())
    }

    /// Returns the number of counters of the given type (0 if none).
    pub fn counter_count(&self, counter_type: CounterType) -> u32 {
        self.counters.get(&counter_type).map(|s| s.count).unwrap_or(0)
    }

    /// How many counters of `kind` it has, or of every kind for `None`.
    pub fn counters_of(&self, kind: Option<CounterType>) -> u32 {
        match kind {
            Some(kind) => self.counter_count(kind),
            // ORDER-FREE: a sum.
            None => self.counters.values().map(|stack| stack.count).sum(),
        }
    }

    /// The CR 613.7c timestamp shared by every counter of this kind, if any.
    pub fn counter_timestamp(&self, counter_type: CounterType) -> Option<Timestamp> {
        self.counters.get(&counter_type).map(|s| s.timestamp)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::new_object_id;

    fn make_permanent_state() -> PermanentState {
        PermanentState::new(new_object_id(), 0, 1)
    }

    #[test]
    fn test_add_counters() {
        let mut e = make_permanent_state();
        e.add_counters(CounterType::PlusOnePlusOne, 3, 1);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 3);
        e.add_counters(CounterType::PlusOnePlusOne, 2, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 5);
    }

    /// A count a scenario set at the most, then a counter put on in play.
    #[test]
    fn test_counters_past_what_a_count_holds_stay_at_the_most() {
        let mut e = make_permanent_state();
        e.add_counters(CounterType::Charge, u32::MAX, 1);
        e.add_counters(CounterType::Charge, 1, 2);
        assert_eq!(e.counter_count(CounterType::Charge), u32::MAX);
        assert_eq!(e.counters[&CounterType::Charge].timestamp, 2, "CR 613.7c: the put still stamps the kind");
    }

    #[test]
    fn test_remove_counters() {
        let mut e = make_permanent_state();
        e.add_counters(CounterType::PlusOnePlusOne, 3, 3);

        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 2);
        assert_eq!(removed, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 1);

        // Remove more than available — clamped at 0
        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 5);
        assert_eq!(removed, 1);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);

        // Remove from empty — returns 0
        let removed = e.remove_counters(CounterType::PlusOnePlusOne, 1);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_counter_count_default_zero() {
        let e = make_permanent_state();
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);
        assert_eq!(e.counter_count(CounterType::Flying), 0);
        assert_eq!(e.counter_count(CounterType::Loyalty), 0);
    }

    #[test]
    fn test_multiple_counter_types() {
        let mut e = make_permanent_state();
        e.add_counters(CounterType::PlusOnePlusOne, 2, 4);
        e.add_counters(CounterType::MinusOneMinusOne, 1, 5);
        e.add_counters(CounterType::Flying, 1, 6);
        e.add_counters(CounterType::Charge, 5, 7);

        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 2);
        assert_eq!(e.counter_count(CounterType::MinusOneMinusOne), 1);
        assert_eq!(e.counter_count(CounterType::Flying), 1);
        assert_eq!(e.counter_count(CounterType::Charge), 5);
        assert_eq!(e.counter_count(CounterType::Loyalty), 0);

        // Remove one type, others unaffected
        e.remove_counters(CounterType::PlusOnePlusOne, 2);
        assert_eq!(e.counter_count(CounterType::PlusOnePlusOne), 0);
        assert_eq!(e.counter_count(CounterType::MinusOneMinusOne), 1);
        assert_eq!(e.counter_count(CounterType::Flying), 1);
    }

    #[test]
    fn test_attachment_defaults_none() {
        let e = make_permanent_state();
        assert_eq!(e.attached_to, None);
        assert!(e.attached_by.is_empty());
    }
}
