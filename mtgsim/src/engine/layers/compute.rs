//! Core computation: `compute_characteristics(game, id)`.
//!
//! Produces `EffectiveCharacteristics` for a game object by applying every
//! continuous effect in layer order (1→2→3→4→5→6→7a→7b→7c→7d). All oracle
//! queries route through this function.
//!
//! The walk itself is `board.rs`'s: one pass per layer over the whole
//! working set, so that what an application reads is what applied earlier in
//! the same layer (`layers-architecture.md` §13b). This file is the entry
//! point, the memo, the walk of an object no row can reach, and the
//! evaluators the pass calls — filters, amounts, and a modification's
//! resolution against the board before it is written to a frame.

use std::borrow::Cow;
use std::sync::Arc;

use crate::engine::layers::board::{compute_board, compute_board_to, membership, Board, Membership};
use crate::engine::layers::lookahead::Lookahead;
use crate::engine::layers::types::*;
use crate::objects::card_data::CardData;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};

/// The layers, in application order (CR 613.1). Index into this array is the
/// "layer ceiling" a non-member walk stops at: ceiling `n` means layers
/// `LAYER_ORDER[..n]` have been applied, i.e. the frame as of the end of
/// layer `n - 1`.
///
/// This mirrors the `Layer` enum exactly, and the enum is **not** the CR's full
/// list. One sublayer split is still missing:
///
/// - **1a / 1b.** CR 613.2a is copy effects (1a); CR 613.2b is face-down (1b),
///   applied after copy. `Layer1Copy` collapses the two. CV-1 gives 1a a
///   producer (`EffectModification::CopyFrom`); 1b arrives with CV-6.
///
/// Splitting the slot later just lengthens this array — the ceiling is an
/// index into it, computed at runtime, so nothing else moves except
/// `layers::copy::END_OF_LAYER_1`, which a `debug_assert` there pins.
/// → `layers-architecture.md` §7.
pub(super) const LAYER_ORDER: [Layer; 10] = [
    Layer::Layer1Copy,
    Layer::Layer2Control,
    Layer::Layer3Text,
    Layer::Layer4Type,
    Layer::Layer5Color,
    Layer::Layer6Ability,
    Layer::Layer7aCdaPT,
    Layer::Layer7bSetPT,
    Layer::Layer7cModifyPT,
    Layer::Layer7dSwitchPT,
];

/// Layer 0 — an object's printed characteristics, plus the two things that
/// are not printed: its controller and CR 302.6's clock.
pub(super) fn seed_frame(card: &CardData, controller: PlayerId, control_since_turn: u32) -> EffectiveCharacteristics {
    EffectiveCharacteristics {
        name: card.name.clone(),
        mana_cost: card.mana_cost.clone(),
        colors: card.colors.clone(),
        types: card.types.clone(),
        subtypes: card.subtypes.clone(),
        supertypes: card.supertypes.clone(),
        keyword_flags: card.keyword_flags.clone(),
        abilities: card.abilities.clone(),
        power: card.power,
        toughness: card.toughness,
        controller,
        control_since_turn,
    }
}

/// Compute the effective characteristics of a game object after applying
/// all active continuous effects in layer order.
///
/// Returns `None` if the object doesn't exist.
///
/// **Memoized across calls** (`layers-architecture.md` §12 "7a"). The frame
/// for `id` is served from `GameState::layer_memo` while nothing has written
/// a walk input since it was computed — `GameState::layer_epoch` is that
/// test — and computed and stored otherwise. Shared rather than owned, so a
/// hit never clones the ability list. A `None` is never stored: an object
/// that does not exist costs a probe of the object map, not a walk.
///
/// **A miss for a member of the working set runs the whole pass and stores
/// every member's frame** at this epoch, so the next member asked is a hit
/// (§13b, decision 2). A miss for anything else walks that object alone.
///
/// Two readers bypass the memo on purpose. The CR 614.12 look-ahead
/// (`lookahead::compute_as_entering`) computes a hypothetical board, and the
/// CR 603.10a LKI capture (`compute_characteristics_uncached`) computes the
/// frame an event will store. Neither consults nor fills the memo.
pub fn compute_characteristics(game: &GameState, id: ObjectId) -> Option<Arc<EffectiveCharacteristics>> {
    let epoch = game.layer_epoch();
    if let Some(frame) = game.layer_memo.get(id, epoch) {
        game.counters.record_memo_hit();
        #[cfg(debug_assertions)]
        audit_memo_hit(game, id, &frame);
        return Some(frame);
    }
    // Counted before the store is probed, so a query for an object that does
    // not exist is a walk — the same walk it was before the pass.
    game.counters.record_layer_walk();
    game.objects.get(&id)?;

    let asked = match membership(game, id) {
        Membership::Member => None,
        Membership::ZoneOnly => Some(id),
        Membership::NonMember => {
            let frame = Arc::new(compute_non_member(game, &Board::settled(), id, LAYER_ORDER.len())?);
            game.layer_memo.insert(id, epoch, Arc::clone(&frame));
            return Some(frame);
        }
    };
    let mut wanted = None;
    for (member, frame) in compute_board_to(game, None, asked, LAYER_ORDER.len()).into_frames() {
        let frame = Arc::new(frame);
        if member == id {
            wanted = Some(Arc::clone(&frame));
        }
        game.layer_memo.insert(member, epoch, frame);
    }
    debug_assert!(wanted.is_some(), "a member's frame comes out of the pass");
    wanted
}

/// The debug mode §12 required in the same commit as the cache: every hit is
/// checked against a fresh walk, which makes `cargo test` and a debug
/// `fuzz_games` run the invalidation-completeness test. A coarse key can be
/// wrong in exactly one way — a write to a walk input that skipped its epoch
/// bump — and this turns that into a panic instead of a stale answer.
///
/// The fresh walk is not engine work: its counts are rewound, so a debug
/// build's `Layer walks` and `Layer frames` are the release build's.
#[cfg(debug_assertions)]
fn audit_memo_hit(game: &GameState, id: ObjectId, served: &EffectiveCharacteristics) {
    let (walks, board_walks, frames, checks) = (
        game.counters.layer_walks(),
        game.counters.board_walks(),
        game.counters.layer_frames(),
        game.counters.dependency_checks(),
    );
    let fresh = compute_characteristics_uncached(game, id);
    game.counters.rewind_layer_work(walks, board_walks, frames, checks);
    debug_assert_eq!(
        fresh.as_ref(),
        Some(served),
        "layer memo served a stale frame for {} at epoch {}: a write to a layer-walk \
         input skipped `GameState::bump_layer_epoch`",
        id,
        game.layer_epoch()
    );
}

/// One full layer walk of `id`, owned by the caller — a memo **miss**, and
/// the walk `EngineCounters::layer_walks` counts. For a member that is a
/// whole pass, of which one frame is kept.
///
/// The CR 603.10a LKI capture reads through here: the frame it takes is about
/// to be *stored*, on an event, as the record of what the permanent was, and
/// it is built from a fresh walk rather than from anything shared.
pub(crate) fn compute_characteristics_uncached(
    game: &GameState,
    id: ObjectId,
) -> Option<EffectiveCharacteristics> {
    game.counters.record_layer_walk();
    game.objects.get(&id)?;
    match membership(game, id) {
        Membership::Member => compute_board(game, None).take(id),
        Membership::ZoneOnly => compute_board_to(game, None, Some(id), LAYER_ORDER.len()).take(id),
        Membership::NonMember => compute_non_member(game, &Board::settled(), id, LAYER_ORDER.len()),
    }
}

