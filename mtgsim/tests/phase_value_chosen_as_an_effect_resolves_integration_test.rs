//! `value-chosen-as-an-effect-resolves` (`choices-architecture.md`), the
//! first of `values-chosen-at-resolution`'s three: a number read off an
//! object and compared, a player verb whose player is gone, and a value or
//! objects chosen as an effect resolves, kept, and used.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use mtgsim::cards::authoring::{
    choose_card_type_among, choose_color, choose_creature_type, choose_number, choose_objects, choose_player, creature, destroy, draw_cards,
    card_type, color, creature_type, exile, lose_life, power_at_least, power_at_most, the_chosen_card_type, the_chosen_color,
    the_chosen_creature_type,
    return_to_the_battlefield, spell_ability, tap, target_creature, target_opponent, the_chosen_number, the_chosen_objects, the_chosen_player,
    the_target, toughness_at_least, toughness_at_most, you, you_control,
};
use mtgsim::cards::fixtures::{bestowed_favors, chosen_champion, consequences_at_random, damnations_tally};
use mtgsim::cards::printings::{
    by_invitation_only, combat_tutorial, duneblast, expel_the_interlopers, glorious_anthem, grizzly_bears, kindred_dominance,
    the_nipton_lottery, wash_out, world_queller,
};
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
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, ObjectValue, PlayerRef, PlayerSet,
    ReturnUnder, TargetCount, ValueIs,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::decision::decision_types::{DecisionContext, DecisionKind, DecisionOption};
use mtgsim::decision::provider::{DecisionProvider, ScriptedDecisionProvider, SeatMode};
use mtgsim::policies::mana_window_stop::ManaWindowStop;

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
fn picks(answers: &ScriptedDecisionProvider, chosen: Vec<DecisionOption>) {
    answers.expect_choice(DecisionKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

/// What each prompt offered: who was asked, the question, and its options,
/// a number's as its least and most.
#[derive(Default)]
struct Asked {
    of_options: RefCell<Vec<(PlayerId, DecisionKind, Vec<DecisionOption>)>>,
    of_numbers: RefCell<Vec<(PlayerId, DecisionKind, u64, u64)>>,
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
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &DecisionContext, options: &[DecisionOption], bounds: (usize, usize)) -> Vec<usize> {
        self.asked.of_options.borrow_mut().push((player, context.kind.clone(), options.to_vec()));
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &DecisionContext, min: u64, max: u64) -> u64 {
        self.asked.of_numbers.borrow_mut().push((player, context.kind.clone(), min, max));
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        total: u64,
        buckets: &[DecisionOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &DecisionContext, items: &[DecisionOption]) -> Vec<usize> {
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

/// CR 608.2b — Combat Tutorial resolves for the target still legal and
/// "won't affect" the one that is not: with its target player gone from the
/// game, nobody draws.
#[test]
fn a_draw_whose_target_player_has_left_is_no_ones() {
    let mut game = setup_game(3);
    for seat in 0..3 {
        fill_library(&mut game, seat, 5);
    }
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![DecisionOption::Player(1)]);
    picks(&answers, vec![DecisionOption::Object(bears)]);
    cast_paying(&mut game, combat_tutorial(), &[(ManaType::Blue, 3)], answers);
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
/// Forest, a land type, is not a `CreatureType` at all. The rules text is
/// not in the repository, so a checkout without it (CI's) checks the three
/// named types alone, and says so.
#[test]
fn the_creature_types_are_the_ones_the_rule_lists() {
    let mut ours: Vec<String> = CreatureType::ALL.iter().map(|creature_type| Subtype::Creature(*creature_type).word()).collect();
    ours.sort();
    assert!(ours.contains(&"Human".to_string()) && ours.contains(&"Time Lord".to_string()));
    assert!(!ours.contains(&"Forest".to_string()));

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../MTG-Rules/versions/tmnt.txt");
    let Ok(rules) = std::fs::read_to_string(path) else {
        eprintln!("no rules file at {path}: the creature types are not compared with CR 205.3m's list here");
        return;
    };
    let rule = rules.lines().find(|line| line.starts_with("205.3m ")).expect("the rules file has CR 205.3m");
    let one_word = rule.split("one word long: ").nth(1).expect("the rule lists its one-word types").trim_end_matches('.');
    let mut printed: Vec<String> =
        one_word.split(", ").map(|word| word.trim_start_matches("and ").replace('\u{2019}', "'")).chain(["Time Lord".to_string()]).collect();
    printed.sort();
    assert_eq!(ours, printed);
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
    picks(&answers, vec![DecisionOption::Player(1)]);
    cast_paying(&mut game, combat_tutorial(), &[(ManaType::Blue, 3)], answers);

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].hand.len(), 2);
    assert_eq!(game.players[0].hand.len(), 0);
}

// ---------------------------------------------------------------------------
// A value chosen as the effect resolves, kept, and read (CR 608.2d)
// ---------------------------------------------------------------------------

/// Expel the Interlopers over a 0/3, a 2/2, a 3/3 and a 5/5 and a land,
/// with `number` chosen as it resolves: what was asked, and which of the
/// four creatures are left. With `anthem`, player 0's two get +1/+1 from a
/// Glorious Anthem.
fn expel_choosing(number: u64, anthem: bool) -> (Rc<Asked>, ObjectId, [bool; 4], bool) {
    let mut game = setup_two_player_game();
    if anthem {
        put_on_battlefield(&mut game, glorious_anthem(), 0);
    }
    let creatures = [
        put_on_battlefield(&mut game, vanilla_creature(0, 3, &[]), 0),
        put_on_battlefield(&mut game, grizzly_bears(), 0),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(5, 5, &[KeywordFlag::Hexproof]), 1),
    ];
    let land = put_on_battlefield(&mut game, forest(), 1);
    let (spell, _) = cast_paying(&mut game, expel_the_interlopers(), &[(ManaType::White, 5)], ScriptedDecisionProvider::new());
    assert!(game.stack.contains(&spell), "cast with nothing chosen: the number is no part of casting it");

    let answers = ScriptedDecisionProvider::new();
    let any_number = DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 0, at_most: 10 }) };
    answers.expect_number(any_number, number);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");
    (asked, spell, creatures.map(|id| game.battlefield.contains_key(&id)), game.battlefield.contains_key(&land))
}

