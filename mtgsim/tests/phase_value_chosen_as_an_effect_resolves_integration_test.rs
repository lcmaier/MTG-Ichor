//! `value-chosen-as-an-effect-resolves` (`choices-architecture.md`), the
//! first of `values-chosen-at-resolution`'s three: a number read off an
//! object and compared, a player verb whose player is gone, and a value or
//! objects chosen as an effect resolves, kept, and used.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use mtgsim::cards::authoring::{
    at_random, choose_number, choose_objects, choose_player, chosen, creature, creatures_you_control, destroy, draw_cards, exile, power_at_least,
    power_at_most, put_counters, return_to_the_battlefield, spell_ability, target_creature, target_player, the_chosen, the_target,
    toughness_at_least, toughness_at_most, you,
};
use mtgsim::cards::fixtures::{
    bestowed_favors, chosen_champion, consequences_at_random, damnations_tally, duneblast_fixture, kindred_dominance_fixture,
    the_nipton_lottery_fixture, wash_out_fixture, world_queller_fixture,
};
use mtgsim::cards::printings::grizzly_bears;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_colors, get_effective_controller, get_effective_power, get_effective_toughness, has_keyword};
use mtgsim::oracle::x_values::XOffer;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    card_of_type, fill_library, forest, pass_turn, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_ctx, test_dp,
    vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, ChoiceName, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, ObjectValue, PlayerRef, PlayerSet,
    ReturnUnder, SelectionFilter, TargetCount, ValueIs,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// The boards' tools
// ---------------------------------------------------------------------------

/// A one-mana sorcery of `color` whose text is `text` and whose effect is
/// `effect`: what each fixture below is.
fn sorcery(name: &str, color: (ManaType, Color), text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[color.0], 0))
        .color(color.1)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

/// `card` in player 0's hand, cast from a pool of exactly its one mana, with
/// `answers` answering the cast. The spell's id, and the answers back.
fn cast(game: &mut GameState, card: Arc<CardData>, mana: ManaType, answers: ScriptedDecisionProvider) -> (ObjectId, ScriptedDecisionProvider) {
    cast_paying(game, card, &[(mana, 1)], answers)
}

/// [`cast`] for a cost of more than one mana: `pool` is exactly the cost.
fn cast_paying(
    game: &mut GameState,
    card: Arc<CardData>,
    pool: &[(ManaType, u64)],
    answers: ScriptedDecisionProvider,
) -> (ObjectId, ScriptedDecisionProvider) {
    let id = put_in_hand(game, card, 0);
    for (mana, amount) in pool {
        game.players[0].mana_pool.add(*mana, *amount);
    }
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, id, &seat).expect("castable from exactly its cost");
    (id, seat.into_inner())
}

/// The script's next answer to a choice of objects or players: `chosen`.
fn picks(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

/// What each prompt offered: who was asked, the question, and its options,
/// a number's as its least and most.
#[derive(Default)]
struct Asked {
    of_options: RefCell<Vec<(PlayerId, ChoiceKind, Vec<ChoiceOption>)>>,
    of_numbers: RefCell<Vec<(PlayerId, ChoiceKind, u64, u64)>>,
}

/// A provider that notes each prompt in `asked` and answers through `inner`.
struct Watching<D> {
    inner: D,
    asked: Rc<Asked>,
}

impl<D> Watching<D> {
    fn new(inner: D) -> (Self, Rc<Asked>) {
        let asked = Rc::new(Asked::default());
        (Watching { inner, asked: Rc::clone(&asked) }, asked)
    }
}

impl<D: DecisionProvider> DecisionProvider for Watching<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.asked.of_options.borrow_mut().push((player, context.kind.clone(), options.to_vec()));
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.asked.of_numbers.borrow_mut().push((player, context.kind.clone(), min, max));
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.inner.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

/// A 2/2 creature of one color and one creature type.
fn creature_of(name: &str, color: Color, creature_type: CreatureType) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .color(color)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(creature_type))
        .power_toughness(2, 2)
        .build()
}

/// A game of `seats` players, each with cards to draw.
fn stocked(seats: usize) -> GameState {
    let mut game = setup_game(seats);
    for seat in 0..seats {
        fill_library(&mut game, seat, 20);
    }
    game
}

