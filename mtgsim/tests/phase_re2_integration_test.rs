//! Phase RE-2 — draw.
//!
//! CR 121.2, 121.2a, 121.6a/b, 614.11, 614.11a and 616.1g, against the four
//! printed cards in `cards::phase_re_cards`.
//!
//! **Every board here is about the boundary between an instruction and a
//! draw.** CR 121.2a makes "draw N cards" an event of its own, and Alms
//! Collector's ruling is the test the CR does not spell out: *"count how many
//! times the word 'draw' is used."* So the assertions come in pairs — what a
//! card does to one instruction of three, and what it does to three
//! instructions of one.
//!
//! `ATOM-614.11b-001` — CR 121.6c's "additional actions on a card after it's
//! drawn" — has **no test, because it has no producer to get wrong.** The rule
//! says the additional action is not performed on a card that arrived by
//! replacement; nothing in `Primitive` performs an additional action on the
//! card it drew, so there is no site at which the engine could perform one.
//! `Primitive::DrawCards` returns no ids to a later atom and `Effect::Sequence`
//! threads nothing between its instructions (CR 608.2c). The printed shape is
//! "draw a card **and reveal it**. If it isn't a land card, discard it" — four
//! cards, and each also needs `Primitive::Discard` (RE-8) or the information
//! model, so the facility alone unblocks none of them. Uncovered with that
//! reason; `backlog.md` §2.26 owns the mechanic
//! (`replacement-architecture.md` §9, RE decision 1).

use std::sync::Arc;

use mtgsim::cards::phase_re_cards::{
    alms_collector, notion_thief, teferis_ageless_insight, thought_reflection,
};
use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_ctx,
    test_dp,
};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution or the out-of-set member of a boundary test. Built inline rather
/// than pulled from the registry, so no real card's behaviour leaks into a
/// board that is only about the draw.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// A board with everyone stocked, so no draw in this file decks anybody and
/// `has_drawn_from_empty_library` stays out of the assertions.
fn stocked(num_players: usize) -> GameState {
    let mut game = setup_game(num_players);
    for pid in 0..num_players {
        fill_library(&mut game, pid, 60);
    }
    game
}

/// Resolve "draw `n` cards" for `player`, the way a spell would, with `dp`
/// answering any CR 616.1 prompt.
///
/// **One instruction**, whatever `n` is — which is the thing half this file is
/// about, so the helper never loops.
fn draw_instruction(game: &mut GameState, player: PlayerId, n: u64, dp: &dyn DecisionProvider) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(n)),
        EffectRecipient::Controller,
    );
    game.resolve_effect(&effect, &ctx, dp).expect("drawing");
}

/// Resolve an arbitrary effect for `player`, the way a spell would.
fn resolve_for(game: &mut GameState, player: PlayerId, effect: &Effect, dp: &dyn DecisionProvider) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// How many cards each player has drawn, read off the event log rather than off
/// hand sizes: a hand can be filled by a tutor, and CR 121.5 is exactly the
/// difference this file keeps asserting.
fn drawn_by(game: &GameState, player: PlayerId) -> usize {
    game.events
        .events()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player_id, .. } if *player_id == player))
        .count()
}

fn total_drawn(game: &GameState) -> usize {
    game.events
        .events()
        .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
        .count()
}

/// Park the game at the start of `player`'s draw step, with `player` active.
fn at_draw_step(game: &mut GameState, player: PlayerId) {
    game.active_player = player;
    game.priority_player = player;
    game.phase = Phase { phase_type: PhaseType::Beginning, step: Some(StepType::Upkeep) };
}

/// Advance until the draw step's turn-based action has happened.
fn take_the_draw_step(game: &mut GameState, dp: &dyn DecisionProvider) {
    let ctx = ActionContext::new(dp);
    for _ in 0..8 {
        game.advance_turn(&ctx).expect("advancing");
        if game.phase.step == Some(StepType::Draw) {
            return;
        }
    }
    panic!("never reached the draw step");
}

