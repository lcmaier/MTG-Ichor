//! `mode-rules-and-costs`: the rules of CR 700.2 a plain "choose one" leaves
//! off — "another target" over any target (CR 115.4), each mode targeting a
//! different player, a mode chosen more than once (700.2d), another player
//! choosing (700.2e), pawprints (700.2i), a mode chosen before not offered,
//! and a mode's own cost (700.2h, spree) — `choices-architecture.md`,
//! decisions 2 and 3.
//!
//! **Every board is cast, activated or triggered**: the modes and targets are
//! questions asked on that path, and a staged entry cannot check them. Two
//! tests ask the enumeration directly, an earlier instance or the seats
//! given, and one writes Demonic Pact's full record into the history, which
//! only a borrowed Pact made to choose the fourth mode would leave.

use mtgsim::cards::authoring::{
    activated_ability, choose_one_not_chosen_before, dies, enchantment, gain_control_of, gain_life, spell_ability, target_permanent,
    triggered_ability, whenever, you,
};
use mtgsim::cards::fixtures::{divided_envoy, echoing_reckoning, pawprint_parade, tribute_offering, twofold_tithe};
use mtgsim::cards::printings::{arc_trail, demonic_pact, explosive_derailment, grizzly_bears, hill_giant, mind_stone, vindictive_lich};
use mtgsim::engine::layers::compute::compute_characteristics;
use mtgsim::scenario::Scenario;
use mtgsim::state::game_state::{AbilityIdentity, StepType};
use mtgsim::state::history::ModesChosen;
use mtgsim::test_support::{pending_triggers_of, test_ctx};
use mtgsim::types::triggers::{TriggerSubject, TriggerTier};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::engine::object_filter::EarlierTargets;
use mtgsim::engine::targeting::{ChosenTargets, FilterIdentity};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::legality::enumerate_legal_selections_excluding;
use mtgsim::state::game_state::GameState;
use mtgsim::oracle::characteristics::get_effective_abilities;
use mtgsim::oracle::mana_helpers::{can_activate, can_cast, CannotActivate};
use mtgsim::test_support::{fill_library, pass_turn, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_dp};
use mtgsim::types::costs::Cost;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{ChosenBefore, ModeIndex, PlayerSet, SelectionFilter};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayer, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// What a provider was shown: each prompt's kind, who it asked, and the
/// options, kept as `inner` answers it. An offer of one option is answered
/// unasked and never reaches it.
#[derive(Default)]
struct Shown {
    prompts: RefCell<Vec<(PlayerId, ChoiceKind, Vec<ChoiceOption>)>>,
}

impl Shown {
    /// The option lists of every `ChooseModes` prompt, as the log writes them.
    fn mode_offers(&self, game: &GameState) -> Vec<Vec<String>> {
        self.prompts
            .borrow()
            .iter()
            .filter(|(_, kind, _)| matches!(kind, ChoiceKind::ChooseModes { .. }))
            .map(|(_, _, options)| options.iter().map(|option| option.as_logged(game)).collect())
            .collect()
    }

    /// Who each `ChooseModes` prompt asked.
    fn mode_choosers(&self) -> Vec<PlayerId> {
        self.prompts.borrow().iter().filter(|(_, kind, _)| matches!(kind, ChoiceKind::ChooseModes { .. })).map(|(player, _, _)| *player).collect()
    }
}

/// A provider that tells `shown` each prompt and answers through `inner`.
struct Showing<D> {
    inner: D,
    shown: Rc<Shown>,
}

impl<D> Showing<D> {
    fn new(inner: D) -> (Self, Rc<Shown>) {
        let shown = Rc::new(Shown::default());
        (Showing { inner, shown: Rc::clone(&shown) }, shown)
    }
}

impl<D: DecisionProvider> DecisionProvider for Showing<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.shown.prompts.borrow_mut().push((player, context.kind.clone(), options.to_vec()));
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
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

/// The script's next answer: the modes printed at `places`.
fn chooses_modes(answers: &ScriptedDecisionProvider, places: &[u8]) {
    answers.expect_choice(ChoiceKind::ChooseModes { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::modes(places)]);
}

/// The modes the stack object `id` announced, by printed place.
fn chosen_modes(game: &GameState, id: ObjectId) -> Vec<ModeIndex> {
    game.stack_entries[&id].announced.chosen_modes().collect()
}

fn modes(places: &[u8]) -> Vec<ModeIndex> {
    places.iter().map(|&place| ModeIndex::printed(place)).collect()
}