// ---------------------------------------------------------------------------
// A number read off an object, and compared (`ObjectFilter::WithValue`)
// ---------------------------------------------------------------------------

/// Every permanent `filter` describes becomes red, so a test reads the
/// filter's answer for an object off its colors: the layer walk's asker.
fn redden_what_matches(game: &mut GameState, source: ObjectId, filter: ObjectFilter) {
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: EffectId::UNASSIGNED,
        source,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer5Color,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected_objects: ObjectSet::battlefield_filter(filter),
        modification: EffectModification::AddColor(Color::Red),
    });
}

/// Which of `objects` `filter` describes, on a fresh board each time so one
/// filter's answer is never read through another's.
fn matched(filter: ObjectFilter) -> [bool; 4] {
    let mut game = setup_two_player_game();
    let small = put_on_battlefield(&mut game, vanilla_creature(1, 4, &[]), 0);
    let even = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let big = put_on_battlefield(&mut game, vanilla_creature(5, 2, &[]), 0);
    let land = put_on_battlefield(&mut game, forest(), 0);
    redden_what_matches(&mut game, even, filter);
    [small, even, big, land].map(|id| get_effective_colors(&game, id).contains(&Color::Red))
}

/// Dusk's "power 3 or greater" and Dawn's "power 2 or less", Citywide Bust's
/// "toughness 4 or greater": one filter, the number and the comparison its
/// two parts. Over a 1/4, a 3/3, a 5/2 and a land.
#[test]
fn a_power_or_a_toughness_is_compared_at_least_and_at_most() {
    assert_eq!(matched(power_at_least(3)), [false, true, true, false], "Dusk: power 3 or greater");
    assert_eq!(matched(power_at_most(2)), [true, false, false, false], "Dawn: power 2 or less");
    assert_eq!(matched(toughness_at_least(4)), [true, false, false, false], "Citywide Bust: toughness 4 or greater");
    assert_eq!(matched(toughness_at_most(2)), [false, false, true, false], "toughness 2 or less");
}

/// "Equal to" (Void's shape, over a power) and "less than" (mentor's, CR
/// 702.134a), which an at-least and an at-most pair could not say.
#[test]
fn a_number_is_compared_exactly_and_strictly_less() {
    let power = |is: ValueIs| ObjectFilter::WithValue { of: ObjectValue::Power, is: Box::new(is) };
    assert_eq!(matched(power(ValueIs::Exactly(AmountExpr::Fixed(3)))), [false, true, false, false]);
    assert_eq!(matched(power(ValueIs::LessThan(AmountExpr::Fixed(3)))), [true, false, false, false], "3 is not less than 3");
    assert_eq!(matched(power(ValueIs::LessThan(AmountExpr::Fixed(6)))), [true, true, true, false]);
}

/// CR 208.3: a noncreature has no power or toughness, so it is neither side
/// of a comparison. The land is no "toughness 0 or greater", and no "not
/// toughness 0 or greater" either.
// COVERS-PARTIAL: ATOM-208.3-001
#[test]
fn an_object_with_no_such_number_is_neither_side_of_the_comparison() {
    assert_eq!(matched(toughness_at_least(0)), [true, true, true, false]);
    assert_eq!(matched(ObjectFilter::Not(Box::new(toughness_at_least(0)))), [false, false, false, false]);
}

/// The table compares plain numbers. An amount nobody read first, here X on
/// a board with no X, is a card asking where its site cannot answer: loud,
/// never a silent "no".
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "compares a number with an amount nothing read first")]
fn an_amount_nothing_read_is_refused_loudly() {
    matched(power_at_least(AmountExpr::X));
}

// ---------------------------------------------------------------------------
// A player verb whose player is gone (CR 608.2b, 800.4a)
// ---------------------------------------------------------------------------

/// **Fixture, modeled on Combat Tutorial**: "Target player draws two cards.
/// Put a +1/+1 counter on up to one target creature you control." Two
/// instances of "target" in one spell, the first a player a verb acts on.
fn combat_tutorial_fixture() -> Arc<CardData> {
    let text = "Target player draws two cards. Put a +1/+1 counter on up to one target creature you control.";
    let up_to_one_creature = EffectRecipient::Target(SelectionFilter::Permanent(creatures_you_control()), TargetCount::UpTo(1));
    sorcery(
        "Fixture Combat Tutorial",
        (ManaType::Blue, Color::Blue),
        text,
        Effect::Sequence(vec![draw_cards(target_player(), 2), put_counters(up_to_one_creature, CounterType::PlusOnePlusOne, 1)]),
    )
}