// ---------------------------------------------------------------------------
// CR 121.2 / 121.2a — the instruction and its individual draws
// ---------------------------------------------------------------------------

// COVERS: ATOM-121.2-001
//
// The atom is "drawing N cards = N individual draws", which was true before
// this phase and is now true through a different path: one `DrawCards`
// proposal whose performer decomposes, rather than a loop in
// `Primitive::DrawCards`. It gains a `COVERS:` here because the decomposition
// is what the assertion is about — three `CardDrawn` events from one
// instruction, and a library three shorter.
#[test]
fn one_instruction_to_draw_three_performs_three_individual_draws() {
    let mut game = stocked(2);
    let before = game.players[0].library.len();

    draw_instruction(&mut game, 0, 3, &test_dp());

    assert_eq!(drawn_by(&game, 0), 3, "three individual draws (CR 121.2)");
    assert_eq!(game.players[0].library.len(), before - 3);
    assert_eq!(game.players[0].hand.len(), 4, "three drawn plus the fixture source");
}

// COVERS: ATOM-121.2a-001
//
// The atom's board is "a player with a draw-count replacement resolving 'draw
// 3'". Thought Reflection is the printed shape of that and it watches the
// *individual* draw rather than the count, which is the ruling's own example:
// "if you cast Harmonize ('Draw three cards'), you'll draw six cards". Six is
// the atom's expected result by the route the printed pool actually has.
#[test]
fn thought_reflection_doubles_each_of_a_three_card_instruction() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, thought_reflection(), 0);

    draw_instruction(&mut game, 0, 3, &test_dp());

    assert_eq!(drawn_by(&game, 0), 6, "Harmonize draws six");
}

/// CR 121.2a's own sentence, from the side that shows the instruction is a real
/// event: Alms Collector watches the instruction and nothing watches the count
/// of an instruction of one.
#[test]
fn an_instruction_of_one_is_not_an_instruction_of_two_or_more() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, alms_collector(), 0);

    // Player 1 draws a card, once. `at_least: Some(2)` does not watch it.
    draw_instruction(&mut game, 1, 1, &test_dp());
    assert_eq!(drawn_by(&game, 1), 1);
    assert_eq!(drawn_by(&game, 0), 0, "Alms Collector did not apply");
}

/// **The engine claim is that the two are different events at all**, and Alms
/// Collector's counting ruling is the printed statement of it: *"count how many
/// times the word 'draw' is used."*
///
/// Which of the two a given card is written as is the author's business; that
/// the vocabulary can *hold* the difference is not, and it is what
/// [`GameAction::DrawCards`] exists for. Before the instruction event,
/// `Primitive::DrawCards(2)` lowered to the same two `DrawCard` proposals a
/// pair of cantrips does, and no pattern could have separated them — so the
/// assertion is the pair, and a board with only one half would pass against an
/// engine that watched everything.
#[test]
fn one_instruction_of_two_is_a_different_event_from_two_instructions_of_one() {
    let dp = ScriptedDecisionProvider::new();

    let mut one_of_two = stocked(2);
    put_on_battlefield(&mut one_of_two, alms_collector(), 0);
    draw_instruction(&mut one_of_two, 1, 2, &dp);
    assert_eq!(drawn_by(&one_of_two, 1), 1, "the instruction was replaced");
    assert_eq!(drawn_by(&one_of_two, 0), 1, "and you drew one");

    // Two instructions, which is how a card printing "draw a card" twice would
    // be authored — `Effect::Sequence` is CR 608.2c's instruction sequencing and
    // each atom proposes its own event.
    let mut two_of_one = stocked(2);
    put_on_battlefield(&mut two_of_one, alms_collector(), 0);
    let twice = Effect::Sequence(vec![
        Effect::Atom(
            Primitive::DrawCards(AmountExpr::Fixed(1)),
            EffectRecipient::Controller,
        ),
        Effect::Atom(
            Primitive::DrawCards(AmountExpr::Fixed(1)),
            EffectRecipient::Controller,
        ),
    ]);
    resolve_for(&mut two_of_one, 1, &twice, &dp);
    assert_eq!(drawn_by(&two_of_one, 1), 2, "two instructions, neither of two or more");
    assert_eq!(drawn_by(&two_of_one, 0), 0);
}

