//! The mode picker over the engine's own offers, a board for each of
//! `ModalCount`'s seven arms (exactly, a range, one or more, none or n, X,
//! pawprints, a raised count) and for the node's rules (a repeat, a mode's own
//! cost, a different player each): a card cast, or a trigger placed, and the
//! window's prompt kept as the seat is asked for its modes. Every set of ticks the picker can reach
//! is walked: Done is live exactly where the ticks are an offered choice and
//! answers with that option, a tick is live exactly where some offered choice
//! still holds the ticks with it, and every offered choice is reached.

#[path = "support/modes_asked.rs"]
#[allow(dead_code, reason = "the walk starts from the prompt alone; the snapshot is the pictures'")]
mod modes_asked;

use std::collections::BTreeSet;

use devgui::prompt::{Answer, Prompt, Reply};
use devgui::view_model::{Input, ModeControl, ModeRow, Selection, WindowState};
use mtgsim::cards::fixtures::{divided_envoy, echoing_reckoning, kindled_option, tally_of_choices};
use mtgsim::cards::printings::{casualties_of_war, cryptic_command, explosive_derailment, grizzly_bears, vindictive_lich};
use mtgsim::test_support::{card_of_type, fill_library, lightning_bolt, put_in_hand, put_on_battlefield, setup_game, test_ctx};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::ModeIndex;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use modes_asked::{cast, cast_with_x, keeping};

fn modes(places: &[u8]) -> Vec<ModeIndex> {
    places.iter().map(|&place| ModeIndex::printed(place)).collect()
}

/// Whether `choice` has every mode of `ticks`, as often: each tick crossed off
/// a copy of the choice.
fn contains(choice: &[ModeIndex], ticks: &[ModeIndex]) -> bool {
    let mut left = choice.to_vec();
    ticks.iter().all(|tick| left.iter().position(|mode| mode == tick).map(|at| left.remove(at)).is_some())
}

/// What the walk found: each set of ticks reached, and each at which Done answered.
struct Walked {
    reached: BTreeSet<Vec<ModeIndex>>,
    confirmed: usize,
    /// The rows at no ticks.
    rows: Vec<ModeRow>,
}

/// Every set of ticks the picker offers a way to, from none.
fn walk(prompt: &Prompt) -> Walked {
    let offered: Vec<Vec<ModeIndex>> = prompt.options.iter().map(|option| option.modes.clone().expect("a choice of modes")).collect();
    let start = WindowState { selection: Some(Selection::start(prompt)), prompt: Some(prompt.clone()), ..WindowState::default() };
    let rows = start.prompt_view().expect("the question is open").modes;
    let mut walked = Walked { reached: BTreeSet::new(), confirmed: 0, rows };
    let mut to_visit = vec![start];
    while let Some(state) = to_visit.pop() {
        let Some(Selection::Modes(ticks)) = state.selection.clone() else { panic!("not the mode picker: {:?}", state.selection) };
        if !walked.reached.insert(ticks.clone()) {
            continue;
        }
        let view = state.prompt_view().expect("the question is open");
        let at = offered.iter().position(|choice| choice.len() == ticks.len() && contains(choice, &ticks));
        assert_eq!(view.done.as_ref().map(|done| done.live), Some(at.is_some()), "Done at {ticks:?} among {offered:?}");
        if let Some(at) = at {
            assert_eq!(state.clone().input(Input::Done), Some(Reply::Answer(Answer::Picks(vec![at]))), "the option holding {ticks:?}");
            walked.confirmed += 1;
        }
        for row in &view.modes {
            let mut more = ticks.clone();
            more.push(row.mode);
            let holds_one_more = offered.iter().any(|choice| contains(choice, &more));
            let (can_tick, can_untick) = match &row.control {
                ModeControl::Checkbox { ticked: true, live, .. } => (false, *live),
                ModeControl::Checkbox { ticked: false, live, .. } => (*live, false),
                ModeControl::Count(amount) => (amount.can_raise, amount.can_lower),
            };
            let ticked = ticks.contains(&row.mode);
            assert_eq!(can_tick, holds_one_more && !matches!(row.control, ModeControl::Checkbox { ticked: true, .. }), "{} at {ticks:?}", row.label);
            assert_eq!(can_untick, ticked, "{} at {ticks:?}", row.label);
            if can_tick {
                let mut next = state.clone();
                assert_eq!(next.input(Input::TickMode(row.mode)), None);
                to_visit.push(next);
            }
        }
    }
    assert_eq!(walked.confirmed, offered.len(), "every offered choice reached: {offered:?}");
    walked
}