/// CR 608.2b — the spell resolves for the target still legal and "won't
/// affect" the one that is not: with its target player gone from the game,
/// nobody draws. The draw fell to the spell's controller before.
#[test]
fn a_draw_whose_target_player_has_left_is_no_ones() {
    let mut game = setup_game(3);
    for seat in 0..3 {
        fill_library(&mut game, seat, 5);
    }
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![ChoiceOption::Player(1)]);
    picks(&answers, vec![ChoiceOption::Object(bears)]);
    cast(&mut game, combat_tutorial_fixture(), ManaType::Blue, answers);
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves: one target is still legal");
    assert_eq!(game.players[0].hand.len(), 0, "the departed player's draw is no one's");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 1, "the creature is still affected");
}

// ---------------------------------------------------------------------------
// What a choice of a kind offers
// ---------------------------------------------------------------------------

/// CR 205.3m — the creature types are the rule's list, word for word as the
/// rules file prints it: Human is one, the two-word Time Lord is one, and
/// Forest, a land type, is not a `CreatureType` at all.
#[test]
fn the_creature_types_are_the_ones_the_rule_lists() {
    let rules = include_str!("../../MTG-Rules/versions/tmnt.txt");
    let rule = rules.lines().find(|line| line.starts_with("205.3m ")).expect("the rules file has CR 205.3m");
    let one_word = rule.split("one word long: ").nth(1).expect("the rule lists its one-word types").trim_end_matches('.');
    let mut printed: Vec<String> =
        one_word.split(", ").map(|word| word.trim_start_matches("and ").replace('\u{2019}', "'")).chain(["Time Lord".to_string()]).collect();
    let mut ours: Vec<String> = CreatureType::ALL.iter().map(|creature_type| Subtype::Creature(*creature_type).word()).collect();
    printed.sort();
    ours.sort();
    assert_eq!(ours, printed);
    assert!(ours.contains(&"Human".to_string()) && ours.contains(&"Time Lord".to_string()));
    assert!(!ours.contains(&"Forest".to_string()));
}

/// CR 105.4 — "choose a color" is one of the five; "multicolored" and
/// "colorless" are not colors, and `Color` has no arm to say either.
#[test]
fn the_colors_are_the_five() {
    assert_eq!(Color::ALL, [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]);
}

/// The same spell with its player still here: that player draws, and its
/// controller does not.
#[test]
fn a_draw_goes_to_its_target_player() {
    let mut game = setup_game(3);
    for seat in 0..3 {
        fill_library(&mut game, seat, 5);
    }
    // No creature to target, so "up to one" is none and is not asked.
    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![ChoiceOption::Player(1)]);
    cast(&mut game, combat_tutorial_fixture(), ManaType::Blue, answers);

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].hand.len(), 2);
    assert_eq!(game.players[0].hand.len(), 0);
}

// ---------------------------------------------------------------------------
// A value chosen as the effect resolves, kept, and read (CR 608.2d)
// ---------------------------------------------------------------------------

