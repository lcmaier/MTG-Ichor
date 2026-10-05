//! SU-6, the why panel's engine half (`setup-architecture.md` §7c):
//! `engine::layers::explain`, the layer pass recorded for one object, and
//! `ui::why`, its words.

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::cards::{creatures, phase5_pre_cards, phase_ld_cards, phase_le_cards, phase_li_cards, phase_lj_cards};
use mtgsim::engine::layers::{compute_characteristics, explain, AppliedBy, Layer, LayerExplanation, StepResult};
use mtgsim::scenario::Scenario;
use mtgsim::state::decision_log::GameStart;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_graveyard, put_in_library, put_on_battlefield, setup_two_player_game};
use mtgsim::types::effects::CounterType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, SeatMode};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use mtgsim::ui::why::why;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// A sample board's text from `mtgsim/scenarios/`, its lines ended by `\n`
/// whatever the checkout wrote.
fn sample_text(name: &str) -> String {
    let path = format!("{}/scenarios/{name}.scenario", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}")).replace("\r\n", "\n")
}

fn built(text: &str) -> GameState {
    let scenario = Scenario::parse(text).unwrap_or_else(|refusal| panic!("{refusal}"));
    scenario.build(&CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}")).game.state
}

fn on_battlefield(game: &GameState, name: &str) -> ObjectId {
    game.battlefield_ids_ordered()
        .into_iter()
        .find(|id| game.objects[id].card_data.name == name)
        .unwrap_or_else(|| panic!("no {name} on the battlefield"))
}

/// Each step as (layer, the effect's source card or what of the object's
/// own applied, what it did to the object).
fn steps(game: &GameState, explanation: &LayerExplanation) -> Vec<(Layer, String, &'static str)> {
    explanation
        .steps
        .iter()
        .map(|step| {
            let by = match step.by {
                AppliedBy::Effect { source, .. } => game.objects[&source].card_data.name.clone(),
                own => format!("{own:?}"),
            };
            let result = match step.result {
                StepResult::Applied { .. } => "applied",
                StepResult::NotMatched => "not matched",
                StepResult::Gone => "gone",
                StepResult::LockedOut => "locked out",
            };
            (step.layer, by, result)
        })
        .collect()
}

fn explained(game: &GameState, id: ObjectId) -> LayerExplanation {
    let explanation = explain(game, id).expect("the object exists");
    assert_eq!(Some(&explanation.result), compute_characteristics(game, id).as_deref(), "the explanation is the walk");
    explanation
}

fn step(layer: Layer, by: &str, result: &'static str) -> (Layer, String, &'static str) {
    (layer, by.to_string(), result)
}

/// Opalescence arrived first: Humility makes every creature a 1/1 with no
/// abilities, the Angel among them, and Opalescence's two parts miss the
/// Angel, at layer 4 by its filter and at 7b by the set CR 613.6 fixed at 4.
#[test]
fn serra_angel_under_humility_says_what_each_layer_did_and_what_missed_it() {
    let game = built(&sample_text("humility-opalescence"));
    let angel = on_battlefield(&game, "Serra Angel");
    let explanation = explained(&game, angel);
    assert_eq!(
        steps(&game, &explanation),
        [
            step(Layer::Layer4Type, "Opalescence", "not matched"),
            step(Layer::Layer6Ability, "Humility", "applied"),
            step(Layer::Layer7bSetPT, "Opalescence", "locked out"),
            step(Layer::Layer7bSetPT, "Humility", "applied"),
        ]
    );
    let StepResult::Applied { before, after } = &explanation.steps[1].result else { unreachable!() };
    assert!(before.keyword_flags.contains(&KeywordFlag::Flying) && after.keyword_flags.is_empty());
    let StepResult::Applied { before, after } = &explanation.steps[3].result else { unreachable!() };
    assert_eq!((before.power, before.toughness, after.power, after.toughness), (Some(4), Some(4), Some(1), Some(1)));
    let opalescence = on_battlefield(&game, "Opalescence");
    let humility = on_battlefield(&game, "Humility");
    assert_eq!(explanation.steps[0].affected, [humility], "what Opalescence did apply to");
    assert!(explanation.steps.iter().all(|step| step.waited_for.is_empty()), "nothing here depends on anything");
    let _ = opalescence;
}