/// `card` cast by player 0 from `mana` floating, and resolved: it enters,
/// and what it triggered waits to be placed.
fn cast_and_resolve(game: &mut GameState, card: Arc<CardData>, mana: &[(ManaType, u64)]) -> ObjectId {
    let id = put_in_hand(game, card, 0);
    for &(mana_type, n) in mana {
        game.players[0].mana_pool.add(mana_type, n);
    }
    game.cast_spell(0, id, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    id
}

/// The script's next answer: `chosen` for an instance of "target".
fn names_targets(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    let kind = ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED };
    answers.expect_choice(kind, chosen);
}


// ---------------------------------------------------------------------------
// "Another target" over any target
// ---------------------------------------------------------------------------

/// "Any other target" excludes a *player* the earlier instance took, which
/// the exclusion could not say while it was a leaf of a permanent's filter:
/// with player 1 named first, the second instance offers player 0 and the
/// creature, never player 1 again.
#[test]
fn any_other_target_is_not_the_player_the_first_instance_took() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let first = ChosenTargets::one(vec![ObjectOrPlayerId::Player(1)]);
    let identity = FilterIdentity::for_text_of(ObjectId::UNASSIGNED, EarlierTargets::Chosen(&first));
    let other = SelectionFilter::Another(Box::new(SelectionFilter::Any));

    let offered = enumerate_legal_selections_excluding(&game, &other, None, 0, identity);

    assert_eq!(offered, [ObjectOrPlayerId::Player(0), ObjectOrPlayerId::Object(bears)]);
}


// ---------------------------------------------------------------------------
// "Target opponent"
// ---------------------------------------------------------------------------

/// At three seats "target opponent" is each opponent and never the
/// controller, whoever's turn it is. Vindictive Lich's three-seat test plays
/// it on a trigger and resolves it.
#[test]
fn target_opponent_is_each_opponent_and_never_the_controller() {
    let game = setup_game(3);
    let opponents = SelectionFilter::Player(PlayerSet::Opponents);
    for controller in 0..3 {
        let others: Vec<ObjectOrPlayerId> = (0..3).filter(|&seat| seat != controller).map(ObjectOrPlayerId::Player).collect();
        assert_eq!(enumerate_legal_selections_excluding(&game, &opponents, None, controller, FilterIdentity::NONE), others);
    }
}

// ---------------------------------------------------------------------------
// CR 700.2d: a mode chosen more than once
// ---------------------------------------------------------------------------

/// ATOM-700.2d-002's board: the damage mode twice, at two creatures, then the
/// draw. The spell resolves as though the damage mode were printed twice in
/// sequence: 2 to the first creature, 2 to the second, then the card.
// COVERS: ATOM-700.2d-002
#[test]
fn a_mode_chosen_twice_resolves_as_that_mode_printed_twice_in_sequence() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 1);
    let first = put_on_battlefield(&mut game, hill_giant(), 1);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, echoing_reckoning(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 3);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 1, 2]);
    names_targets(&answers, vec![ChoiceOption::Object(first)]);
    names_targets(&answers, vec![ChoiceOption::Object(second)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    assert_eq!(chosen_modes(&game, spell), modes(&[1, 1, 2]));
    let mark = game.recorded_events().records().len();
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let order: Vec<String> = game.recorded_events().records()[mark..]
        .iter()
        .filter_map(|record| match &record.event {
            GameEvent::DamageDealt { recipient: ObjectOrPlayer::Object(hit), amount, .. } => {
                Some(format!("{amount} to {}", if hit.id == first { "first" } else { "second" }))
            }
            GameEvent::CardDrawn { .. } => Some("draw".to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["2 to first", "2 to second", "draw"]);
}

/// "Up to four" of two modes that may repeat lists every multiset of up to
/// four, the empty one among them: 15, Moment of Reckoning's count. With no
/// creature to target, the damage mode is in none, and the draw fills them.
#[test]
fn a_repeating_count_lists_every_multiset_up_to_its_most() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 4);
    let spell = put_in_hand(&mut game, echoing_reckoning(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 3);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2, 2, 2, 2]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable");
    assert_eq!(shown.mode_offers(&game), [["no modes", "mode 2", "modes 2, 2", "modes 2, 2, 2", "modes 2, 2, 2, 2"]]);

    let _ = put_on_battlefield(&mut game, hill_giant(), 1);
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), 4, "four draws, as four copies of the mode");
    let again = put_in_hand(&mut game, echoing_reckoning(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 3);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, again, &ManaWindowStop::new(showing)).expect("castable");
    assert_eq!(shown.mode_offers(&game)[0].len(), 15);
}

// ---------------------------------------------------------------------------
// CR 700.2i: pawprints
// ---------------------------------------------------------------------------