/// CR 105.4 — the color is chosen as the spell resolves, from the five, and
/// the step after it reads the color kept: every permanent of it, whoever
/// controls it, and nothing else. `Color` has five arms and none for
/// "colorless" or "multicolored", so no sixth answer can be written, and
/// this board shows the offer, not a refusal.
// COVERS-PARTIAL: ATOM-105.4-001
#[test]
fn wash_out_returns_the_permanents_of_the_color_chosen_as_it_resolves() {
    let mut game = setup_two_player_game();
    let mine = put_on_battlefield(&mut game, creature_of("Blue One", Color::Blue, CreatureType::Merfolk), 0);
    let theirs = put_on_battlefield(&mut game, creature_of("Blue Two", Color::Blue, CreatureType::Merfolk), 1);
    let red = put_on_battlefield(&mut game, creature_of("Red One", Color::Red, CreatureType::Goblin), 1);
    let land = put_on_battlefield(&mut game, forest(), 0);
    let (spell, _) = cast_paying(&mut game, wash_out_fixture(), &[(ManaType::Blue, 4)], ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(ChoiceKind::ChooseColor { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::Color(Color::Blue)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(chooser, kind, offered)] = asked.as_slice() else { panic!("one question, asked as it resolves: {asked:?}") };
    assert_eq!(*chooser, 0, "its controller chooses");
    assert!(matches!(kind, ChoiceKind::ChooseColor { source } if *source == spell), "the question names the spell: {kind:?}");
    assert_eq!(*offered, Color::ALL.map(ChoiceOption::Color).to_vec(), "the five colors");
    assert!(game.players[0].hand.contains(&mine) && game.players[1].hand.contains(&theirs), "each blue permanent to its owner's hand");
    assert!(game.battlefield.contains_key(&red) && game.battlefield.contains_key(&land), "a red creature and a colorless land stay");
}

/// CR 205.3m — the type is chosen from the rule's list
/// (`the_creature_types_are_the_ones_the_rule_lists` reads it off the rules
/// file), and "aren't of the chosen type" is the filter's `Not` over the
/// type kept. Forest is a `LandType` and cannot be offered.
// COVERS-PARTIAL: ATOM-205.3m-001
#[test]
fn kindred_dominance_spares_the_creature_type_chosen_as_it_resolves() {
    let mut game = setup_two_player_game();
    let elf = put_on_battlefield(&mut game, creature_of("An Elf", Color::Green, CreatureType::Elf), 0);
    let their_elf = put_on_battlefield(&mut game, creature_of("Another Elf", Color::Green, CreatureType::Elf), 1);
    let goblin = put_on_battlefield(&mut game, creature_of("A Goblin", Color::Red, CreatureType::Goblin), 1);
    cast_paying(&mut game, kindred_dominance_fixture(), &[(ManaType::Black, 7)], ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(
        ChoiceKind::ChooseCreatureType { source: ObjectId::UNASSIGNED },
        vec![ChoiceOption::CreatureType(CreatureType::Elf)],
    );
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(_, _, offered)] = asked.as_slice() else { panic!("one question: {asked:?}") };
    assert_eq!(*offered, CreatureType::ALL.map(ChoiceOption::CreatureType).to_vec(), "every creature type the rule lists");
    assert!(game.battlefield.contains_key(&elf) && game.battlefield.contains_key(&their_elf), "the Elves are of the chosen type");
    assert!(!game.battlefield.contains_key(&goblin), "the Goblin is not");
}

/// Walk `game` to player 0's next upkeep, where a "beginning of your
/// upkeep" ability has triggered and waits to be placed.
fn to_player_0s_upkeep(game: &mut GameState) {
    for _ in 0..game.num_players() {
        pass_turn(game);
    }
    assert_eq!(game.active_player, 0, "a rotation later it is player 0's turn again");
    let (_, step) = game.advance_turn(&test_dp()).expect("the untap step ends");
    assert_eq!(step, Some(StepType::Upkeep));
}

/// A triggered ability keeps what it chooses as a spell does, on the
/// ability resolving. World Queller's card type is offered from CR 205.2a's
/// list and read by each player's edict: a player picks from their
/// permanents of that type, and one with none sacrifices nothing. Declined,
/// nothing is chosen and nothing sacrificed.
#[test]
fn world_queller_keeps_a_card_type_for_each_players_edict() {
    for chooses in [true, false] {
        let mut game = stocked(3);
        let queller = put_on_battlefield(&mut game, world_queller_fixture(), 0);
        let relics = [0, 1].map(|seat| put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), seat));
        let bears = [0, 1, 2].map(|seat| put_on_battlefield(&mut game, grizzly_bears(), seat));
        to_player_0s_upkeep(&mut game);
        assert!(game.place_pending_triggers(&test_dp()).expect("the trigger is placed"));

        let answers = ScriptedDecisionProvider::new();
        let may = ChoiceKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED };
        if chooses {
            answers.expect_take(may);
            answers.expect_choice(ChoiceKind::ChooseCardType { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::CardType(CardType::Artifact)]);
        } else {
            answers.expect_choice(may, Vec::new());
        }
        let (seat, asked) = Watching::new(answers);
        game.resolve_top_of_stack(&seat).expect("the trigger resolves");

        let asked = asked.of_options.borrow();
        if chooses {
            // Each player's one artifact is their only candidate, so no
            // player is asked which.
            let [_, (chooser, _, offered)] = asked.as_slice() else { panic!("the may, then the card type: {asked:?}") };
            assert_eq!(*chooser, 0);
            assert_eq!(*offered, CardType::ALL.map(ChoiceOption::CardType).to_vec(), "every card type");
        } else {
            assert_eq!(asked.len(), 1, "declined, nothing more is asked");
        }
        for relic in relics {
            assert_eq!(game.battlefield.contains_key(&relic), !chooses, "an artifact sacrificed: {chooses}");
        }
        assert!(bears.iter().all(|id| game.battlefield.contains_key(id)) && game.battlefield.contains_key(&queller), "no creature is an artifact");
    }
}