/// CR 608.2d — the number is chosen as the spell resolves, never as it is
/// cast, from 0 through 10, both ends offered. Each creature whose power is
/// the number or more is destroyed, whoever controls it and hexproof or
/// not: nothing is targeted. This board asks the number of a spell; the
/// atom's own board is a "you may sacrifice".
// RULING: Expel the Interlopers #1 - "The numbers you may choose include 0 and 10."
// COVERS-PARTIAL: ATOM-608.2d-001
#[test]
fn expel_the_interlopers_offers_0_through_10_and_destroys_from_the_number_up() {
    let (asked, spell, left, land_left) = expel_choosing(3, false);
    let asked = asked.of_numbers.borrow();
    let [(chooser, kind, least, most)] = asked.as_slice() else { panic!("one question, asked as it resolves: {asked:?}") };
    assert_eq!((*chooser, *least, *most), (0, 0, 10), "its controller, from 0 through 10");
    assert!(matches!(kind, DecisionKind::ChooseNumber { source, .. } if *source == spell), "the question names the spell: {kind:?}");
    assert_eq!(left, [true, true, false, false], "power 3 and power 5 are 3 or more");
    assert!(land_left, "a land has no power, and is no creature");

    assert_eq!(expel_choosing(0, false).2, [false; 4], "0: every creature's power is 0 or more");
    assert_eq!(expel_choosing(10, false).2, [true; 4], "10: no creature here has power 10");
    assert_eq!(expel_choosing(5, false).2, [true, true, true, false], "the comparison is 'or equal'");
}