/// ATOM-700.2i-001's board: modes of one, two and one pawprints under a
/// budget of three. The second and third together spend three and are
/// offered; all three would spend four and are not.
// COVERS: ATOM-700.2i-001
#[test]
fn pawprints_offer_the_modes_within_the_budget() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 1);
    fill_library(&mut game, 1, 2);
    let spell = put_in_hand(&mut game, pawprint_parade(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 2);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2, 3]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable");

    let offered = &shown.mode_offers(&game)[0];
    assert!(offered.contains(&"modes 2, 3".to_string()));
    assert!(offered.contains(&"modes 1, 1, 3".to_string()), "a mode repeats within the budget");
    assert!(!offered.contains(&"modes 1, 2, 3".to_string()), "four pawprints is past three");
    assert!(!offered.contains(&"modes 2, 2".to_string()));
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), 1);
    assert_eq!(game.players[1].graveyard.len(), 2);
}

// ---------------------------------------------------------------------------
// CR 700.2e: another player chooses
// ---------------------------------------------------------------------------

/// ATOM-700.2e-001's board: "an opponent chooses one", and player 1 is the
/// only opponent, so player 1 is asked the modes and the spell does what they
/// chose for its caster.
// COVERS: ATOM-700.2e-001
#[test]
fn an_opponent_chooses_the_modes_of_a_spell_that_says_so() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 1);
    let _ = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, tribute_offering(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable");

    assert_eq!(shown.mode_choosers(), [1], "the opponent chose");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), 1, "the caster drew");
}

/// With two opponents the caster first picks which one chooses (CR 700.2e's
/// last sentence), and the targets stay the caster's to announce.
#[test]
fn with_two_opponents_the_caster_picks_which_chooses() {
    let mut game = setup_game(3);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, tribute_offering(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);
    let answers = ScriptedDecisionProvider::new();
    answers.expect_choice(ChoiceKind::ChooseOpponent { source: ObjectId::UNASSIGNED }, vec![ChoiceOption::Player(2)]);
    chooses_modes(&answers, &[2]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable");

    assert_eq!(shown.mode_choosers(), [2], "the opponent the caster picked chose");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.battlefield[&bears].damage_marked, 3, "the one creature, the caster's target");
}

// ---------------------------------------------------------------------------
// CR 700.2h: a mode's own cost
// ---------------------------------------------------------------------------

