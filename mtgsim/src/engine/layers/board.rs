//! The board-wide sequential pass — CR 613.3, 613.6, 613.7 and 604.2 applied
//! the way the CR states them (`layers-architecture.md` §13b, LI-1).
//!
//! CR 613 applies a layer to the whole board at once: each application's
//! reads — whether the ability generating it still exists, who "you" is, what
//! it applies to, what a count comes to — see everything applied *earlier in
//! the same layer*. The per-object walk this replaced answered every read of
//! another object at the end of the previous layer, which is exact exactly
//! while no application in a layer changes what a later one reads. Humility
//! beside Citanul Hierophants is the board in the pool that breaks it: the
//! grant's CR 604.2 existence check could not see Humility's strip, applied
//! earlier in layer 6, and a creature under Humility tapped for {G}.
//!
//! One [`Board`] is one pass: the working set, one live frame per member,
//! advanced layer by layer. The unit of ordering inside a layer is an
//! [`Application`] — one effect's rows in the layer, one member's own CDA,
//! or one of its counters — sorted once on a key that is CR 613.3 and
//! CR 613.7c read together, and then applied in CR 613.8's order (LI-2): the
//! key's, except that an application waits for anything it depends on, the
//! dependencies being decided against the live board and re-decided after
//! every application (`resolve_order_within_layer`). "Depends on" is
//! CR 613.8a(b) read literally — would applying the other change this one's
//! existence, what it applies to, or what it does — answered by a static
//! check on the frame fields each reads and writes (`Channels`) and, for the
//! pairs that leaves, by applying the other under a journal and looking
//! (`depends_on`).
//!
//! **What is a member: every object some row can reach**, read off the
//! `AffectedSet` variants. `Filter` and `Host` rows reach the battlefield;
//! `SourceOnly` rows reach their source, a permanent; `Fixed` rows name what
//! they name, anywhere. So: every battlefield entity, then the look-ahead's
//! entering object, then whatever `Fixed` rows name. A variant that reaches
//! another zone — `codebase-state.md` layers item 9, Wonder's graveyard
//! static — extends `Board::seed` by one clause. Everything else keeps a
//! walk of its own that applies only its CDAs (CR 604.3, all zones) and reads
//! a member's frame from the live board when nested inside a pass, or from
//! the memo otherwise (`compute::compute_non_member`).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{BitOr, BitOrAssign, Deref};
use std::sync::Arc;

use crate::engine::layers::cda;
use crate::engine::layers::compute::{
    apply_resolved, base_controller, compute_non_member, permanent_matches_filter,
    resolve_modification, seed_frame, FilterPlayers, Resolved, LAYER_ORDER,
};
use crate::engine::layers::lookahead::Lookahead;
use crate::engine::layers::types::*;
use crate::state::battlefield::PermanentState;
use crate::state::game_state::GameState;
use crate::types::card_types::Subtype;
use crate::types::effects::{AmountExpr, CounterType, PermanentFilter, PlayerRef, Selector};
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::zones::Zone;

/// One pass's state: the working set and its live frames.
///
/// Also the read-side view every evaluator takes. The two ways to hold one:
/// **live**, inside a pass, where a member's frame is the map's; and
/// **settled**, for a top-level non-member query, where there are no live
/// frames and a member's frame is the memo's (`compute_characteristics`).
/// The evaluators ask [`Board::frame_of`] and never care which.
pub(super) struct Board<'l> {
    /// Members in walk order: battlefield entities by CR 613.7 timestamp,
    /// then the entering object, then `Fixed`-named objects in row order.
    members: Vec<ObjectId>,
    /// How many of `members`, from the front, are battlefield entities —
    /// the prefix a count over the battlefield enumerates (§5b's boundary:
    /// the entering object is visible to filters and invisible to counts).
    entities: usize,
    frames: HashMap<ObjectId, EffectiveCharacteristics>,
    live: bool,
    /// CR 613.6 — the set of members a CR-level effect first applied to,
    /// which its later-layer rows apply to without re-running the filter or
    /// the existence check. Keyed by `EffectGroup`, not `EffectId`: March of
    /// the Machines is two rows and one effect.
    started: HashMap<EffectGroup, Vec<ObjectId>>,
    /// ...but only when some group has more than one row. For a single-row
    /// group the set is written after its only row and never read, and
    /// maintaining it was 70% of a static-heavy walk before this gate.
    track_started: bool,
    pub(super) lookahead: Option<&'l Lookahead>,
    /// Non-member frames at a layer ceiling, for reads that leave the
    /// working set — Tarmogoyf counting graveyard cards. Keyed the way the
    /// old per-call frame cache was, and bounded the way it was: a read at
    /// ceiling `c` only ever requests ceilings below `c`.
    sub: RefCell<HashMap<(ObjectId, usize), Arc<EffectiveCharacteristics>>>,
}

/// A frame handed out by [`Board::frame_of`]: borrowed from the live map, or
/// shared from the memo or the non-member cache.
pub(super) enum FrameRef<'a> {
    Live(&'a EffectiveCharacteristics),
    Shared(Arc<EffectiveCharacteristics>),
}

impl Deref for FrameRef<'_> {
    type Target = EffectiveCharacteristics;
    fn deref(&self) -> &EffectiveCharacteristics {
        match self {
            FrameRef::Live(frame) => frame,
            FrameRef::Shared(frame) => frame,
        }
    }
}

impl<'l> Board<'l> {
    /// The read-side view for a top-level non-member query: no live frames,
    /// members answered by the memo.
    pub(super) fn settled() -> Self {
        Board {
            members: Vec::new(),
            entities: 0,
            frames: HashMap::new(),
            live: false,
            started: HashMap::new(),
            track_started: false,
            lookahead: None,
            sub: RefCell::new(HashMap::new()),
        }
    }