/// "Choose a player" offers every player, "choose an opponent" every other
/// one, and the step after each acts on the player kept. The second choice
/// takes the first's place under the name.
#[test]
fn a_player_chosen_as_the_spell_resolves_is_who_the_next_step_acts_on() {
    let mut game = stocked(3);
    cast(&mut game, bestowed_favors(), ManaType::Green, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    let a_player = || ChoiceKind::ChoosePlayer { source: ObjectId::UNASSIGNED };
    answers.expect_choice(a_player(), vec![ChoiceOption::Player(2)]);
    answers.expect_choice(a_player(), vec![ChoiceOption::Player(1)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let offered: Vec<Vec<ChoiceOption>> = asked.of_options.borrow().iter().map(|(_, _, options)| options.clone()).collect();
    assert_eq!(offered, [[0, 1, 2].map(ChoiceOption::Player).to_vec(), [1, 2].map(ChoiceOption::Player).to_vec()]);
    assert_eq!([0, 1, 2].map(|seat| game.players[seat].hand.len()), [0, 0, 1], "the first player chosen draws");
    let tokens: Vec<Option<PlayerId>> =
        game.battlefield.keys().filter(|id| game.objects[id].is_token).map(|id| get_effective_controller(&game, id)).collect();
    assert_eq!(tokens, [Some(1)], "the second creates the token");
}

/// "At random" asks nobody: the player is drawn from the game's seeded rng,
/// among the opponents still in the game, so a seed replays and the seeds
/// between them reach each opponent.
#[test]
fn an_opponent_chosen_at_random_is_drawn_from_the_seed() {
    let damaged_under = |seed: u64| {
        let mut game = setup_game(3);
        game.reseed(seed);
        cast(&mut game, consequences_at_random(), ManaType::Red, ScriptedDecisionProvider::new());
        game.resolve_top_of_stack(&test_dp()).expect("it resolves, and nobody is asked");
        let damaged: Vec<PlayerId> = (0..3).filter(|&seat| game.players[seat].life_total == 17).collect();
        assert_eq!(damaged.len(), 1, "one player takes the 3 damage");
        damaged[0]
    };
    let drawn: Vec<PlayerId> = (0..16).map(damaged_under).collect();
    assert!(!drawn.contains(&0), "never its controller");
    assert!(drawn.contains(&1) && drawn.contains(&2), "either opponent, by the seed: {drawn:?}");
    assert_eq!(drawn, (0..16).map(damaged_under).collect::<Vec<_>>(), "a seed replays");
}

/// A number drawn at random is one of the numbers the text allows, and the
/// amount after it reads it.
#[test]
fn a_number_chosen_at_random_is_within_its_bounds() {
    let text = "Choose a number between 2 and 3 at random. You draw that many cards.";
    let drawn_under = |seed: u64| {
        let mut game = stocked(2);
        game.reseed(seed);
        let effect = Effect::Sequence(vec![choose_number(at_random(), 2, 3), draw_cards(you(), chosen(ChoiceName::Number))]);
        cast(&mut game, sorcery("Fixture Lot", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, ScriptedDecisionProvider::new());
        game.resolve_top_of_stack(&test_dp()).expect("it resolves, and nobody is asked");
        game.players[0].hand.len()
    };
    let drawn: Vec<usize> = (0..16).map(drawn_under).collect();
    assert!(drawn.iter().all(|cards| (2..=3).contains(cards)) && drawn.contains(&2) && drawn.contains(&3), "{drawn:?}");
}

/// The Nipton Lottery: one creature drawn at random, three steps on "that
/// creature" (gained, untapped, hasty) and one on every other. Whichever
/// the seed draws, it is the one left, and across seeds an opponent's is.
#[test]
fn the_nipton_lottery_acts_on_the_creature_drawn_and_destroys_the_rest() {
    let mut drew_an_opponents = false;
    for seed in 0..8 {
        let mut game = setup_two_player_game();
        game.reseed(seed);
        let creatures = [0, 1, 1, 1].map(|seat| put_on_battlefield(&mut game, grizzly_bears(), seat));
        for id in creatures {
            game.battlefield.get_mut(&id).expect("on the battlefield").tapped = true;
        }
        cast_paying(&mut game, the_nipton_lottery_fixture(), &[(ManaType::Black, 3), (ManaType::Red, 1)], ScriptedDecisionProvider::new());
        game.resolve_top_of_stack(&test_dp()).expect("it resolves, and nobody is asked");

        let left: Vec<ObjectId> = creatures.into_iter().filter(|id| game.battlefield.contains_key(id)).collect();
        let [drawn] = left.as_slice() else { panic!("one creature is left, the rest destroyed: {left:?}") };
        assert_eq!(get_effective_controller(&game, *drawn), Some(0), "you gain control of that creature");
        assert!(!game.battlefield[drawn].tapped, "untap it");
        assert!(has_keyword(&game, *drawn, KeywordFlag::Haste), "it gains haste");
        drew_an_opponents |= *drawn != creatures[0];
    }
    assert!(drew_an_opponents, "three of the four creatures are an opponent's");
}

/// Duneblast: the creature chosen is no target, so an opponent's hexproof
/// creature can be the one spared (CR 115.10a), and "the rest" is every
/// creature other than it. "Up to one" may be none, and then the rest is
/// every creature.
#[test]
fn duneblast_spares_the_creature_chosen_and_destroys_the_rest() {
    for spares_one in [true, false] {
        let mut game = setup_two_player_game();
        let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let hexproof = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Hexproof]), 1);
        let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let pool = [(ManaType::White, 5), (ManaType::Black, 1), (ManaType::Green, 1)];
        cast_paying(&mut game, duneblast_fixture(), &pool, ScriptedDecisionProvider::new());

        let answers = ScriptedDecisionProvider::new();
        picks(&answers, if spares_one { vec![ChoiceOption::Object(hexproof)] } else { Vec::new() });
        game.resolve_top_of_stack(&answers).expect("it resolves");
        assert_eq!(game.battlefield.contains_key(&hexproof), spares_one);
        assert!(!game.battlefield.contains_key(&mine) && !game.battlefield.contains_key(&theirs), "the rest are destroyed");
    }
}

/// "It" after "choose a creature you control" is the creature chosen: one
/// question, among the chooser's creatures, and both changes on its answer.
#[test]
fn two_changes_after_a_choice_act_on_the_one_creature_chosen() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    cast(&mut game, chosen_champion(), ManaType::Green, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![ChoiceOption::Object(second)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(_, _, offered)] = asked.as_slice() else { panic!("asked once: {asked:?}") };
    assert_eq!(*offered, [first, second].map(ChoiceOption::Object).to_vec(), "the creatures you control");
    assert_eq!((get_effective_power(&game, second), get_effective_toughness(&game, second)), (Some(4), Some(4)));
    assert!(has_keyword(&game, second, KeywordFlag::Trample));
    assert_eq!(get_effective_power(&game, first), Some(2));
    assert!(!has_keyword(&game, first, KeywordFlag::Trample));
}

/// "Target opponent chooses a number": the question goes to the target, its
/// bounds the text's, and the amount after it reads the answer.
#[test]
fn a_number_a_target_opponent_chooses_is_asked_of_that_opponent() {
    let mut game = setup_game(3);
    let targets = ScriptedDecisionProvider::new();
    picks(&targets, vec![ChoiceOption::Player(2)]);
    let (spell, _) = cast(&mut game, damnations_tally(), ManaType::Black, targets);

    let answers = ScriptedDecisionProvider::new();
    let any_number = ChoiceKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 0, at_most: 3 }) };
    answers.expect_number(any_number, 2);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_numbers.borrow();
    let [(chooser, kind, least, most)] = asked.as_slice() else { panic!("one question: {asked:?}") };
    assert_eq!((*chooser, *least, *most), (2, 0, 3), "the target opponent, between 0 and 3");
    assert!(matches!(kind, ChoiceKind::ChooseNumber { source, .. } if *source == spell));
    assert_eq!([0, 1, 2].map(|seat| game.players[seat].life_total), [20, 20, 18]);
}