/// ATOM-700.2h-001's board: both modes chosen, so both their costs are in
/// the total and paid as the spell is cast — 2 life and a creature — beside
/// its mana. The modes are asked before the targets, and the costs after.
// COVERS: ATOM-700.2h-001
#[test]
fn each_chosen_modes_own_cost_is_paid_as_the_spell_is_cast() {
    let mut game = setup_two_player_game();
    let fodder = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let victim = put_on_battlefield(&mut game, hill_giant(), 1);
    let spell = put_in_hand(&mut game, twofold_tithe(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 2);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    names_targets(&answers, vec![ChoiceOption::Object(victim)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable with both modes' costs");
    assert_eq!(game.players[0].life_total, 18, "the first mode's 2 life, paid as it was cast");
    assert!(!game.battlefield.contains_key(&fodder), "the second mode's sacrifice");
    let paid = &game.stack_entries[&spell].cost_choices.additional;
    assert_eq!(paid.len(), 2, "each mode's cost is an additional cost the spell was cast with");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].life_total, 17);
    assert!(!game.battlefield.contains_key(&victim));
}

/// A mode whose own cost can't be paid makes no set the spell can be cast
/// with: with no creature to sacrifice and too little life to pay 2 (CR
/// 119.4), neither mode is payable, so the spell is not offered; with the
/// life back, the first mode alone is.
#[test]
fn a_spell_is_castable_only_while_some_set_of_modes_is_payable() {
    let mut game = setup_two_player_game();
    let _ = put_on_battlefield(&mut game, hill_giant(), 1);
    let spell = put_in_hand(&mut game, twofold_tithe(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 2);
    game.players[0].life_total = 1;
    assert!(can_cast(&game, 0, spell).is_err(), "no creature to sacrifice, and 1 life");

    game.players[0].life_total = 20;
    assert_eq!(can_cast(&game, 0, spell), Ok(()), "the first mode's 2 life is payable");
}

// ---------------------------------------------------------------------------
// "Choose one that hasn't been chosen this turn"
// ---------------------------------------------------------------------------

/// An artifact whose `{1}` ability chooses one mode not chosen this turn.
/// The printed "this turn" cards are triggers in `tdc` that no card writes
/// yet: Teval's Judgment makes a Treasure (any-color mana,
/// `any-color-plus-painlands-plus-signets`), and Parapet Thrasher names "that
/// opponent" and plays an exiled card (`multiplayer-rules`,
/// `casting-from-other-zones`). A trigger writes and reads the record as
/// Demonic Pact's tests show; the scope is one function both read.
fn turnwise_lever() -> Arc<CardData> {
    let text = "{1}: Choose one that hasn't been chosen this turn —\n• You gain 1 life.\n• You gain 2 life.";
    CardDataBuilder::new("Turnwise Lever")
        .mana_cost(ManaCost::build(&[], 1))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(activated_ability(
            text,
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            choose_one_not_chosen_before(ChosenBefore::ThisTurn, [gain_life(you(), 1), gain_life(you(), 2)]),
        ))
        .build()
}

/// A mode the ability chose this turn is not offered again this turn: the
/// second activation takes the other mode unasked, a third has none to take,
/// and the next turn offers both again.
#[test]
fn a_mode_chosen_this_turn_is_not_offered_again_until_the_next() {
    let mut game = setup_two_player_game();
    let lever = put_on_battlefield(&mut game, turnwise_lever(), 0);
    let activate = |game: &mut GameState, answers: ScriptedDecisionProvider| {
        game.players[0].mana_pool.add(ManaType::Green, 1);
        game.activate_ability(0, lever, 0, &ManaWindowStop::new(answers)).expect("activatable");
        let ability = *game.stack.last().expect("on the stack");
        let chosen = chosen_modes(game, ability);
        game.resolve_top_of_stack(&test_dp()).expect("it resolves");
        chosen
    };

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1]);
    assert_eq!(activate(&mut game, answers), modes(&[1]));
    assert_eq!(activate(&mut game, test_dp()), modes(&[2]), "the other mode, the one legal set");
    game.players[0].mana_pool.add(ManaType::Green, 1);
    let ability = get_effective_abilities(&game, lever)[0].clone();
    assert_eq!(can_activate(&game, 0, lever, &ability), Err(CannotActivate::NoLegalModes));
    assert_eq!(game.players[0].life_total, 23);
    let keeps_modes_unoffered = |game: &GameState| Scenario::write(game).unwritten.iter().any(|line| line.contains("hasn't been chosen"));
    assert!(keeps_modes_unoffered(&game), "the save has no word for the record yet, and says so");

    pass_turn(&mut game);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    assert_eq!(can_activate(&game, 0, lever, &ability), Ok(()), "a new turn, nothing chosen in it");
    assert!(!keeps_modes_unoffered(&game), "last turn's record keeps nothing unoffered, so it holds no save back");
}

// ---------------------------------------------------------------------------
// "You may choose two. Each mode must target a different player."
// ---------------------------------------------------------------------------

/// Zero modes or two, never one (Shadrix Silverquill's ruling), and at two
/// seats every pair: each mode targets a player, and there are two players.
/// The second mode is offered only the player the first did not take.
#[test]
fn each_mode_must_target_a_different_player() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 2);
    fill_library(&mut game, 1, 2);
    cast_and_resolve(&mut game, divided_envoy(), &[(ManaType::White, 1), (ManaType::Black, 1), (ManaType::Green, 3)]);

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    assert_eq!(shown.mode_offers(&game), [["no modes", "modes 1, 2", "modes 1, 3", "modes 2, 3"]]);
    let recipient_offers: Vec<usize> = shown
        .prompts
        .borrow()
        .iter()
        .filter(|(_, kind, _)| matches!(kind, ChoiceKind::SelectRecipients { .. }))
        .map(|(_, _, options)| options.len())
        .collect();
    assert_eq!(recipient_offers, [2], "the first mode had both players; the second, one, taken unasked");

    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(game.players[1].graveyard.len(), 2, "player 1 milled two");
    assert_eq!((game.players[0].hand.len(), game.players[0].life_total), (1, 19), "player 0 drew and lost 1");
}

