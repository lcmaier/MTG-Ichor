//! MA-2, the legal values of X (`mana-architecture.md` §6.1), and the random
//! agent's payment through a pool multiplier it rides on (`codebase-state.md`
//! item 214): what MA-1's check offers, the agent now pays.

use mtgsim::cards::basic_lands::{mountain, plains};
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_re9_cards::doubling_cube;
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::oracle::mana_helpers::can_cast;
use mtgsim::state::game_state::{GameState, Phase, PhaseType};
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
use mtgsim::types::zones::Zone;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

/// Player 0's precombat main phase, where a sorcery-speed spell can be cast.
fn main_phase() -> GameState {
    let mut game = setup_two_player_game();
    game.set_turn_position(Phase::new(PhaseType::Precombat));
    game.active_player = 0;
    game
}

/// Whether the random agent, from `seed`, completes the cast of `card`.
fn the_agent_casts(game: &GameState, card: mtgsim::types::ids::ObjectId, seed: u64) -> bool {
    let mut game = game.clone();
    let cast = game.cast_spell(0, card, &ManaWindowStop::new(RandomDecisionProvider::seeded(seed)));
    cast.is_ok() && game.objects[&card].zone == Zone::Stack
}

/// Item 214's board: Humility's `{2}{W}{W}` from one Plains, four Mountains
/// and Doubling Cube, where the Cube's doubling is the only second `{W}`. The
/// agent taps everything else first, keeps the white out of the Cube's
/// `{3}`, and doubles `{W}{R}` into the cost. Before, it tapped the Plains,
/// found no source of a second `{W}`, and declined, so the cast was reversed.
#[test]
fn the_agent_pays_humility_through_doubling_cube() {
    let mut game = main_phase();
    put_on_battlefield(&mut game, plains(), 0);
    for _ in 0..4 {
        put_on_battlefield(&mut game, mountain(), 0);
    }
    put_on_battlefield(&mut game, doubling_cube(), 0);
    let card = put_in_hand(&mut game, humility(), 0);
    assert!(can_cast(&game, 0, card).is_ok(), "the check counts the doubling");
    for seed in 0..32 {
        assert!(the_agent_casts(&game, card, seed), "seed {seed}");
    }
}

/// Nine Plains and the Cube against Darksteel Colossus's `{11}`: the Cube is
/// activated once nothing else is offered, so it doubles the six white its
/// `{3}` leaves. Before, the agent counted it among the sources any of which
/// would do, and from some seeds tapped it with three white in the pool.
#[test]
fn the_agent_activates_the_cube_last() {
    let mut game = main_phase();
    for _ in 0..9 {
        put_on_battlefield(&mut game, plains(), 0);
    }
    put_on_battlefield(&mut game, doubling_cube(), 0);
    let card = put_in_hand(&mut game, darksteel_colossus(), 0);
    assert!(can_cast(&game, 0, card).is_ok());
    for seed in 0..32 {
        assert!(the_agent_casts(&game, card, seed), "seed {seed}");
    }
}

/// Lightning Bolt costing `{X}{R}`, registered nowhere: what the gate reads
/// of an X spell before Blaze is in the registry.
fn ember_lesson(x_cannot_be_zero: bool) -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
    let bolt = mtgsim::test_support::lightning_bolt();
    std::sync::Arc::new(mtgsim::objects::card_data::CardData {
        name: "Ember Lesson".to_string(),
        mana_cost: Some(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)])),
        x_cannot_be_zero,
        ..(*bolt).clone()
    })
}

/// The gate asks whether the least legal X is payable (CR 107.3a): `{X}{R}`
/// is offered from one Mountain, at X = 0, and not from none. Before MA-2 a
/// cost with `{X}` had no demand and was never offered.
#[test]
fn a_spell_with_x_is_offered_when_its_least_x_is_payable() {
    let mut game = main_phase();
    let card = put_in_hand(&mut game, ember_lesson(false), 0);
    assert!(can_cast(&game, 0, card).is_err(), "no red, no X");
    put_on_battlefield(&mut game, mountain(), 0);
    assert!(can_cast(&game, 0, card).is_ok(), "{{R}} pays X = 0");
}

/// "X can't be 0" makes the least X 1, so one Mountain does not offer the
/// spell and two do.
#[test]
fn x_cannot_be_zero_asks_the_gate_for_one_x() {
    let mut game = main_phase();
    put_on_battlefield(&mut game, mountain(), 0);
    let card = put_in_hand(&mut game, ember_lesson(true), 0);
    assert!(can_cast(&game, 0, card).is_err(), "{{R}} does not pay X = 1");
    put_on_battlefield(&mut game, mountain(), 0);
    assert!(can_cast(&game, 0, card).is_ok(), "{{1}}{{R}} does");
}