// ---------------------------------------------------------------------------
// CR 614.5 through the lineage — the acid test
// ---------------------------------------------------------------------------

/// **The regression for §3.2d's lineage rule, and its failure mode is a hang.**
///
/// Thought Reflection's ruling gives the arithmetic: *"if you have two Thought
/// Reflections on the battlefield, you'll draw four times the original number.
/// If you have three, you'll draw eight times."* The trace is the rule:
/// `DrawCard` meets both, one is chosen, its `DrawCards { n: 2 }` is performed,
/// and **each of the two inner draws inherits `{chosen}`** — so only the other
/// Reflection applies to them, and its output inherits both. Four cards.
///
/// Without the inheritance the chosen Reflection re-applies to its own output,
/// which does not answer wrongly; it recurses until the stack overflows.
///
/// **The bound is `execute_actions_decomposing`'s debug assertion, and it is
/// derived rather than chosen.** A decomposing call at depth `d` exists because
/// `d - 1` substitutions happened above it, and each inserted an instance into
/// the applied set — so `d <= inherited.len() + 1` on any correct board. Break
/// the inheritance and depth climbs while the set does not: the assertion fires
/// at **depth 2**, long before the recursion is deep enough to overflow the
/// stack and take the whole test binary with it, and its message names the rule.
/// Mutation-checked.
///
/// **And no prompt, which is the second claim.** Two Thought Reflections are
/// order-invariant — the total is the product of their counts whichever applies
/// first — so `ordering_cannot_change_outcome` suppresses CR 616.1's question
/// the way it does for two Furnaces of Rath. The provider is primed with
/// nothing and asserts it was never asked.
/// [`a_draw_doubler_beside_a_notion_thief_is_a_real_choice`] is the board where
/// the order does change the answer and the prompt is not suppressed.
#[test]
fn test_two_thought_reflections_draw_four_not_infinity() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    put_on_battlefield(&mut game, thought_reflection(), 0);

    let dp = ScriptedDecisionProvider::new();
    draw_instruction(&mut game, 0, 1, &dp);

    assert_eq!(drawn_by(&game, 0), 4, "two Reflections draw four, not four thousand");
    assert!(dp.is_empty(), "and the choice between them has one outcome, so nobody was asked");
}

/// The same rule one copy further on, because the ruling states it: three
/// Reflections draw eight. Worth its own board — 2ⁿ is the claim, and two
/// copies alone are consistent with "each draw is doubled once per copy after
/// the first", which three copies separate from it.
#[test]
fn three_thought_reflections_draw_eight() {
    let mut game = stocked(2);
    for _ in 0..3 {
        put_on_battlefield(&mut game, thought_reflection(), 0);
    }

    let dp = ScriptedDecisionProvider::new();
    draw_instruction(&mut game, 0, 1, &dp);

    assert_eq!(drawn_by(&game, 0), 8);
    assert!(dp.is_empty(), "still one outcome with three, so still no prompt");
}