/// The power compared is the creature's as the step runs, every effect
/// applied (CR 613): under a Glorious Anthem the 2/2 is a 3/3 and a 3
/// destroys it, and the 0/3 is a 1/4 that a 1 destroys and a 2 spares.
#[test]
fn expel_the_interlopers_compares_each_creatures_power_as_it_is() {
    assert_eq!(expel_choosing(3, true).2, [true, false, false, false]);
    assert_eq!(expel_choosing(1, true).2, [false; 4]);
    assert_eq!(expel_choosing(2, true).2, [true, false, false, false]);
}

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
    let (spell, _) = cast_paying(&mut game, wash_out(), &[(ManaType::Blue, 4)], ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(DecisionKind::ChooseColor { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Color(Color::Blue)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(chooser, kind, offered)] = asked.as_slice() else { panic!("one question, asked as it resolves: {asked:?}") };
    assert_eq!(*chooser, 0, "its controller chooses");
    assert!(matches!(kind, DecisionKind::ChooseColor { source } if *source == spell), "the question names the spell: {kind:?}");
    assert_eq!(*offered, Color::ALL.map(DecisionOption::Color).to_vec(), "the five colors");
    assert!(game.players[0].hand.contains(&mine) && game.players[1].hand.contains(&theirs), "each blue permanent to its owner's hand");
    assert!(game.battlefield.contains_key(&red) && game.battlefield.contains_key(&land), "a red creature and a colorless land stay");
}

/// CR 205.3m — the type is chosen from the rule's list
/// (`the_creature_types_are_the_ones_the_rule_lists` reads it off the rules
/// file), and "aren't of the chosen type" is the filter's `Not` over the
/// type kept. Forest is a `LandType` and cannot be offered. One type is
/// chosen, and an Elf Warrior is an Elf; the one question asked is the type,
/// so no player acts between the choice and the destruction.
// RULING: Kindred Dominance #1 - "Once Kindred Dominance begins to resolve, no players may take other actions until it's done. Notably, players can't try to save their creatures after you've chosen a creature type."
// RULING: Kindred Dominance #2 - "You must choose an existing creature type, such as Vampire or Cat. Card types such as "artifact" can't be chosen."
// RULING: Kindred Dominance #3 - "You can't choose multiple creature types, such as "Cat Warrior." A Cat Warrior is both a Cat and a Warrior. It's affected by anything that affects either type and unaffected by things that affect non-Cat or non-Warrior creatures."
// COVERS-PARTIAL: ATOM-205.3m-001
#[test]
fn kindred_dominance_spares_the_creature_type_chosen_as_it_resolves() {
    let mut game = setup_two_player_game();
    let elf = put_on_battlefield(&mut game, creature_of("An Elf", Color::Green, CreatureType::Elf), 0);
    let their_elf = put_on_battlefield(&mut game, creature_of("Another Elf", Color::Green, CreatureType::Elf), 1);
    let goblin = put_on_battlefield(&mut game, creature_of("A Goblin", Color::Red, CreatureType::Goblin), 1);
    let elf_warrior = CardDataBuilder::new("An Elf Warrior")
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(2, 2)
        .build();
    let elf_warrior = put_on_battlefield(&mut game, elf_warrior, 1);
    cast_paying(&mut game, kindred_dominance(), &[(ManaType::Black, 7)], ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(
        DecisionKind::ChooseCreatureType { source: ObjectId::UNASSIGNED },
        vec![DecisionOption::CreatureType(CreatureType::Elf)],
    );
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(_, _, offered)] = asked.as_slice() else { panic!("one question: {asked:?}") };
    assert_eq!(*offered, CreatureType::ALL.map(DecisionOption::CreatureType).to_vec(), "every creature type the rule lists");
    assert!(game.battlefield.contains_key(&elf) && game.battlefield.contains_key(&their_elf), "the Elves are of the chosen type");
    assert!(game.battlefield.contains_key(&elf_warrior), "an Elf Warrior is an Elf");
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
// RULING: World Queller #2 - "You don't choose a card type until the ability resolves. Once you choose a type, it's too late for players to respond."
#[test]
fn world_queller_keeps_a_card_type_for_each_players_edict() {
    for chooses in [true, false] {
        let mut game = stocked(3);
        let queller = put_on_battlefield(&mut game, world_queller(), 0);
        let relics = [0, 1].map(|seat| put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), seat));
        let bears = [0, 1, 2].map(|seat| put_on_battlefield(&mut game, grizzly_bears(), seat));
        to_player_0s_upkeep(&mut game);
        assert!(game.place_pending_triggers(&test_dp()).expect("the trigger is placed"));
        let trigger = *game.stack.last().expect("the trigger is on the stack");

        let answers = ScriptedDecisionProvider::new();
        let may = DecisionKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED };
        if chooses {
            answers.expect_take(may);
            answers.expect_choice(DecisionKind::ChooseCardType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CardType(CardType::Artifact)]);
        } else {
            answers.expect_choice(may, Vec::new());
        }
        let (seat, asked) = Watching::new(answers);
        game.resolve_top_of_stack(&seat).expect("the trigger resolves");

        let asked = asked.of_options.borrow();
        if chooses {
            // Each player's one artifact is their only candidate, so no
            // player is asked which.
            let [_, (chooser, kind, offered)] = asked.as_slice() else { panic!("the may, then the card type: {asked:?}") };
            assert_eq!(*chooser, 0);
            assert!(matches!(kind, DecisionKind::ChooseCardType { source } if *source == trigger), "the question names the ability resolving");
            assert_eq!(*offered, CardType::ALL.map(DecisionOption::CardType).to_vec(), "every card type");
        } else {
            assert_eq!(asked.len(), 1, "declined, nothing more is asked");
        }
        for relic in relics {
            assert_eq!(game.battlefield.contains_key(&relic), !chooses, "an artifact sacrificed: {chooses}");
        }
        assert!(bears.iter().all(|id| game.battlefield.contains_key(id)) && game.battlefield.contains_key(&queller), "no creature is an artifact");
    }
}