/// Humility in both orders: Opalescence makes it a creature at layer 4, its
/// own ability strips it at layer 6 and still applies at 7b (CR 613.6), and
/// at 7b the two set power and toughness in timestamp order (CR 613.7), so
/// whichever arrived last decides it.
#[test]
fn humility_is_whichever_set_its_power_and_toughness_last() {
    let opalescence_first = sample_text("humility-opalescence");
    let humility_first = opalescence_first.replace(
        "battlefield: Opalescence | controller 0\nbattlefield: Humility | controller 1",
        "battlefield: Humility | controller 1\nbattlefield: Opalescence | controller 0",
    );
    assert_ne!(humility_first, opalescence_first, "the two lines swapped");
    for (text, last, power) in [(&opalescence_first, "Humility", 1), (&humility_first, "Opalescence", 4)] {
        let game = built(text);
        let humility = on_battlefield(&game, "Humility");
        let explanation = explained(&game, humility);
        let first = if last == "Humility" { "Opalescence" } else { "Humility" };
        assert_eq!(
            steps(&game, &explanation),
            [
                step(Layer::Layer4Type, "Opalescence", "applied"),
                step(Layer::Layer6Ability, "Humility", "applied"),
                step(Layer::Layer7bSetPT, first, "applied"),
                step(Layer::Layer7bSetPT, last, "applied"),
            ],
            "{last} arrived last"
        );
        assert_eq!((explanation.result.power, explanation.result.toughness), (Some(power), Some(power)));
    }
}

/// Urborg arrived first and still applies second, since Blood Moon would
/// take its ability (CR 613.8); by its turn the ability is gone (CR 604.2).
#[test]
fn urborg_waits_for_blood_moon_and_is_gone_by_its_turn() {
    let mut game = setup_two_player_game();
    let urborg = put_on_battlefield(&mut game, phase_li_cards::urborg_tomb_of_yawgmoth(), 0);
    let moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
    let explanation = explained(&game, urborg);
    assert_eq!(
        steps(&game, &explanation),
        [step(Layer::Layer4Type, "Blood Moon", "applied"), step(Layer::Layer4Type, "Urborg, Tomb of Yawgmoth", "gone")]
    );
    assert!(explanation.steps[0].waited_for.is_empty(), "Blood Moon waited for nothing");
    assert_eq!(explanation.steps[1].waited_for, [moon], "Urborg waited for Blood Moon");

    let words = why(&game, urborg);
    let texts: Vec<&str> = words.sections.iter().flat_map(|s| &s.lines).map(|l| l.text.as_str()).collect();
    assert!(texts.contains(&"Its source no longer has the ability, so the effect does not exist."), "{texts:#?}");
    let waited = words.sections.iter().flat_map(|s| &s.lines).find(|l| l.rule == Some("613.8")).expect("the wait is said");
    assert_eq!(waited.names.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [moon]);
}

/// An anthem that does not reach a creature says so, and names the ones it
/// did pump.
#[test]
fn an_anthem_that_misses_a_creature_names_the_creatures_it_pumped() {
    let mut game = setup_two_player_game();
    let anthem = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let giant = put_on_battlefield(&mut game, creatures::hill_giant(), 1);
    let missed = explained(&game, giant);
    assert_eq!(steps(&game, &missed), [step(Layer::Layer7cModifyPT, "Glorious Anthem", "not matched")]);
    assert_eq!(missed.steps[0].affected, [bears]);
    let pumped = explained(&game, bears);
    assert_eq!(steps(&game, &pumped), [step(Layer::Layer7cModifyPT, "Glorious Anthem", "applied")]);
    assert_eq!((pumped.result.power, pumped.result.toughness), (Some(3), Some(3)));
    let words = why(&game, bears);
    let head = &words.sections[0].lines[2];
    assert_eq!(head.rule, Some("613.4c"));
    assert_eq!(head.names.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [anthem], "{head:?}");
    assert!(words.sections[0].lines.iter().any(|l| l.text == "Power and toughness from 2/2 to 3/3"), "{words:#?}");
}