    /// The working set, seeded from printed characteristics — layer 0.
    ///
    /// `asked` is the object the pass is for when that object is in the
    /// battlefield zone with no entity — a token whose entry is being decided,
    /// which the CR 614.17 predicate asks about through the plain entry. Such
    /// an object is a member of the pass that asks about it and of no other:
    /// it is invisible to a count, no row's affected set changes for its
    /// presence, and so every other member's frame is the same with it as
    /// without. That is also why two of them never need an order between
    /// them.
    fn seed(game: &GameState, lookahead: Option<&'l Lookahead>, asked: Option<ObjectId>) -> Self {
        let mut members = game.battlefield_ids_ordered();
        let entities = members.len();
        let mut seen: HashSet<ObjectId> = members.iter().copied().collect();
        if let Some(l) = lookahead {
            if seen.insert(l.object) {
                members.push(l.object);
            }
        }
        if let Some(id) = asked {
            if game.objects.contains_key(&id) && seen.insert(id) {
                members.push(id);
            }
        }
        for effect in game.continuous_effects.iter() {
            if let AffectedSet::Fixed(ids) = &effect.affected {
                for id in ids {
                    if game.objects.contains_key(id) && seen.insert(*id) {
                        members.push(*id);
                    }
                }
            }
        }

        let track_started = game.continuous_effects.summary().any_multi_row_group
            || lookahead.is_some_and(|l| l.summary.any_multi_row_group);

        let mut board = Board {
            members,
            entities,
            frames: HashMap::new(),
            live: true,
            started: HashMap::new(),
            track_started,
            lookahead,
            sub: RefCell::new(HashMap::new()),
        };
        for &id in &board.members {
            let Some(obj) = game.objects.get(&id) else { continue };
            game.counters.record_layer_frame();
            // The controller seed and CR 302.6's clock come from the entity —
            // the real one, or the one the performer would build for the
            // entering object — and from CR 108.4's other arms otherwise.
            let (controller, since) = match board.entity(game, id) {
                Some(entity) => (entity.controller, entity.controller_since_turn),
                None => (base_controller(game, id, lookahead).unwrap_or(obj.owner), 0),
            };
            let frame = seed_frame(&obj.card_data, controller, since);
            board.frames.insert(id, frame);
        }
        board
    }

    fn entering(&self, id: ObjectId) -> Option<&'l Lookahead> {
        self.lookahead.filter(|l| l.object == id)
    }

    /// **Accessor 1**: the battlefield entity a member seeds from and reads
    /// counters off — the real one for a permanent, the one the performer
    /// would build for the entering object (`Lookahead::entity`). `None` for
    /// anything that is neither.
    ///
    /// The look-ahead answers first even when a real entity exists for the
    /// same id: the caller asked what the object would be under the proposal.
    pub(super) fn entity<'a>(&self, game: &'a GameState, id: ObjectId) -> Option<&'a PermanentState>
    where
        'l: 'a,
    {
        if let Some(l) = self.entering(id) {
            return Some(&l.entity);
        }
        game.battlefield.get(&id)
    }

    /// Is `id` in the battlefield zone, or is it the object entering it?
    ///
    /// The gate a filter row asks before matching. The *zone* rather than
    /// entity membership (RC-3), which admits a token created in the zone
    /// with no entity yet; the look-ahead admits the entering object, still
    /// in its source zone while its entry is decided (RC-4b) — and nothing
    /// else. A `Fixed`-named member in a graveyard fails it.
    pub(super) fn in_battlefield_zone_or_entering(&self, game: &GameState, id: ObjectId) -> bool {
        self.entering(id).is_some()
            || matches!(game.objects.get(&id), Some(obj) if obj.zone == Zone::Battlefield)
    }

    fn has_frame(&self, id: ObjectId) -> bool {
        self.frames.contains_key(&id)
    }

    /// The battlefield, in timestamp order, for a count over it.
    pub(super) fn battlefield_ids(&self, game: &GameState) -> Vec<ObjectId> {
        if self.live {
            self.members[..self.entities].to_vec()
        } else {
            game.battlefield_ids_ordered()
        }
    }

    /// `id`'s frame as the walk should see it *now*: a member's live frame
    /// mid-pass, or its memoized frame from outside one; a non-member walked
    /// to `ceiling`, the frame as of the end of layer `ceiling - 1`.
    ///
    /// `None` only when `id` is not in the object store.
    pub(super) fn frame_of(&self, game: &GameState, id: ObjectId, ceiling: usize) -> Option<FrameRef<'_>> {
        if let Some(frame) = self.frames.get(&id) {
            return Some(FrameRef::Live(frame));
        }
        if !self.live && game.battlefield.contains_key(&id) {
            return crate::engine::layers::compute::compute_characteristics(game, id).map(FrameRef::Shared);
        }
        if let Some(frame) = self.sub.borrow().get(&(id, ceiling)) {
            return Some(FrameRef::Shared(Arc::clone(frame)));
        }
        let frame = Arc::new(compute_non_member(game, self, id, ceiling)?);
        self.sub.borrow_mut().insert((id, ceiling), Arc::clone(&frame));
        Some(FrameRef::Shared(frame))
    }

    /// Hand the frames over, for the memo. Order is a `HashMap`'s and is
    /// unobservable: the memo is keyed, never iterated.
    pub(super) fn into_frames(self) -> HashMap<ObjectId, EffectiveCharacteristics> {
        self.frames
    }

    /// One member's frame, consuming the pass.
    pub(super) fn take(mut self, id: ObjectId) -> Option<EffectiveCharacteristics> {
        self.frames.remove(&id)
    }
}

/// One thing a layer applies — CR 613.8's unit of ordering.
pub(super) struct Application<'a> {
    kind: Kind<'a>,
    timestamp: Timestamp,
    tiebreak: Tiebreak,
    /// CR 613.8a(b)'s questions as the channels answering them reads, fixed
    /// at the start of the layer. The static half of the dependency check.
    reads: Reads,
    /// The channels this application writes on whatever it applies to.
    writes: Channels,
}