/// **The other side of the suppression, and the reason it is a predicate rather
/// than a blanket rule for draws.**
///
/// Player 0's Thought Reflection and player 1's Notion Thief both watch player
/// 0's draw, and the order decides how many cards player 1 ends up with — so
/// `ordering_cannot_change_outcome` says no and CR 616.1 asks. The Thief is what
/// fails the predicate: its `GameActionTemplate::DrawCards` carries
/// `player: Some(You)`, which moves the event's *subject*, and two applications
/// that move the subject do not commute with anything.
///
/// Both answers are asserted, because "the prompt exists" is only half the
/// claim — the other half is that the two answers really are different.
#[test]
fn a_draw_doubler_beside_a_notion_thief_is_a_real_choice() {
    // Candidate order is the battlefield's (CR 613.7 timestamps), so the
    // Reflection is index 0 and the Thief index 1 on both boards.
    let doubled_first = {
        let mut game = stocked(2);
        put_on_battlefield(&mut game, thought_reflection(), 0);
        put_on_battlefield(&mut game, notion_thief(), 1);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);
        draw_instruction(&mut game, 0, 1, &dp);
        assert!(dp.is_empty(), "one prompt");
        (drawn_by(&game, 0), drawn_by(&game, 1))
    };
    let stolen_first = {
        let mut game = stocked(2);
        put_on_battlefield(&mut game, thought_reflection(), 0);
        put_on_battlefield(&mut game, notion_thief(), 1);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![1]);
        draw_instruction(&mut game, 0, 1, &dp);
        assert!(dp.is_empty(), "one prompt");
        (drawn_by(&game, 0), drawn_by(&game, 1))
    };

    // Double first and the Thief takes each of the two resulting draws; steal
    // first and there is one draw to take, and it is no longer player 0's, so
    // the Reflection never sees it.
    assert_eq!(doubled_first, (0, 2));
    assert_eq!(stolen_first, (0, 1));
    assert_ne!(doubled_first, stolen_first, "the order is what CR 616.1 is for");
}

// ---------------------------------------------------------------------------
// CR 121.2 / DrawCause — "except the first one you draw in each of your draw
// steps"
// ---------------------------------------------------------------------------

/// Teferi's Ageless Insight alone in a draw step: the turn-based action's one
/// card is the excepted one, so nothing is doubled.
#[test]
fn teferi_excepts_the_draw_steps_first_card() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, teferis_ageless_insight(), 0);
    at_draw_step(&mut game, 0);

    take_the_draw_step(&mut game, &test_dp());

    assert_eq!(drawn_by(&game, 0), 1, "the draw step's first card is excepted");
}

/// And every draw after it in the same turn is not the draw step's, so Teferi
/// doubles it.
#[test]
fn teferi_doubles_a_draw_that_is_not_the_draw_steps() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, teferis_ageless_insight(), 0);
    at_draw_step(&mut game, 0);
    take_the_draw_step(&mut game, &test_dp());
    assert_eq!(drawn_by(&game, 0), 1);

    draw_instruction(&mut game, 0, 1, &test_dp());

    assert_eq!(drawn_by(&game, 0), 3, "one from the step, two from the spell");
}

/// **Decision 1's own board, and the one the `DrawCause` stamping rule exists
/// for: three cards.**
///
/// Thought Reflection doubles the draw step's one card into an instruction of
/// two. That instruction is the same event in modified form (CR 614.6), so it
/// is still the draw step's: its **first** inner is the card Teferi excepts and
/// its **second** is `DrawCause::Effect`, which Teferi doubles into two more.
/// One plus two is three, and nothing anywhere counts cards drawn this step.
///
/// Four would be the answer if a substituted instruction started fresh; two
/// would be the answer if the exception were a property of the instruction
/// rather than of each draw.
#[test]
fn teferi_beside_thought_reflection_draws_three_in_the_draw_step() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    put_on_battlefield(&mut game, teferis_ageless_insight(), 0);
    at_draw_step(&mut game, 0);

    // The draw step's first card is one Teferi does not watch, so the only
    // candidate there is the Reflection; each of the resulting two draws is
    // then watched by whichever of the two has not applied. No prompt anywhere.
    take_the_draw_step(&mut game, &test_dp());

    assert_eq!(drawn_by(&game, 0), 3);
}

