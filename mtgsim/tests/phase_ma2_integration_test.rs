//! MA-2, the legal values of X (`mana-architecture.md` §3.7), and the random
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
fn ember_lesson(x_cant_be_zero: bool) -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
    let bolt = mtgsim::test_support::lightning_bolt();
    std::sync::Arc::new(mtgsim::objects::card_data::CardData {
        name: "Ember Lesson".to_string(),
        mana_cost: Some(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)])),
        x_cant_be_zero,
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
fn x_cant_be_zero_asks_the_gate_for_one_x() {
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
    use mtgsim::engine::cost_determination::total_cost_in_x;
    use mtgsim::oracle::x_values::legal_x_values;
    let data = &game.objects[&card].card_data;
    let total = total_cost_in_x(game, card, data.mana_cost.as_ref().unwrap(), None, &[]);
    legal_x_values(game, 0, &total, data.x_cant_be_zero).unwrap()
}

fn choose_x(card: mtgsim::types::ids::ObjectId, values: mtgsim::oracle::x_values::LegalXValues) -> mtgsim::ui::choice_types::ChoiceKind {
    mtgsim::ui::choice_types::ChoiceKind::ChooseXValue { spell_id: card, values: Box::new(values) }
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
        "X above 3: alongside {R}, Player 0 can pay at most 3 generic mana, and each X adds {1}.".to_string(),
        Some("601.2h")
    )), "{lines:#?}");

    put_on_battlefield(&mut game, mtgsim::cards::phase_cm_cards::goblin_electromancer(), 0);
    let lines = why_lines(&game, choose_x(card, legal_values(&game, card)), card);
    assert!(lines.contains(&(
        "X above 4: alongside {R}, Player 0 can pay at most 3 generic mana, and each X adds {1}, less the {1} its reductions take off."
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

// --- The scenario word `x N` (setup-architecture §5.1) -----------------------

use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::{BuiltScenario, Scenario, SetupDriver};
use mtgsim::state::game::Halt;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider, SeatMode, Stop};

/// Protean Hydra in player 0's hand, `forests` Forests to pay with, and
/// `then`, the setup action.
fn hydra_board(forests: usize, then: &str) -> String {
    format!(
        "turn 3\nactive 0\nstep precombat main\n\nhand 0: Protean Hydra\nlibrary 0: Forest | x8\nlibrary 1: Forest | x8\n\n\
         battlefield: Forest | controller 0, x{forests}\n\n{then}\n"
    )
}

fn built(text: &str) -> BuiltScenario {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{r}"))
}

/// The board after `rounds` priority rounds, or the refusal the line stopped
/// with. The line is cast in the first, and confirmed at the second's first
/// prompt, where the seats then pass and the stack resolves.
fn played_through(text: &str, rounds: usize, as_a_set: bool) -> Result<GameState, String> {
    let BuiltScenario { mut game, setup } = built(text);
    let seats = ScriptedDecisionProvider::new();
    let driver = SetupDriver::new(setup, &seats);
    let set = AsTamiyosSet(&driver);
    let asked: &dyn DecisionProvider = if as_a_set { &set } else { &driver };
    let played = game.until_stopped(|game| {
        for _ in 0..rounds {
            let _ = game.state.run_priority_round(asked);
        }
        Ok(())
    });
    match played {
        Err(Halt::Stopped(stop @ Stop::SetupRefused { .. })) => Err(stop.to_string()),
        Err(other) => panic!("{text}: {other:?}"),
        Ok(()) => Ok(game.state),
    }
}

/// The board with the line cast.
fn played(text: &str) -> Result<GameState, String> {
    played_through(text, 1, false)
}

/// The refusal the line meets, in its cast or once it is confirmed.
fn refused(text: &str, as_a_set: bool) -> String {
    played_through(text, 2, as_a_set).map(|_| ()).unwrap_err()
}

/// The driver, asked X as Tamiyo's set of 0, 1, 4, 6 and 7 where the engine
/// asks an interval: no rule MA-2 builds cuts a gap, so this hands the
/// driver's set form a real cast's question (`mana-architecture.md` §3.7's
/// set form).
struct AsTamiyosSet<'a>(&'a dyn DecisionProvider);

impl DecisionProvider for AsTamiyosSet<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.0.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        use mtgsim::oracle::x_values::{LegalXValues, XOffer};
        let ChoiceKind::ChooseXValue { spell_id, .. } = &context.kind else {
            return self.0.pick_number(game, player, context, min, max);
        };
        let set = vec![0, 1, 4, 6, 7];
        let values = LegalXValues { offered: XOffer::Set(set.clone()), ..LegalXValues::zero_through(7) };
        let kind = ChoiceKind::ChooseXValue { spell_id: *spell_id, values: Box::new(values) };
        let options: Vec<ChoiceOption> = set.iter().map(|&x| ChoiceOption::Number(x)).collect();
        set[self.0.pick_n(game, player, &ChoiceContext::new(kind), &options, (1, 1))[0]]
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
        self.0.allocate(game, player, context, total, buckets, mins, maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.0.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.0.seat_mode(player)
    }
}