/// The why's lines at `kind`'s question to player 0, about `about`, each as
/// its text and its rule.
fn why_lines(game: &GameState, kind: mtgsim::ui::choice_types::ChoiceKind, about: mtgsim::types::ids::ObjectId) -> Vec<(String, Option<&'static str>)> {
    use mtgsim::ui::why::{why, OpenQuestion, WhyAbout};
    let context = mtgsim::ui::choice_types::ChoiceContext::new(kind);
    let answer = why(game, WhyAbout::Object(about), Some(&OpenQuestion { player: 0, context: &context, options: &[] }));
    assert_eq!(answer.sections[0].heading, "At this question");
    answer.sections[0].lines.iter().map(|line| (line.text.clone(), line.rule)).collect()
}

/// `card`'s legal values of X, cast for its mana cost with nothing else
/// announced.
fn legal_values(game: &GameState, card: mtgsim::types::ids::ObjectId) -> mtgsim::oracle::x_values::LegalXValues {
    use mtgsim::oracle::x_values::{legal_x_values, XAnnouncement};
    let data = &game.objects[&card].card_data;
    let announced = XAnnouncement {
        base_mana_cost: data.mana_cost.as_ref().unwrap(),
        chosen_alt_cost: None,
        chosen_additional_costs: &[],
        x_cannot_be_zero: data.x_cannot_be_zero,
    };
    legal_x_values(game, 0, card, &announced).unwrap()
}

fn choose_x(card: mtgsim::types::ids::ObjectId, values: mtgsim::oracle::x_values::LegalXValues) -> mtgsim::ui::choice_types::ChoiceKind {
    mtgsim::ui::choice_types::ChoiceKind::ChooseXValue { spell_id: card, x_count: 1, values: Box::new(values) }
}

/// The why of X's bound says what the closed form read (setup-architecture
/// §7c, `mana-architecture.md` §3.13): the mana beside the pips, and what
/// each X adds, less what a reduction absorbs.
#[test]
fn the_why_of_x_names_the_bound_and_what_set_it() {
    let mut game = main_phase();
    for _ in 0..4 {
        put_on_battlefield(&mut game, mountain(), 0);
    }
    let card = put_in_hand(&mut game, ember_lesson(false), 0);
    let lines = why_lines(&game, choose_x(card, legal_values(&game, card)), card);
    assert!(lines.contains(&(
        "X above 3: beside {R}, Player 0 can pay at most 3 generic mana, and each X adds {1}.".to_string(),
        Some("601.2h")
    )), "{lines:#?}");

    put_on_battlefield(&mut game, mtgsim::cards::phase_cm_cards::goblin_electromancer(), 0);
    let lines = why_lines(&game, choose_x(card, legal_values(&game, card)), card);
    assert!(lines.contains(&(
        "X above 4: beside {R}, Player 0 can pay at most 3 generic mana, and each X adds {1}, less the {1} its reductions take off."
            .to_string(),
        Some("601.2h")
    )), "{lines:#?}");
}

/// "X can't be 0" says why the least is one.
#[test]
fn the_why_of_x_names_the_printed_least() {
    let mut game = main_phase();
    for _ in 0..2 {
        put_on_battlefield(&mut game, mountain(), 0);
    }
    let card = put_in_hand(&mut game, ember_lesson(true), 0);
    let lines = why_lines(&game, choose_x(card, legal_values(&game, card)), card);
    assert!(lines.contains(&(format!("X = 0: Ember Lesson ({card}) says X can't be 0."), None)), "{lines:#?}");
}

/// The generic split's why names the clamp: two red in the pool, one of them
/// the `{R}` pip's, so at most one red pays the generic part
/// (`ui::ask::generic_split_caps`, the ask's own caps).
#[test]
fn the_why_of_the_generic_split_names_the_clamp() {
    use mtgsim::types::mana::{ManaCost, ManaType};
    let mut game = main_phase();
    let card = put_in_hand(&mut game, ember_lesson(false), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    let kind = mtgsim::ui::choice_types::ChoiceKind::GenericManaAllocation {
        spell_or_ability_id: card,
        mana_cost: ManaCost::build(&[ManaType::Red], 1),
    };
    let lines = why_lines(&game, kind, card);
    assert!(lines.contains(&(
        "More than 1 red toward the generic part: 2 red in the pool, less the cost's {R}.".to_string(),
        Some("601.2h")
    )), "{lines:#?}");
    assert!(!lines.iter().any(|(text, _)| text.contains("green")), "green pays the generic part freely: {lines:#?}");
}