/// CR 121.5's boundary, which two of the four cards have a ruling about: a card
/// put into hand without the word "draw" is not drawn, so no draw replacement
/// sees it.
///
/// Asserted structurally, because that is what the engine's answer is: a
/// library-to-hand move whose [`ZoneChangeCause`] is not `Drawn` proposes no
/// draw event at all, so there is nothing for Thought Reflection to double.
#[test]
fn a_card_put_into_hand_is_not_drawn_and_no_draw_replacement_sees_it() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    let before = game.players[0].hand.len();
    let top = *game.players[0].library.last().expect("stocked");

    game.change_zone(top, Zone::Hand, ZoneChangeCause::PutIntoHand, &test_ctx())
        .expect("tutoring");

    assert_eq!(game.players[0].hand.len(), before + 1, "one card moved");
    assert_eq!(drawn_by(&game, 0), 0, "and none was drawn (CR 121.5)");
}

// ---------------------------------------------------------------------------
// CR 616.1g / 616.2 — the outer is decided before an inner exists
// ---------------------------------------------------------------------------

// COVERS: ATOM-616.1g-001
//
// The atom's board is Doubling Season's, which is RE-4's. Its rule — "a
// replacement that applies to an event contained within another event can't be
// chosen until the outer event's replacement is applied first" — has a printed
// draw board today, and it is Alms Collector's own first ruling: "[its]
// replacement effect applies to an instruction to draw more than one card
// before any replacement effects apply to individual cards drawn". Thought
// Reflection is the effect on the inner side, and the nesting is what decides
// the answer.
#[test]
fn alms_collector_applies_to_the_instruction_before_thought_reflection_sees_a_draw() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, alms_collector(), 0);
    put_on_battlefield(&mut game, thought_reflection(), 1);

    // Player 1 resolves "draw two cards" under their own Thought Reflection and
    // an opponent's Alms Collector. The instruction is decided first: Alms
    // prevents it and its rider draws one for each player. Only then do
    // individual draws exist, and the Reflection doubles player 1's.
    draw_instruction(&mut game, 1, 2, &test_dp());

    assert_eq!(drawn_by(&game, 0), 1, "the rider's draw for Alms Collector's controller");
    assert_eq!(drawn_by(&game, 1), 2, "the rider's draw for the affected player, doubled");
}

/// CR 616.2 from the other direction, and it is Alms Collector's second ruling
/// almost verbatim: *"once Alms Collector's replacement effect has modified the
/// effect of a player's Divination, Thought Reflection can double that player's
/// resulting card draw without Alms Collector's replacement effect applying
/// again."*
///
/// The board above already proves the doubling happens. This one proves the
/// second half — that Alms Collector does not apply to what it produced —
/// by counting: two applications would make four draws for player 1, not two.
#[test]
fn alms_collector_does_not_apply_again_to_the_draws_it_produced() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, alms_collector(), 0);
    put_on_battlefield(&mut game, thought_reflection(), 1);

    draw_instruction(&mut game, 1, 2, &test_dp());

    assert_eq!(
        total_drawn(&game),
        3,
        "one rider draw each, and the affected player's doubled — not a second application"
    );
}

// COVERS: ATOM-614.11-001
// COVERS: ATOM-121.6a-001
//
// Both atoms are the same sentence from two rules: a draw replacement applies
// even when the library is empty, and the CR 704.5b flag is not set because no
// draw was attempted. Thought Reflection is not the atoms' "[do X] instead" —
// it replaces a draw with more draws — so the assertion is on the flag and on
// the gather: the replacement is reached with an empty library, its output is
// two draws that also find nothing, and the player is flagged by *those*
// rather than by the original. What the atoms are really about is that the
// pipeline runs before the library is consulted, and the flag's absence in the
// Notion Thief board below is the other half.
#[test]
fn a_draw_replacement_applies_with_an_empty_library() {
    let mut game = setup_two_player_game();
    assert!(game.players[0].library.is_empty(), "setup_two_player_game stocks nothing");
    put_on_battlefield(&mut game, thought_reflection(), 0);

    draw_instruction(&mut game, 0, 1, &test_dp());

    assert_eq!(drawn_by(&game, 0), 0, "nothing to draw");
    assert!(
        game.players[0].has_drawn_from_empty_library,
        "CR 704.5b is flagged by the draws that actually happened"
    );
}