enum Kind<'a> {
    /// One CR-level effect's rows in this layer, in id order — an
    /// `EffectGroup` restricted to the layer — or, `would_be`, the entering
    /// object's own rows under a look-ahead, which CR 614.12 clause (2)
    /// applies to the entering object alone.
    ///
    /// **The effect is the unit, not the row.** Ashaya's "Forest lands in
    /// addition to their other types" is one effect and two rows here
    /// (`AddType(Land)`, `AddSubtype(Forest)`); ordered row by row, Blood
    /// Moon could apply between them, and the CR 613.6 locked set would then
    /// paint Forest onto lands Blood Moon had just made Mountains. CR 613.8
    /// says "effect" throughout, and the judge answer in
    /// `plans/references/` applies Ashaya as one step.
    Effect { rows: Vec<&'a ContinuousEffect>, would_be: bool },
    /// One member's own application: its CDA (which must still be on it
    /// when its turn comes, CR 604.2), a keyword counter (CR 122.1b), or a
    /// P/T counter kind (CR 122.1a). Affects the member and nothing else.
    Own { object: ObjectId, cda: Option<AbilityId>, modification: EffectModification },
}

/// The last component of the sort key — process-independent, every arm
/// (CLAUDE.md, determinism). Rows and counters never share a timestamp
/// (both come from one counter), so only the first two arms ever decide a
/// tie: an object's several CDA modifications, and one object's several
/// effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tiebreak {
    /// Member index in walk order, then the modification's index in the
    /// object's CDA list.
    Cda(usize, usize),
    /// The effect's first registry id — assigned in registration order and
    /// never reused, which is CR 613.7a's "relative order remains the same".
    Row(EffectId),
    /// Index in `Lookahead::rows` of the effect's first row; those carry no id.
    WouldBeRow(usize),
    Keyword(KeywordFlag),
    Counter(u8),
}

impl Application<'_> {
    fn is_cda(&self) -> bool {
        matches!(self.kind, Kind::Own { cda: Some(_), .. })
    }

    /// CR 613.3 — CDAs first, then timestamp order; CR 613.7c puts counters
    /// on the same clock as rows, so they interleave rather than follow.
    fn key(&self) -> (bool, Timestamp, Tiebreak) {
        (!self.is_cda(), self.timestamp, self.tiebreak)
    }

    /// The object whose frame this application's existence and "you" are
    /// read from: an effect's source, or the member an `Own` belongs to.
    fn source_object(&self) -> ObjectId {
        match &self.kind {
            Kind::Effect { rows, .. } => rows[0].source,
            Kind::Own { object, .. } => *object,
        }
    }
}

// ---------------------------------------------------------------------------
// Channels — the static half of CR 613.8a(b).
//
// "Applying the other would change the text or the existence of the first
// effect, what it applies to, or what it does to any of the things it applies
// to." Each of those is a read of some frame field, and every modification
// writes a known set of them, so disjoint sets settle a pair without touching
// the board. Text is layer 3 and is not modelled.
// ---------------------------------------------------------------------------

/// A set of `EffectiveCharacteristics` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct Channels(u16);

impl Channels {
    const NONE: Channels = Channels(0);
    const TYPES: Channels = Channels(1 << 0);
    const SUBTYPES: Channels = Channels(1 << 1);
    const SUPERTYPES: Channels = Channels(1 << 2);
    const COLORS: Channels = Channels(1 << 3);
    const ABILITIES: Channels = Channels(1 << 4);
    const KEYWORDS: Channels = Channels(1 << 5);
    const CONTROLLER: Channels = Channels(1 << 6);
    const POWER: Channels = Channels(1 << 7);
    const TOUGHNESS: Channels = Channels(1 << 8);
    const MANA_COST: Channels = Channels(1 << 9);
    const ALL: Channels = Channels((1 << 10) - 1);