/// World Queller's trigger on the stack at player 0's upkeep, in `game`.
fn world_queller_trigger_waiting(game: &mut GameState) {
    put_on_battlefield(game, world_queller(), 0);
    to_player_0s_upkeep(game);
    assert!(game.place_pending_triggers(&test_dp()).expect("the trigger is placed"));
}

/// The script's answers to World Queller's trigger: "you may", taken, then
/// `card_type`.
fn queller_chooses(card_type: CardType) -> ScriptedDecisionProvider {
    let answers = ScriptedDecisionProvider::new();
    answers.expect_take(DecisionKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED });
    answers.expect_choice(DecisionKind::ChooseCardType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CardType(card_type)]);
    answers
}

/// Any card type may be chosen, and one no permanent has (instant) has each
/// player sacrifice nothing.
// RULING: World Queller #1 - "You may choose any card type. Choosing anything other than artifact, battle, creature, enchantment, kindred, land, or planeswalker will result in no permanents being sacrificed, though, since these are the only types permanents can have."
#[test]
fn world_queller_may_choose_a_type_no_permanent_has() {
    let mut game = stocked(2);
    let permanents = [0, 1].map(|seat| put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), seat));
    world_queller_trigger_waiting(&mut game);
    game.resolve_top_of_stack(&queller_chooses(CardType::Instant)).expect("the trigger resolves");
    assert!(permanents.iter().all(|id| game.battlefield.contains_key(id)), "no permanent is an instant");
}