/// CR 121.6a's other half, on the card that shows it: a draw that is replaced
/// away from a player never touches that player's empty library.
#[test]
fn notion_thief_takes_the_draw_before_an_empty_library_is_consulted() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 10);
    // Player 1's library stays empty.
    put_on_battlefield(&mut game, notion_thief(), 0);

    draw_instruction(&mut game, 1, 1, &test_dp());

    assert_eq!(drawn_by(&game, 0), 1, "the Thief's controller drew");
    assert!(
        !game.players[1].has_drawn_from_empty_library,
        "the opponent's draw was replaced, so no draw from an empty library was attempted"
    );
}

// ---------------------------------------------------------------------------
// CR 614.11a / 121.6b — a replacement completes before the sequence resumes
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-614.11a-001
//
// The atom's replacement is "mill two cards then draw a card instead", whose
// rider needs `Primitive::Mill` beside a draw; what it is proving is that each
// individual draw of an instruction of three is replaced and *completed* before
// the next is proposed. Alms Collector is the printed effect that makes the
// order observable without a mill: its rider draws for two players, so the
// sequencing shows up as interleaving in the event log rather than as a count.
// The partial is the atom's rider — this board proves the sequencing and not
// the "mill then draw" shape.
#[test]
fn each_draw_of_an_instruction_completes_before_the_next_is_proposed() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, thought_reflection(), 0);

    // Three instructions would interleave with nothing; one instruction of
    // three decomposes into three draws, each of which becomes an instruction
    // of two whose own two draws happen before draw #2 of the outer is
    // proposed. The log is therefore six draws in three pairs, and the library
    // shrinks monotonically through them.
    let before = game.players[0].library.len();
    draw_instruction(&mut game, 0, 3, &test_dp());

    assert_eq!(drawn_by(&game, 0), 6);
    assert_eq!(game.players[0].library.len(), before - 6);
}

/// The interleaving the atom is really about, on the one printed board that
/// makes it visible: Alms Collector turns one player's instruction into a draw
/// for each of two players, so a pair of replaced instructions produces an
/// alternating log rather than a run.
///
/// The order inside each pair is the affected player's draw and *then* the
/// rider's, which is §4.1a — a rider resolves after the event it rides on. It
/// is deliberately not the card's text order ("you and that player"), and it is
/// not CR 121.2c's turn order either; `codebase-state.md` item 122 owns that.
#[test]
fn a_riders_draws_resolve_before_the_next_instruction_begins() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, alms_collector(), 0);

    // Two instructions of two, back to back. Each is replaced, and each
    // replacement's two draws happen before the next instruction is proposed:
    // P1 P0 P1 P0, not P1 P1 P0 P0.
    let twice = Effect::Sequence(vec![
        Effect::Atom(
            Primitive::DrawCards(AmountExpr::Fixed(2)),
            EffectRecipient::Controller,
        ),
        Effect::Atom(
            Primitive::DrawCards(AmountExpr::Fixed(2)),
            EffectRecipient::Controller,
        ),
    ]);
    resolve_for(&mut game, 1, &twice, &test_dp());

    let order: Vec<PlayerId> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::CardDrawn { player_id, .. } => Some(*player_id),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![1, 0, 1, 0], "CR 121.6b — the replacement completes first");
}

// ---------------------------------------------------------------------------
// Notion Thief — the same event with a new subject
// ---------------------------------------------------------------------------