/// The walk of an object no row can reach — a card in a hand, library or
/// graveyard, or a spell — up to `ceiling`: its printed characteristics and
/// its own CDAs, which CR 604.3 makes function in every zone.
///
/// No row applies here by construction of the working set: a filter row
/// needs the battlefield zone, a `Fixed` row's targets are members, a `Host`
/// row's host is a permanent. What a CDA here reads of *another* object goes
/// through `board.frame_of` — a member's live frame inside a pass, its
/// memoized frame outside one, and another non-member at a strictly lower
/// ceiling, which is what bounds the recursion (§13b, decision 4).
pub(super) fn compute_non_member(
    game: &GameState,
    board: &Board<'_>,
    id: ObjectId,
    ceiling: usize,
) -> Option<EffectiveCharacteristics> {
    let obj = game.objects.get(&id)?;
    debug_assert!(
        board.entity(game, id).is_none(),
        "a battlefield entity is a member of every pass, never walked alone"
    );
    game.counters.record_layer_frame();

    let controller = base_controller(game, id, board.lookahead).unwrap_or(obj.owner);
    let mut chars = seed_frame(&obj.card_data, controller, 0);

    // The common case, and worth its own exit: with no CDA there is nothing
    // any layer can do to an object off the battlefield.
    if !crate::engine::layers::cda::has_any_cda(&chars) {
        return Some(chars);
    }

    for (layer_index, &layer) in LAYER_ORDER.iter().enumerate().take(ceiling) {
        if !crate::engine::layers::cda::CDA_LAYERS.contains(&layer) {
            continue;
        }
        // Collected before applying, because applying mutates the list
        // being read.
        for (_, modification) in crate::engine::layers::cda::cda_modifications(&chars, layer) {
            let resolved = resolve_modification(&modification, game, board, id, layer_index, None);
            apply_resolved(&resolved, &mut chars, id);
        }
    }
    Some(chars)
}

/// The controller an object has before Layer 2 touches it — CR 110.2's default,
/// or the one CR 614.12's entering object would enter under.
///
/// Past the look-ahead arm, the arms are CR 108.4's sentence in order: a permanent reads
/// `PermanentState`, a spell reads its `StackEntry`, and a card in a hand or
/// graveyard has no controller at all — owner is what this reports for it,
/// because `EffectiveCharacteristics.controller` is not an `Option`.
///
/// **The single definition of the pre-Layer-2 seed.** The oracle's
/// `any_control_changing` gate returns this instead of walking, and it is
/// only exact while it and the pass's seed agree — which they do by both
/// calling this.
///
/// **The third arm is a resolving object, and it is not the owner fallback.**
/// `resolve_top_of_stack` takes the `StackEntry` before it resolves anything
/// (CR 608.2's object stays on the stack, but its *entry* is owned by the
/// resolution), so between there and the end of the resolution the first two
/// probes both miss and the owner fallback answers. That is right for a land
/// drop, where owner and controller coincide, and wrong for a spell cast by a
/// player who does not own it — which CR 110.2b calls out by name. `resolving`
/// carries that default across exactly this window, so it belongs above the
/// fallback rather than inside it.
///
/// RC-3 is where this is fixed because RC-3 is where it became askable of an
/// *entering* permanent: `effect_applies_to` no longer stops a filter at the
/// battlefield boundary, so `ObjectFilter::ByController` now reads this
/// value for every entry. It was already wrong on the replacement path, where
/// `set_affects` has never had a gate.
pub(crate) fn base_controller(
    game: &GameState,
    id: ObjectId,
    lookahead: Option<&Lookahead>,
) -> Option<PlayerId> {
    // CR 614.12's entering object answers with the controller it would enter
    // under — the proposal's, or a CR 616.1b rewrite of it. Ahead of the
    // battlefield probe, which never finds an entity for it, and of the owner
    // fallback, which is wrong for every permanent spell cast by a non-owner.
    if let Some(l) = lookahead.filter(|l| l.object == id) {
        return Some(l.entity.controller);
    }
    // Battlefield first, so the common case is one probe rather than three.
    if let Some(entry) = game.battlefield.get(&id) {
        return Some(entry.controller);
    }
    if let Some(entry) = game.stack_entries.get(&id) {
        return Some(entry.controller);
    }
    if let Some(resolving) = game.resolving {
        if resolving.id == id {
            return Some(resolving.default_controller);
        }
    }
    game.objects.get(&id).map(|obj| obj.owner)
}

/// The players a continuous effect's `ObjectFilter` can name — resolved
/// lazily, at most once per filter tree.
///
/// Laziness is load-bearing, not tidiness. A filter with no `ByController` node
/// (Cloudspire Mesa's bare "creatures have flying") must cost what it cost
/// before this refactor, and `And(ByType(Creature), ByController(You))` must
/// cost nothing extra for a land, because `&&` never reaches the second arm.
///
/// The two origins get their "you" from different rules, and the difference is
/// not a shortcut:
///
/// - **`StaticAbility` (CR 613.7a).** CR 109.5: "For a static ability, this is
///   the *current* controller of the object it's on." So ask the source for its
///   controller as the pass has it *now* — the same live frame the existence
///   check reads. At layer 2 that is the partially-applied layer: two Layer 2
///   effects where applying one changes what the other applies to are
///   dependent under CR 613.8a, and LI-2 orders them; a mutual pair is a
///   loop, applied in timestamp order, which is the order this pass already
///   uses.
///
/// - **`Resolution` (CR 613.7b).** "You" was fixed when the spell or ability
///   resolved — CR 611.2c, "the set of objects it affects is determined when
///   that continuous effect begins" — and `effect.controller` is that player.
///   The source permanent may since have changed hands, or left the
///   battlefield entirely, without moving the effect's allegiance.
///
/// `effect.controller` is also the fallback when the source object is gone from
/// the store. A `StaticAbility` effect in that state is about to be retired by
/// `static_ability_still_exists` anyway, so the value only has to be defined,
/// not meaningful.
///
/// # Two ways to build one
///
/// A registry row supplies `effect`, and the two players are derived from it
/// lazily as above. A CDA has no row (CR 604.3a(3)) and its "you" is the
/// object's own controller as of this layer — `chars.controller`, CR 109.5 —
/// so `AmountExpr::CountOf` inside one builds a `FilterPlayers` with both
/// players already resolved and no row. The `Option` is that second
/// constructor; a row-less `FilterPlayers` with an unresolved player is a
/// construction error, and `expect`s.
pub(super) struct FilterPlayers<'a, 'l> {
    effect: Option<&'a ContinuousEffect>,
    /// The object the filter is relative to — the row's source, or the
    /// object whose CDA is counting — which is what `ObjectFilter::EachOther`
    /// is other than.
    source: ObjectId,
    game: &'a GameState,
    board: &'a Board<'l>,
    layer_index: usize,
    you: Option<PlayerId>,
    owner: Option<PlayerId>,
}

impl<'a, 'l> FilterPlayers<'a, 'l> {
    /// The players of a registry row's filter, resolved on first use.
    pub(super) fn for_row(
        effect: &'a ContinuousEffect,
        game: &'a GameState,
        board: &'a Board<'l>,
        layer_index: usize,
    ) -> Self {
        FilterPlayers {
            effect: Some(effect),
            source: effect.source,
            game,
            board,
            layer_index,
            you: None,
            owner: None,
        }
    }