/// Each player picks which permanent they sacrifice in turn order, the
/// player whose turn it is first, and the picks are sacrificed together as
/// the edict's one instruction.
// RULING: World Queller #3 - "First you choose which permanent you'll sacrifice (if you control any of the chosen type), then each other player in turn order does the same, then all chosen permanents are sacrificed at the same time."
#[test]
fn world_queller_asks_each_player_in_turn_order() {
    let mut game = stocked(3);
    let relics = [0, 1, 2].map(|seat| [(); 2].map(|_| put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), seat)));
    world_queller_trigger_waiting(&mut game);
    let answers = queller_chooses(CardType::Artifact);
    for [_, picked] in relics {
        picks(&answers, vec![DecisionOption::Object(picked)]);
    }
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("the trigger resolves");

    let pickers: Vec<PlayerId> = asked.of_options.borrow().iter().skip(2).map(|(player, ..)| *player).collect();
    assert_eq!(pickers, [0, 1, 2], "in turn order, from the player whose turn it is");
    for [kept, picked] in relics {
        assert!(game.battlefield.contains_key(&kept) && !game.battlefield.contains_key(&picked), "each player's pick, and only it");
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
    let a_player = || DecisionKind::ChoosePlayer { source: ObjectId::UNASSIGNED };
    answers.expect_choice(a_player(), vec![DecisionOption::Player(2)]);
    answers.expect_choice(a_player(), vec![DecisionOption::Player(1)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let offered: Vec<Vec<DecisionOption>> = asked.of_options.borrow().iter().map(|(_, _, options)| options.clone()).collect();
    assert_eq!(offered, [[0, 1, 2].map(DecisionOption::Player).to_vec(), [1, 2].map(DecisionOption::Player).to_vec()]);
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

/// By Invitation Only at three seats: the number kept is an edict's count.
/// The number is offered from 0 through 13; each player picks that many of
/// their creatures in turn order, the player whose turn it is first, and a
/// player with fewer sacrifices all of theirs, unasked.
// RULING: By Invitation Only #1 - "The numbers you may choose include 0 and 13."
// RULING: By Invitation Only #2 - "If a player controls fewer than the chosen number of creatures, they will sacrifice all their creatures."
// RULING: By Invitation Only #3 - "Starting with the player whose turn it is, each player in turn order chooses which creatures they will sacrifice, then all the creatures chosen by all players are sacrificed at the same time. Players get to know the choices made by players who chose before them."
#[test]
fn by_invitation_only_has_each_player_sacrifice_the_number_chosen() {
    let mut game = setup_game(3);
    let creatures = [0, 1].map(|seat| [(); 3].map(|_| put_on_battlefield(&mut game, grizzly_bears(), seat)));
    let lone = put_on_battlefield(&mut game, grizzly_bears(), 2);
    cast_paying(&mut game, by_invitation_only(), &[(ManaType::White, 5)], ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    let any_number = DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 0, at_most: 13 }) };
    answers.expect_number(any_number, 2);
    picks(&answers, vec![DecisionOption::Object(creatures[0][0]), DecisionOption::Object(creatures[0][1])]);
    picks(&answers, vec![DecisionOption::Object(creatures[1][1]), DecisionOption::Object(creatures[1][2])]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    assert!(matches!(asked.of_numbers.borrow().as_slice(), [(0, _, 0, 13)]), "its controller, from 0 through 13");
    let pickers: Vec<PlayerId> = asked.of_options.borrow().iter().map(|(player, ..)| *player).collect();
    assert_eq!(pickers, [0, 1], "in turn order; player 2 has fewer than two, and is not asked");
    assert_eq!(creatures[0].map(|id| game.battlefield.contains_key(&id)), [false, false, true]);
    assert_eq!(creatures[1].map(|id| game.battlefield.contains_key(&id)), [true, false, false]);
    assert!(!game.battlefield.contains_key(&lone), "fewer than two: all of player 2's");
}

/// A card type chosen from the few the text prints: Turnabout's "choose
/// artifact, creature, or land", read by a sweep. Its "target player
/// controls" is no filter word yet, so this taps every permanent of the type.
#[test]
fn a_card_type_is_chosen_from_the_few_the_text_prints() {
    let text = "Choose artifact, creature, or land. Tap all permanents of the chosen type.";
    let printed = [CardType::Artifact, CardType::Creature, CardType::Land];
    let effect = Effect::Sequence(vec![choose_card_type_among(you(), printed), tap(each_of_the_chosen_card_type())]);
    let mut game = setup_two_player_game();
    let relic = put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    cast(&mut game, sorcery("Fixture Turn", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(DecisionKind::ChooseCardType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CardType(CardType::Artifact)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");
    assert_eq!(asked.of_options.borrow()[0].2, printed.map(DecisionOption::CardType).to_vec(), "the three printed, and no other");
    assert!(game.battlefield[&relic].tapped && !game.battlefield[&bears].tapped);
}

/// "Each permanent of the chosen card type".
fn each_of_the_chosen_card_type() -> EffectRecipient {
    EffectRecipient::FilteredPermanents(card_type(the_chosen_card_type()))
}

/// A choice reads a value chosen before it: "choose a color, then a
/// creature of that color" offers the creatures of the color kept, and the
/// question's label carries the color, never the name.
#[test]
fn a_choice_of_objects_is_among_what_an_earlier_choice_describes() {
    let text = "Choose a color. Choose a creature of that color. Destroy it.";
    let effect = Effect::Sequence(vec![
        choose_color(you()),
        choose_objects(you(), creature().and(color(the_chosen_color())), TargetCount::Exactly(1)),
        destroy(the_chosen_objects()),
    ]);
    let mut game = setup_two_player_game();
    let blue = [0, 1].map(|seat| put_on_battlefield(&mut game, creature_of("A Blue", Color::Blue, CreatureType::Merfolk), seat));
    let red = put_on_battlefield(&mut game, creature_of("A Red", Color::Red, CreatureType::Goblin), 1);
    cast(&mut game, sorcery("Fixture Hue", (ManaType::Black, Color::Black), text, effect), ManaType::Black, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(DecisionKind::ChooseColor { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Color(Color::Blue)]);
    picks(&answers, vec![DecisionOption::Object(blue[1])]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");
    assert_eq!(asked.of_options.borrow()[1].2, blue.map(DecisionOption::Object).to_vec(), "the blue creatures, and not the red one");
    assert!(game.battlefield.contains_key(&blue[0]) && !game.battlefield.contains_key(&blue[1]) && game.battlefield.contains_key(&red));
}

/// Distant Melody's text: a count reads the type kept, "for each permanent
/// you control of that type".
#[test]
fn a_count_reads_the_creature_type_chosen() {
    let text = "Choose a creature type. Draw a card for each permanent you control of that type.";
    let of_that_type = ObjectSet::battlefield_filter(creature_type(the_chosen_creature_type()).and(you_control()));
    let effect = Effect::Sequence(vec![choose_creature_type(you()), draw_cards(you(), AmountExpr::CountOf(of_that_type))]);
    let mut game = stocked(2);
    for seat in [0, 0, 1] {
        put_on_battlefield(&mut game, creature_of("An Elf", Color::Green, CreatureType::Elf), seat);
    }
    put_on_battlefield(&mut game, creature_of("A Goblin", Color::Red, CreatureType::Goblin), 0);
    cast(&mut game, sorcery("Fixture Melody", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(DecisionKind::ChooseCreatureType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CreatureType(CreatureType::Elf)]);
    game.resolve_top_of_stack(&answers).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), 2, "your two Elves, and not an opponent's");
}

/// `effect`, cast by player 0 at three seats with player 1 its target
/// opponent and `creature` its target creature, resolved with `answers`
/// after player 1 has left the game, when `leaves`.
fn resolve_at_an_opponent_who(leaves: bool, game: &mut GameState, text: &'static str, effect: Effect, creature: ObjectId, answers: ScriptedDecisionProvider) {
    let targets = ScriptedDecisionProvider::new();
    picks(&targets, vec![DecisionOption::Player(1)]);
    picks(&targets, vec![DecisionOption::Object(creature)]);
    cast(game, sorcery("Fixture Absent", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, targets);
    if leaves {
        game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");
    }
    game.resolve_top_of_stack(&answers).expect("it resolves: the creature is still a legal target");
}

/// CR 608.2b — a chooser who is an illegal target makes no choice, and a
/// step that reads the choice does nothing: it does not act on what an
/// earlier step kept under the name. With the chooser still in the game,
/// the second choice replaces the first.
#[test]
fn a_choice_never_made_does_not_leave_an_earlier_answer_for_the_next_step() {
    let text = "Choose a player. That player draws a card. Target opponent chooses a player. That player draws two cards. Tap target creature.";
    for leaves in [true, false] {
        let effect = Effect::Sequence(vec![
            choose_player(you(), PlayerSet::Everyone),
            draw_cards(the_chosen_player(), 1),
            choose_player(target_opponent(), PlayerSet::Everyone),
            draw_cards(the_chosen_player(), 2),
            tap(target_creature()),
        ]);
        let mut game = stocked(3);
        let creatures = [0, 2].map(|seat| put_on_battlefield(&mut game, grizzly_bears(), seat));
        let answers = ScriptedDecisionProvider::new();
        let a_player = || DecisionKind::ChoosePlayer { source: ObjectId::UNASSIGNED };
        answers.expect_choice(a_player(), vec![DecisionOption::Player(2)]);
        if !leaves {
            answers.expect_choice(a_player(), vec![DecisionOption::Player(0)]);
        }
        resolve_at_an_opponent_who(leaves, &mut game, text, effect, creatures[0], answers);
        let drawn = [0, 2].map(|seat| game.players[seat].hand.len());
        assert_eq!(drawn, if leaves { [0, 1] } else { [2, 1] }, "the chooser left: {leaves}");
        assert!(game.battlefield[&creatures[0]].tapped, "the rest of the spell happens");
    }
}

/// The same for a filter: "aren't of the chosen type" with no type chosen
/// describes no creature, so nothing is destroyed, where a filter over an
/// empty choice would have described every creature.
#[test]
fn a_filter_naming_a_choice_never_made_describes_nothing() {
    let text = "Target opponent chooses a creature type. Destroy all creatures that aren't of the chosen type. Tap target creature.";
    for leaves in [true, false] {
        let effect = Effect::Sequence(vec![
            choose_creature_type(target_opponent()),
            destroy(EffectRecipient::FilteredPermanents(creature().other_than(creature_type(the_chosen_creature_type())))),
            tap(target_creature()),
        ]);
        let mut game = setup_game(3);
        let elf = put_on_battlefield(&mut game, creature_of("An Elf", Color::Green, CreatureType::Elf), 0);
        let goblin = put_on_battlefield(&mut game, creature_of("A Goblin", Color::Red, CreatureType::Goblin), 2);
        let answers = ScriptedDecisionProvider::new();
        if !leaves {
            answers.expect_choice(DecisionKind::ChooseCreatureType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CreatureType(CreatureType::Elf)]);
        }
        resolve_at_an_opponent_who(leaves, &mut game, text, effect, elf, answers);
        assert_eq!(game.battlefield.contains_key(&goblin), leaves, "no type chosen, nothing destroyed: {leaves}");
        assert!(game.battlefield[&elf].tapped);
    }
}

/// "That player chooses" after a targeted verb: the chooser is the player
/// the earlier instance of "target" named.
#[test]
fn the_player_an_earlier_target_named_can_be_the_chooser() {
    let text = "Target opponent loses 1 life. That player chooses a number between 0 and 2. You draw that many cards.";
    let effect = Effect::Sequence(vec![lose_life(target_opponent(), 1), choose_number(the_target(), 0, 2), draw_cards(you(), the_chosen_number())]);
    let mut game = stocked(3);
    let targets = ScriptedDecisionProvider::new();
    picks(&targets, vec![DecisionOption::Player(2)]);
    cast(&mut game, sorcery("Fixture Toll", (ManaType::Black, Color::Black), text, effect), ManaType::Black, targets);

    let answers = ScriptedDecisionProvider::new();
    let any_number = DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 0, at_most: 2 }) };
    answers.expect_number(any_number, 2);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");
    assert!(matches!(asked.of_numbers.borrow().as_slice(), [(2, ..)]), "the opponent targeted is asked");
    assert_eq!((game.players[2].life_total, game.players[0].hand.len()), (19, 2));
}

/// The Nipton Lottery: one creature drawn at random, three steps on "that
/// creature" (gained, untapped, hasty) and one on every other. Whichever
/// the seed draws, it is the one left, and across seeds an opponent's
/// hexproof creature is. Nobody is asked anything as it resolves, so no
/// player acts between the draw and the destruction.
// RULING: The Nipton Lottery #1 - "The creature chosen at random isn’t targeted, so a creature with shroud or an opponent’s creature with hexproof can be chosen this way."
// RULING: The Nipton Lottery #2 - "Players can’t take actions between the time the creature is randomly chosen and the time at which the rest of the effects occur."
#[test]
fn the_nipton_lottery_acts_on_the_creature_drawn_and_destroys_the_rest() {
    let mut drew_an_opponents = false;
    for seed in 0..8 {
        let mut game = setup_two_player_game();
        game.reseed(seed);
        let hexproof = || vanilla_creature(2, 2, &[KeywordFlag::Hexproof]);
        let creatures = [(0, grizzly_bears()), (1, hexproof()), (1, hexproof()), (1, hexproof())]
            .map(|(seat, creature)| put_on_battlefield(&mut game, creature, seat));
        for id in creatures {
            game.battlefield.get_mut(&id).expect("on the battlefield").tapped = true;
        }
        cast_paying(&mut game, the_nipton_lottery(), &[(ManaType::Black, 3), (ManaType::Red, 1)], ScriptedDecisionProvider::new());
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
// RULING: Duneblast #1 - "You decide which creature to spare as Duneblast resolves. This choice doesn't target the creature. If you don't choose a creature, then all creatures will be destroyed."
#[test]
fn duneblast_spares_the_creature_chosen_and_destroys_the_rest() {
    for spares_one in [true, false] {
        let mut game = setup_two_player_game();
        let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let hexproof = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Hexproof]), 1);
        let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let pool = [(ManaType::White, 5), (ManaType::Black, 1), (ManaType::Green, 1)];
        cast_paying(&mut game, duneblast(), &pool, ScriptedDecisionProvider::new());

        let answers = ScriptedDecisionProvider::new();
        picks(&answers, if spares_one { vec![DecisionOption::Object(hexproof)] } else { Vec::new() });
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
    picks(&answers, vec![DecisionOption::Object(second)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let [(_, _, offered)] = asked.as_slice() else { panic!("asked once: {asked:?}") };
    assert_eq!(*offered, [first, second].map(DecisionOption::Object).to_vec(), "the creatures you control");
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
    picks(&targets, vec![DecisionOption::Player(2)]);
    let (spell, _) = cast(&mut game, damnations_tally(), ManaType::Black, targets);

    let answers = ScriptedDecisionProvider::new();
    let any_number = DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 0, at_most: 3 }) };
    answers.expect_number(any_number, 2);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_numbers.borrow();
    let [(chooser, kind, least, most)] = asked.as_slice() else { panic!("one question: {asked:?}") };
    assert_eq!((*chooser, *least, *most), (2, 0, 3), "the target opponent, between 0 and 3");
    assert!(matches!(kind, DecisionKind::ChooseNumber { source, .. } if *source == spell));
    assert_eq!([0, 1, 2].map(|seat| game.players[seat].life_total), [20, 20, 18]);
}

/// A player kept is a player a later step can ask: "that player chooses a
/// number", and "that player may", each asked of the opponent chosen.
#[test]
fn a_chosen_player_makes_the_choices_a_later_step_gives_them() {
    let text = "Choose an opponent. That player chooses a number between 1 and 2, then may draw a card. You draw cards equal to the chosen number.";
    let that_player = || the_chosen_player();
    let effect = Effect::Sequence(vec![
        choose_player(you(), PlayerSet::Opponents),
        choose_number(that_player(), 1, 2),
        Effect::Optional { chooser: PlayerRef::NamedBy(Box::new(that_player())), effect: Box::new(draw_cards(that_player(), 1)) },
        draw_cards(you(), the_chosen_number()),
    ]);
    let mut game = stocked(3);
    cast(&mut game, sorcery("Fixture Parley", (ManaType::Blue, Color::Blue), text, effect), ManaType::Blue, ScriptedDecisionProvider::new());

    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(DecisionKind::ChoosePlayer { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Player(2)]);
    let any_number = DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(XOffer::Interval { at_least: 1, at_most: 2 }) };
    answers.expect_number(any_number, 2);
    answers.expect_take(DecisionKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED });
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
            destroy(the_chosen_objects()),
        ])
    };
    for chosen_is_flickered in [true, false] {
        let mut game = setup_two_player_game();
        let flickered = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let targets = ScriptedDecisionProvider::new();
        picks(&targets, vec![DecisionOption::Object(flickered)]);
        cast(&mut game, sorcery("Fixture Blink", (ManaType::White, Color::White), text, effect()), ManaType::White, targets);

        let answers = ScriptedDecisionProvider::new();
        let chosen = if chosen_is_flickered { flickered } else { other };
        picks(&answers, vec![DecisionOption::Object(chosen)]);
        game.resolve_top_of_stack(&answers).expect("it resolves");

        // The card is back under the id it left with, so the id alone would
        // still find it: what the choice kept is that id's earlier existence.
        assert!(game.battlefield.contains_key(&flickered), "the flickered creature returned");
        let creatures = game.battlefield.keys().count();
        assert_eq!(creatures, if chosen_is_flickered { 2 } else { 1 }, "chosen and flickered: {chosen_is_flickered}");
        assert_eq!(game.players[0].graveyard.len(), if chosen_is_flickered { 1 } else { 2 }, "the spell, and the creature destroyed");
    }
}
