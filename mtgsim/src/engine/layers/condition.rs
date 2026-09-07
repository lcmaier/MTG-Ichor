//! "As long as [X]" — a static ability's condition, evaluated during the pass
//! (`layers-architecture.md` §13b, LI-3).
//!
//! CR 604.2 makes a static ability's effect active "as long as the permanent
//! with the ability remains on the battlefield and has the ability". A
//! conditional static adds one clause to that same sentence: the effect
//! exists while the condition holds. So this is not a new kind of row — the
//! lowering drops the conditional wrapper and registers the inner atom's rows
//! exactly as an unconditional static's, and the condition is re-read off the
//! ability itself every time `board::static_ability_still_exists` asks
//! whether the effect is there. Nothing about a `ContinuousEffect` changes.
//!
//! **The frame is the pass's live board**, which is what makes the answer the
//! CR's: a condition reading types sees layer 4 applied, and Kird Ape loses
//! its bonus at 7c because a Taiga stopped being a Forest at 4. CR 613.6 then
//! covers the later layers of a multi-layer effect — once it has started
//! applying, a condition that goes false does not retract it.
//!
//! `Condition` is `types::effects`' own enum, written for CR 603.4's
//! intervening "if" and shared rather than duplicated (§13b decision 5), so
//! two arms here are resolution-only and assert the way `evaluate_amount`'s
//! resolution-only amounts do.

use crate::engine::layers::board::Board;
use crate::engine::layers::compute::{evaluate_amount, permanent_matches_filter, FilterPlayers};
use crate::state::game_state::GameState;
use crate::types::effects::{AmountExpr, CardFilter, Condition, PermanentFilter};
use crate::types::ids::{ObjectId, PlayerId};

/// Does `source`'s "as long as" clause hold against the board as the pass has
/// it right now?
///
/// `source` is the object the ability is on — CR 109.5's "you" is its
/// *current* controller, read off its live frame exactly as a filter row's
/// is, which is why this takes an object rather than a player.
pub(super) fn holds(
    condition: &Condition,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
) -> bool {
    // Every arm below reads the source's frame or its owner, and a condition
    // on an object that is not in the store has no answer. The existence
    // check has already found the ability on that frame, so this is a guard
    // rather than a case.
    if !game.objects.contains_key(&source) {
        return false;
    }

    match condition {
        // "as long as you control a Forest" / "as long as an opponent
        // controls a creature". The controller test is the *variant's*, not
        // the filter's — that is what separates the two — and it reads the
        // effective controller, so a Layer 2 steal moves the answer.
        Condition::ControlPermanent(filter) => {
            controls_matching(filter, game, board, source, layer_index, true)
        }
        Condition::OpponentControlsPermanent(filter) => {
            controls_matching(filter, game, board, source, layer_index, false)
        }

        Condition::LifeAtLeast(expr) => life_compare(expr, game, board, source, layer_index, true),
        Condition::LifeAtMost(expr) => life_compare(expr, game, board, source, layer_index, false),

        // "as long as there's a [X] card in your graveyard". The variant
        // carries no player, and the printed shape it is written for is
        // *your* graveyard; a condition over somebody else's is §15.1's
        // `ZoneContainsCard`, which has an owner on it, when a card wants
        // one. A graveyard card is a non-member of the pass, so its frame is
        // its own CDA walk at this ceiling (CR 604.3).
        Condition::CardInGraveyard(filter) => {
            let Some(you) = controller_of(game, board, source, layer_index) else {
                return false;
            };
            let Some(player) = game.players.get(you) else { return false };
            player.graveyard.iter().any(|&card| {
                board.frame_of(game, card, layer_index).is_some_and(|chars| match filter {
                    CardFilter::All => true,
                    CardFilter::ByType(t) => chars.types.contains(t),
                    CardFilter::ByColor(c) => chars.colors.contains(c),
                })
            })
        }

        // The zone gate a filter row asks, for the same reason: under a CR
        // 614.12 look-ahead the entering object's own conditional static has
        // to function, and it is still in its source zone while its entry is
        // being decided (RC-4b).
        Condition::SourceOnBattlefield => board.in_battlefield_zone_or_entering(game, source),

        // CR 303.4m — whatever the source is attached to *now*, re-read at
        // every layer, exactly as `AffectedSet::Host` is. An unattached
        // source matches nothing, so its conditional effect does not exist.
        Condition::HostMatches(filter) => {
            let Some(host) = game.battlefield.get(&source).and_then(|e| e.attached_to) else {
                return false;
            };
            let Some(chars) = board.frame_of(game, host, layer_index) else {
                return false;
            };
            let mut players = FilterPlayers::for_source(source, game, board, layer_index);
            permanent_matches_filter(filter, host, &chars, &mut players)
        }

        // Both are answers a *resolution* had and a static ability never
        // does: CR 702.33a's kicker was paid as a spell was cast, and CR
        // 700.2's modes were chosen then too. A card author reaching for one
        // on a static ability is making the `evaluate_amount` mistake and
        // gets the same treatment — stopped in debug, declining in release,
        // never inventing an answer.
        Condition::SpellWasKicked | Condition::ModeChosen(_) => {
            debug_assert!(
                false,
                "{:?} has no static-context evaluator: it reads a choice made \
                 while a spell was cast, and a static ability's condition is \
                 re-read every layer with no cast to read from. The effect \
                 would simply never exist.",
                condition
            );
            false
        }
    }
}

/// CR 109.5's "you", off the source's live frame.
fn controller_of(
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
) -> Option<PlayerId> {
    board
        .frame_of(game, source, layer_index)
        .map(|frame| frame.controller)
        .or_else(|| game.objects.get(&source).map(|obj| obj.owner))
}

/// Is there a battlefield permanent matching `filter` that "you" control
/// (`mine`), or that somebody else does?
///
/// `Board::battlefield_ids` rather than the working set: this is a look over
/// the battlefield, and §5b's boundary makes a merely *entering* permanent
/// visible to filters and invisible to counts.
fn controls_matching(
    filter: &PermanentFilter,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    mine: bool,
) -> bool {
    let mut players = FilterPlayers::for_source(source, game, board, layer_index);
    let you = players.you();
    board.battlefield_ids(game).into_iter().any(|id| {
        let Some(chars) = board.frame_of(game, id, layer_index) else {
            return false;
        };
        // CR 102.2/102.3 — "an opponent" is "somebody who isn't you", the
        // same answer in two-player and multiplayer without the type having
        // to name a player.
        (chars.controller == you) == mine
            && permanent_matches_filter(filter, id, &chars, &mut players)
    })
}

/// "as long as you have N or more life", and its mirror. The amount is
/// resolved against the source's own frame, which is where a static
/// ability's dynamic number is read everywhere else.
fn life_compare(
    expr: &AmountExpr,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    at_least: bool,
) -> bool {
    let Some(chars) = board.frame_of(game, source, layer_index) else {
        return false;
    };
    let Some(threshold) = evaluate_amount(expr, game, &chars, source, layer_index, board, None)
    else {
        return false;
    };
    let Some(player) = game.players.get(chars.controller) else { return false };
    if at_least {
        player.life_total >= threshold as i64
    } else {
        player.life_total <= threshold as i64
    }
}