    /// The players of a *condition*'s filter (CR 604.2's "as long as", read
    /// through CR 109.5): "you" is the source's current controller off its
    /// live frame, exactly as a static row's is, and the source is what
    /// `ObjectFilter::EachOther` is other than. Both are resolved up
    /// front, since there is no row to re-derive them from.
    pub(super) fn for_source(
        source: ObjectId,
        game: &'a GameState,
        board: &'a Board<'l>,
        layer_index: usize,
    ) -> Self {
        let owner = game.objects.get(&source).map(|obj| obj.owner);
        let you = board
            .frame_of(game, source, layer_index)
            .map(|frame| frame.controller)
            .or(owner);
        FilterPlayers { effect: None, source, game, board, layer_index, you, owner }
    }

    /// CR 109.5's "you".
    pub(super) fn you(&mut self) -> PlayerId {
        if let Some(you) = self.you {
            return you;
        }
        let effect = self
            .effect
            .expect("a FilterPlayers built without a row is built with both players resolved");
        let you = match effect.origin {
            EffectOrigin::Resolution => effect.controller,
            EffectOrigin::StaticAbility { .. } => self
                .board
                .frame_of(self.game, effect.source, self.layer_index)
                .map(|frame| frame.controller)
                .unwrap_or(effect.controller),
        };
        self.you = Some(you);
        you
    }

    /// The source object's owner (CR 108.3 / 110.2).
    fn owner(&mut self) -> PlayerId {
        if let Some(owner) = self.owner {
            return owner;
        }
        let effect = self
            .effect
            .expect("a FilterPlayers built without a row is built with both players resolved");
        let owner = self
            .game
            .objects
            .get(&effect.source)
            .map(|obj| obj.owner)
            .unwrap_or(effect.controller);
        self.owner = Some(owner);
        owner
    }
}

/// Resolve a Layer 2 `SetController`'s `PlayerRef` to the player who ends up
/// controlling `object_id` (CR 613.1b).
///
/// `You` routes through the same `FilterPlayers` a filter's `ByController`
/// uses, so CR 109.5 and CR 611.2c have one implementation between them.
///
/// `Owner` is the owner of the object being *moved*, not of the effect's
/// source. Homeward Path's "each player gains control of all creatures they
/// own" hands each creature to its own owner, which is the opposite of what the
/// same variant means inside an `ObjectFilter`, where it describes the source.
///
/// # Which player identities may stay symbolic in a row
///
/// The walk is a pure read: it cannot prompt, and it runs many times per game
/// state. So a `PlayerRef` survives into the registry only when it must be
/// **re-derived** on every walk — `You`, because CR 109.5 makes a static
/// ability's "you" the source's *current* controller, and `Owner`, which is
/// fixed but free to recompute (CR 108.3).
///
/// Everything else is settled **when the effect is created** and stored as
/// `Player(pid)`. That covers every card whose new controller is chosen or
/// computed rather than named:
///
/// - "An opponent gains control" (Akroan Horse, Fateful Handoff, Rainbow Vale
///   — 9 cards) does not target, and its ruling is explicit that in a
///   multiplayer game *you choose the opponent as the ability resolves*.
/// - "That player gains control" (Risky Move), "choose a player at random"
///   (Scrambleverse), an auction winner (Illicit Auction) — all resolution-time
///   computations over game state.
///
/// None of those need a new `PlayerRef` variant, and none are boxed out by this
/// function; they need the *lowering* to make the choice, which is the piece
/// that does not exist yet (`codebase-state.md` item 13).
///
/// `Opponent` therefore resolves here only in a two-player game, where CR 102.2
/// leaves nothing to choose. Above two players it means the resolution step
/// skipped a choice it owed, so it asserts rather than inventing one.
fn resolve_set_controller(
    player_ref: &crate::types::effects::PlayerRef,
    object_id: ObjectId,
    effect: &ContinuousEffect,
    game: &GameState,
    board: &Board<'_>,
    layer_index: usize,
) -> Option<PlayerId> {
    use crate::types::effects::PlayerRef;

    let mut players = FilterPlayers::for_row(effect, game, board, layer_index);

    match player_ref {
        PlayerRef::You => Some(players.you()),
        PlayerRef::Player(pid) => Some(*pid),
        PlayerRef::Owner => game.objects.get(&object_id).map(|obj| obj.owner),
        PlayerRef::Opponent => {
            let you = players.you();
            let mut opponents = (0..game.num_players()).filter(|&pid| pid != you);
            match (opponents.next(), opponents.next()) {
                // CR 102.2 — exactly one opponent, so nothing was ever chosen.
                (Some(only), None) => Some(only),
                _ => {
                    debug_assert!(
                        false,
                        concat!(
                            "SetController(PlayerRef::Opponent) with {} players. ",
                            "Which opponent is a choice the effect's controller ",
                            "makes as it resolves, so the row should already carry ",
                            "PlayerRef::Player(..); reaching the layer walk means ",
                            "the lowering skipped it."
                        ),
                        game.num_players()
                    );
                    None
                }
            }
        }
    }
}

/// Check if a permanent's current characteristics match a filter.
///
/// `players` resolves the `PlayerRef` in a `ByController` node. Controller is
/// matched here rather than beside the filter, which is how the snapshot bug
/// hid: `ByController` used to return `true` unconditionally and defer to a
/// field on `AffectedSet::Filter`, so the two halves of one question lived in
/// two places and only one of them was re-asked during the walk.
///
/// `id` is the object `chars` describes. Almost every leaf answers from the
/// frame alone — that is what "post-layers" means — but CR 707.2 excludes
/// tokenness from copiable values, so `ObjectFilter::Token` is a property of
/// the `GameObject` that no layer can reach and no frame can carry.
pub(super) fn object_matches_filter(
    filter: &crate::types::effects::ObjectFilter,
    id: ObjectId,
    chars: &EffectiveCharacteristics,
    players: &mut FilterPlayers<'_, '_>,
) -> bool {
    use crate::types::effects::{ObjectFilter, PlayerRef};
    match filter {
        ObjectFilter::All => true,
        ObjectFilter::ByType(t) => chars.types.contains(t),
        ObjectFilter::BySubtype(s) => chars.subtypes.contains(s),
        ObjectFilter::BySupertype(s) => chars.supertypes.contains(s),
        ObjectFilter::ByColor(c) => chars.colors.contains(c),
        // `chars.controller` is the *effective* controller of the object being
        // tested — Layer 2 will write it, and this comparison then costs
        // nothing to keep correct.
        //
        // Every variant resolves; none of them asserts. `Opponent` is a
        // predicate rather than an id on purpose: CR 102.2 makes it exactly one
        // player in a two-player game, but CR 102.3 makes "your opponents" a
        // set in multiplayer, and "controlled by someone who isn't you" is the
        // same answer in both without the type having to lie.
        ObjectFilter::ByController(player_ref) => match player_ref {
            PlayerRef::You => chars.controller == players.you(),
            PlayerRef::Opponent => chars.controller != players.you(),
            PlayerRef::Owner => chars.controller == players.owner(),
            PlayerRef::Player(pid) => chars.controller == *pid,
        },
        // Read off the object, not the frame: CR 707.2 excludes tokenness from
        // copiable values, so no layer can change the answer and there is
        // nothing on `chars` to consult. `players.game` is the same board the
        // frame was computed against.
        ObjectFilter::Token => {
            players.game.objects.get(&id).map(|obj| obj.is_token).unwrap_or(false)
        }
        // Off the object for the same reason as `Token`: CR 108.3 ownership is
        // fixed when the game starts and no layer touches it. An object with no
        // entry cannot match anybody's ownership question.
        ObjectFilter::ByOwner(player_ref) => {
            let Some(owner) = players.game.objects.get(&id).map(|obj| obj.owner) else {
                return false;
            };
            match player_ref {
                PlayerRef::You => owner == players.you(),
                PlayerRef::Opponent => owner != players.you(),
                PlayerRef::Owner => owner == players.owner(),
                PlayerRef::Player(pid) => owner == *pid,
            }
        }
        ObjectFilter::PowerLE(n) => {
            chars.power.map(|p| p <= *n).unwrap_or(false)
        }
        // Identity, off the ids: Opalescence does not animate itself, and no
        // layer can make an object something other than itself.
        ObjectFilter::EachOther => id != players.source,
        ObjectFilter::And(a, b) => {
            object_matches_filter(a, id, chars, players)
                && object_matches_filter(b, id, chars, players)
        }
        ObjectFilter::Or(a, b) => {
            object_matches_filter(a, id, chars, players)
                || object_matches_filter(b, id, chars, players)
        }
        ObjectFilter::Not(inner) => !object_matches_filter(inner, id, chars, players),
    }
}