fn hydra_x(state: &GameState) -> Option<u64> {
    let top = *state.stack.last().expect("the Hydra on the stack");
    state.stack_entries[&top].x_value
}

/// `x N` answers the X prompt: three Forests pay Protean Hydra's `{X}{G}` up
/// to X = 2, and the line casts it with 2.
#[test]
fn the_word_x_answers_the_x_prompt() {
    let state = played(&hydra_board(3, "then: player 0 casts Protean Hydra | x 2")).unwrap();
    assert_eq!(hydra_x(&state), Some(2));
}

/// An X the engine does not offer stops the line, and so does a line with
/// no `x N` that is asked one.
#[test]
fn the_word_x_is_refused_where_the_engine_offers_no_such_x() {
    let refusal = refused(&hydra_board(3, "then: player 0 casts Protean Hydra | x 3"), false);
    assert!(refusal.contains("its `x 3` is not offered: X can be 0 to 2"), "{refusal}");
    let refusal = refused(&hydra_board(3, "then: player 0 casts Protean Hydra"), false);
    assert!(refusal.contains("and the line has no `x N`"), "{refusal}");
}

/// One Forest pays X = 0 only, so nothing is asked, and a line saying
/// another X is refused once the cast shows what it was cast with.
#[test]
fn an_unasked_x_must_be_the_lines() {
    assert_eq!(hydra_x(&played(&hydra_board(1, "then: player 0 casts Protean Hydra | x 0")).unwrap()), Some(0));
    let refusal = refused(&hydra_board(1, "then: player 0 casts Protean Hydra | x 1"), false);
    assert!(refusal.contains("it says `x 1`, and the spell was cast with X = 0"), "{refusal}");
}

/// The set form: the line's X is answered from the set by value, and an X
/// the set leaves out stops the line.
#[test]
fn the_word_x_answers_a_set_of_x_by_value() {
    let state = played_through(&hydra_board(5, "then: player 0 casts Protean Hydra | x 4"), 1, true).unwrap();
    assert_eq!(hydra_x(&state), Some(4));
    let refusal = refused(&hydra_board(5, "then: player 0 casts Protean Hydra | x 2"), true);
    assert!(refusal.contains("its `x 2` is not among the values of X offered"), "{refusal}");
}

/// `x N` on a card with no X, or on an activation, is refused as the file is
/// read.
#[test]
fn the_word_x_is_refused_where_nothing_has_an_x() {
    let text = "turn 3\nactive 0\nstep precombat main\n\nhand 0: Grizzly Bears\n\nbattlefield: Forest | controller 0, x2\n\n\
                then: player 0 casts Grizzly Bears | x 2\n";
    let refusal = Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).map(|_| ()).unwrap_err();
    assert!(refusal.to_string().contains("Grizzly Bears's mana cost has none"), "{refusal}");
    let text = "turn 3\nactive 0\nstep precombat main\n\nbattlefield: Merfolk Thaumaturgist | controller 0\n\n\
                then: player 0 activates Merfolk Thaumaturgist | x 1\n";
    let refusal = Scenario::parse(text).map(|_| ()).unwrap_err();
    assert!(refusal.to_string().contains("an activated ability's X is not announced yet"), "{refusal}");
}

// --- Blaze, and Protean Hydra at the priority question ----------------------

use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::phase_ma2_cards::blaze;
use mtgsim::cards::phase_tr5b_cards::protean_hydra;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::legality::candidate_priority_actions;
use mtgsim::oracle::x_values::{LegalXValues, XOffer};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::decision::PriorityAction;

/// What the X prompt offered, read off the provider that answered it.
fn offered_x(seat: &ScriptedDecisionProvider) -> XOffer {
    seat.asked()
        .into_iter()
        .find_map(|kind| match kind {
            ChoiceKind::ChooseXValue { values, .. } => Some(values.offered),
            _ => None,
        })
        .expect("X was asked")
}