/// At three seats a mode targeting a player is not offered the one another
/// mode took: the third mode, after the first named player 2, is offered
/// players 0 and 1.
#[test]
fn a_later_mode_is_not_offered_the_player_an_earlier_mode_took() {
    let mut game = setup_game(3);
    cast_and_resolve(&mut game, divided_envoy(), &[(ManaType::White, 1), (ManaType::Black, 1), (ManaType::Green, 3)]);
    let trigger_source = game.pending_triggers[0].origin.identity().source.id;
    let legal = enumerate_legal_selections_excluding(&game, &SelectionFilter::Player(PlayerSet::Everyone), None, 0, FilterIdentity::NONE);
    assert_eq!(legal.len(), 3, "the control: every seat is a player target");

    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 3]);
    names_targets(&answers, vec![ChoiceOption::Player(2)]);
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    let second_offer: Vec<ChoiceOption> = shown
        .prompts
        .borrow()
        .iter()
        .filter(|(_, kind, _)| matches!(kind, ChoiceKind::SelectRecipients { .. }))
        .nth(1)
        .map(|(_, _, options)| options.clone())
        .expect("the second mode was asked");
    assert_eq!(second_offer, [ChoiceOption::Player(0), ChoiceOption::Player(1)], "player 2 is the first mode's");
    assert_eq!(game.stack_entries[game.stack.last().unwrap()].this_object(), trigger_source);
}

// ---------------------------------------------------------------------------
// The cards
// ---------------------------------------------------------------------------

/// Arc Trail cast from hand: 2 to player 1, and 1 to the creature, the one
/// other target there is once player 1 is taken and player 0 is not chosen.
#[test]
fn arc_trail_deals_its_two_amounts_to_two_different_targets() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, arc_trail(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 2);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable");
    let second: Vec<ChoiceOption> = shown.prompts.borrow()[1].2.clone();
    assert_eq!(second, [ChoiceOption::Player(0), ChoiceOption::Object(bears)], "any other target: player 1 is the first's");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].life_total, 18);
    assert_eq!(game.battlefield[&bears].damage_marked, 1);
}

/// A board for Explosive Derailment: a creature and an artifact of player
/// 1's, and five red mana for player 0, its {R} and both modes' {2}.
fn derailment_board() -> (GameState, ObjectId, ObjectId, ObjectId) {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let stone = put_on_battlefield(&mut game, mind_stone(), 1);
    let spell = put_in_hand(&mut game, explosive_derailment(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 5);
    (game, giant, stone, spell)
}

/// Spree's "one or more": the empty set is never offered, and both modes
/// chosen pay {R} and each mode's {2}, every mana floating spent. The modes
/// are on the spell as it was cast (CR 702.172a).
// RULING: Explosive Derailment #3 - "You must choose at least one of the listed modes and pay its associated additional cost in order to cast a spell with spree."
// RULING: Explosive Derailment #4 - "You choose the modes as you cast the spell with spree. Once modes are chosen, they can't be changed."
// COVERS: ATOM-702.172a-002
#[test]
fn explosive_derailment_needs_a_mode_and_pays_each_chosen_modes_cost() {
    // One creature and one artifact, so each target is taken unasked.
    let (mut game, _, _, spell) = derailment_board();
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    let (showing, shown) = Showing::new(answers);
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable with both modes");

    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 2", "modes 1, 2"]], "no choice of none");
    assert_eq!(chosen_modes(&game, spell), modes(&[1, 2]));
    assert_eq!(game.players[0].mana_pool.total(), 0, "{{R}}, then {{2}} and {{2}}");
}

/// A mode whose target has no legal choice is not offered (CR 700.2a): with
/// no artifact, only the damage mode, taken unasked, and the spell costs {R}
/// and its {2}. Neither offer repeats a mode.
// RULING: Explosive Derailment #5 - "If a mode requires a target, you can select that mode only if there's a legal target available. Ignore the targeting requirements for modes you don't choose."
// RULING: Explosive Derailment #7 - "You can't choose the same mode more than once."
// COVERS-PARTIAL: ATOM-702.172a-001
#[test]
fn explosive_derailment_offers_only_the_modes_with_a_legal_target() {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let spell = put_in_hand(&mut game, explosive_derailment(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 3);
    let (showing, shown) = Showing::new(test_dp());
    game.cast_spell(0, spell, &ManaWindowStop::new(showing)).expect("castable with the one mode it can take");

    assert!(shown.mode_offers(&game).is_empty(), "one legal set, announced unasked");
    assert_eq!(chosen_modes(&game, spell), modes(&[1]));
    assert_eq!(game.players[0].mana_pool.total(), 0, "{{R}} and the first mode's {{2}}, not the second's");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.battlefield[&giant].damage_marked, 4, "the chosen mode applies, and no other");
}

/// An artifact creature of player 1's that gains its controller 1 life as
/// it dies.
fn mourning_engine() -> Arc<CardData> {
    let text = "When this creature dies, you gain 1 life.";
    CardDataBuilder::new("Mourning Engine")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(dies(TriggerSubject::ThisObject), gain_life(you(), 1))))
        .build()
}

