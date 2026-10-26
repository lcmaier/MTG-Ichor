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

use crate::engine::layers::board::{Board, FrameRef};
use crate::engine::layers::compute::LAYER_ORDER;
use crate::engine::layers::compute::{evaluate_amount, object_matches_filter, FilterPlayers};
use crate::engine::object_filter;
use crate::state::battlefield::CostChoices;
use crate::state::game_state::GameState;
use crate::state::player::PlayerState;
use crate::types::costs::AdditionalCost;
use crate::types::effects::{PlayerRef, Antecedent, Condition, ObjectFilter, PlayerFact, PlayerSet};
use crate::types::history::{CountIs, HistoryCount, HistorySpan};
use crate::oracle::characteristics::ObjectInformation;
use crate::engine::triggers::binding::TriggeringEventView;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::Lki;
use crate::state::layer_memo::GatedWalkInput;

/// What a condition's words "this" and "it" refer to, when the caller knows
/// more than the board does.
///
/// A condition is answered by looking at the game. But a triggered ability's
/// "if" is often asked after its objects have left the battlefield: persist's
/// "if it had no -1/-1 counters on it" is asked of a creature that has just
/// died, and the board now holds only the card in the graveyard. CR 113.7a
/// and 608.2h answer from how the object last existed, so the caller passes
/// that here (`triggers-architecture.md` §6.1):
/// - "this", the ability's source, once it has left: `source_lki`;
/// - "it", an object of the triggering event: `triggering_event`.
///
/// A static ability's "as long as" refers to neither, and passes [`Self::NONE`].
#[derive(Clone, Copy)]
pub struct ConditionReferents<'a> {
    /// The source as it last existed, once it has left; `None` while it is
    /// still there, where the board answers.
    pub source_lki: Option<&'a Lki>,
    /// The objects of the triggering event, each read live while it is where
    /// the event left it, or as it last existed when its record framed it or
    /// it has left since. None outside a trigger, and for a "one or more"
    /// trigger, which has no single triggering event.
    pub triggering_event: TriggeringEventView<'a>,
}

impl ConditionReferents<'_> {
    /// Nothing beyond the board: every static ability's condition.
    pub const NONE: ConditionReferents<'static> =
        ConditionReferents { source_lki: None, triggering_event: TriggeringEventView::None };
}

impl<'a> ConditionReferents<'a> {
    /// Finds the object a text's word for one means. `ThisObject` is
    /// `source`, the object whose ability it is; `ThisObjectsHost` what it is
    /// attached to now, or was as it left (CR 303.4m, 608.2h);
    /// `Triggering(role)` the object that played that part in the triggering
    /// event. Each comes back as `ObjectInformation`: the object as it is
    /// now, or as it last existed once it has left (CR 608.2h). `None` when
    /// there is no such object, and for `Referred`: a condition has no
    /// remembered object.
    pub fn object_for(&self, game: &GameState, word: Antecedent, source: ObjectId) -> Option<ObjectInformation<'a>> {
        match word {
            Antecedent::ThisObject => Some(ObjectInformation::of(source, self.source_lki)),
            Antecedent::ThisObjectsHost => {
                let host = match self.source_lki {
                    Some(lki) => lki.host(),
                    None => crate::oracle::board::host_of(game, source),
                }?;
                // The triggering event's own object, where the host is one,
                // as its record framed it.
                Some(match self.triggering_event.object_with_id(host) {
                    Some(framed @ ObjectInformation::LastKnown(_)) => framed,
                    _ => ObjectInformation::Current(host),
                })
            }
            Antecedent::Triggering(role) => self.triggering_event.object(role),
            Antecedent::Referred => None,
        }
    }
}

