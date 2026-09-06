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
//! [`Application`] — a registry row, one member's own CDA, or one of its
//! counters — sorted once on a key that is CR 613.3 and CR 613.7c read
//! together. Dependency (CR 613.8) is LI-2's; here the order is the key's.
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
use std::ops::Deref;
use std::sync::Arc;

use crate::engine::layers::cda;
use crate::engine::layers::compute::{
    apply_resolved, base_controller, compute_non_member, permanent_matches_filter,
    resolve_modification, seed_frame, FilterPlayers, LAYER_ORDER,
};
use crate::engine::layers::lookahead::Lookahead;
use crate::engine::layers::types::*;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::{AbilityId, ObjectId};
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
    pub(super) fn entity<'a>(&self, game: &'a GameState, id: ObjectId) -> Option<&'a BattlefieldEntity>
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

/// One thing a layer applies, in the order CR 613.3 and 613.7 give it.
pub(super) struct Application<'a> {
    kind: Kind<'a>,
    timestamp: Timestamp,
    tiebreak: Tiebreak,
}

enum Kind<'a> {
    /// A registry row, or — `would_be` — one of the entering object's own
    /// rows under a look-ahead, which CR 614.12 clause (2) applies to the
    /// entering object alone.
    Row { effect: &'a ContinuousEffect, would_be: bool },
    /// One member's own application: its CDA (which must still be on it
    /// when its turn comes, CR 604.2), a keyword counter (CR 122.1b), or a
    /// P/T counter kind (CR 122.1a). Affects the member and nothing else.
    Own { object: ObjectId, cda: Option<AbilityId>, modification: EffectModification },
}

/// The last component of the sort key — process-independent, every arm
/// (CLAUDE.md, determinism). Rows and counters never share a timestamp
/// (both come from one counter), so only the first two arms ever decide a
/// tie: an object's several CDA modifications, and its several rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tiebreak {
    /// Member index in walk order, then the modification's index in the
    /// object's CDA list.
    Cda(usize, usize),
    /// The registry id — assigned in registration order and never reused,
    /// which is CR 613.7a's "relative order remains the same".
    Row(EffectId),
    /// Index in `Lookahead::rows`, which carry no id.
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
                apps.push(Application {
                    kind: Kind::Own { object, cda: Some(ability), modification },
                    timestamp,
                    tiebreak: Tiebreak::Cda(index, i),
                });
            }
        }
    }

    for (effect, would_be) in rows_in_layer(game, board.lookahead, layer) {
        apps.push(Application {
            kind: Kind::Row { effect, would_be: would_be.is_some() },
            timestamp: effect.timestamp,
            tiebreak: would_be.map(Tiebreak::WouldBeRow).unwrap_or(Tiebreak::Row(effect.id)),
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
                            apps.push(Application {
                                kind: Kind::Own {
                                    object,
                                    cda: None,
                                    modification: EffectModification::GrantKeywordFlag(keyword),
                                },
                                timestamp: stack.timestamp,
                                tiebreak: Tiebreak::Keyword(keyword),
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
                        apps.push(Application {
                            kind: Kind::Own {
                                object,
                                cda: None,
                                modification: EffectModification::ModifyPowerToughness {
                                    power: PtValue::Fixed(delta),
                                    toughness: PtValue::Fixed(delta),
                                },
                            },
                            timestamp: stack.timestamp,
                            tiebreak: Tiebreak::Counter(rank),
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
/// inside a layer, which the sort key and (LI-2) CR 613.8 decide.
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

/// Apply one application to every member it affects.
fn apply_one(game: &GameState, board: &mut Board<'_>, layer_index: usize, app: &Application<'_>) {
    let (targets, lock, modification, origin): (
        Vec<ObjectId>,
        Option<EffectGroup>,
        &EffectModification,
        Option<&ContinuousEffect>,
    ) = match &app.kind {
        Kind::Row { effect, would_be } => {
            let group = effect.group();
            let locked = if board.track_started { board.started.get(&group).cloned() } else { None };
            match locked {
                // CR 613.6 — the effect started applying in an earlier layer;
                // this row applies to that set, and neither the filter nor the
                // existence check is asked again.
                Some(targets) => (targets, None, &effect.modification, Some(*effect)),
                None => {
                    if !static_ability_still_exists(game, board, effect, layer_index) {
                        return;
                    }
                    let targets = affected_members(game, board, effect, *would_be, layer_index);
                    let lock = board.track_started.then_some(group);
                    (targets, lock, &effect.modification, Some(*effect))
                }
            }
        }
        Kind::Own { object, cda, modification } => {
            if let Some(ability) = cda {
                // CR 604.2 for a CDA: still on the object when its turn comes.
                let still_there = board.frames[object]
                    .abilities
                    .iter()
                    .any(|a| a.id == *ability && a.is_characteristic_defining);
                if !still_there {
                    return;
                }
            }
            (vec![*object], None, modification, None)
        }
    };

    if let Some(group) = lock {
        // Recorded even when empty: CR 613.6 locks the set at the layer the
        // effect starts in, and an effect that found nothing there finds
        // nothing later either.
        board.started.insert(group, targets.clone());
    }

    for target in targets {
        // Resolved before the frame is mutated: a dynamic amount may read the
        // member being modified — a creature counting "creatures you control"
        // counts itself — and a half-applied frame must not be what it sees.
        let resolved = resolve_modification(modification, game, board, target, layer_index, origin);
        let frame = board.frames.get_mut(&target).expect("a target is a member");
        apply_resolved(&resolved, frame, target);
    }
}

/// Apply `layer` over its applications in key order.
///
/// LI-2 replaces "in key order" with CR 613.8's loop — dependencies decided
/// against the live board, a loop's members in timestamp order, the order
/// re-evaluated after every application — and nothing else here moves.
fn apply_layer(game: &GameState, board: &mut Board<'_>, layer_index: usize, apps: &[Application<'_>]) {
    for app in apps {
        apply_one(game, board, layer_index, app);
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
    game.counters.record_board_walk();
    let mut board = Board::seed(game, lookahead, asked);
    for (layer_index, &layer) in LAYER_ORDER.iter().enumerate().take(ceiling) {
        let apps = applications_in_layer(game, &board, layer);
        apply_layer(game, &mut board, layer_index, &apps);
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