/// A player kept is a player a later step can ask: "that player chooses a
/// number", and "that player may", each asked of the opponent chosen.
#[test]
fn a_chosen_player_makes_the_choices_a_later_step_gives_them() {
    let text = "Choose an opponent. That player chooses a number between 1 and 2, then may draw a card. You draw cards equal to the chosen number.";
    let that_player = || the_chosen(ChoiceName::Player);
    let effect = Effect::Sequence(vec![
        choose_player(you(), PlayerSet::Opponents),
        choose_number(that_player(), 1, 2),
        Effect::Optional { chooser: PlayerRef::NamedBy(Box::new(that_player())), effect: Box::new(draw_cards(that_player(), 1)) },
        draw_cards(you(), chosen(ChoiceName::Number)),
    ]);
    let mut game = stocked(3);
    cast(&mut game, sorcery("Fixture Parley", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(ChoiceKind::ChoosePlayer { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::Player(2)]);
    let any_number = ChoiceKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 1, at_most: 2 }) };
    answers.expect_number(any_number, 2);
    answers.expect_take(ChoiceKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED });
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked_of: Vec<PlayerId> = asked.of_options.borrow().iter().map(|(player, _, _)| *player).collect();
    assert_eq!(asked_of, [0, 2], "you choose the opponent, and that opponent is asked the may");
    assert_eq!(asked.of_numbers.borrow().iter().map(|(player, ..)| *player).collect::<Vec<_>>(), [2], "that player chooses the number");
    assert_eq!([0, 1, 2].map(|seat| game.players[seat].hand.len()), [2, 0, 1]);
}