/// Resolve one side of a P/T modification against the frame so far.
///
/// `None` means the expression has no meaning in a static context — `Variable`
/// is CR 107.3's X, chosen as a spell is cast, and the `Target*`/`DamageDealt`
/// arms read a resolution that already happened. A continuous effect asking for
/// one of those is a card-authoring error, so it asserts in debug and declines
/// to apply in release rather than inventing a number.
///
/// Evaluated fresh at every layer: that is the point of `PtValue::Dynamic`.
///
/// `object_id` is the object `chars` describes and `origin` the registry row
/// the value came from, or `None` for a CDA — `AmountExpr::CountOf` needs both
/// to resolve "you" (CR 109.5) the way the filter leaves do.
pub(super) fn evaluate_pt_value(
    value: &PtValue,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    object_id: ObjectId,
    layer_index: usize,
    board: &Board<'_>,
    origin: Option<&ContinuousEffect>,
) -> Option<i32> {
    match value {
        PtValue::Fixed(n) => Some(*n),
        PtValue::Dynamic(expr) => {
            evaluate_amount(expr, game, chars, object_id, layer_index, board, origin)
        }
    }
}

/// Evaluate a card-definition amount inside the layer walk.
///
/// Anything reading *another* object goes through `board.frame_of`, never
/// through `card_data`: a member's frame as the pass has it now, a non-member
/// at this `layer_index` as its ceiling. Both halves of that matter: it keeps
/// the layer-system invariant (effective characteristics, not printed ones),
/// and it keeps the one recursion left bounded — a non-member read at ceiling
/// `layer_index` is strictly below the ceiling of the walk that made it. A
/// Tarmogoyf in a graveyard counting itself bottoms out for exactly that reason.
pub(super) fn evaluate_amount(
    expr: &crate::types::effects::AmountExpr,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    object_id: ObjectId,
    layer_index: usize,
    board: &Board<'_>,
    origin: Option<&ContinuousEffect>,
) -> Option<i32> {
    use crate::types::effects::{AmountExpr, ObjectFilter, PlayerRef, Selector};

    match expr {
        AmountExpr::Fixed(n) => Some(*n as i32),

        // CR 202.3b — an object with no mana cost has mana value 0.
        AmountExpr::AffectedManaValue => {
            Some(chars.mana_cost.as_ref().map(|c| c.mana_value()).unwrap_or(0) as i32)
        }

        AmountExpr::Plus(inner, n) => {
            evaluate_amount(inner, game, chars, object_id, layer_index, board, origin)
                .map(|v| v + *n as i32)
        }

        // A count over the battlefield, taken at this layer — Keldon Warlord's
        // "the number of non-Wall creatures you control".
        //
        // **Enumerates the real battlefield**, which a permanent that is only
        // *entering* is not on. That is §5a's boundary
        // (`replacement-architecture.md`) falling out of the structure rather
        // than being special-cased: the entering object is visible to filters
        // — the frame this count runs inside is its own — and invisible to
        // counts. Thassa's ruling says exactly that: "the mana symbols in its
        // mana cost won't be counted", because replacement effects are
        // considered before the God is on the battlefield.
        //
        // Each member's frame is the live one — including the object doing
        // the counting, which is why a modification is resolved before its
        // frame is written. One filter evaluation per permanent per query is
        // `layers-architecture.md` §12's quadratic by design, and Keldon
        // Warlord is the card that measures it.
        //
        // "You" is the affected object's own controller for a CDA (CR 109.5,
        // read off `chars` as of this layer) and the row's controller for a
        // registry row, exactly as a filter leaf resolves it.
        AmountExpr::CountOf(selector) => {
            let filter: Cow<'_, ObjectFilter> = match selector {
                Selector::PermanentsMatching(filter) => Cow::Borrowed(filter),
                Selector::ControlledCreatures => Cow::Owned(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(crate::types::card_types::CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                )),
                other => {
                    debug_assert!(
                        false,
                        "CountOf({:?}) has no static-context evaluator yet (on '{}')",
                        other, chars.name
                    );
                    return None;
                }
            };
            let (you, owner) = match origin {
                Some(_) => (None, None),
                None => (
                    Some(chars.controller),
                    Some(game.objects.get(&object_id).map(|obj| obj.owner).unwrap_or(chars.controller)),
                ),
            };
            let mut players = FilterPlayers {
                effect: origin,
                source: origin.map(|e| e.source).unwrap_or(object_id),
                game,
                board,
                layer_index,
                you,
                owner,
            };
            let mut count = 0;
            for other in board.battlefield_ids(game) {
                let Some(other_chars) = board.frame_of(game, other, layer_index) else {
                    continue;
                };
                if object_matches_filter(&filter, other, &other_chars, &mut players) {
                    count += 1;
                }
            }
            Some(count)
        }

        // Card *types*, not cards: ten artifact creatures in a graveyard are
        // still two types.
        AmountExpr::CardTypesAmong(selector) => match selector {
            Selector::CardsInGraveyard(None) => {
                let mut types: std::collections::HashSet<crate::types::card_types::CardType> =
                    std::collections::HashSet::new();
                for player in &game.players {
                    for card_id in &player.graveyard {
                        if let Some(card) = board.frame_of(game, *card_id, layer_index) {
                            types.extend(card.types.iter().copied());
                        }
                    }
                }
                Some(types.len() as i32)
            }
            other => {
                debug_assert!(
                    false,
                    "CardTypesAmong({:?}) has no evaluator yet (on '{}')",
                    other, chars.name
                );
                None
            }
        },

        // CR 107.3's X is chosen as a spell is cast, and the Target*/DamageDealt
        // arms read a resolution that already happened. A continuous effect
        // asking for one of those is a card-authoring error: assert in debug,
        // decline to apply in release, never invent a number.
        other => {
            debug_assert!(
                false,
                "continuous effect on '{}' carries {:?}, which has no static-context evaluator",
                chars.name, other
            );
            None
        }
    }
}