/// Each row's label, and whether it is a count.
fn kinds(rows: &[ModeRow]) -> Vec<(&str, bool)> {
    rows.iter().map(|row| (row.label.as_str(), matches!(row.control, ModeControl::Count(_)))).collect()
}

/// Cryptic Command's "choose two", each mode with a choice: four checkboxes,
/// the six pairs, and nothing at one tick confirms.
#[test]
fn cryptic_command_ticks_two_of_four() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = put_in_hand(&mut game, lightning_bolt(), 1);
    game.players[1].mana_pool.add(ManaType::Red, 1);
    game.cast_spell(1, bolt, &ManaWindowStop::new(keeping().0)).expect("a spell to counter");
    let prompt = cast(&mut game, cryptic_command(), &[(ManaType::Blue, 4)]).prompt;
    let walked = walk(&prompt);
    let checkboxes = [
        ("1. Counter target spell.", false),
        ("2. Return target permanent to its owner's hand.", false),
        ("3. Tap all creatures your opponents control.", false),
        ("4. Draw a card.", false),
    ];
    assert_eq!(kinds(&walked.rows), checkboxes, "each mode by its place and its line of the text");
    assert_eq!(walked.confirmed, 6);
    assert!(walked.reached.iter().all(|ticks| ticks.len() <= 2), "no third tick once two are in");
}

/// Casualties of War's "one or more" of five: every one of the 31 choices,
/// and none at no ticks.
#[test]
fn casualties_of_war_ticks_one_or_more_of_five() {
    let mut game = setup_game(2);
    for card_type in [CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker] {
        put_on_battlefield(&mut game, card_of_type("Target", card_type), 1);
    }
    let prompt = cast(&mut game, casualties_of_war(), &[(ManaType::Black, 3), (ManaType::Green, 3)]).prompt;
    let walked = walk(&prompt);
    assert_eq!(walked.rows.len(), 5);
    assert_eq!(walked.confirmed, 31);
    assert!(!prompt.options.iter().any(|option| option.modes == Some(Vec::new())), "one or more, so Done waits for a tick");
}

/// The Echoing Reckoning fixture's "up to four", a mode as often as the count
/// allows: two counts, and every choice from none to four.
#[test]
fn echoing_reckoning_counts_a_mode_chosen_again() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let prompt = cast(&mut game, echoing_reckoning(), &[(ManaType::Red, 3)]).prompt;
    let walked = walk(&prompt);
    assert_eq!(kinds(&walked.rows), [("1. This spell deals 2 damage to target creature.", true), ("2. Draw a card.", true)]);
    assert_eq!(walked.confirmed, 15, "none, then one to four of two");
    assert!(walked.reached.contains(&modes(&[1, 1, 1, 1])));
}

/// The Pawprint Parade fixture's three pawprints over modes of one, two and
/// one: the two-pawprint mode fits once, so it is a checkbox beside two
/// counts, and with it ticked the others fit once more between them.
#[test]
fn pawprint_parade_ticks_within_its_budget() {
    let walked = walk(&modes_asked::pawprint_parade().prompt);
    let rows = [("1. {P} — You gain 1 life.", true), ("2. {P}{P} — Draw a card.", false), ("3. {P} — Target player mills two cards.", true)];
    assert_eq!(kinds(&walked.rows), rows, "a mode's pawprints are in its words");
    assert!(walked.reached.contains(&modes(&[2, 3])) && walked.reached.contains(&modes(&[1, 1, 3])));
    assert!(!walked.reached.contains(&modes(&[1, 2, 3])), "four pawprints is past three");
}