/// CR 400.7 — what is kept is the object as it was chosen. Flickered
/// between the choice and the step that names it, the creature is a new
/// object, and "the chosen creature" finds nothing. With another creature
/// flickered, the chosen one is found and destroyed.
#[test]
fn a_chosen_creature_that_left_and_came_back_is_no_longer_the_one_chosen() {
    let text = "Choose a creature. Exile target creature, then return it to the battlefield under its owner's control. Destroy the chosen creature.";
    let effect = || {
        Effect::Sequence(vec![
            choose_objects(you(), creature(), TargetCount::Exactly(1)),
            exile(target_creature()),
            return_to_the_battlefield(the_target(), ReturnUnder::Owner),
            destroy(the_chosen(ChoiceName::Objects)),
        ])
    };
    for chosen_is_flickered in [true, false] {
        let mut game = setup_two_player_game();
        let flickered = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let targets = ScriptedDecisionProvider::new();
        picks(&targets, vec![ChoiceOption::Object(flickered)]);
        cast(&mut game, sorcery("Fixture Blink", (ManaType::White, Color::White), text, effect()), ManaType::White, targets);

        let answers = ScriptedDecisionProvider::new();
        let chosen = if chosen_is_flickered { flickered } else { other };
        picks(&answers, vec![ChoiceOption::Object(chosen)]);
        game.resolve_top_of_stack(&answers).expect("it resolves");

        // The card is back under the id it left with, so the id alone would
        // still find it: what the choice kept is that id's earlier existence.
        assert!(game.battlefield.contains_key(&flickered), "the flickered creature returned");
        let creatures = game.battlefield.keys().count();
        assert_eq!(creatures, if chosen_is_flickered { 2 } else { 1 }, "chosen and flickered: {chosen_is_flickered}");
        assert_eq!(game.players[0].graveyard.len(), if chosen_is_flickered { 1 } else { 2 }, "the spell, and the creature destroyed");
    }
}
