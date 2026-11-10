//! The mode picker over the engine's own offers, one board per count rule the
//! engine knows: a card cast, or a trigger placed, and the window's prompt kept
//! as the seat is asked for its modes. Every set of ticks the picker can reach
//! is walked: Done is live exactly where the ticks are an offered choice and
//! answers with that option, a tick is live exactly where some offered choice
//! still holds the ticks with it, and every offered choice is reached.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::Arc;

use devgui::prompt::{Answer, Prompt, Reply};
use devgui::view_model::{Input, ModeControl, ModeRow, Selection, WindowState};
use mtgsim::cards::fixtures::{echoing_reckoning, pawprint_parade};
use mtgsim::cards::printings::{casualties_of_war, cryptic_command, explosive_derailment, grizzly_bears, vindictive_lich};
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{card_of_type, fill_library, lightning_bolt, put_in_hand, put_on_battlefield, setup_game, test_ctx};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::ModeIndex;
use mtgsim::types::ids::PlayerId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// A seat that keeps the window's prompt for the first choice of modes it is
/// asked, and gives the first legal answer to everything.
struct Keeping {
    kept: Rc<RefCell<Option<Prompt>>>,
}

impl DecisionProvider for Keeping {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::ChooseModes { .. }) && self.kept.borrow().is_none() {
            *self.kept.borrow_mut() = Some(Prompt::pick_n(game, player, context, options, bounds));
        }
        (0..bounds.0).collect()
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }

    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        panic!("nothing on these boards divides")
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        (0..items.len()).collect()
    }
}

fn keeping() -> (Keeping, Rc<RefCell<Option<Prompt>>>) {
    let kept = Rc::new(RefCell::new(None));
    (Keeping { kept: Rc::clone(&kept) }, kept)
}

/// The window's prompt for `card`'s modes, cast by player 0 from `mana` floating.
fn cast(game: &mut GameState, card: Arc<CardData>, mana: &[(ManaType, u64)]) -> Prompt {
    let id = put_in_hand(game, card, 0);
    for &(mana_type, n) in mana {
        game.players[0].mana_pool.add(mana_type, n);
    }
    let (seat, kept) = keeping();
    game.cast_spell(0, id, &ManaWindowStop::new(seat)).expect("castable from the mana floating");
    kept.take().expect("its modes were asked")
}

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

fn kinds(rows: &[ModeRow]) -> Vec<(String, bool)> {
    rows.iter().map(|row| (row.label.clone(), matches!(row.control, ModeControl::Count(_)))).collect()
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
    let prompt = cast(&mut game, cryptic_command(), &[(ManaType::Blue, 4)]);
    let walked = walk(&prompt);
    let checkboxes = (1..=4).map(|mode| (format!("Mode {mode}"), false)).collect::<Vec<_>>();
    assert_eq!(kinds(&walked.rows), checkboxes);
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
    let prompt = cast(&mut game, casualties_of_war(), &[(ManaType::Black, 3), (ManaType::Green, 3)]);
    let walked = walk(&prompt);
    assert_eq!(walked.rows.len(), 5);
    assert_eq!(walked.confirmed, 31);
    assert!(walked.reached.contains(&Vec::new()) && !prompt.options.iter().any(|option| option.modes == Some(Vec::new())));
}

/// The Echoing Reckoning fixture's "up to four", a mode as often as the count
/// allows: two counts, and every choice from none to four.
#[test]
fn echoing_reckoning_counts_a_mode_chosen_again() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let prompt = cast(&mut game, echoing_reckoning(), &[(ManaType::Red, 3)]);
    let walked = walk(&prompt);
    assert_eq!(kinds(&walked.rows), [("Mode 1".to_string(), true), ("Mode 2".to_string(), true)]);
    assert_eq!(walked.confirmed, 15, "none, then one to four of two");
    assert!(walked.reached.contains(&modes(&[1, 1, 1, 1])));
}

/// The Pawprint Parade fixture's three pawprints over modes of one, two and
/// one: the two-pawprint mode fits once, so it is a checkbox beside two
/// counts, and with it ticked the others fit once more between them.
#[test]
fn pawprint_parade_ticks_within_its_budget() {
    let mut game = setup_game(2);
    let prompt = cast(&mut game, pawprint_parade(), &[(ManaType::Green, 2)]);
    let walked = walk(&prompt);
    assert_eq!(kinds(&walked.rows), [("Mode 1".to_string(), true), ("Mode 2".to_string(), false), ("Mode 3".to_string(), true)]);
    assert!(walked.reached.contains(&modes(&[2, 3])) && walked.reached.contains(&modes(&[1, 1, 3])));
    assert!(!walked.reached.contains(&modes(&[1, 2, 3])), "four pawprints is past three");
}

/// Explosive Derailment's spree: one mode or both, each with its own cost.
#[test]
fn explosive_derailment_ticks_one_mode_or_both() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 1);
    let prompt = cast(&mut game, explosive_derailment(), &[(ManaType::Red, 5)]);
    let walked = walk(&prompt);
    assert_eq!(kinds(&walked.rows), [("Mode 1".to_string(), false), ("Mode 2".to_string(), false)]);
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
    let prompt = kept.take().expect("its modes were asked");
    let walked = walk(&prompt);
    assert_eq!(walked.rows.len(), 3);
    assert_eq!(walked.confirmed, 6);
    assert!(walked.reached.iter().all(|ticks| ticks.len() <= 2), "no third mode at two opponents");
}