/// Both modes, in printed order: the damage, then the destruction. The
/// artifact creature's dies trigger waits until the spell has finished: it
/// is pending, not on the stack, once the spell is gone.
// RULING: Explosive Derailment #6 - "No matter which modes you choose, you always follow the instructions in the order they are written."
// RULING: Explosive Derailment #9 - "No player can cast spells or activate abilities in between the modes of a resolving spell. Any abilities that trigger won't be put onto the stack until the spell is done resolving."
#[test]
fn explosive_derailment_resolves_its_modes_in_printed_order() {
    let (mut game, giant, _, spell) = derailment_board();
    let engine = put_on_battlefield(&mut game, mourning_engine(), 1);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    names_targets(&answers, vec![ChoiceOption::Object(giant)]);
    names_targets(&answers, vec![ChoiceOption::Object(engine)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    let mark = game.recorded_events().records().len();
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let order: Vec<&str> = game.recorded_events().records()[mark..]
        .iter()
        .filter_map(|record| match &record.event {
            GameEvent::DamageDealt { .. } => Some("damage"),
            GameEvent::ZoneChange { from: Zone::Battlefield, to: Zone::Graveyard, .. } => Some("destroyed"),
            _ => None,
        })
        .collect();
    assert_eq!(order.first(), Some(&"damage"));
    assert!(order.contains(&"destroyed"));
    assert!(game.stack.is_empty() && game.pending_triggers.len() == 1, "the dies trigger waited for the spell to finish");
}

/// Its mana value is its mana cost's, 1, whatever its modes' costs added.
// RULING: Explosive Derailment #8 - "The mana value of a spell with spree is determined only by its mana cost (in the upper right corner of the card). It doesn't matter which modes you choose or which additional costs you pay, including any additional costs imposed by other effects."
#[test]
fn explosive_derailment_has_mana_value_one_whatever_was_paid() {
    let (mut game, _, _, spell) = derailment_board();
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1, 2]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    let chars = compute_characteristics(&game, spell).expect("on the stack");
    assert_eq!(chars.mana_cost.as_ref().map(ManaCost::mana_value), Some(1));
}

/// With the artifact gone before it resolves, the spell still resolves
/// against the creature, and with both targets gone it does not resolve at
/// all (CR 608.2b).
// RULING: Explosive Derailment #11 - "If all targets for the chosen modes become illegal before a spell with spree resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
#[test]
fn explosive_derailment_resolves_against_whichever_target_is_left() {
    for both_gone in [false, true] {
        let (mut game, giant, stone, spell) = derailment_board();
        let answers = ScriptedDecisionProvider::new();
        chooses_modes(&answers, &[1, 2]);
        game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
        game.change_zone(stone, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the artifact leaves");
        if both_gone {
            game.change_zone(giant, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the creature leaves");
        }
        game.resolve_top_of_stack(&test_dp()).expect("it resolves or does not");
        let resolved = game.recorded_events().records().iter().any(|record| matches!(record.event, GameEvent::DamageDealt { .. }));
        assert_eq!(resolved, !both_gone, "both gone: {both_gone}");
    }
}

/// Walk `game` to `player`'s next upkeep, where its "beginning of your
/// upkeep" abilities have triggered and wait to be placed: whole turns passed
/// until `player`'s begins, then its untap step ended (CR 502, 503).
fn to_upkeep(game: &mut GameState, player: PlayerId) {
    for seat in 0..game.num_players() {
        if game.players[seat].library.len() < 10 {
            fill_library(game, seat, 20);
        }
    }
    for _ in 0..game.num_players() {
        pass_turn(game);
        if game.active_player == player {
            break;
        }
    }
    assert_eq!(game.active_player, player, "player {player}'s turn began within one rotation");
    assert_eq!(game.phase.step, Some(StepType::Untap), "a turn begins with its untap step");
    let (_, step) = game.advance_turn(&test_dp()).expect("the untap step ends");
    assert_eq!(step, Some(StepType::Upkeep), "the upkeep follows the untap step");
}

/// Demonic Pact's trigger placed at player 0's upkeep with `answers`, and
/// what it was offered.
fn place_pact(game: &mut GameState, answers: ScriptedDecisionProvider) -> (ObjectId, Rc<Shown>) {
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    (*game.stack.last().expect("on the stack"), shown)
}

/// The modes are chosen as the trigger goes on the stack, among the ones
/// left: all four the first time, and once the third is chosen, the others.
// RULING: Demonic Pact #1 - "You choose the mode as the triggered ability goes on the stack. You can choose a mode that requires targets only if there are legal targets available."
#[test]
fn demonic_pact_chooses_as_it_triggers_among_the_modes_left() {
    let mut game = setup_two_player_game();
    let _ = put_on_battlefield(&mut game, demonic_pact(), 0);
    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[3]);
    let hand = game.players[0].hand.len();
    let (ability, shown) = place_pact(&mut game, answers);
    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 2", "mode 3", "mode 4"]]);
    assert_eq!(chosen_modes(&game, ability), modes(&[3]));
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[0].hand.len(), hand + 2);

    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2]);
    let (_, shown) = place_pact(&mut game, answers);
    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 2", "mode 4"]]);
}