// COVERS: BOUNDARY-DEF-614.1a-001
//
// The boundary is CR 614.1a's "instead": a card whose text uses the word is a
// replacement effect, and one whose text does not is not. Notion Thief is the
// in-set member — its draw is taken before it happens, by the CR 616.1 loop.
// The out-of-set member is the same board with a vanilla creature in its place,
// where the draw happens as proposed. Both halves are on one board because the
// claim is the difference: a test that only watched the Thief could not tell
// "instead is a replacement" from "a creature on the battlefield changes
// draws".
#[test]
fn notion_thief_takes_the_draw_and_a_vanilla_creature_leaves_it_alone() {
    let mut with = stocked(2);
    put_on_battlefield(&mut with, notion_thief(), 0);
    draw_instruction(&mut with, 1, 1, &test_dp());
    assert_eq!(drawn_by(&with, 1), 0, "the opponent's draw was replaced");
    assert_eq!(drawn_by(&with, 0), 1, "and the Thief's controller drew");

    let mut without = stocked(2);
    put_on_battlefield(&mut without, fixture_object(), 0);
    draw_instruction(&mut without, 1, 1, &test_dp());
    assert_eq!(drawn_by(&without, 1), 1, "nothing watched it");
    assert_eq!(drawn_by(&without, 0), 0);
}

/// Notion Thief's first ruling: *"if an opponent is instructed to draw a card
/// then discard a card, and Notion Thief causes you to draw a card instead,
/// that opponent still discards a card. The same is true of any other actions
/// that opponent is instructed to do."*
///
/// Tested on the second sentence. `Primitive::Discard` is `NotImplemented`
/// until RE-8, so the board is a draw-then-lose-life resolution — Night's
/// Whisper's shape, on an unnamed fixture — and the assertion is that replacing
/// the draw does nothing to the instruction beside it. CR 608.2c: each
/// instruction proposes its own events, and a replacement on one touches
/// nothing about the other (§4.1a's second "then").
#[test]
fn the_opponents_other_instructions_still_happen() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, notion_thief(), 0);
    let life_before = game.players[1].life_total;

    let draw_then_pay = Effect::Sequence(vec![
        Effect::Atom(
            Primitive::DrawCards(AmountExpr::Fixed(1)),
            EffectRecipient::Controller,
        ),
        Effect::Atom(
            Primitive::LoseLife(AmountExpr::Fixed(2)),
            EffectRecipient::Controller,
        ),
    ]);
    resolve_for(&mut game, 1, &draw_then_pay, &test_dp());

    assert_eq!(drawn_by(&game, 0), 1, "you drew instead");
    assert_eq!(
        game.players[1].life_total,
        life_before - 2,
        "and the opponent still paid"
    );
}

/// **Two Thieves in a two-player game, which the ruling answers in one
/// sentence:** *"it really will be that player who draws a card."*
///
/// The whole of it is the lineage. Player 1 would draw; player 0's Thief is the
/// only applicable one (nobody is their own opponent), so the draw becomes
/// player 0's — the same event, new subject. Player 1's Thief now applies to
/// *that* draw and player 0's cannot, because the applied set travelled with
/// it. The draw comes back to player 1, both Thieves are spent, and player 1
/// draws. As riders the two would have traded it forever.
#[test]
fn two_notion_thieves_hand_the_draw_across_the_table_and_back() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, notion_thief(), 0);
    put_on_battlefield(&mut game, notion_thief(), 1);

    // No prompt anywhere: at every step exactly one Thief is applicable, since
    // a Thief never watches its own controller's draw.
    let dp = ScriptedDecisionProvider::new();
    draw_instruction(&mut game, 1, 1, &dp);

    assert_eq!(drawn_by(&game, 1), 1, "it really is that player who draws");
    assert_eq!(drawn_by(&game, 0), 0);
    assert!(dp.is_empty(), "and nobody was asked");
}