/// `seat` casts `card` at the priority question, announces `x`, targets
/// `target` if it names one, and taps the first source each of `windows`
/// windows offers.
fn script_the_cast(seat: &ScriptedDecisionProvider, card: ObjectId, x: u64, target: Option<PlayerId>, windows: usize) {
    seat.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(card))]);
    seat.expect_number(choose_x(card, LegalXValues::zero_through(x)), x);
    if let Some(target) = target {
        seat.expect_choice(blaze_target(card), vec![ChoiceOption::Player(target)]);
    }
    for _ in 0..windows {
        seat.expect_pick_n(ChoiceKind::ManaAbilityWindow { spell_or_ability_id: card, remaining_cost: ManaCost::zero() }, vec![0]);
    }
}

/// Blaze's "any target".
fn blaze_target(card: ObjectId) -> ChoiceKind {
    ChoiceKind::SelectRecipients {
        recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        spell_id: card,
    }
}

/// CR 107.3a and 601.2b on the atoms' board: `{X}{R}` with five mana, X
/// announced as 3, kept on the stack entry, and paid as `{3}{R}`, leaving
/// one. The prompt offered 0 to 4, what five mana pays beside `{R}`.
// COVERS: ATOM-107.3a-001, ATOM-601.2b-004
#[test]
fn blaze_is_cast_with_the_x_its_caster_announces() {
    let mut game = main_phase();
    game.players[0].mana_pool.add(ManaType::Red, 5);
    let card = put_in_hand(&mut game, blaze(), 0);
    let seat = ScriptedDecisionProvider::new();
    seat.expect_number(choose_x(card, LegalXValues::zero_through(3)), 3);
    seat.expect_choice(blaze_target(card), vec![ChoiceOption::Player(1)]);
    game.cast_spell(0, card, &seat).expect("castable");
    assert_eq!(offered_x(&seat), XOffer::Interval { at_least: 0, at_most: 4 });
    assert_eq!(game.stack_entries[&card].x_value, Some(3));
    assert_eq!(game.players[0].mana_pool.total(), 1, "{{3}}{{R}} paid of five");
}

/// A staged fixture cannot check the question (`engineering-practices.md`):
/// Blaze cast from hand at a real priority question, from four untapped
/// Mountains, under `ManaWindowStop`. The question offers it, X runs 0 to 3,
/// and X = 3 taps every Mountain.
#[test]
fn blaze_is_offered_and_cast_from_untapped_lands() {
    let mut game = main_phase();
    for _ in 0..4 {
        put_on_battlefield(&mut game, mountain(), 0);
    }
    let card = put_in_hand(&mut game, blaze(), 0);
    assert!(candidate_priority_actions(&game, 0).contains(&PriorityAction::CastSpell(card)));
    let seat = ManaWindowStop::new(ScriptedDecisionProvider::new());
    script_the_cast(seat.inner(), card, 3, Some(1), 4);
    assert_eq!(game.run_priority_round(&seat), Ok(PriorityResult::ActionTaken));
    assert_eq!(offered_x(seat.inner()), XOffer::Interval { at_least: 0, at_most: 3 });
    assert_eq!(game.stack_entries[&card].x_value, Some(3));
    let tapped = game.battlefield_ordered().into_iter().filter(|(_, entry)| entry.tapped).count();
    assert_eq!(tapped, 4, "every Mountain");
}

/// TR-5b's reachability pass cast Protean Hydra 0 times in 400 games: the
/// gate refused its `{X}`. The priority question offers it now, and three
/// Forests cast it with X up to 2.
#[test]
fn protean_hydra_is_offered_at_the_priority_question() {
    let mut game = main_phase();
    for _ in 0..3 {
        put_on_battlefield(&mut game, forest(), 0);
    }
    let card = put_in_hand(&mut game, protean_hydra(), 0);
    assert!(candidate_priority_actions(&game, 0).contains(&PriorityAction::CastSpell(card)));
    let seat = ManaWindowStop::new(ScriptedDecisionProvider::new());
    script_the_cast(seat.inner(), card, 2, None, 3);
    assert_eq!(game.run_priority_round(&seat), Ok(PriorityResult::ActionTaken));
    assert_eq!(offered_x(seat.inner()), XOffer::Interval { at_least: 0, at_most: 2 });
    assert_eq!(game.stack_entries[&card].x_value, Some(2));
}