    fn intersects(self, other: Channels) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for Channels {
    type Output = Channels;
    fn bitor(self, rhs: Channels) -> Channels {
        Channels(self.0 | rhs.0)
    }
}

impl BitOrAssign for Channels {
    fn bitor_assign(&mut self, rhs: Channels) {
        self.0 |= rhs.0;
    }
}

/// Does gaining this subtype also gain an ability? CR 305.6: a land with a
/// basic land type has that type's mana ability, and `land_types` writes it.
fn subtype_writes_abilities(subtype: &Subtype) -> bool {
    matches!(subtype, Subtype::Land(lt) if lt.is_basic_land_type())
}

/// What a modification writes on each object it is applied to.
///
/// Over-approximate where the target decides: `AddSubtype(Forest)` on a
/// creature writes no ability, but the static check has no target in hand,
/// and an extra hypothetical costs a frame clone where a missed one costs a
/// wrong order.
fn writes_of(modification: &EffectModification) -> Channels {
    use EffectModification::*;
    match modification {
        CopyFrom(_) => Channels::ALL,
        SetController(_) => Channels::CONTROLLER,
        AddType(_) | RemoveType(_) | SetTypes(_) => Channels::TYPES,
        AddSubtype(s) => {
            if subtype_writes_abilities(s) {
                Channels::SUBTYPES | Channels::ABILITIES
            } else {
                Channels::SUBTYPES
            }
        }
        RemoveSubtype(_) => Channels::SUBTYPES,
        // CR 305.7 — setting a land to a basic land type strips its
        // abilities and keywords and grants a mana ability.
        SetSubtypes(set) => {
            if set.iter().any(subtype_writes_abilities) {
                Channels::SUBTYPES | Channels::ABILITIES | Channels::KEYWORDS
            } else {
                Channels::SUBTYPES
            }
        }
        AddSupertype(_) | RemoveSupertype(_) | SetSupertypes(_) => Channels::SUPERTYPES,
        AddColor(_) | SetColors(_) | RemoveAllColors => Channels::COLORS,
        GrantKeywordFlag(_) | RemoveKeywordFlag(_) => Channels::KEYWORDS,
        GrantAbility(_) | LoseAbility(_) => Channels::ABILITIES,
        LoseAllAbilities => Channels::ABILITIES | Channels::KEYWORDS,
        SetPowerToughness { .. } | ModifyPowerToughness { .. } | SwitchPowerToughness => {
            Channels::POWER | Channels::TOUGHNESS
        }
    }
}

/// What an application reads, split by whose frame: `source` is read off the
/// object the application belongs to (its existence, CR 604.2; its "you",
/// CR 109.5), `members` off every member it might apply to or count.
///
/// The split is what makes the static check sharp. A source read can only
/// change if the other application *reaches the source*, which is a
/// membership test on the other's targets; a member read can change if the
/// other reaches anything at all.
#[derive(Debug, Clone, Copy, Default)]
struct Reads {
    source: Channels,
    members: Channels,
}

/// The channels a filter's leaves read. `you` is what resolving "you" costs
/// on the source's frame: `CONTROLLER` for a static ability (CR 109.5 reads
/// the source's current controller off its live frame) and nothing for a
/// resolution row, whose "you" was fixed when it resolved.
fn filter_reads(filter: &PermanentFilter, out: &mut Reads, you: Channels) {
    match filter {
        PermanentFilter::All | PermanentFilter::Token | PermanentFilter::Other => {}
        PermanentFilter::ByType(_) => out.members |= Channels::TYPES,
        PermanentFilter::BySubtype(_) => out.members |= Channels::SUBTYPES,
        PermanentFilter::BySupertype(_) => out.members |= Channels::SUPERTYPES,
        PermanentFilter::ByColor(_) => out.members |= Channels::COLORS,
        PermanentFilter::ByController(player) => {
            out.members |= Channels::CONTROLLER;
            if matches!(player, PlayerRef::You | PlayerRef::Opponent) {
                out.source |= you;
            }
        }
        // Ownership is off the object, but "you" still resolves off the source.
        PermanentFilter::ByOwner(player) => {
            if matches!(player, PlayerRef::You | PlayerRef::Opponent) {
                out.source |= you;
            }
        }
        PermanentFilter::PowerLE(_) => out.members |= Channels::POWER,
        PermanentFilter::And(a, b) | PermanentFilter::Or(a, b) => {
            filter_reads(a, out, you);
            filter_reads(b, out, you);
        }
        PermanentFilter::Not(inner) => filter_reads(inner, out, you),
    }
}

/// The channels a dynamic amount reads, mirroring `compute::evaluate_amount`'s
/// arms: a count reads its filter's leaves on every member; a graveyard type
/// count reads types (of non-members, which no application reaches — kept
/// exact rather than clever); mana value is the affected object's own.
fn amount_reads(expr: &AmountExpr, out: &mut Reads, you: Channels) {
    match expr {
        AmountExpr::CountOf(Selector::PermanentsMatching(filter)) => filter_reads(filter, out, you),
        AmountExpr::CountOf(Selector::ControlledCreatures) => {
            out.members |= Channels::TYPES | Channels::CONTROLLER;
            out.source |= you;
        }
        AmountExpr::CardTypesAmong(_) => out.members |= Channels::TYPES,
        AmountExpr::AffectedManaValue => out.members |= Channels::MANA_COST,
        AmountExpr::Plus(inner, _) => amount_reads(inner, out, you),
        _ => {}
    }
}

/// What a modification reads while being resolved — "what it does to the
/// things it applies to". Only the three resolving arms read anything.
fn modification_reads(modification: &EffectModification, out: &mut Reads, you: Channels) {
    match modification {
        EffectModification::SetController(PlayerRef::You | PlayerRef::Opponent) => out.source |= you,
        EffectModification::SetPowerToughness { power, toughness }
        | EffectModification::ModifyPowerToughness { power, toughness } => {
            for value in [power, toughness] {
                if let PtValue::Dynamic(expr) = value {
                    amount_reads(expr, out, you);
                }
            }
        }
        _ => {}
    }
}

/// Does resolving this modification read the board at all? The cheap gate
/// on computing an `Observation`'s outcomes.
fn is_dynamic(modification: &EffectModification) -> bool {
    match modification {
        EffectModification::SetController(_) => true,
        EffectModification::SetPowerToughness { power, toughness }
        | EffectModification::ModifyPowerToughness { power, toughness } => {
            matches!(power, PtValue::Dynamic(_)) || matches!(toughness, PtValue::Dynamic(_))
        }
        _ => false,
    }
}

/// An effect's reads and writes for the layer, from its rows.
fn effect_channels(board: &Board<'_>, rows: &[&ContinuousEffect]) -> (Reads, Channels) {
    let first = rows[0];
    let is_static = matches!(first.origin, EffectOrigin::StaticAbility { .. });
    let you = if is_static { Channels::CONTROLLER } else { Channels::NONE };
    let mut reads = Reads::default();
    // CR 613.6 — an effect that started in an earlier layer has its set and
    // its existence locked; only what it does can still depend on anything.
    let locked = board.track_started && board.started.contains_key(&first.group());
    if !locked {
        if is_static {
            reads.source |= Channels::ABILITIES;
        }
        if let AffectedSet::Filter { filter } = &first.affected {
            filter_reads(filter, &mut reads, you);
        }
    }
    let mut writes = Channels::NONE;
    for row in rows {
        modification_reads(&row.modification, &mut reads, you);
        writes |= writes_of(&row.modification);
    }
    (reads, writes)
}

/// The rows that apply in `layer`: the registry's slice in CR 613.7 order,
/// then the entering object's would-be rows (CR 614.12 clause 2), which
/// carry the timestamp it would get and so sort last.
fn rows_in_layer<'a>(
    game: &'a GameState,
    lookahead: Option<&'a Lookahead>,
    layer: Layer,
) -> impl Iterator<Item = (&'a ContinuousEffect, Option<usize>)> + 'a {
    let own: &'a [ContinuousEffect] = lookahead.map(|l| l.rows.as_slice()).unwrap_or(&[]);
    game.continuous_effects
        .effects_in_layer(layer)
        .iter()
        .map(|row| (row, None))
        .chain(
            own.iter()
                .enumerate()
                .filter(move |(_, row)| row.layer == layer)
                .map(|(i, row)| (row, Some(i))),
        )
}

/// Everything `layer` applies, sorted on the one key.
fn applications_in_layer<'a, 'l: 'a>(
    game: &'a GameState,
    board: &Board<'l>,
    layer: Layer,
) -> Vec<Application<'a>> {
    let mut apps: Vec<Application<'a>> = Vec::new();

    // CR 613.3 — a member's own CDAs, read off its live ability list at the
    // start of the layer: that list is what earlier layers have already done
    // to it, which is how Humility strips Tarmogoyf's CDA before 7a reads it.
    if cda::CDA_LAYERS.contains(&layer) {
        for (index, &object) in board.members.iter().enumerate() {
            let frame = &board.frames[&object];
            let timestamp = board.entity(game, object).map(|e| e.timestamp).unwrap_or(Timestamp::MAX);
            for (i, (ability, modification)) in cda::cda_modifications(frame, layer).into_iter().enumerate() {
                // A CDA's "you" is the object's own controller (CR 109.5),
                // which is its own frame's — the source, here.
                let mut reads = Reads { source: Channels::ABILITIES, members: Channels::NONE };
                modification_reads(&modification, &mut reads, Channels::CONTROLLER);
                let writes = writes_of(&modification);
                apps.push(Application {
                    kind: Kind::Own { object, cda: Some(ability), modification },
                    timestamp,
                    tiebreak: Tiebreak::Cda(index, i),
                    reads,
                    writes,
                });
            }
        }
    }

    // Rows bundled into effects: one `EffectGroup` per layer, rows in id
    // order. First-seen order here is unobservable — the sort below decides.
    let mut by_group: HashMap<(EffectGroup, bool), usize> = HashMap::new();
    let mut effects: Vec<(Vec<&'a ContinuousEffect>, Option<usize>)> = Vec::new();
    for (effect, would_be) in rows_in_layer(game, board.lookahead, layer) {
        let key = (effect.group(), would_be.is_some());
        match by_group.get(&key) {
            Some(&i) => effects[i].0.push(effect),
            None => {
                by_group.insert(key, effects.len());
                effects.push((vec![effect], would_be));
            }
        }
    }
    for (rows, would_be) in effects {
        let first = rows[0];
        debug_assert!(
            rows.iter().all(|r| r.timestamp == first.timestamp),
            "one effect's rows carry one timestamp (CR 613.7a/b)"
        );
        let (reads, writes) = effect_channels(board, &rows);
        apps.push(Application {
            kind: Kind::Effect { rows, would_be: would_be.is_some() },
            timestamp: first.timestamp,
            tiebreak: would_be.map(Tiebreak::WouldBeRow).unwrap_or(Tiebreak::Row(first.id)),
            reads,
            writes,
        });
    }

    // Keyword counters (CR 122.1b) are a second source of layer 6 effects,
    // and CR 613.7c timestamps them; P/T counters (CR 122.1a) are layer 7c's
    // (CR 613.4c) on the same clock. Both read off accessor 1, so the
    // entering object's are the counters it would enter with.
    if layer == Layer::Layer6Ability || layer == Layer::Layer7cModifyPT {
        for &object in &board.members {
            let Some(entity) = board.entity(game, object) else { continue };
            if entity.counters.is_empty() {
                continue;
            }
            for (kind, stack) in &entity.counters {
                if stack.count == 0 {
                    continue;
                }
                match layer {
                    Layer::Layer6Ability => {
                        if let Some(keyword) = kind.keyword_granted() {
                            let modification = EffectModification::GrantKeywordFlag(keyword);
                            let writes = writes_of(&modification);
                            apps.push(Application {
                                kind: Kind::Own { object, cda: None, modification },
                                timestamp: stack.timestamp,
                                tiebreak: Tiebreak::Keyword(keyword),
                                reads: Reads::default(),
                                writes,
                            });
                        }
                    }
                    Layer::Layer7cModifyPT => {
                        // TODO: other P/T-modifying counter kinds (+2/+2, +0/+1)
                        // when `CounterType` grows them — `codebase-state.md`,
                        // "Before card breadth" item 3.
                        let (delta, rank) = match kind {
                            CounterType::PlusOnePlusOne => (stack.count as i32, 0),
                            CounterType::MinusOneMinusOne => (-(stack.count as i32), 1),
                            _ => continue,
                        };
                        let modification = EffectModification::ModifyPowerToughness {
                            power: PtValue::Fixed(delta),
                            toughness: PtValue::Fixed(delta),
                        };
                        let writes = writes_of(&modification);
                        apps.push(Application {
                            kind: Kind::Own { object, cda: None, modification },
                            timestamp: stack.timestamp,
                            tiebreak: Tiebreak::Counter(rank),
                            reads: Reads::default(),
                            writes,
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    apps.sort_by_key(Application::key);
    apps
}

/// CR 604.2 — does the static ability that generates `effect` still exist,
/// *now*, on the source's frame as the pass has built it so far? ("These
/// effects are active as long as the permanent with the ability remains on
/// the battlefield and has the ability"; CR 611.3b says the same.)
///
/// Registry membership does not answer this: the row was registered when the
/// permanent entered, and CR 305.7 or Layer 6 can take the ability away later
/// without touching the registry — including earlier in this very layer,
/// which is the read the per-object walk could not make.
///
/// `EffectOrigin::Resolution` effects (CR 613.7b) always exist: a resolution
/// already happened and cannot be taken back.
///
/// Existence is not the same as surviving, and only existence is decided
/// here. An instant that grants first strike until end of turn creates an
/// effect that exists for the turn no matter what — but Humility, applying
/// later in layer 6, still clears the keyword it granted. That is ordering
/// inside a layer, which CR 613.8 and the sort key decide.
fn static_ability_still_exists(
    game: &GameState,
    board: &Board<'_>,
    effect: &ContinuousEffect,
    layer_index: usize,
) -> bool {
    let ability_id = match effect.origin {
        EffectOrigin::Resolution => return true,
        EffectOrigin::StaticAbility { ability } => ability,
    };
    match board.frame_of(game, effect.source, layer_index) {
        Some(frame) => frame.abilities.iter().any(|a| a.id == ability_id),
        // Source is gone from the object store entirely.
        None => false,
    }
}

/// CR 604.2 for a CDA: still on the object, and still characteristic-defining
/// there, when its turn comes.
fn cda_still_there(board: &Board<'_>, object: ObjectId, ability: AbilityId) -> bool {
    board.frames[&object]
        .abilities
        .iter()
        .any(|a| a.id == ability && a.is_characteristic_defining)
}

/// The members `effect` applies to, decided against the live board (CR
/// 611.2c's "determined when the effect begins" is per application here).
fn affected_members(
    game: &GameState,
    board: &Board<'_>,
    effect: &ContinuousEffect,
    would_be: bool,
    layer_index: usize,
) -> Vec<ObjectId> {
    match &effect.affected {
        AffectedSet::SourceOnly => {
            if board.has_frame(effect.source) { vec![effect.source] } else { Vec::new() }
        }
        AffectedSet::Fixed(ids) => ids.iter().copied().filter(|id| board.has_frame(*id)).collect(),
        // CR 303.4m — whatever the source enchants *now*, read off the entity
        // at every layer. `attached_to` only ever names a permanent
        // (`cleanup_zone_state` clears it when the host leaves), so no zone
        // gate is needed; an unattached source matches nothing.
        AffectedSet::Host => game
            .battlefield
            .get(&effect.source)
            .and_then(|e| e.attached_to)
            .filter(|host| board.has_frame(*host))
            .into_iter()
            .collect(),
        AffectedSet::Filter { filter } => {
            // §5b's asymmetry (`replacement-architecture.md`): the entering
            // object's own row is in its frame and reaches no other member,
            // because it is not on the battlefield yet and CR 604.3 makes
            // its static abilities function there.
            let candidates: &[ObjectId] = if would_be {
                std::slice::from_ref(&effect.source)
            } else {
                &board.members
            };
            let mut players = FilterPlayers::for_row(effect, game, board, layer_index);
            candidates
                .iter()
                .copied()
                .filter(|&id| {
                    // In the battlefield zone, checked first so a `Fixed`-named
                    // graveyard card costs no filter evaluation.
                    board.in_battlefield_zone_or_entering(game, id)
                        && permanent_matches_filter(filter, id, &board.frames[&id], &mut players)
                })
                .collect()
        }
    }
}

/// Where a row's members come from, right now.
enum RowPlan {
    /// The generating ability is gone (CR 604.2): the row applies to nothing.
    Gone,
    /// CR 613.6 — the effect started in an earlier layer, or an earlier row
    /// of it in this one; this is the set it locked, and neither the filter
    /// nor the existence check is asked again.
    Locked(Vec<ObjectId>),
    /// Decided against the live board just now.
    Fresh(Vec<ObjectId>),
}

fn plan_row(
    game: &GameState,
    board: &Board<'_>,
    effect: &ContinuousEffect,
    would_be: bool,
    layer_index: usize,
) -> RowPlan {
    if board.track_started {
        if let Some(locked) = board.started.get(&effect.group()) {
            return RowPlan::Locked(locked.clone());
        }
    }
    if !static_ability_still_exists(game, board, effect, layer_index) {
        return RowPlan::Gone;
    }
    RowPlan::Fresh(affected_members(game, board, effect, would_be, layer_index))
}

/// What `app` would apply to if it applied now. Empty when its ability is
/// gone, when its filter matches nothing, or when a CDA has been stripped.
fn targets_of(game: &GameState, board: &Board<'_>, layer_index: usize, app: &Application<'_>) -> Vec<ObjectId> {
    match &app.kind {
        Kind::Effect { rows, would_be } => match plan_row(game, board, rows[0], *would_be, layer_index) {
            RowPlan::Gone => Vec::new(),
            RowPlan::Locked(t) | RowPlan::Fresh(t) => t,
        },
        Kind::Own { object, cda, .. } => {
            if cda.is_some_and(|a| !cda_still_there(board, *object, a)) {
                Vec::new()
            } else {
                vec![*object]
            }
        }
    }
}

/// The pre-images a hypothetical application overwrote, so it can be taken
/// back: each frame it touched (saved before its first write), and each CR
/// 613.6 lock it recorded (with what was there, if anything).
///
/// The hypothetical applies to the live board through the same functions
/// the real application uses, so every read the check makes afterwards —
/// existence, "you", the filter, a count — sees it without a second code
/// path; this is what makes that safe.
#[derive(Default)]
struct Journal {
    frames: Vec<(ObjectId, EffectiveCharacteristics)>,
    locks: Vec<(EffectGroup, Option<Vec<ObjectId>>)>,
}

impl Journal {
    fn save_frame(&mut self, board: &Board<'_>, id: ObjectId) {
        if !self.frames.iter().any(|(saved, _)| *saved == id) {
            self.frames.push((id, board.frames[&id].clone()));
        }
    }

    fn restore(self, board: &mut Board<'_>) {
        for (group, previous) in self.locks.into_iter().rev() {
            match previous {
                Some(set) => board.started.insert(group, set),
                None => board.started.remove(&group),
            };
        }
        for (id, frame) in self.frames {
            board.frames.insert(id, frame);
        }
    }
}

/// Apply one modification to `targets`, resolving each against the board
/// before any frame is written.
fn write_targets(
    game: &GameState,
    board: &mut Board<'_>,
    layer_index: usize,
    modification: &EffectModification,
    origin: Option<&ContinuousEffect>,
    targets: &[ObjectId],
    journal: &mut Option<Journal>,
) {
    for &target in targets {
        // Resolved before the frame is mutated: a dynamic amount may read the
        // member being modified — a creature counting "creatures you control"
        // counts itself — and a half-applied frame must not be what it sees.
        let resolved = resolve_modification(modification, game, board, target, layer_index, origin);
        if let Some(journal) = journal.as_mut() {
            journal.save_frame(board, target);
        }
        let frame = board.frames.get_mut(&target).expect("a target is a member");
        apply_resolved(&resolved, frame, target);
    }
}

/// One application, applied: what it belonged to and what it reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Applied {
    pub(super) source: ObjectId,
    pub(super) targets: Vec<ObjectId>,
}

/// Apply `app` to every member it affects — for real, or under a `journal`
/// that lets it be taken back.
fn apply_app(
    game: &GameState,
    board: &mut Board<'_>,
    layer_index: usize,
    app: &Application<'_>,
    journal: &mut Option<Journal>,
) -> Applied {
    match &app.kind {
        Kind::Effect { rows, would_be } => {
            let mut reached: Vec<ObjectId> = Vec::new();
            for row in rows {
                let (targets, lock) = match plan_row(game, board, row, *would_be, layer_index) {
                    RowPlan::Gone => continue,
                    RowPlan::Locked(targets) => (targets, None),
                    RowPlan::Fresh(targets) => (targets, board.track_started.then(|| row.group())),
                };
                if let Some(group) = lock {
                    // Recorded even when empty: CR 613.6 locks the set at the
                    // layer the effect starts in, and an effect that found
                    // nothing there finds nothing later either.
                    if let Some(journal) = journal.as_mut() {
                        journal.locks.push((group, board.started.get(&group).cloned()));
                    }
                    board.started.insert(group, targets.clone());
                }
                write_targets(game, board, layer_index, &row.modification, Some(row), &targets, journal);
                for target in targets {
                    if !reached.contains(&target) {
                        reached.push(target);
                    }
                }
            }
            Applied { source: rows[0].source, targets: reached }
        }
        Kind::Own { object, cda, modification } => {
            if cda.is_some_and(|a| !cda_still_there(board, *object, a)) {
                return Applied { source: *object, targets: Vec::new() };
            }
            write_targets(game, board, layer_index, modification, None, &[*object], journal);
            Applied { source: *object, targets: vec![*object] }
        }
    }
}

// ---------------------------------------------------------------------------
// The hypothetical — CR 613.8a(b) read literally.
// ---------------------------------------------------------------------------

/// A resolving arm's answer, comparable. `AsIs` modifications carry their own
/// answer and are never observed.
#[derive(Debug, PartialEq)]
enum Outcome {
    Controller(Option<(PlayerId, u32)>),
    SetPt(Option<(i32, i32)>),
    ModifyPt(Option<i32>, Option<i32>),
}

/// CR 613.8a(b)'s three questions about one application, answered against
/// the board as it is: does the effect exist, what does it apply to, and
/// what does it do to each of those things. Two observations of the same
/// application, one on either side of a hypothetical, differ exactly when
/// the hypothetical is a dependency.
#[derive(Debug, PartialEq)]
struct Observation {
    exists: bool,
    targets: Vec<ObjectId>,
    does: Vec<Outcome>,
}

fn observe(game: &GameState, board: &Board<'_>, layer_index: usize, app: &Application<'_>) -> Observation {
    let (exists, targets, modifications): (bool, Vec<ObjectId>, Vec<(&EffectModification, Option<&ContinuousEffect>)>) =
        match &app.kind {
            Kind::Effect { rows, would_be } => {
                let (exists, targets) = match plan_row(game, board, rows[0], *would_be, layer_index) {
                    RowPlan::Gone => (false, Vec::new()),
                    RowPlan::Locked(t) | RowPlan::Fresh(t) => (true, t),
                };
                (exists, targets, rows.iter().map(|r| (&r.modification, Some(*r))).collect())
            }
            Kind::Own { object, cda, modification } => {
                let exists = !cda.is_some_and(|a| !cda_still_there(board, *object, a));
                let targets = if exists { vec![*object] } else { Vec::new() };
                (exists, targets, vec![(modification, None)])
            }
        };
    let mut does = Vec::new();
    for (modification, origin) in modifications {
        if !is_dynamic(modification) {
            continue;
        }
        for &target in &targets {
            let outcome = match resolve_modification(modification, game, board, target, layer_index, origin) {
                Resolved::AsIs(_) => continue,
                Resolved::Controller(c) => Outcome::Controller(c),
                Resolved::SetPt(pt) => Outcome::SetPt(pt),
                Resolved::ModifyPt(p, t) => Outcome::ModifyPt(p, t),
            };
            does.push(outcome);
        }
    }
    Observation { exists, targets, does }
}

/// CR 613.8a — does `a` depend on `b`?
///
/// Clause (a), the same layer, holds by construction. Clause (c) is a flag
/// comparison. Clause (b) is decided in two steps: the static channel check
/// (what `a` reads against what `b` writes, and — for a read of `a`'s own
/// source — whether `b` reaches that source at all), then, for the pairs it
/// cannot settle, the hypothetical: apply `b` to the live board under a
/// journal, observe `a` again, take `b` back. "The dependency has to be
/// actual for the current game state being built, not theoretical" (the
/// judge answer in `plans/references/`): Blood Moon depends on Ashaya only
/// while there is a nontoken creature for Ashaya to reach.
///
/// `b_targets` memoizes `b`'s targets across the pairs of one iteration.
fn depends_on(
    game: &GameState,
    board: &mut Board<'_>,
    layer_index: usize,
    a: &Application<'_>,
    b: &Application<'_>,
    b_targets: &mut Option<Vec<ObjectId>>,
) -> bool {
    // CR 613.8a(c).
    if a.is_cda() != b.is_cda() {
        return false;
    }
    let on_source = a.reads.source.intersects(b.writes);
    let on_members = a.reads.members.intersects(b.writes);
    if !on_source && !on_members {
        return false;
    }
    let targets = b_targets.get_or_insert_with(|| targets_of(game, board, layer_index, b));
    if targets.is_empty() {
        return false;
    }
    if !on_members && !targets.contains(&a.source_object()) {
        return false;
    }

    game.counters.record_dependency_check();
    let before = observe(game, board, layer_index, a);
    let mut journal = Some(Journal::default());
    apply_app(game, board, layer_index, b, &mut journal);
    let after = observe(game, board, layer_index, a);
    journal.expect("the journal was handed in").restore(board);
    before != after
}

/// Which pending application applies next — a position in `pending`, which
/// is in key order.
///
/// CR 613.8b: an application with no unapplied dependency is ready; the
/// members of a dependency loop are ready together and apply in timestamp
/// order. Both are "in a source component of the dependency graph's
/// condensation", and the earliest such application by the layer's key is
/// the next one — which is also what "waits to apply until just after all of
/// those effects have been applied" gives, since the moment the last
/// dependency applies the waiter is a candidate and sorts by its own key.
///
/// The fast path is the common case: the key-first application depends on
/// nothing, which the channel check almost always settles without a
/// hypothetical, and then it is next. Only when it does depend on something
/// is the whole graph built — over a handful of applications, so the closure
/// is Floyd–Warshall rather than anything cleverer.
fn next_application(
    game: &GameState,
    board: &mut Board<'_>,
    layer_index: usize,
    apps: &[Application<'_>],
    pending: &[usize],
) -> usize {
    let m = pending.len();
    if m == 1 {
        return 0;
    }
    let mut targets: Vec<Option<Vec<ObjectId>>> = (0..m).map(|_| None).collect();
    let mut depends = vec![vec![false; m]; m];

    let head = &apps[pending[0]];
    let mut head_waits = false;
    for j in 1..m {
        depends[0][j] = depends_on(game, board, layer_index, head, &apps[pending[j]], &mut targets[j]);
        head_waits |= depends[0][j];
    }
    if !head_waits {
        return 0;
    }

    for i in 1..m {
        for j in 0..m {
            if i != j {
                depends[i][j] =
                    depends_on(game, board, layer_index, &apps[pending[i]], &apps[pending[j]], &mut targets[j]);
            }
        }
    }
    // Transitive closure: `depends[i][j]` becomes "i waits, directly or
    // through others, on j".
    for k in 0..m {
        for i in 0..m {
            if depends[i][k] {
                for j in 0..m {
                    if depends[k][j] {
                        depends[i][j] = true;
                    }
                }
            }
        }
    }
    // A source component: everything i waits on waits back on i.
    for i in 0..m {
        if (0..m).all(|j| j == i || !depends[i][j] || depends[j][i]) {
            return i;
        }
    }
    unreachable!("a finite dependency graph's condensation has a source component")
}

/// Apply `layer` — CR 613.8's loop: decide dependencies against the live
/// board, apply the earliest ready application, re-evaluate (CR 613.8c),
/// until nothing is pending.
///
/// It applies as it orders, which is why §9's reserved `Vec<EffectId>`
/// return could not exist: after the k-th application the order of the rest
/// is a function of the first k. `trace`, when given, receives what each
/// application reached, in the order applied.
fn resolve_order_within_layer(
    game: &GameState,
    board: &mut Board<'_>,
    layer_index: usize,
    apps: Vec<Application<'_>>,
    mut trace: Option<&mut Vec<Applied>>,
) {
    let mut pending: Vec<usize> = (0..apps.len()).collect();
    while !pending.is_empty() {
        let next = next_application(game, board, layer_index, &apps, &pending);
        let index = pending.remove(next);
        let applied = apply_app(game, board, layer_index, &apps[index], &mut None);
        if let Some(trace) = trace.as_deref_mut() {
            trace.push(applied);
        }
    }
}

/// One pass under a look-ahead: the working set plus the entering object,
/// through every layer.
pub(super) fn compute_board<'l>(game: &GameState, lookahead: Option<&'l Lookahead>) -> Board<'l> {
    compute_board_to(game, lookahead, None, LAYER_ORDER.len())
}

/// One pass stopped at `ceiling`: layers `LAYER_ORDER[..ceiling]` applied,
/// the frames as of the end of layer `ceiling - 1`. CR 613.2c's copiable
/// values are the one reader that wants less than the whole walk. `asked`
/// is [`Board::seed`]'s.
pub(super) fn compute_board_to<'l>(
    game: &GameState,
    lookahead: Option<&'l Lookahead>,
    asked: Option<ObjectId>,
    ceiling: usize,
) -> Board<'l> {
    compute_board_traced(game, lookahead, asked, ceiling, None)
}

/// [`compute_board_to`], recording the order one layer applied its
/// applications in — the test hook for CR 613.8's sequence.
pub(super) fn compute_board_traced<'l>(
    game: &GameState,
    lookahead: Option<&'l Lookahead>,
    asked: Option<ObjectId>,
    ceiling: usize,
    mut trace: Option<(usize, &mut Vec<Applied>)>,
) -> Board<'l> {
    game.counters.record_board_walk();
    let mut board = Board::seed(game, lookahead, asked);
    for (layer_index, &layer) in LAYER_ORDER.iter().enumerate().take(ceiling) {
        let apps = applications_in_layer(game, &board, layer);
        let sink: Option<&mut Vec<Applied>> = match trace.as_mut() {
            Some((traced, sink)) if *traced == layer_index => Some(sink),
            _ => None,
        };
        resolve_order_within_layer(game, &mut board, layer_index, apps, sink);
    }
    board
}

/// Whether `id` belongs to the working set, which decides how the
/// top-level entry computes it.
pub(super) enum Membership {
    /// A battlefield entity, or an object a `Fixed` row names: a member of
    /// every pass. Rows are scanned rather than summarised — `Fixed` rows
    /// are few, and a miss is already a walk.
    Member,
    /// In the battlefield zone with no entity: a member of the pass that
    /// asks about it (see [`Board::seed`]).
    ZoneOnly,
    /// Reachable by no row: its own CDA walk.
    NonMember,
}

pub(super) fn membership(game: &GameState, id: ObjectId) -> Membership {
    if game.battlefield.contains_key(&id) {
        return Membership::Member;
    }
    if matches!(game.objects.get(&id), Some(obj) if obj.zone == Zone::Battlefield) {
        return Membership::ZoneOnly;
    }
    let fixed_named = game
        .continuous_effects
        .iter()
        .any(|e| matches!(&e.affected, AffectedSet::Fixed(ids) if ids.contains(&id)));
    if fixed_named { Membership::Member } else { Membership::NonMember }
}

/// `id`'s frame as of the end of layer `ceiling - 1`, from outside any pass:
/// through a pass stopped there for a member, through its own walk otherwise.
pub(super) fn frame_at_ceiling(game: &GameState, id: ObjectId, ceiling: usize) -> Option<EffectiveCharacteristics> {
    game.objects.get(&id)?;
    match membership(game, id) {
        Membership::Member => compute_board_to(game, None, None, ceiling).take(id),
        Membership::ZoneOnly => compute_board_to(game, None, Some(id), ceiling).take(id),
        Membership::NonMember => compute_non_member(game, &Board::settled(), id, ceiling),
    }
}

