//! "As long as [X]" — a static ability's condition, evaluated during the pass
//! (`layers-architecture.md` §13b).
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
//! `ModeChosen` is resolution-only here and asserts the way `evaluate_amount`'s
//! resolution-only amounts do.

use crate::engine::layers::board::Board;
use crate::engine::layers::compute::LAYER_ORDER;
use crate::engine::layers::compute::{evaluate_amount, object_matches_filter, FilterPlayers};
use crate::state::battlefield::CostChoices;
use crate::state::game_state::GameState;
use crate::state::player::PlayerState;
use crate::types::costs::AdditionalCost;
use crate::types::effects::{Condition, ObjectFilter, PlayerFact, PlayerSet};
use crate::types::history::HistoryCount;
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

    // **A new leaf lands here and in `board::condition_reads`.** Both matches
    // are exhaustive with no wildcard, so the compiler refuses to build until
    // each has an arm — but it can only make you *write* the second one, not
    // get it right. An arm here that reads a frame and a `condition_reads`
    // arm that declares nothing gives a correct answer in the wrong *order*:
    // the pair is settled "independent" by the static check and never reaches
    // CR 613.8's hypothetical. `phase_li3_integration_test`'s Simian Clause
    // board is what that failure looks like.
    match condition {
        // "As long as you control a Forest", "as long as an opponent has 10 or
        // less life": a fact about each player the set names.
        Condition::Player { whose, fact } => player_fact_holds(whose, fact, game, board, source, layer_index),

        // CR 113.6b's clause, and **the leg that retires Wonder's row**: the
        // grant exists while the card is in the graveyard, and this is asked at
        // every layer, so a Wonder that is exiled stops granting on the very next
        // walk without anything reconciling the registry (§13d decision 3).
        //
        // The gate is `in_zones_or_entering` rather than zone equality: under a
        // CR 614.12 look-ahead the entering object is still in its source zone,
        // and 614.12 asks what it *would* be on the battlefield — so equality
        // would answer `false` for the one question being asked counterfactually.
        //
        // **Battlefield sources never see a `false` here**, because leaving the
        // battlefield calls `remove_by_source`, which drops every row of that
        // source whatever its duration. That is still not a second spelling of the
        // duration — the duration decides whether the row is in the registry, and
        // this decides whether the effect exists given that it is.
        Condition::SourceInZone(zones) => board.in_zones_or_entering(game, source, *zones),

        // Every clause. Short-circuits, so a `SourceInZone` written first —
        // which is how a card's text reads — costs nothing on the boards
        // where it is false.
        Condition::All(clauses) => clauses
            .iter()
            .all(|c| holds(c, game, board, source, layer_index)),

        // "As long as this artifact is untapped" — Trinisphere, Winter Orb,
        // Static Orb. A status (CR 110.5), read off the entity — under a
        // look-ahead, the entering one — and never off a frame, since no
        // layer writes it. A source with no entity is not on the battlefield
        // and so is not untapped either.
        Condition::SourceUntapped => board.entity(game, source).is_some_and(|entity| !entity.tapped),

        // CR 303.4m — whatever the source is attached to *now*, re-read at
        // every layer, exactly as `ObjectSet::Host` is. An unattached
        // source matches nothing, so its conditional effect does not exist.
        Condition::HostMatches(filter) => {
            let Some(host) = game.battlefield.get(&source).and_then(|e| e.attached_to) else {
                return false;
            };
            let Some(chars) = board.frame_of(game, host, layer_index) else {
                return false;
            };
            let mut players = FilterPlayers::for_source(source, game, board, layer_index);
            object_matches_filter(filter, host, &chars, &mut players)
        }

        // A turn summary's count (§3.10), for "you" as every leaf here reads it:
        // the source's controller. The counts are off `GameState`.
        Condition::ThisTurn(count) => history_holds(count, HistorySpan::ThisTurn, game, board, source, layer_index),
        Condition::LastTurn(count) => history_holds(count, HistorySpan::LastTurn, game, board, source, layer_index),
        Condition::SinceYourLastTurn(count) => {
            history_holds(count, HistorySpan::SinceYourLastTurn, game, board, source, layer_index)
        }
        Condition::ThisGame(count) => history_holds(count, HistorySpan::ThisGame, game, board, source, layer_index),
        // CR 603.7h: the resolving ability's count, which its own resolution
        // has not advanced yet. False outside a resolution, where nothing is
        // resolving to be counted.
        Condition::ResolvedThisTurn(n) => game
            .resolving
            .as_ref()
            .and_then(|r| r.identity)
            .is_some_and(|identity| game.resolutions_this_turn_of(identity) + 1 == *n),

        // CR 702.33d, read off the cost decisions and not off the cast: a copy
        // of a kicked spell isn't cast and is kicked (CR 707.10), and so is
        // the token it becomes; a permanent that was never a spell is not.
        Condition::SpellWasKicked => cost_choices(game, source).is_some_and(|choices| {
            choices.additional.iter().any(|c| matches!(c, AdditionalCost::Kicker(_)))
        }),

        // An answer a *resolution* had and a static ability never does:
        // CR 700.2's modes are chosen as a spell is cast, and nothing carries
        // them past it. A card author reaching for one on a static ability is
        // making the `evaluate_amount` mistake and gets the same treatment —
        // stopped in debug, declining in release, never inventing an answer.
        Condition::ModeChosen(_) => {
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

/// [`holds`] against the settled board at the full ceiling — the read-side
/// view a top-level query takes, so every leaf answers as the live pass
/// would have at its last layer.
///
/// The reader a *post-layer* consumer of `Condition` uses:
/// `engine::cost_determination::cost_modifications_for` today (CR 613.11 applies cost effects
/// after every layer, so a conditional one is asked here), and critical-path
/// item 6's intervening "if" next. A reader, not a language — the leaves
/// and their evaluators are [`holds`]'s, unchanged.
pub fn settled_holds(condition: &Condition, game: &GameState, source: ObjectId) -> bool {
    holds(condition, game, &Board::settled(), source, LAYER_ORDER.len())
}

/// Which turns a history leaf reads.
#[derive(Clone, Copy)]
enum HistorySpan {
    ThisTurn,
    LastTurn,
    SinceYourLastTurn,
    ThisGame,
}

/// A history leaf: the counts `count.whose` names, each over `span`, summed.
fn history_holds(
    count: &HistoryCount,
    span: HistorySpan,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
) -> bool {
    let Some(you) = you_for(game, board, source, layer_index) else {
        return false;
    };
    let now = game.turn_number;
    let yours = game.players.get(you).map(|p| &p.history);
    let total: u64 = game
        .players
        .iter()
        .enumerate()
        .filter(|(player, _)| count.whose.contains(you, *player))
        .map(|(player, state)| {
            let theirs = &state.history;
            let counts = match (span, yours) {
                (HistorySpan::ThisTurn, _) => theirs.this_turn(now),
                (HistorySpan::LastTurn, _) => theirs.last_turn(now),
                (HistorySpan::SinceYourLastTurn, Some(yours)) => yours.since_your_last_turn(player, theirs),
                (HistorySpan::ThisGame, _) | (HistorySpan::SinceYourLastTurn, None) => theirs.this_game(),
            };
            counts.count(count.fact)
        })
        .sum();
    count.is.met_by(total)
}

/// CR 707.10's cost decisions for `source`: the resolving spell's, whose
/// `StackEntry` resolution has taken, or the permanent's it became.
fn cost_choices(game: &GameState, source: ObjectId) -> Option<&CostChoices> {
    match &game.resolving {
        Some(r) if r.id == source => Some(&r.cost_choices),
        _ => game.battlefield.get(&source).map(|e| &e.cost_choices),
    }
}

/// CR 109.5's "you": the source's controller, off its live frame, or its
/// owner if it has no controller. `None` only when the source no longer exists
/// anywhere (a token that has ceased to exist), where no "you" can be read and
/// a leaf answers false; a trigger's recheck gets its locked controller
/// instead (item 169, TR-2b).
fn you_for(
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

/// [`Condition::Player`]: does any player `whose` names, among those still in
/// the game, meet `fact`? "Whose" is resolved against CR 109.5's "you", the
/// source's controller off its live frame. A departed player's permanents
/// left with them (CR 800.4a), so no fact about the board can name them.
fn player_fact_holds(
    whose: &PlayerSet,
    fact: &PlayerFact,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
) -> bool {
    let Some(you) = you_for(game, board, source, layer_index) else {
        return false;
    };
    let named = |player: PlayerId| game.in_game(player) && whose.contains(you, player);
    let any_named = |meets: &dyn Fn(&PlayerState) -> bool| {
        game.players.iter().enumerate().any(|(player, state)| named(player) && meets(state))
    };
    match fact {
        PlayerFact::ControlsPermanent(filter) => {
            controls_matching(filter, game, board, source, layer_index, &named)
        }
        // The threshold is resolved against the source's own frame, which is
        // where a static ability's dynamic number is read everywhere else.
        PlayerFact::LifeAtLeast(expr) | PlayerFact::LifeAtMost(expr) => {
            let Some(chars) = board.frame_of(game, source, layer_index) else {
                return false;
            };
            let Some(threshold) = evaluate_amount(expr, game, &chars, source, layer_index, board, None)
            else {
                return false;
            };
            let threshold = threshold as i64;
            match fact {
                PlayerFact::LifeAtLeast(_) => any_named(&|state| state.life_total >= threshold),
                _ => any_named(&|state| state.life_total <= threshold),
            }
        }
        PlayerFact::LibraryEmpty => any_named(&|state| state.library.is_empty()),
        // A graveyard card may be a non-member of the pass, in which case its
        // frame is its own CDA walk at this ceiling (CR 604.3).
        PlayerFact::CardInGraveyard(filter) => {
            let mut players = FilterPlayers::for_source(source, game, board, layer_index);
            game.players.iter().enumerate().any(|(player, state)| {
                named(player)
                    && state.graveyard.iter().any(|&card| {
                        board
                            .frame_of(game, card, layer_index)
                            .is_some_and(|chars| object_matches_filter(filter, card, &chars, &mut players))
                    })
            })
        }
    }
}

/// Is there a battlefield permanent matching `filter` whose controller
/// `named` accepts?
///
/// `Board::battlefield_ids` rather than the working set: this is a look over
/// the battlefield, and §5b's boundary makes a merely *entering* permanent
/// visible to filters and invisible to counts.
fn controls_matching(
    filter: &ObjectFilter,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    named: &dyn Fn(PlayerId) -> bool,
) -> bool {
    let mut players = FilterPlayers::for_source(source, game, board, layer_index);
    board.battlefield_ids(game).into_iter().any(|id| {
        let Some(chars) = board.frame_of(game, id, layer_index) else {
            return false;
        };
        named(chars.controller) && object_matches_filter(filter, id, &chars, &mut players)
    })
}

// ---------------------------------------------------------------------------
// The leaves with no registered consumer.
//
// Kird Ape covers `PlayerFact::ControlsPermanent` and the Flight Clause covers
// `HostMatches`, both end to end in `tests/phase_li3_integration_test.rs`.
// The rest are exercised here against a settled board, because a leaf no
// card reaches is exactly the kind of code that is wrong and quiet — the
// failure mode `static_ability_atoms`' doc is written about.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{basic_lands, creatures};
    use crate::test_support::{
        card_of_type, put_in_graveyard, put_on_battlefield, setup_game, setup_two_player_game,
        vanilla_creature,
    };
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::effects::AmountExpr;
    use crate::types::zones::ZoneSet;

    /// A status leaf reads the entity, not a frame: tapping the source flips
    /// it with no zone change and no registry write.
    #[test]
    fn source_untapped_reads_the_entitys_status() {
        let mut game = setup_two_player_game();
        let sphere = put_on_battlefield(&mut game, card_of_type("Sphere", CardType::Artifact), 0);
        assert!(settled_holds(&Condition::SourceUntapped, &game, sphere));
        game.battlefield.get_mut(&sphere).unwrap().tapped = true;
        assert!(!settled_holds(&Condition::SourceUntapped, &game, sphere));
        let dead = put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
        assert!(!settled_holds(&Condition::SourceUntapped, &game, dead), "no entity, not untapped");
    }

    fn yours(fact: PlayerFact) -> Condition {
        Condition::Player { whose: PlayerSet::You, fact }
    }

    fn an_opponents(fact: PlayerFact) -> Condition {
        Condition::Player { whose: PlayerSet::Opponents, fact }
    }

    #[test]
    fn life_thresholds_read_the_source_controllers_total() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        game.players[0].life_total = 20;
        game.players[1].life_total = 3;

        assert!(settled_holds(&yours(PlayerFact::LifeAtLeast(AmountExpr::Fixed(20))), &game, bears));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtLeast(AmountExpr::Fixed(21))), &game, bears));
        assert!(settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(20))), &game, bears));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(19))), &game, bears));

        // CR 109.5 — the *source's* controller, not either player at large.
        let theirs = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
        assert!(settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(3))), &game, theirs));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(3))), &game, bears));
    }

    /// A set of players asks each one alone: Bloodghast's "an opponent has 10
    /// or less life" is one opponent at 10, not the opponents' total, and a
    /// player who has left the game is no opponent (CR 800.4a).
    #[test]
    fn an_opponent_is_any_one_opponent_still_in_the_game() {
        let mut game = setup_game(4);
        let ghast = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        for seat in 1..4 {
            game.players[seat].life_total = 12;
        }
        let ten_or_less = an_opponents(PlayerFact::LifeAtMost(AmountExpr::Fixed(10)));
        assert!(!settled_holds(&ten_or_less, &game, ghast), "three at 12, and no sum");

        game.players[2].life_total = 0;
        game.player_lost[2] = true;
        assert!(!settled_holds(&ten_or_less, &game, ghast), "the one at 0 has left the game");

        game.players[3].life_total = 10;
        assert!(settled_holds(&ten_or_less, &game, ghast), "one opponent at 10 is enough");
    }

    #[test]
    fn card_in_graveyard_reads_your_graveyard_through_the_card_filter() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let card_in_yours = |filter: ObjectFilter| yours(PlayerFact::CardInGraveyard(filter));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::All), &game, bears));

        put_in_graveyard(&mut game, basic_lands::forest(), 0);
        assert!(settled_holds(&card_in_yours(ObjectFilter::All), &game, bears));
        assert!(settled_holds(&card_in_yours(ObjectFilter::ByType(CardType::Land)), &game, bears));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::ByType(CardType::Creature)), &game, bears));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::ByColor(Color::Red)), &game, bears));

        // Your graveyard, not everybody's: the same card under the opponent
        // answers for their graveyard, which is empty.
        let theirs = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
        assert!(!settled_holds(&card_in_yours(ObjectFilter::All), &game, theirs));
    }

    #[test]
    fn you_and_an_opponent_control_each_others_complement() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let forest = ObjectFilter::BySubtype(crate::types::card_types::Subtype::Land(
            crate::types::card_types::LandType::Forest,
        ));
        let you_control = yours(PlayerFact::ControlsPermanent(forest.clone()));
        let an_opponent_controls = an_opponents(PlayerFact::ControlsPermanent(forest));

        assert!(!settled_holds(&you_control, &game, bears));
        assert!(!settled_holds(&an_opponent_controls, &game, bears));

        put_on_battlefield(&mut game, basic_lands::forest(), 1);
        assert!(!settled_holds(&you_control, &game, bears));
        assert!(settled_holds(&an_opponent_controls, &game, bears));

        put_on_battlefield(&mut game, basic_lands::forest(), 0);
        assert!(settled_holds(&you_control, &game, bears));
        assert!(settled_holds(&an_opponent_controls, &game, bears));
    }

    /// CR 113.6b's leaf, both directions: the battlefield spelling answers
    /// what `SourceOnBattlefield` answered, and the graveyard spelling is the
    /// one Wonder needs.
    #[test]
    fn source_in_zone_is_the_zone_gate_in_both_directions() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let dead = put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
        let on_bf = Condition::SourceInZone(ZoneSet::BATTLEFIELD);
        let in_gy = Condition::SourceInZone(ZoneSet::GRAVEYARD);
        assert!(settled_holds(&on_bf, &game, bears));
        assert!(!settled_holds(&on_bf, &game, dead));
        assert!(settled_holds(&in_gy, &game, dead));
        assert!(!settled_holds(&in_gy, &game, bears));

        // A set, so one clause can name several zones — Mycosynth Lattice's
        // shape on the source side.
        let anywhere_else = Condition::SourceInZone(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD);
        assert!(settled_holds(&anywhere_else, &game, dead));
        assert!(!settled_holds(&anywhere_else, &game, bears));
    }

    /// Every clause, and an empty conjunction is vacuously true.
    #[test]
    fn all_holds_only_when_every_clause_does() {
        let mut game = setup_two_player_game();
        let dead = put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
        let island = ObjectFilter::BySubtype(crate::types::card_types::Subtype::Land(
            crate::types::card_types::LandType::Island,
        ));
        // Wonder's own condition, clause for clause.
        let wonder = Condition::All(vec![
            Condition::SourceInZone(ZoneSet::GRAVEYARD),
            yours(PlayerFact::ControlsPermanent(island)),
        ]);
        assert!(!settled_holds(&wonder, &game, dead), "in the graveyard, no Island");
        put_on_battlefield(&mut game, basic_lands::island(), 0);
        assert!(settled_holds(&wonder, &game, dead), "in the graveyard, an Island");

        assert!(settled_holds(&Condition::All(Vec::new()), &game, dead));
    }

    /// An unattached source is attached to nothing, so nothing matches — the
    /// same answer `ObjectSet::Host` gives, and the reason an Aura's
    /// conditional effect simply does not exist before it is attached.
    #[test]
    fn an_unattached_source_matches_no_host() {
        let mut game = setup_two_player_game();
        let aura = put_on_battlefield(&mut game, card_of_type("Loose Aura", CardType::Enchantment), 0);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert!(!settled_holds(&Condition::HostMatches(ObjectFilter::All), &game, aura));

        assert!(game.attach(aura, bears));
        assert!(settled_holds(&Condition::HostMatches(ObjectFilter::All), &game, aura));
        assert!(settled_holds(
            &Condition::HostMatches(ObjectFilter::ByType(CardType::Creature)),
            &game,
            aura
        ));
        assert!(!settled_holds(
            &Condition::HostMatches(ObjectFilter::ByType(CardType::Artifact)),
            &game,
            aura
        ));
    }

    /// A condition on an object the store has never heard of has no answer,
    /// and says so rather than reading somebody else's frame.
    #[test]
    fn a_condition_on_a_missing_source_is_false() {
        let game = setup_two_player_game();
        let anywhere = Condition::SourceInZone(ZoneSet::ALL);
        assert!(!settled_holds(&anywhere, &game, ObjectId::UNASSIGNED));
    }

    /// "If you put a permanent with a kicker ability onto the battlefield
    /// without casting it, you can't kick it" (Archangel of Wrath's ruling):
    /// it was never a spell, so there were no costs to decide.
    #[test]
    fn a_permanent_that_was_never_a_spell_was_not_kicked() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        assert!(!settled_holds(&Condition::SpellWasKicked, &game, bears));
    }

    #[test]
    #[should_panic(expected = "no static-context evaluator")]
    fn mode_chosen_on_a_static_is_loud() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let _ = settled_holds(&Condition::ModeChosen(0), &game, bears);
    }
}