/// A mode whose ability did not resolve was still chosen: the damage mode at
/// a creature that leaves first does nothing, and is not offered again.
// RULING: Demonic Pact #2 - "If the ability doesn't resolve (either for having its target become illegal or because a spell or ability counters it), the mode chosen for that instance of the ability still counts as being chosen."
#[test]
fn demonic_pact_counts_a_mode_whose_ability_did_not_resolve() {
    let mut game = setup_two_player_game();
    let _ = put_on_battlefield(&mut game, demonic_pact(), 0);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1]);
    names_targets(&answers, vec![ChoiceOption::Object(giant)]);
    let _ = place_pact(&mut game, answers);
    game.change_zone(giant, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("the creature leaves");
    game.resolve_top_of_stack(&test_dp()).expect("it does not resolve");
    assert_eq!(game.players[0].life_total, 20, "no life gained: it did not resolve");

    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[3]);
    let (_, shown) = place_pact(&mut game, answers);
    assert_eq!(shown.mode_offers(&game), [["mode 2", "mode 3", "mode 4"]]);
}

/// A second Demonic Pact's record is its own: the first chose the third
/// mode, and the second is offered all four.
// RULING: Demonic Pact #3 - "The phrase \"that hasn't been chosen\" refers only to that specific Demonic Pact. If you control one and cast another one, you can choose any mode for the second one the first time its ability triggers."
#[test]
fn a_second_demonic_pact_chooses_any_mode() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, demonic_pact(), 0);
    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[3]);
    let _ = place_pact(&mut game, answers);
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let second = put_on_battlefield(&mut game, demonic_pact(), 0);
    to_upkeep(&mut game, 0);
    let answers = ScriptedDecisionProvider::new();
    let order: Vec<ChoiceOption> = [first, second].iter().flat_map(|&pact| pending_triggers_of(&game, pact)).collect();
    answers.expect_order(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, order);
    chooses_modes(&answers, &[4]);
    chooses_modes(&answers, &[3]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("both are placed"));
    assert_eq!(shown.mode_offers(&game), [vec!["mode 1", "mode 2", "mode 4"], vec!["mode 1", "mode 2", "mode 3", "mode 4"]]);
}

/// A Demonic Pact taken by an opponent keeps what it chose: player 0 chose
/// the second and third modes, and player 1, controlling it at their upkeep,
/// is offered the first and fourth.
// RULING: Demonic Pact #4 - "It doesn't matter who has chosen any particular mode. For example, say you control Demonic Pact and have chosen the first two modes. If an opponent gains control of Demonic Pact, that player can choose only the third or fourth mode."
#[test]
fn demonic_pact_under_a_new_controller_offers_only_the_modes_left() {
    let mut game = setup_two_player_game();
    let pact = put_on_battlefield(&mut game, demonic_pact(), 0);
    for mode in [2, 3] {
        to_upkeep(&mut game, 0);
        let answers = ScriptedDecisionProvider::new();
        chooses_modes(&answers, &[mode]);
        let _ = place_pact(&mut game, answers);
        game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    }

    let text = "Gain control of target enchantment.";
    let theft = CardDataBuilder::new("Invented Theft")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, gain_control_of(target_permanent(enchantment())).until_end_of_game()))
        .build();
    let steal = put_in_hand(&mut game, theft, 1);
    game.players[1].mana_pool.add(ManaType::Blue, 1);
    game.cast_spell(1, steal, &ManaWindowStop::new(test_dp())).expect("one enchantment to take");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    to_upkeep(&mut game, 1);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[1]);
    names_targets(&answers, vec![ChoiceOption::Player(0)]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    assert_eq!(game.stack_entries[game.stack.last().unwrap()].controller, 1, "its new controller's upkeep");
    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 4"]]);
    assert_eq!(game.stack_entries[game.stack.last().unwrap()].this_object(), pact);
}