/// Does `source`'s "as long as" clause hold against the board as the pass has
/// it right now?
///
/// `source` is the object the ability is on. CR 109.5's "you" is
/// `locked_you` when the asker has fixed it ([`settled_holds`] says who
/// does), and otherwise the source's *current* controller, read off its live
/// frame exactly as a filter row's is.
pub(super) fn holds(
    condition: &Condition,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    locked_you: Option<PlayerId>,
    referents: ConditionReferents<'_>,
) -> bool {
    // Every arm below reads the source's frame or its owner, and a condition
    // on an object that is not in the store has no answer unless its last
    // known information gives one. The existence check has already found the
    // ability on that frame, so this is a guard rather than a case.
    if referents.source_lki.is_none() && !game.objects.contains_key(&source) {
        return false;
    }
    // "Its owner" for a filter, once the store has lost the source (CR 108.3).
    let owner_if_gone = referents.source_lki.map(Lki::owner);
    let history_players = || FilterPlayers::for_source(source, game, board, layer_index, locked_you, owner_if_gone);

    // **A new leaf lands here and in `board::condition_reads`**, both exhaustive,
    // so the compiler asks for the second arm but cannot check it: one that
    // declares nothing it reads settles the pair "independent" and skips CR
    // 613.8's hypothetical (`phase_li3_integration_test`'s Simian Clause board).
    match condition {
        // "As long as you control a Forest", "as long as an opponent has 10 or
        // less life": a fact about each player the set names.
        Condition::Player { players, fact: player_fact } => {
            let named;
            let players = match players {
                // CR 508.5: the creature's defending player. It attacks what
                // the battlefield says, or what the history kept.
                PlayerSet::Player(PlayerRef::DefendingPlayer(of)) => {
                    let note = |input| board.note_gated_read(input);
                    match referents.object_for(game, *of, source).and_then(|it| it.defending_player(game, &note)) {
                        Some(defending) => {
                            named = PlayerSet::Fixed(vec![defending]);
                            &named
                        }
                        None => return false,
                    }
                }
                PlayerSet::Player(PlayerRef::Player(id)) => {
                    named = PlayerSet::Fixed(vec![*id]);
                    &named
                }
                // Defending player with no creature to settle which is the
                // controller's choice (CR 802.2a), and an event's, a
                // remembered or an acted-on player is no condition's to read
                // yet: each waits for its first card, loud in a debug build.
                PlayerSet::DefendingPlayers | PlayerSet::Player(_) => {
                    debug_assert!(false, "a condition asks {players:?}, which no condition reads yet");
                    return false;
                }
                PlayerSet::Nobody | PlayerSet::You | PlayerSet::Opponents | PlayerSet::Everyone | PlayerSet::Fixed(_) => players,
            };
            player_fact_holds(players, player_fact, game, board, source, layer_index, locked_you, owner_if_gone)
        }

        // Every clause, short-circuiting.
        Condition::All(clauses) => clauses
            .iter()
            .all(|c| holds(c, game, board, source, layer_index, locked_you, referents)),
        Condition::Not(clause) => !holds(clause, game, board, source, layer_index, locked_you, referents),

        // "As long as this artifact is untapped" — Trinisphere, Winter Orb,
        // Static Orb. A status (CR 110.5), read off the entity — under a
        // look-ahead, the entering one — and never off a frame, since no
        // layer writes it. A source with no entity is not on the battlefield
        // and so is not untapped either; one that left reads as it left.
        Condition::SourceUntapped => match referents.source_lki {
            Some(lki) => lki.was_tapped() == Some(false),
            None => {
                board.note_gated_read(GatedWalkInput::TappedStatus);
                board.entity(game, source).is_some_and(|entity| !entity.tapped)
            }
        },
        Condition::SourceTapped => match referents.source_lki {
            Some(lki) => lki.was_tapped() == Some(true),
            None => {
                board.note_gated_read(GatedWalkInput::TappedStatus);
                board.entity(game, source).is_some_and(|entity| entity.tapped)
            }
        },
        // "This" or "it", as it last existed once it has left (persist's
        // creature, which died), and off the entity otherwise.
        Condition::CounterCount { object, kind, count } => {
            let Some(asked) = referents.object_for(game, *object, source) else { return false };
            let n = match asked.lki() {
                Some(lki) => lki.counters_of(*kind),
                None => match board.entity(game, asked.id()) {
                    Some(entity) => entity.counters_of(*kind),
                    None => return false,
                },
            };
            count.satisfied_by(u64::from(n))
        }

        // CR 303.4m — whatever the source is attached to *now*, re-read at
        // every layer, exactly as `ObjectSet::Host` is. An unattached
        // source matches nothing, so its conditional effect does not exist.
        // A source that left was attached to what its last known information
        // says, and CR 608.2h reads that host where it was, or as it left
        // when it is the triggering object. "This" and "it" read the same
        // way, as `CounterCount` reads them.
        Condition::ObjectMatches(object, filter) => {
            let Some(asked) = referents.object_for(game, *object, source) else {
                return false;
            };
            let chars = match asked {
                ObjectInformation::LastKnown(lki) => Some(FrameRef::Shared(std::sync::Arc::clone(lki.frame()))),
                _ if *object == Antecedent::ThisObjectsHost
                    && referents.source_lki.is_some()
                    && !game.battlefield.contains_key(&asked.id()) =>
                {
                    None
                }
                _ => board.frame_of(game, asked.id(), layer_index),
            };
            let Some(chars) = chars else {
                return false;
            };
            let mut players = FilterPlayers::for_source(source, game, board, layer_index, locked_you, owner_if_gone);
            object_matches_filter(filter, asked.id(), &chars, &mut players)
        }

        // A history count, for "you" as every leaf here reads it: the
        // source's controller. The rows are off `GameState`.
        Condition::ThisTurn(history, count) => {
            history_holds(history, *count, HistorySpan::ThisTurn, game, board, layer_index, &mut history_players())
        }
        Condition::LastTurn(history, count) => {
            history_holds(history, *count, HistorySpan::LastTurn, game, board, layer_index, &mut history_players())
        }
        Condition::SinceYourLastTurn(history, count) => {
            history_holds(history, *count, HistorySpan::SinceYourLastTurn, game, board, layer_index, &mut history_players())
        }
        Condition::ThisGame(history, count) => {
            history_holds(history, *count, HistorySpan::ThisGame, game, board, layer_index, &mut history_players())
        }
        // CR 122.7, 714.2b: some record of the trigger's left the count at
        // least `n` from below it. Off the records, never the board, so it
        // answers alike at both instants; false outside a trigger.
        Condition::CounterCountReached { counter_type, n } => referents.triggering_event.any_record(|record| {
            matches!(
                record.event,
                crate::events::event::GameEvent::CountersPutOn { counter_type: kind, n: put, now, .. }
                    if kind == *counter_type && now.saturating_sub(put) < *n && *n <= now
            )
        }),

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
        Condition::SpellWasKicked => {
            let kicked =
                |choices: &CostChoices| choices.additional.iter().any(|c| matches!(c, AdditionalCost::Kicker(_)));
            match referents.source_lki {
                Some(lki) => kicked(lki.cost_choices()),
                None => game.cost_choices_of(source).is_some_and(|choices| kicked(&choices)),
            }
        }

        // CR 118.12's answer is the resolver's walk's, which answers it before
        // any evaluator is asked; reaching here is a condition the walk never
        // saw, so it has no answer.
        Condition::CostAnswer(_) => {
            debug_assert!(false, "{:?} is answered by the resolver's walk, and nothing else has an answer", condition);
            false
        }

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

/// Does `condition` hold now, outside the layer walk?
///
/// [`holds`] is the one evaluator, and it is written for the walk, which asks
/// at each layer against a board partway built. Every other asker wants the
/// finished board, and this hides that: `Board::settled()` at the last layer,
/// where every leaf answers as the walk would have at its end. Its askers: CR
/// 613.11's cost effects, the static replacement and restriction gathers, and
/// a triggered ability's intervening "if" at the trigger and at resolution
/// (CR 603.4) and a resolving effect's own "if".
///
/// `locked_you` is CR 109.5's "you" when the asker has already fixed it: a
/// triggered ability's controller as it triggered (CR 603.3a), or a
/// resolution's controller. It stays that player whatever has become of the
/// source since — stolen, or in its owner's graveyard. `None` reads it live,
/// as a static ability's is: the source's current controller. `referents` is
/// [`ConditionReferents::NONE`] for a static ability's condition.
pub fn settled_holds(
    condition: &Condition,
    game: &GameState,
    source: ObjectId,
    locked_you: Option<PlayerId>,
    referents: ConditionReferents<'_>,
) -> bool {
    holds(condition, game, &Board::settled(), source, LAYER_ORDER.len(), locked_you, referents)
}

/// Whether a "this turn", "last turn", "since your last turn" or "this game"
/// condition holds: `history` totalled over `span` and compared with `count`,
/// as "if you've gained life this turn" is life gained this turn at least 1.
/// A condition whose source has no controller or owner left to be "you" is
/// false.
fn history_holds(
    history: &HistoryCount,
    count: CountIs,
    span: HistorySpan,
    game: &GameState,
    board: &Board<'_>,
    layer_index: usize,
    players: &mut FilterPlayers<'_, '_>,
) -> bool {
    players.has_you() && count.satisfied_by(history_total(history, span, game, board, layer_index, players))
}

/// `history` totalled over `span`: the rows of the players it names, read
/// against `players`' "you" and narrowed by its filter over each row's frame in
/// their context, a condition's or an amount's.
pub(super) fn history_total(
    history: &HistoryCount,
    span: HistorySpan,
    game: &GameState,
    board: &Board<'_>,
    layer_index: usize,
    players: &mut FilterPlayers<'_, '_>,
) -> u64 {
    let you = players.you();
    // A row read live is read off the board at the walk's own ceiling, never
    // through the top-level entry (`Board::frame_of`).
    let mut filter_matches = |filter: &ObjectFilter, object: ObjectInformation<'_>| match object {
        ObjectInformation::Characteristics(_, frame, _) => object_filter::evaluate(game, filter, object, &|| Some(frame), players),
        ObjectInformation::LastKnown(lki) => object_filter::evaluate(game, filter, object, &|| Some(lki.characteristics()), players),
        ObjectInformation::Current(id) => {
            board.frame_of(game, id, layer_index).is_some_and(|frame| object_matches_filter(filter, id, &frame, players))
        }
    };
    let note = |input| board.note_gated_read(input);
    game.history.total(history, span, you, game.turn_number, &mut filter_matches, &note)
}

/// CR 109.5's "you": `locked_you` when the asker fixed it, or else the
/// source's controller off its live frame, or its owner if it has no
/// controller. `None` only when none is there — a static ability's source that
/// no longer exists anywhere — where no "you" can be read and a leaf answers
/// false.
pub(super) fn you_for(
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    locked_you: Option<PlayerId>,
) -> Option<PlayerId> {
    locked_you.or_else(|| {
        board
            .frame_of(game, source, layer_index)
            .map(|frame| frame.controller)
            .or_else(|| game.objects.get(&source).map(|obj| obj.owner))
    })
}

/// [`Condition::Player`]: does any player `players` names, among those still
/// in the game, meet `fact`? The set is resolved against CR 109.5's "you"
/// ([`you_for`]). A departed player's permanents left with them (CR 800.4a),
/// so no fact about the board can name them.
fn player_fact_holds(
    players: &PlayerSet,
    fact: &PlayerFact,
    game: &GameState,
    board: &Board<'_>,
    source: ObjectId,
    layer_index: usize,
    locked_you: Option<PlayerId>,
    owner_if_gone: Option<PlayerId>,
) -> bool {
    let Some(you) = you_for(game, board, source, layer_index, locked_you) else {
        return false;
    };
    let filter_players = || FilterPlayers::for_source(source, game, board, layer_index, locked_you, owner_if_gone);
    let named = |player: PlayerId| game.in_game(player) && players.contains(you, player);
    let any_named = |meets: &dyn Fn(&PlayerState) -> bool| {
        game.players.iter().enumerate().any(|(player, state)| named(player) && meets(state))
    };
    match fact {
        PlayerFact::ControlsPermanent(filter) => controls_matching(filter, game, board, layer_index, filter_players(), &named),
        // The threshold is resolved against the source's own frame, which is
        // where a static ability's dynamic number is read everywhere else.
        PlayerFact::LifeAtLeast(expr) | PlayerFact::LifeAtMost(expr) => {
            board.note_gated_read(GatedWalkInput::LifeTotals);
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
        PlayerFact::HandEmpty => any_named(&|state| state.hand.is_empty()),
        // A graveyard card may be a non-member of the pass, in which case its
        // frame is its own CDA walk at this ceiling (CR 604.3).
        PlayerFact::CardInGraveyard(filter) => {
            let mut players = filter_players();
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
    layer_index: usize,
    mut players: FilterPlayers<'_, '_>,
    named: &dyn Fn(PlayerId) -> bool,
) -> bool {
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
// `ObjectMatches` of the host, both end to end in `tests/phase_li3_integration_test.rs`.
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

    /// A status leaf reads the entity, not a frame: tapping the source flips
    /// it with no zone change and no registry write.
    #[test]
    fn source_untapped_reads_the_entitys_status() {
        let mut game = setup_two_player_game();
        let sphere = put_on_battlefield(&mut game, card_of_type("Sphere", CardType::Artifact), 0);
        assert!(settled_holds(&Condition::SourceUntapped, &game, sphere, None, ConditionReferents::NONE));
        game.battlefield.get_mut(&sphere).unwrap().tapped = true;
        assert!(!settled_holds(&Condition::SourceUntapped, &game, sphere, None, ConditionReferents::NONE));
        let dead = put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
        assert!(!settled_holds(&Condition::SourceUntapped, &game, dead, None, ConditionReferents::NONE), "no entity, not untapped");
    }

    fn yours(fact: PlayerFact) -> Condition {
        Condition::Player { players: PlayerSet::You, fact }
    }

    fn an_opponents(fact: PlayerFact) -> Condition {
        Condition::Player { players: PlayerSet::Opponents, fact }
    }

    /// Emperor Crocodile's "you control no other creatures": the positive fact
    /// negated, read for "you" alone, so an opponent's creature leaves it true.
    #[test]
    fn a_negated_fact_holds_exactly_when_the_fact_does_not() {
        use crate::types::effects::ObjectFilter;
        let mut game = setup_two_player_game();
        let crocodile = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let other_creature = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::NotSource));
        let none_other = Condition::Not(Box::new(yours(PlayerFact::ControlsPermanent(other_creature))));
        put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
        assert!(settled_holds(&none_other, &game, crocodile, None, ConditionReferents::NONE), "only the opponent has another creature");
        put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        assert!(!settled_holds(&none_other, &game, crocodile, None, ConditionReferents::NONE));
    }

    /// CR 603.8's example board: "you have no cards in hand", asked of "you".
    #[test]
    fn an_empty_hand_is_the_named_players_hand() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        game.players[0].hand.clear();
        game.players[1].hand.clear();
        assert!(settled_holds(&yours(PlayerFact::HandEmpty), &game, source, None, ConditionReferents::NONE));
        let card = game.add_object(crate::objects::object::GameObject::new(creatures::grizzly_bears(), 1, crate::types::zones::Zone::Hand));
        game.players[1].hand.push(card);
        assert!(settled_holds(&yours(PlayerFact::HandEmpty), &game, source, None, ConditionReferents::NONE), "the opponent's card is not yours");
        assert!(!settled_holds(&an_opponents(PlayerFact::HandEmpty), &game, source, None, ConditionReferents::NONE));
    }

    #[test]
    fn life_thresholds_read_the_source_controllers_total() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        game.players[0].life_total = 20;
        game.players[1].life_total = 3;

        assert!(settled_holds(&yours(PlayerFact::LifeAtLeast(AmountExpr::Fixed(20))), &game, bears, None, ConditionReferents::NONE));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtLeast(AmountExpr::Fixed(21))), &game, bears, None, ConditionReferents::NONE));
        assert!(settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(20))), &game, bears, None, ConditionReferents::NONE));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(19))), &game, bears, None, ConditionReferents::NONE));

        // CR 109.5 — the *source's* controller, not either player at large.
        let theirs = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
        assert!(settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(3))), &game, theirs, None, ConditionReferents::NONE));
        assert!(!settled_holds(&yours(PlayerFact::LifeAtMost(AmountExpr::Fixed(3))), &game, bears, None, ConditionReferents::NONE));
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
        assert!(!settled_holds(&ten_or_less, &game, ghast, None, ConditionReferents::NONE), "three at 12, and no sum");

        game.players[2].life_total = 0;
        game.player_lost[2] = true;
        assert!(!settled_holds(&ten_or_less, &game, ghast, None, ConditionReferents::NONE), "the one at 0 has left the game");

        game.players[3].life_total = 10;
        assert!(settled_holds(&ten_or_less, &game, ghast, None, ConditionReferents::NONE), "one opponent at 10 is enough");
    }

    #[test]
    fn card_in_graveyard_reads_your_graveyard_through_the_card_filter() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let card_in_yours = |filter: ObjectFilter| yours(PlayerFact::CardInGraveyard(filter));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::All), &game, bears, None, ConditionReferents::NONE));

        put_in_graveyard(&mut game, basic_lands::forest(), 0);
        assert!(settled_holds(&card_in_yours(ObjectFilter::All), &game, bears, None, ConditionReferents::NONE));
        assert!(settled_holds(&card_in_yours(ObjectFilter::ByType(CardType::Land)), &game, bears, None, ConditionReferents::NONE));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::ByType(CardType::Creature)), &game, bears, None, ConditionReferents::NONE));
        assert!(!settled_holds(&card_in_yours(ObjectFilter::ByColor(Color::Red)), &game, bears, None, ConditionReferents::NONE));

        // Your graveyard, not everybody's: the same card under the opponent
        // answers for their graveyard, which is empty.
        let theirs = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
        assert!(!settled_holds(&card_in_yours(ObjectFilter::All), &game, theirs, None, ConditionReferents::NONE));
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

        assert!(!settled_holds(&you_control, &game, bears, None, ConditionReferents::NONE));
        assert!(!settled_holds(&an_opponent_controls, &game, bears, None, ConditionReferents::NONE));

        put_on_battlefield(&mut game, basic_lands::forest(), 1);
        assert!(!settled_holds(&you_control, &game, bears, None, ConditionReferents::NONE));
        assert!(settled_holds(&an_opponent_controls, &game, bears, None, ConditionReferents::NONE));

        put_on_battlefield(&mut game, basic_lands::forest(), 0);
        assert!(settled_holds(&you_control, &game, bears, None, ConditionReferents::NONE));
        assert!(settled_holds(&an_opponent_controls, &game, bears, None, ConditionReferents::NONE));
    }

    /// Every clause, and an empty conjunction is vacuously true.
    #[test]
    fn all_holds_only_when_every_clause_does() {
        let mut game = setup_two_player_game();
        let dead = put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
        let island = ObjectFilter::BySubtype(crate::types::card_types::Subtype::Land(
            crate::types::card_types::LandType::Island,
        ));
        let island_at_twenty = Condition::All(vec![
            yours(PlayerFact::LifeAtLeast(AmountExpr::Fixed(20))),
            yours(PlayerFact::ControlsPermanent(island)),
        ]);
        assert!(!settled_holds(&island_at_twenty, &game, dead, None, ConditionReferents::NONE), "at 20, no Island");
        put_on_battlefield(&mut game, basic_lands::island(), 0);
        assert!(settled_holds(&island_at_twenty, &game, dead, None, ConditionReferents::NONE), "at 20, an Island");

        assert!(settled_holds(&Condition::All(Vec::new()), &game, dead, None, ConditionReferents::NONE));
    }

    /// An unattached source is attached to nothing, so nothing matches — the
    /// same answer `ObjectSet::Host` gives, and the reason an Aura's
    /// conditional effect simply does not exist before it is attached.
    #[test]
    fn an_unattached_source_matches_no_host() {
        let mut game = setup_two_player_game();
        let aura = put_on_battlefield(&mut game, card_of_type("Loose Aura", CardType::Enchantment), 0);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        assert!(!settled_holds(&Condition::ObjectMatches(Antecedent::ThisObjectsHost, ObjectFilter::All), &game, aura, None, ConditionReferents::NONE));

        assert!(game.attach(aura, bears));
        assert!(settled_holds(&Condition::ObjectMatches(Antecedent::ThisObjectsHost, ObjectFilter::All), &game, aura, None, ConditionReferents::NONE));
        assert!(settled_holds(
            &Condition::ObjectMatches(Antecedent::ThisObjectsHost, ObjectFilter::ByType(CardType::Creature)),
            &game,
            aura, None, ConditionReferents::NONE
        ));
        assert!(!settled_holds(
            &Condition::ObjectMatches(Antecedent::ThisObjectsHost, ObjectFilter::ByType(CardType::Artifact)),
            &game,
            aura, None, ConditionReferents::NONE
        ));
    }

    /// A condition on an object the store has never heard of has no answer,
    /// and says so rather than reading somebody else's frame.
    #[test]
    fn a_condition_on_a_missing_source_is_false() {
        let game = setup_two_player_game();
        assert!(!settled_holds(&Condition::SourceUntapped, &game, ObjectId::UNASSIGNED, None, ConditionReferents::NONE));
    }

    /// "If you put a permanent with a kicker ability onto the battlefield
    /// without casting it, you can't kick it" (Archangel of Wrath's ruling):
    /// it was never a spell, so there were no costs to decide.
    #[test]
    fn a_permanent_that_was_never_a_spell_was_not_kicked() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        assert!(!settled_holds(&Condition::SpellWasKicked, &game, bears, None, ConditionReferents::NONE));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "no static-context evaluator")]
    fn mode_chosen_on_a_static_is_loud() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let _ = settled_holds(&Condition::ModeChosen(0), &game, bears, None, ConditionReferents::NONE);
    }
}