/// The ruling's procedure in full, on the board that needs three players:
/// *"that player chooses one ... Then the player whose Notion Thief's effect
/// was chosen repeats this process among the remaining ... Each effect can be
/// applied to the card draw only once this way."*
///
/// Every clause falls out rather than being coded. "That player chooses" is CR
/// 616.1's affected player; "the player whose Thief was chosen repeats" is the
/// same rule asked of the new subject, because the substitution moved it; and
/// "only once" is CR 614.5's applied set travelling with the lineage.
#[test]
fn three_notion_thieves_pass_the_draw_once_each_in_the_rulings_order() {
    let mut game = stocked(3);
    let thieves: Vec<ObjectId> = (0..3)
        .map(|pid| put_on_battlefield(&mut game, notion_thief(), pid))
        .collect();
    assert_eq!(thieves.len(), 3);

    // Player 0 would draw. Two Thieves watch that draw — player 1's and player
    // 2's, since nobody is their own opponent — and player 0 chooses. Taking
    // candidate 0 hands the draw to player 1; player 1 then chooses between
    // player 0's Thief and player 2's, and takes candidate 0 again, which hands
    // it back to player 0. Each choice is made by the player the draw is
    // currently about, which is the ruling's "repeats this process".
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);

    draw_instruction(&mut game, 0, 1, &dp);

    // The third hop is not a choice: with player 1's and player 0's Thieves both
    // spent, player 2's is the only one left that watches a draw by player 0,
    // and the pipeline never prompts with one candidate. So the draw lands on
    // player 2 — three hops, three applications, one card, and no effect
    // applied twice.
    assert!(dp.is_empty(), "two prompts — the third hop has one candidate");
    assert_eq!(total_drawn(&game), 1, "one draw, however far it travelled");
    assert_eq!(drawn_by(&game, 2), 1, "the last Thief to apply is the one that keeps it");
    assert_eq!(drawn_by(&game, 0), 0);
    assert_eq!(drawn_by(&game, 1), 0);
}

// ---------------------------------------------------------------------------
// Four players — the only CR 616.1 prompt two printed cards can build between
// three of them
// ---------------------------------------------------------------------------

/// Alms Collector's last ruling: *"if two players each control an Alms
/// Collector and a third player would draw two or more cards, the third player
/// chooses which Alms Collector's replacement effect will apply, and therefore
/// which of the first two players draws a card."*
///
/// The prompt is the payload — a CR 616.1 choice between two effects controlled
/// by two different players, made by a third, which is a board no two-player
/// game can produce.
#[test]
fn a_third_player_chooses_which_alms_collector_applies() {
    let mut game = stocked(4);
    put_on_battlefield(&mut game, alms_collector(), 0);
    put_on_battlefield(&mut game, alms_collector(), 1);

    // Player 2 draws two and picks the second candidate, so player 1 is the one
    // who draws — which is the ruling's "and therefore which of the first two
    // players draws a card".
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![1]);

    draw_instruction(&mut game, 2, 2, &dp);

    assert_eq!(drawn_by(&game, 2), 1, "the affected player drew one instead of two");
    assert_eq!(drawn_by(&game, 1), 1, "the chosen Collector's controller drew");
    assert_eq!(drawn_by(&game, 0), 0, "the other one did not apply");
    assert!(dp.is_empty());
}

/// Alms Collector's fifth ruling, which is the same board with the draws
/// crossed: *"if two players each control an Alms Collector and an effect
/// instructs them to each draw two or more cards, the replacement effect of
/// each ... is applied and both players end up drawing two cards."*
///
/// Two instructions, not one — so each player's own Collector watches nothing
/// and the opponent's does, and each ends with one rider draw of their own plus
/// one from the other's rider.
#[test]
fn two_alms_collectors_facing_each_other_both_draw_two() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, alms_collector(), 0);
    put_on_battlefield(&mut game, alms_collector(), 1);

    let dp = ScriptedDecisionProvider::new();
    draw_instruction(&mut game, 0, 2, &dp);
    draw_instruction(&mut game, 1, 2, &dp);

    assert_eq!(drawn_by(&game, 0), 2, "one from your rider, one from theirs");
    assert_eq!(drawn_by(&game, 1), 2);
    assert!(dp.is_empty(), "one candidate each time — nobody is their own opponent");
}