/// With every mode chosen, none is left, and the ability is removed from the
/// stack with no effect (CR 603.3c). The record holds what a player who took
/// the Pact and was made to choose the fourth mode would leave.
// RULING: Demonic Pact #5 - "In some very unusual situations, you may not be able to choose a mode, either because all modes have previously been chosen or the only remaining modes require targets and there are no legal targets available. In this case, the ability is simply removed from the stack with no effect."
#[test]
fn demonic_pact_with_no_mode_left_is_removed() {
    let mut game = setup_two_player_game();
    let pact = put_on_battlefield(&mut game, demonic_pact(), 0);
    let ability = get_effective_abilities(&game, pact)[0].id;
    let identity = AbilityIdentity { source: game.object_ref(pact).expect("on the battlefield"), ability };
    let record = ModesChosen { turn: game.turn_number, scope: ChosenBefore::WhileItExists, modes: modes(&[1, 2, 3, 4]) };
    game.history.modes_chosen.insert(identity, record);
    to_upkeep(&mut game, 0);

    assert_eq!(game.pending_triggers.len(), 1, "it triggered");
    assert!(!game.place_pending_triggers(&test_dp()).expect("the placement runs"), "nothing was placed");
    assert!(game.stack.is_empty() && !game.player_lost[0]);
}

/// The fourth upkeep: the first three modes chosen, the fourth is the one
/// left, taken unasked, and its controller loses the game.
// RULING: Demonic Pact #6 - "Yes, if the fourth mode is the only one remaining, you must choose it. You read the whole contract, right?"
#[test]
fn demonic_pact_with_only_the_fourth_mode_left_loses_the_game() {
    let mut game = setup_two_player_game();
    let _ = put_on_battlefield(&mut game, demonic_pact(), 0);
    for mode in [1, 2, 3] {
        to_upkeep(&mut game, 0);
        let answers = ScriptedDecisionProvider::new();
        chooses_modes(&answers, &[mode]);
        // "Any target" has both players; "target opponent", one.
        if mode == 1 {
            names_targets(&answers, vec![ChoiceOption::Player(1)]);
        }
        let _ = place_pact(&mut game, answers);
        game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    }

    to_upkeep(&mut game, 0);
    let (ability, shown) = place_pact(&mut game, test_dp());
    assert!(shown.mode_offers(&game).is_empty(), "the one mode left, taken unasked");
    assert_eq!(chosen_modes(&game, ability), modes(&[4]));
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert!(game.player_lost[0]);
}

/// Vindictive Lich dies, and its trigger waits to be placed.
fn lich_dies(seats: usize) -> GameState {
    let mut game = setup_game(seats);
    for seat in 1..seats {
        fill_library(&mut game, seat, 2);
        let _ = put_on_battlefield(&mut game, grizzly_bears(), seat);
    }
    let lich = put_on_battlefield(&mut game, vindictive_lich(), 0);
    game.change_zone(lich, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).expect("it dies");
    assert_eq!(game.pending_triggers.len(), 1, "it triggered");
    game
}

/// At two seats there is one opponent, and each mode must target a different
/// player, so one mode is chosen: the three are offered alone, never two.
// RULING: Vindictive Lich #1 - "You can't choose more modes for Vindictive Lich's triggered ability than you have opponents, but you need to choose at least one target if you can."
#[test]
fn vindictive_lich_at_two_seats_takes_one_mode() {
    let mut game = lich_dies(2);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[3]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 2", "mode 3"]]);
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].life_total, 15);
}

/// At three seats a mode for each opponent is allowed and not required: one
/// mode or two, never three, and two modes take one opponent each.
// RULING: Vindictive Lich #2 - "You don't have to choose a mode for each opponent. For example, you could make one opponent lose 5 life and spare each other opponent. They might not repay your kindness."
#[test]
fn vindictive_lich_at_three_seats_offers_one_mode_or_two() {
    let mut game = lich_dies(3);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, &[2, 3]);
    names_targets(&answers, vec![ChoiceOption::Player(2)]);
    let (showing, shown) = Showing::new(answers);
    assert!(game.place_pending_triggers(&showing).expect("the trigger is placed"));
    assert_eq!(shown.mode_offers(&game), [["mode 1", "mode 2", "mode 3", "modes 1, 2", "modes 1, 3", "modes 2, 3"]]);
    let first_target: Vec<ChoiceOption> = shown
        .prompts
        .borrow()
        .iter()
        .find(|(_, kind, _)| matches!(kind, ChoiceKind::SelectRecipients { .. }))
        .map(|(_, _, options)| options.clone())
        .expect("the second mode's opponent was asked");
    assert_eq!(first_target, [ChoiceOption::Player(1), ChoiceOption::Player(2)], "target opponent: never the Lich's controller");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[2].hand.len(), 0, "player 2 discarded");
    assert_eq!(game.players[1].life_total, 15, "player 1, the one opponent left for the third mode, lost 5");
}