/// A CDA applies at 7a, ahead of everything at its layer (CR 613.3), and
/// +1/+1 counters at 7c (CR 122.1a): Tarmogoyf with two card types in the
/// graveyards and two counters.
#[test]
fn a_cda_and_counters_are_steps_of_the_objects_own() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);
    put_in_graveyard(&mut game, creatures::grizzly_bears(), 0);
    put_in_graveyard(&mut game, mtgsim::test_support::lightning_bolt(), 1);
    let timestamp = game.allocate_timestamp();
    game.battlefield.get_mut(&goyf).expect("on the battlefield").add_counters(CounterType::PlusOnePlusOne, 2, timestamp);
    game.bump_layer_epoch();
    let explanation = explained(&game, goyf);
    assert_eq!(explanation.steps.len(), 2, "{:#?}", steps(&game, &explanation));
    assert!(matches!(explanation.steps[0].by, AppliedBy::Cda { .. }) && explanation.steps[0].layer == Layer::Layer7aCdaPT);
    assert_eq!(explanation.steps[1].by, AppliedBy::PtCounters { kind: CounterType::PlusOnePlusOne, count: 2 });
    assert_eq!((explanation.result.power, explanation.result.toughness), (Some(4), Some(5)));
    let texts: Vec<String> = why(&game, goyf).sections[0].lines.iter().map(|l| l.text.clone()).collect();
    assert!(texts.iter().any(|t| t.ends_with("2 +1/+1 counters (CR 122.1a) · timestamp ".to_string().as_str()) || t.contains("2 +1/+1 counters (CR 122.1a)")), "{texts:#?}");
}

/// A card in a library that a row reaches is walked alone with the pass's
/// notes (`layers-architecture.md` §13e), and a card in a graveyard as a
/// member of the pass: both say the clause made them colorless.
#[test]
fn a_card_off_the_battlefield_says_what_reached_its_zone() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_lj_cards::lattice_colorless_clause(), 0);
    let in_library = put_in_library(&mut game, creatures::grizzly_bears(), 1);
    let in_graveyard = put_in_graveyard(&mut game, creatures::grizzly_bears(), 1);
    for card in [in_library, in_graveyard] {
        let explanation = explained(&game, card);
        assert_eq!(steps(&game, &explanation), [step(Layer::Layer5Color, "Lattice's Colorless Clause", "applied")]);
        assert!(!explanation.seed.colors.is_empty() && explanation.result.colors.is_empty());
    }
}

/// Asks every object's why at every question, then answers as `inner` does.
struct AskingWhy<'a> {
    inner: &'a dyn DecisionProvider,
    asked: std::cell::Cell<usize>,
}

impl AskingWhy<'_> {
    fn ask(&self, game: &GameState) {
        let mut ids: Vec<ObjectId> = game.objects.keys().copied().collect();
        ids.sort_by_key(|id| id.raw());
        for id in ids {
            assert!(!why(game, id).sections.is_empty());
            self.asked.set(self.asked.get() + 1);
        }
    }
}

impl DecisionProvider for AskingWhy<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.ask(game);
        self.inner.pick_n(game, player, context, options, bounds)
    }
    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.ask(game);
        self.inner.pick_number(game, player, context, min, max)
    }
    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.ask(game);
        self.inner.allocate(game, player, context, total, buckets, mins, maxs)
    }
    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.ask(game);
        self.inner.choose_ordering(game, player, context, items)
    }
    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

/// Asking why only reads (§7c's decision 3): a game with every object's why
/// asked at every question plays as the same game asks nothing, and every
/// explanation is the walk's answer, which `explain` asserts in a debug
/// build.
#[test]
fn a_game_with_every_why_asked_is_the_same_game() {
    const TURNS: u32 = 6;
    let pool = CardRegistry::performance_pool();
    let mut deck_rng = StdRng::seed_from_u64(31);
    let decks = (0..2).map(|_| random_deck(&pool, &mut deck_rng, &[], 1, 60).iter().map(|c| c.name.clone()).collect()).collect();
    let start = GameStart::Dealt { seed: 31, config: GameConfig::unrestricted(), decks };
    let mut asked = 0;
    let logs: Vec<Vec<String>> = [false, true]
        .into_iter()
        .map(|asking| {
            let mut built = start.build(&CardRegistry::default_registry()).expect("a dealt start builds");
            built.game_mut().state.record_events();
            let agent = ManaWindowStop::new(RandomDecisionProvider::seeded(31));
            let asking_agent = AskingWhy { inner: &agent, asked: std::cell::Cell::new(0) };
            let seats: &dyn DecisionProvider = if asking { &asking_agent } else { &agent };
            let ran = built.game_mut().until_stopped(|game| {
                game.setup(seats)?;
                while !game.is_over() && game.state.turn_number <= TURNS {
                    game.run_turn(seats)?;
                }
                Ok(())
            });
            ran.expect("the turns play");
            asked += asking_agent.asked.get();
            format_event_log(&built.game().state)
        })
        .collect();
    assert!(logs[0].len() > 50, "the game did something");
    assert!(asked > 1_000, "every object was asked at every question: {asked}");
    assert_eq!(logs[0], logs[1]);
}
