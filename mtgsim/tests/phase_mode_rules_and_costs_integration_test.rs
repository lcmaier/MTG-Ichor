//! `mode-rules-and-costs`: the rules of CR 700.2 a plain "choose one" leaves
//! off — "another target" over any target (CR 115.4), each mode targeting a
//! different player, a mode chosen more than once (700.2d), another player
//! choosing (700.2e), pawprints (700.2i), a mode chosen before not offered,
//! and a mode's own cost (700.2h, spree) — `choices-architecture.md`,
//! decisions 2 and 3.
//!
//! **Every board is cast, activated or triggered, never staged**: the modes
//! and targets are questions asked on that path, and a staged entry cannot
//! check them.

use mtgsim::cards::authoring::{
    activated_ability, any_other_target, any_target, choose_one_not_chosen_before, deal_damage, gain_life, lose_life, spell_ability, target_opponent,
    you,
};
use mtgsim::cards::fixtures::{divided_envoy, echoing_reckoning, pawprint_parade, tribute_offering};
use mtgsim::cards::printings::{grizzly_bears, hill_giant};
use mtgsim::engine::object_filter::EarlierTargets;
use mtgsim::engine::targeting::{ChosenTargets, FilterIdentity};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::legality::enumerate_legal_selections_excluding;
use mtgsim::state::game_state::GameState;
use mtgsim::oracle::characteristics::get_effective_abilities;
use mtgsim::oracle::mana_helpers::{can_activate, CannotActivate};
use mtgsim::test_support::{fill_library, pass_turn, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_dp};
use mtgsim::types::costs::Cost;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{ChosenBefore, Effect, ModeIndex, PlayerSet, SelectionFilter};
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

/// An instant costing {R} with `text`, which `effect` does.
fn invented_instant(name: &'static str, text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

/// The script's next answer: `chosen` for an instance of "target".
fn names_targets(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    let kind = ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED };
    answers.expect_choice(kind, chosen);
}

fn damage_on(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

// ---------------------------------------------------------------------------
// "Another target" over any target (item 154's first half)
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
    let other = SelectionFilter::other_than(SelectionFilter::Any, &[0]);

    let offered = enumerate_legal_selections_excluding(&game, &other, None, 0, identity);

    assert_eq!(offered, [ObjectOrPlayerId::Player(0), ObjectOrPlayerId::Object(bears)]);
}

/// Cast from hand, "2 damage to any target and 1 damage to any other
/// target": player 1 first, the creature second, each dealt its own amount.
#[test]
fn any_other_target_takes_its_own_damage_after_the_first_names_a_player() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let text = "Invented Fork deals 2 damage to any target and 1 damage to any other target.";
    let fork = invented_instant(
        "Invented Fork",
        text,
        Effect::Sequence(vec![deal_damage(any_target(), 2), deal_damage(any_other_target(0), 1)]),
    );
    let spell = put_in_hand(&mut game, fork, 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert_eq!(game.players[1].life_total, 18);
    assert_eq!(damage_on(&game, bears), 1);
}

// ---------------------------------------------------------------------------
// "Target opponent" (item 345)
// ---------------------------------------------------------------------------

/// At three seats "target opponent" offers both opponents and never the
/// caster, and cast from hand it takes the opponent named: player 2 loses
/// the life, player 1 and the caster none.
#[test]
fn target_opponent_offers_each_opponent_and_never_the_caster() {
    let mut game = setup_game(3);
    let opponents = SelectionFilter::Player(PlayerSet::Opponents);
    assert_eq!(
        enumerate_legal_selections_excluding(&game, &opponents, None, 0, FilterIdentity::NONE),
        [ObjectOrPlayerId::Player(1), ObjectOrPlayerId::Player(2)],
    );

    let text = "Target opponent loses 2 life.";
    let drain = invented_instant("Invented Drain", text, lose_life(target_opponent(), 2));
    let spell = put_in_hand(&mut game, drain, 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(2)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let life: Vec<i64> = game.players.iter().map(|player| player.life_total).collect();
    assert_eq!(life, [20, 20, 18]);
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
// "Choose one that hasn't been chosen this turn"
// ---------------------------------------------------------------------------

/// An artifact whose `{1}` ability chooses one mode not chosen this turn.
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

    pass_turn(&mut game);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    assert_eq!(can_activate(&game, 0, lever, &ability), Ok(()), "a new turn, nothing chosen in it");
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