/// [`evaluate_amount`] against the settled board at the full ceiling — the
/// read-side view a *post-layer* consumer takes, so every leaf answers as the
/// live pass would have at its last layer. The mirror of
/// [`condition::settled_holds`](crate::engine::layers::condition::settled_holds),
/// and the same one line.
///
/// `object` is the object the amount is *about*, which for the one caller
/// today — `engine::cost_determination`, at CR 601.2f — is always the cost
/// ability's own source: the permanent for a `Spells` subject, the spell
/// itself for `Itself`.
///
/// **That identity is `codebase-state.md` item 57's answer.** The item warned
/// that a cost evaluator for `SourcePower` would be a *third* evaluator whose
/// entitlement nobody had argued. It is instead a third **caller** of the one
/// leaf table, and the entitlement is argued once (`cost-architecture.md`
/// §3.7): CR 613.11 applies cost effects after every other continuous effect,
/// so no hypothetical frame is in play and `Board::settled()` is the board the
/// rule describes — the memo for a member's frame, the real battlefield for a
/// count.
///
/// **`SourcePower` is answered here and refused by [`evaluate_amount`]**, and
/// that split is the whole point. `object` being the source means this caller
/// reads its own `chars.power` — the *effective* one, so an anthem on the
/// source moves the amount — and makes no cross-object read at all. The walk
/// cannot do that: there `object_id` is the affected object and the row's
/// source is somewhere else on the board, so answering would be a
/// cross-object read mid-pass, which is the CR 613.8 dependency it refuses on
/// purpose. Same leaf, two callers, one of which is entitled to it.
pub fn settled_amount(
    expr: &crate::types::effects::AmountExpr,
    game: &GameState,
    object: ObjectId,
) -> Option<i32> {
    let chars = compute_characteristics(game, object)?;
    // The one leaf this reader answers itself. `None` — a source with no
    // power at all, an enchantment or a creature that stopped being one — is
    // "no amount", which every consumer reads as "reduce nothing" rather than
    // as zero-by-assumption.
    if matches!(expr, crate::types::effects::AmountExpr::SourcePower) {
        return chars.power;
    }
    evaluate_amount(expr, game, &chars, object, LAYER_ORDER.len(), &Board::settled(), None)
}

/// A modification with its reads already made, ready to be written to a
/// frame without touching the board again.
///
/// Only three arms read anything: `SetController` asks who "you" is, and the
/// two P/T arms evaluate their amounts. Everything else carries its own
/// answer and is applied as it is. The split exists so a pass can resolve
/// against every member's live frame — including the one about to be
/// mutated — and only then take that frame mutably.
pub(super) enum Resolved<'m> {
    AsIs(&'m EffectModification),
    /// CR 613.1b, resolved; `None` when the `PlayerRef` could not be.
    Controller(Option<(PlayerId, u32)>),
    /// Layer 7b, both sides; `None` when either amount had no meaning.
    SetPt(Option<(i32, i32)>),
    /// Layer 7c, each side independently.
    ModifyPt(Option<i32>, Option<i32>),
}

/// Make a modification's reads against the board.
///
/// `object_id` is the object about to be modified. `origin` is the registry
/// row this modification came from, or `None` for an intrinsic application
/// (`layers::cda`, a counter), which has no row.
///
/// Only `SetController` needs the row, because it is the only modification
/// that does not carry its own answer. `AddType(Creature)` says what to do;
/// "the controller becomes *you*" does not say who "you" is, and CR 109.5
/// answers that from the ability's source and the row's origin. The row also
/// supplies `created_on_turn`, which is when the new controller's CR 302.6
/// clock starts. CDAs never reach that arm — CR 613.4a lists 7a as their only
/// P/T sublayer and none live in Layer 2 — so it asserts instead of guessing.
pub(super) fn resolve_modification<'m>(
    modification: &'m EffectModification,
    game: &GameState,
    board: &Board<'_>,
    object_id: ObjectId,
    layer_index: usize,
    origin: Option<&ContinuousEffect>,
) -> Resolved<'m> {
    match modification {
        EffectModification::SetController(player_ref) => {
            let Some(effect) = origin else {
                debug_assert!(
                    false,
                    "SetController reached the walk with no registry row. CR 613.4a \
                     puts no characteristic-defining ability in Layer 2, so the only \
                     caller that passes `None` cannot produce this modification."
                );
                return Resolved::Controller(None);
            };
            Resolved::Controller(
                resolve_set_controller(player_ref, object_id, effect, game, board, layer_index)
                    .map(|pid| (pid, effect.created_on_turn)),
            )
        }
        EffectModification::SetPowerToughness { power, toughness } => {
            let chars = board
                .frame_of(game, object_id, layer_index)
                .expect("a modification resolves against the frame it is about to change");
            let p = evaluate_pt_value(power, game, &chars, object_id, layer_index, board, origin);
            let t = evaluate_pt_value(toughness, game, &chars, object_id, layer_index, board, origin);
            Resolved::SetPt(p.zip(t))
        }
        EffectModification::ModifyPowerToughness { power, toughness } => {
            let chars = board
                .frame_of(game, object_id, layer_index)
                .expect("a modification resolves against the frame it is about to change");
            let dp = evaluate_pt_value(power, game, &chars, object_id, layer_index, board, origin);
            let dt = evaluate_pt_value(toughness, game, &chars, object_id, layer_index, board, origin);
            Resolved::ModifyPt(dp, dt)
        }
        other => Resolved::AsIs(other),
    }
}