/// Explosive Derailment's spree: one mode or both, each with its own cost.
#[test]
fn explosive_derailment_ticks_one_mode_or_both() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 1);
    let prompt = cast(&mut game, explosive_derailment(), &[(ManaType::Red, 5)]).prompt;
    let walked = walk(&prompt);
    let rows = [("1. + {2} — Explosive Derailment deals 4 damage to target creature.", false), ("2. + {2} — Destroy target artifact.", false)];
    assert_eq!(kinds(&walked.rows), rows, "a mode's own cost is in its words");
    assert_eq!(walked.confirmed, 3);
}

/// Vindictive Lich's trigger at three seats, each mode a different opponent:
/// one mode or two, so with two ticked the third is greyed.
#[test]
fn vindictive_lich_at_three_seats_ticks_one_mode_or_two() {
    let mut game = setup_game(3);
    for seat in 1..3 {
        fill_library(&mut game, seat, 2);
        put_on_battlefield(&mut game, grizzly_bears(), seat);
    }
    let lich = put_on_battlefield(&mut game, vindictive_lich(), 0);
    game.change_zone(lich, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).expect("it dies");
    let (seat, kept) = keeping();
    assert!(game.place_pending_triggers(&seat).expect("the trigger is placed"));
    let prompt = kept.take().expect("its modes were asked").prompt;
    let walked = walk(&prompt);
    assert_eq!(walked.rows.len(), 3);
    assert_eq!(walked.confirmed, 6);
    assert!(walked.reached.iter().all(|ticks| ticks.len() <= 2), "no third mode at two opponents");
}

/// The Divided Envoy fixture's "you may choose two": Done is live at no
/// ticks, dead at one, and live again at two.
#[test]
fn divided_envoy_ticks_none_or_two() {
    let mut game = setup_game(2);
    let envoy = put_in_hand(&mut game, divided_envoy(), 0);
    for (mana_type, n) in [(ManaType::White, 1), (ManaType::Black, 1), (ManaType::Green, 3)] {
        game.players[0].mana_pool.add(mana_type, n);
    }
    game.cast_spell(0, envoy, &ManaWindowStop::new(keeping().0)).expect("castable from the mana floating");
    game.resolve_top_of_stack(&keeping().0).expect("it enters");
    let (seat, kept) = keeping();
    // The seat's first legal answer is no modes, so CR 603.3c removes the trigger.
    assert!(!game.place_pending_triggers(&seat).expect("its trigger is asked"));
    let walked = walk(&kept.take().expect("its modes were asked").prompt);
    assert_eq!(walked.confirmed, 4, "none, and the three pairs");
    assert!(walked.reached.contains(&modes(&[2])), "one tick is reached, and the walk found Done dead there");
}

/// The Tally of Choices fixture's "choose X", X announced first as 2: Done
/// waits for two ticks, and a third is never live.
#[test]
fn tally_of_choices_ticks_x_modes() {
    let mut game = setup_game(2);
    let prompt = cast_with_x(&mut game, tally_of_choices(), &[(ManaType::Blue, 3)], Some(2)).prompt;
    let walked = walk(&prompt);
    assert_eq!(walked.confirmed, 3);
    assert!(walked.reached.iter().all(|ticks| ticks.len() <= 2));
}

/// The Kindled Option fixture's raised count: with an artifact, one mode or
/// both.
#[test]
fn kindled_option_ticks_one_or_both_with_an_artifact() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let walked = walk(&cast(&mut game, kindled_option(), &[(ManaType::Red, 2)]).prompt);
    assert_eq!(walked.confirmed, 3);
    assert!(walked.reached.contains(&modes(&[1, 2])));
}