/// Write a resolved modification to a frame.
///
/// `object_id` is the object `chars` describes. Layer 4's subtype arms need it
/// to derive stable ids for intrinsic mana abilities (CR 305.6) — see
/// `land_types::intrinsic_mana_ability`.
pub(super) fn apply_resolved(resolved: &Resolved<'_>, chars: &mut EffectiveCharacteristics, object_id: ObjectId) {
    let modification = match resolved {
        // Layer 2. CR 302.6 asks whether control has been *continuous*, so the
        // clock only restarts when control actually moves. Act of Treason
        // legally targets a creature you already control; gaining control of
        // something you control changes nothing, and resetting the epoch here
        // would invent summoning sickness the CR does not give. (Act of
        // Treason grants haste, so it would hide the bug; a card that gains
        // control without haste would not.)
        Resolved::Controller(Some((new_controller, since))) => {
            if chars.controller != *new_controller {
                chars.controller = *new_controller;
                chars.control_since_turn = *since;
            }
            return;
        }
        Resolved::Controller(None) => return,
        // Layer 7b
        Resolved::SetPt(Some((p, t))) => {
            chars.power = Some(*p);
            chars.toughness = Some(*t);
            return;
        }
        Resolved::SetPt(None) => return,
        // Layer 7c
        Resolved::ModifyPt(dp, dt) => {
            if let (Some(dp), Some(p)) = (dp, chars.power.as_mut()) {
                *p += dp;
            }
            if let (Some(dt), Some(t)) = (dt, chars.toughness.as_mut()) {
                *t += dt;
            }
            return;
        }
        Resolved::AsIs(modification) => *modification,
    };

    match modification {
        // Layer 1a — CR 707.2's captured values replace every characteristic
        // channel at once. Everything after this point in the walk modifies the
        // copy, which is CR 613.2c read forwards.
        EffectModification::CopyFrom(values) => values.apply_to(chars),

        EffectModification::SetController(_)
        | EffectModification::SetPowerToughness { .. }
        | EffectModification::ModifyPowerToughness { .. } => {
            unreachable!("resolved above")
        }

        // Layer 4
        EffectModification::AddType(t) => { chars.types.insert(*t); }
        EffectModification::RemoveType(t) => { chars.types.remove(t); }
        EffectModification::SetTypes(types) => { chars.types = types.clone(); }
        // CR 305.6/305.7 land semantics live in `land_types` — see the module
        // docs there for why this is not a Layer 6 concern.
        EffectModification::AddSubtype(s) => {
            crate::engine::layers::land_types::apply_add_subtype(chars, s, object_id);
        }
        EffectModification::RemoveSubtype(s) => { chars.subtypes.remove(s); }
        EffectModification::SetSubtypes(subtypes) => {
            crate::engine::layers::land_types::apply_set_subtypes(chars, subtypes, object_id);
        }
        EffectModification::AddSupertype(s) => { chars.supertypes.insert(*s); }
        EffectModification::RemoveSupertype(s) => { chars.supertypes.remove(s); }
        EffectModification::SetSupertypes(supertypes) => { chars.supertypes = supertypes.clone(); }

        // Layer 5
        EffectModification::AddColor(c) => { chars.colors.insert(*c); }
        EffectModification::SetColors(colors) => { chars.colors = colors.clone(); }
        EffectModification::RemoveAllColors => { chars.colors.clear(); }

        // Layer 6
        EffectModification::GrantKeywordFlag(kw) => { chars.keyword_flags.insert(*kw); }
        // CR 113.10b — "effects that remove an ability remove all instances of
        // it". For a keyword flag that is structural: a `HashSet` never held
        // more than one.
        EffectModification::RemoveKeywordFlag(kw) => { chars.keyword_flags.remove(kw); }
        EffectModification::GrantAbility(def) => {
            // CR 604.3a(2) — an ability that reached an object by being granted
            // is never a characteristic-defining ability, however its text
            // reads. The flag on `AbilityDef` asserts only the four criteria
            // that are properties of the text; provenance is maintained by
            // whoever writes the ability onto an object, and this is that
            // place. Copy (Layer 1) and text-changing (Layer 3) effects hand
            // the def over whole and keep the flag, which is the *other* half
            // of 604.3a(2) and equally deliberate.
            //
            // Not clearing it would let a granted Tarmogoyf ability define P/T
            // at Layer 7a, which is exactly what 604.3a(2) forbids.
            let mut granted = (**def).clone();
            granted.is_characteristic_defining = false;
            chars.abilities.push(granted);
        }
        // CR 113.10b again, and here it is *not* structural: `abilities` is a
        // `Vec` and the same ability can genuinely appear twice — printed on
        // the card and granted on top of it. `retain`, never "remove the first
        // match".
        EffectModification::LoseAbility(ability_id) => {
            chars.abilities.retain(|a| a.id != *ability_id);
        }
        EffectModification::LoseAllAbilities => {
            chars.keyword_flags.clear();
            chars.abilities.clear();
        }

        // Layer 7d
        EffectModification::SwitchPowerToughness => {
            let old_power = chars.power;
            chars.power = chars.toughness;
            chars.toughness = old_power;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::effects::CounterType;
    use crate::types::replacement::EnterMods;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::effects::PlayerRef;
    use crate::types::keywords::KeywordFlag;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::test_support::registered;

    #[test]
    fn test_base_characteristics_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Grizzly Bears");
        assert_eq!(chars.power, Some(2));
        assert_eq!(chars.toughness, Some(2));
        assert!(chars.types.contains(&CardType::Creature));
        assert!(chars.colors.contains(&Color::Green));
        assert_eq!(chars.controller, 0);
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_pump_power_only() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register a +3/+0 effect
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(0) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(2));
    }

    // COVERS: ATOM-122.1a-001
    #[test]
    fn test_counters_modify_pt() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);
        game.add_counters(id, CounterType::PlusOnePlusOne, 2);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(4));
        assert_eq!(chars.toughness, Some(4));
    }

    // COVERS-PARTIAL: ATOM-122.1a-002
    #[test]
    fn test_counters_plus_and_minus() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Big Creature")
            .card_type(CardType::Creature)
            .power_toughness(5, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);
        game.add_counters(id, CounterType::PlusOnePlusOne, 3);
        game.add_counters(id, CounterType::MinusOneMinusOne, 1);

        let chars = compute_characteristics(&game, id).unwrap();
        // Net: +3 -1 = +2
        assert_eq!(chars.power, Some(7));
        assert_eq!(chars.toughness, Some(7));
    }

    #[test]
    fn test_nonexistent_object_returns_none() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(compute_characteristics(&game, fake_id).is_none());
    }

    #[test]
    fn test_non_battlefield_object_no_modifiers() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(Color::Red)
            .build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Lightning Bolt");
        assert!(chars.types.contains(&CardType::Instant));
        assert!(chars.colors.contains(&Color::Red));
        // Not on battlefield, so controller defaults to owner
        assert_eq!(chars.controller, 0);
    }

    #[test]
    fn test_keywords_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .power_toughness(4, 4)
            .keyword_flag(KeywordFlag::Flying)
            .keyword_flag(KeywordFlag::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keyword_flags.contains(&KeywordFlag::Flying));
        assert!(chars.keyword_flags.contains(&KeywordFlag::Vigilance));
        assert!(!chars.keyword_flags.contains(&KeywordFlag::Trample));
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_modifies_pt() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register a +3/+3 effect targeting this creature
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_registered_effect_grants_keyword() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register a "gains flying" effect
        let effect = registered(
            id,
            Layer::Layer6Ability,
            1,
            EffectModification::GrantKeywordFlag(KeywordFlag::Flying),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keyword_flags.contains(&KeywordFlag::Flying));
    }

    #[test]
    fn test_filter_based_effect() {
        use crate::types::effects::{Duration, ObjectFilter};

        let mut game = GameState::new(2, 20);

        // Two creatures controlled by player 0
        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0, &EnterMods::NONE);

        let giant_data = CardDataBuilder::new("Hill Giant")
            .card_type(CardType::Creature)
            .power_toughness(3, 3)
            .build();
        let giant = GameObject::new(giant_data, 0, Zone::Battlefield);
        let giant_id = giant.id;
        game.add_object(giant);
        game.place_on_battlefield(giant_id, 0, &EnterMods::NONE);

        // Register an anthem: "Creatures you control get +1/+1"
        let anthem_source = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: anthem_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ),
            },
            modification: EffectModification::ModifyPowerToughness { power: PtValue::Fixed(1), toughness: PtValue::Fixed(1) },
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert_eq!(bears_chars.power, Some(3));
        assert_eq!(bears_chars.toughness, Some(3));

        let giant_chars = compute_characteristics(&game, giant_id).unwrap();
        assert_eq!(giant_chars.power, Some(4));
        assert_eq!(giant_chars.toughness, Some(4));
    }

    /// CR 303.4m — `Host` is whatever the source is attached to at
    /// the moment of the walk: nothing while unattached, the host once
    /// attached, the new host after a move, nothing again after a detach. Read
    /// through the memo on purpose, so a writer that skipped its epoch bump
    /// would serve the previous answer here.
    #[test]
    fn test_attached_to_source_follows_the_attachment() {
        let mut game = GameState::new(2, 20);
        let place = |game: &mut GameState, name: &str, card_type: CardType| {
            let obj = GameObject::new(
                CardDataBuilder::new(name).card_type(card_type).power_toughness(2, 2).build(),
                0,
                Zone::Battlefield,
            );
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0, &EnterMods::NONE);
            id
        };
        let bears = place(&mut game, "Grizzly Bears", CardType::Creature);
        let giant = place(&mut game, "Hill Giant", CardType::Creature);
        let aura = place(&mut game, "Holy Strength", CardType::Enchantment);

        let timestamp = game.allocate_timestamp();
        game.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: aura,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp,
            affected: AffectedSet::Host,
            modification: EffectModification::ModifyPowerToughness {
                power: PtValue::Fixed(1),
                toughness: PtValue::Fixed(2),
            },
        });
        let pt = |game: &GameState, id| {
            let c = compute_characteristics(game, id).unwrap();
            (c.power.unwrap(), c.toughness.unwrap())
        };

        assert_eq!(pt(&game, bears), (2, 2), "unattached, the row names nothing");

        game.attach(aura, bears);
        assert_eq!(pt(&game, bears), (3, 4));
        assert_eq!(pt(&game, giant), (2, 2));

        game.attach(aura, giant);
        assert_eq!(pt(&game, bears), (2, 2), "a reattachment moves the row");
        assert_eq!(pt(&game, giant), (3, 4));
        assert!(
            !game.battlefield[&bears].attached_by.contains(&aura),
            "and the old host's back-pointer went with it"
        );

        game.detach(aura);
        assert_eq!(pt(&game, giant), (2, 2));
        assert_eq!(game.battlefield[&aura].attached_to, None);
    }

    #[test]
    fn test_set_colors_replaces_base_colors() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register a "becomes blue" effect (SetColors)
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        assert_eq!(chars.colors.len(), 1);
    }

    #[test]
    fn test_add_color_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register an "also red" effect (AddColor)
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::AddColor(Color::Red),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Green));
        assert!(chars.colors.contains(&Color::Red));
        assert_eq!(chars.colors.len(), 2);
    }

    #[test]
    fn test_remove_all_colors_makes_colorless() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register a "becomes colorless" effect
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::RemoveAllColors,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.is_empty());
    }

    #[test]
    fn test_color_change_independent_of_pt() {

        // Color change (L5) should not affect P/T (L7) and vice versa
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // L5: becomes blue
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let color_effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(color_effect);

        // L7c: +3/+3
        let pt_effect = registered(
            id,
            Layer::Layer7cModifyPT,
            game.allocate_timestamp(),
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(pt_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Color should be blue (not green)
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        // P/T should be 5/5 (2+3)
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_filter_based_color_effect() {
        use crate::types::effects::{Duration, ObjectFilter};

        // Static ability: "Creatures you control are also red"
        let mut game = GameState::new(2, 20);

        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0, &EnterMods::NONE);

        // Opponent's creature should NOT be affected
        let opp_data = CardDataBuilder::new("Savannah Lions")
            .card_type(CardType::Creature)
            .color(Color::White)
            .power_toughness(2, 1)
            .build();
        let opp = GameObject::new(opp_data, 1, Zone::Battlefield);
        let opp_id = opp.id;
        game.add_object(opp);
        game.place_on_battlefield(opp_id, 1, &EnterMods::NONE);

        let source_id = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: source_id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert!(bears_chars.colors.contains(&Color::Green));
        assert!(bears_chars.colors.contains(&Color::Red));

        let opp_chars = compute_characteristics(&game, opp_id).unwrap();
        assert!(opp_chars.colors.contains(&Color::White));
        assert!(!opp_chars.colors.contains(&Color::Red));
    }

    // COVERS-PARTIAL: ATOM-613.4d-004
    #[test]
    fn test_counters_applied_before_switch_pt() {

        // Regression test: counters are in 7c, switch is 7d.
        // A 1/4 creature with two +1/+1 counters and a switch effect:
        // 7c: 1+2=3 / 4+2=6, then 7d: swap → 6/3
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall")
            .card_type(CardType::Creature)
            .power_toughness(1, 4)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);
        game.add_counters(id, CounterType::PlusOnePlusOne, 2);

        // Register a switch P/T effect (layer 7d)
        let effect = registered(
            id,
            Layer::Layer7dSwitchPT,
            1,
            EffectModification::SwitchPowerToughness,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // 7c: 1+2=3, 4+2=6; 7d: swap → 6/3
        assert_eq!(chars.power, Some(6));
        assert_eq!(chars.toughness, Some(3));
    }

    // === Layer 4 type-changing tests ===

    // COVERS-PARTIAL: ATOM-205.1b-004
    #[test]
    fn test_add_type_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Register "becomes also a creature" effect
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
    }

    #[test]
    fn test_remove_type() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mycosynth Lattice")
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Remove Creature type
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::RemoveType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(!chars.types.contains(&CardType::Creature));
    }

    // COVERS-PARTIAL: ATOM-205.1a-003
    #[test]
    fn test_set_subtypes_replaces_all() {
        use crate::types::card_types::{LandType, Subtype};
        use std::collections::HashSet;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Steam Vents")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Island))
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // SetSubtypes to just Forest
        let mut forest_set = HashSet::new();
        forest_set.insert(Subtype::Land(LandType::Forest));
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::SetSubtypes(forest_set),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Forest)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Island)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert_eq!(chars.subtypes.len(), 1);
    }

    #[test]
    fn test_add_subtype_preserves_existing() {
        use crate::types::card_types::{LandType, Subtype};

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Add Swamp subtype ("in addition to")
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSubtype(Subtype::Land(LandType::Swamp)),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Swamp)));
        assert_eq!(chars.subtypes.len(), 2);
    }

    #[test]
    fn test_add_supertype() {
        use crate::types::card_types::Supertype;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // Add Legendary supertype
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSupertype(Supertype::Legendary),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.supertypes.contains(&Supertype::Legendary));
    }

    // COVERS-PARTIAL: ATOM-613.1d-001
    #[test]
    fn test_type_change_before_color_change() {
        use crate::types::effects::{Duration, ObjectFilter};

        // Test layer ordering: L4 (type) applies before L5 (color)
        // A filter-based color effect that checks types should see the
        // type as it stands after L4.
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        // L4: Add Creature type
        let l4_effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(l4_effect);

        // L5: "Creatures are also red" (filter-based)
        let l5_source = crate::types::ids::new_object_id();
        let l5_effect = ContinuousEffect {
            id: 0,
            source: l5_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: ObjectFilter::ByType(CardType::Creature),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(l5_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Should be both Artifact and Creature (L4 applied)
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
        // Should be Red (L5 filter sees the L4-modified type = Creature)
        assert!(chars.colors.contains(&Color::Red));
    }

    // -----------------------------------------------------------------------
    // CR 614.12 — the look-ahead overlay (Phase RC-4)
    //
    // What the accessor pair perturbs, and — the one that matters — what it
    // does not: §5b's "one object is hypothetical; nothing else is".
    // -----------------------------------------------------------------------

    use crate::engine::layers::lookahead::{compute_as_entering, Lookahead};
    use crate::engine::layers::board::compute_board;
    use crate::objects::card_data::CardData;
    use crate::test_support::{creature_with_ability, put_on_battlefield, static_ability};
    use crate::types::effects::{
        AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, Primitive,
    };
    use std::sync::Arc;

    /// A 2/2 with "Creatures you control get +1/+1", itself included.
    fn anthem_bear() -> Arc<CardData> {
        creature_with_ability(
            "Anthem Bear",
            2,
            2,
            static_ability(Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                )),
            )),
        )
    }

    fn in_graveyard(game: &mut GameState, data: Arc<CardData>, owner: PlayerId) -> ObjectId {
        let obj = GameObject::new(data, owner, Zone::Graveyard);
        let id = obj.id;
        game.add_object(obj);
        game.players[owner].graveyard.push(id);
        id
    }

    #[test]
    fn test_look_ahead_seeds_the_proposed_controller_and_the_clock() {
        let mut game = GameState::new(2, 20);
        game.turn_number = 7;
        let bears = in_graveyard(
            &mut game,
            CardDataBuilder::new("Bears").card_type(CardType::Creature).power_toughness(2, 2).build(),
            0,
        );

        let frame = compute_as_entering(&game, bears, 1, &EnterMods::NONE).unwrap();
        assert_eq!(frame.controller, 1, "the proposed controller, not the owner");
        assert_eq!(frame.control_since_turn, 7, "CR 302.6's clock starts with the entry");

        let real = compute_characteristics(&game, bears).unwrap();
        assert_eq!(real.controller, 0);
        assert_eq!(real.control_since_turn, 0);
    }

    #[test]
    fn test_look_ahead_counters_feed_layers_6_and_7c() {
        let mut game = GameState::new(2, 20);
        let bears = in_graveyard(
            &mut game,
            CardDataBuilder::new("Bears").card_type(CardType::Creature).power_toughness(2, 2).build(),
            0,
        );
        let mods = EnterMods {
            tapped: false,
            counters: vec![(CounterType::PlusOnePlusOne, 2), (CounterType::Flying, 1)],
        };

        let frame = compute_as_entering(&game, bears, 0, &mods).unwrap();
        assert_eq!(frame.power, Some(4), "CR 122.1a counters it would enter with, at 7c");
        assert!(frame.keyword_flags.contains(&KeywordFlag::Flying), "CR 122.1b, at layer 6");

        let real = compute_characteristics(&game, bears).unwrap();
        assert_eq!(real.power, Some(2));
        assert!(!real.keyword_flags.contains(&KeywordFlag::Flying));
    }

    /// Clause (3) for an object that has not moved yet: the zone gate admits it
    /// under the look-ahead and nowhere else.
    #[test]
    fn test_look_ahead_admits_an_object_not_yet_in_the_zone_to_filters() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let bears = in_graveyard(
            &mut game,
            CardDataBuilder::new("Bears").card_type(CardType::Creature).power_toughness(2, 2).build(),
            0,
        );
        game.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: crate::types::ids::new_object_id(),
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::Indefinite,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ),
            },
            modification: EffectModification::ModifyPowerToughness {
                power: PtValue::Fixed(1),
                toughness: PtValue::Fixed(1),
            },
        });

        assert_eq!(compute_characteristics(&game, bears).unwrap().power, Some(2), "in a graveyard, no filter reaches it");
        assert_eq!(compute_as_entering(&game, bears, 0, &EnterMods::NONE).unwrap().power, Some(3), "as it would exist on the battlefield, the anthem does");
        assert_eq!(compute_as_entering(&game, bears, 1, &EnterMods::NONE).unwrap().power, Some(2), "under the other player it is not \"you control\"");
    }

    /// §5b — the asymmetry that must not be smoothed over. The entering Anthem
    /// Bear's own row is in *its* frame (clause 2) and in no other object's
    /// frame computed under the same look-ahead; once it has actually entered,
    /// the other object gets the anthem like everything else.
    #[test]
    fn test_look_ahead_rows_reach_only_the_entering_object() {
        let mut game = GameState::new(2, 20);
        let other = put_on_battlefield(
            &mut game,
            CardDataBuilder::new("Other Bears").card_type(CardType::Creature).power_toughness(2, 2).build(),
            0,
        );
        let anthem = in_graveyard(&mut game, anthem_bear(), 0);

        assert_eq!(
            compute_as_entering(&game, anthem, 0, &EnterMods::NONE).unwrap().power,
            Some(3),
            "CR 614.12 clause (2): its own anthem, in its own frame"
        );

        let lookahead = Lookahead::new(&game, anthem, 0, &EnterMods::NONE);
        let other_frame = compute_board(&game, Some(&lookahead)).take(other).unwrap();
        assert_eq!(other_frame.power, Some(2), "nothing else is hypothetical");

        game.move_object(anthem, Zone::Battlefield).unwrap();
        game.place_on_battlefield(anthem, 0, &EnterMods::NONE);
        assert_eq!(compute_characteristics(&game, other).unwrap().power, Some(3), "and after the entry it is a plain registered row");
    }

    /// A would-be row is subject to CR 604.2 exactly like a registered one.
    #[test]
    fn test_look_ahead_row_is_stripped_by_humility_before_it_applies() {
        let mut game = GameState::new(2, 20);
        put_on_battlefield(&mut game, crate::cards::phase_lf_cards::humility(), 1);
        let anthem = in_graveyard(&mut game, anthem_bear(), 0);

        let frame = compute_as_entering(&game, anthem, 0, &EnterMods::NONE).unwrap();
        assert_eq!(frame.power, Some(1), "Humility's 1/1 at 7b, and no anthem at 7c: the ability was gone at layer 6");
        assert!(frame.abilities.is_empty());
    }

    /// `settled_amount` is `evaluate_amount` over `Board::settled()`: a count
    /// enumerates the *real* battlefield, and "you" is the asking object's own
    /// controller. That is exactly what a cost effect at CR 601.2f is
    /// entitled to, since CR 613.11 leaves it a finished board
    /// (`cost-architecture.md` §3.7).
    #[test]
    fn settled_amount_counts_the_real_battlefield_with_you_the_objects_controller() {
        use crate::test_support::{card_of_type, put_in_hand, setup_two_player_game, vanilla_creature};
        use crate::types::effects::{AmountExpr, ObjectFilter, Selector};

        let mut game = setup_two_player_game();
        let expr = AmountExpr::CountOf(Selector::PermanentsMatching(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Artifact)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )));

        // A card in hand has no controller of its own, so "you" is its owner
        // (CR 108.4a) — the prospective caster, which is the whole reason the
        // castability preview can ask this at all.
        let mine = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), 0);
        let theirs = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), 1);
        assert_eq!(settled_amount(&expr, &game, mine), Some(0));

        put_on_battlefield(&mut game, card_of_type("Rock", CardType::Artifact), 0);
        put_on_battlefield(&mut game, card_of_type("Their Rock", CardType::Artifact), 1);
        put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        assert_eq!(settled_amount(&expr, &game, mine), Some(1), "P0's artifacts, not P1's and not a bear");
        assert_eq!(settled_amount(&expr, &game, theirs), Some(1), "and P1's, P1's");
    }

    /// `SourcePower` off the source's **effective** frame, which is what
    /// CR 613.11 entitles a cost effect to: an anthem on the source moves the
    /// amount, and a source with no power at all has no amount rather than
    /// zero.
    #[test]
    fn settled_amount_answers_source_power_off_the_effective_frame() {
        use crate::types::effects::AmountExpr::SourcePower;
        let mut game = crate::test_support::setup_two_player_game();
        let bears = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        assert_eq!(settled_amount(&SourcePower, &game, bears), Some(2));

        put_on_battlefield(&mut game, crate::cards::phase5_pre_cards::glorious_anthem(), 0);
        assert_eq!(settled_amount(&SourcePower, &game, bears), Some(3), "the anthem is in the frame");

        let rock = put_on_battlefield(&mut game, crate::test_support::card_of_type("Rock", CardType::Artifact), 0);
        assert_eq!(settled_amount(&SourcePower, &game, rock), None, "no power at all is not zero");
    }

    /// And the walk still refuses it. The reader may answer `SourcePower`
    /// because its `object` *is* the source; `evaluate_amount` is handed an
    /// affected object whose row's source is elsewhere, and reaching that
    /// object's frame mid-pass is the CR 613.8 dependency it declines to
    /// invent (`cost-architecture.md` §3.7).
    #[test]
    #[should_panic(expected = "no static-context evaluator")]
    fn the_walks_evaluator_still_refuses_source_power() {
        use crate::engine::layers::board::Board;
        let mut game = crate::test_support::setup_two_player_game();
        let bears = put_on_battlefield(&mut game, crate::cards::creatures::grizzly_bears(), 0);
        let chars = compute_characteristics(&game, bears).unwrap();
        let _ = evaluate_amount(
            &crate::types::effects::AmountExpr::SourcePower,
            &game,
            &chars,
            bears,
            LAYER_ORDER.len(),
            &Board::settled(),
            None,
        );
    }
}
