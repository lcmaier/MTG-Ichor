//! TR-4d — control, countering, loss, and the records' missing facts
//! (`triggers-architecture.md` §12, TR-4d).
//!
//! 1. Item 253: a countered spell's record carries the spell as it was on the
//!    stack, and a trigger on a spell being countered looks back at it (CR
//!    603.10e). Multani's Presence's "a spell you've cast" reads who cast it.
//! 2. CR 603.9 and 603.10f: a trigger on a player losing the game looks back,
//!    so the loser's permanents, which leave in the same event, are asked as
//!    they were; a trigger the loser would control is kept off the stack (CR
//!    800.4d).
//! 3. Item 131: a mill a replacement keeps in the library still happened
//!    (CR 614.6), and its record says so, with the card as it was milled.

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::authoring::{loses_the_game, triggered_ability, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::cards::phase_tr4d_cards::multanis_presence;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, effect_ctx, fill_library, lightning_bolt, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    put_on_battlefield_under, put_spell_on_stack, setup_game, setup_two_player_game, static_ability, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// The triggers `source` has waiting to be put on the stack.
fn waiting_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|p| p.origin.source() == source).count()
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn hand_size(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
}

/// `primitive` resolving as `player`'s spell aimed at `object`.
fn resolve_at(game: &mut GameState, player: PlayerId, primitive: Primitive, object: ObjectId) {
    let source = put_in_hand(game, counterspell(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(object)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let effect = Effect::Atom(primitive, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
}

/// `player`'s Counterspell countering `spell` (CR 701.6a).
fn counter(game: &mut GameState, spell: ObjectId, player: PlayerId) {
    resolve_at(game, player, Primitive::CounterSpell, spell);
}

/// `card` cast by `player` from exactly `pool`, each instance of "target"
/// answered with the next of `targets`.
fn cast_at(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], targets: &[ChoiceOption]) -> ObjectId {
    let aim = ScriptedDecisionProvider::new();
    for target in targets {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![target.clone()],
        );
    }
    let id = put_in_hand(game, card, player);
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    id
}

/// An enchantment with one triggered ability, `arm` → you gain 2 life.
fn watcher(name: &str, arm: TriggerEvent) -> Arc<CardData> {
    gaining(name, arm, 2)
}

/// An enchantment with one triggered ability, `arm` → you gain `n` life.
fn gaining(name: &str, arm: TriggerEvent, n: u64) -> Arc<CardData> {
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(n)), EffectRecipient::Controller);
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, gain))).build()
}

/// **Fixture.** "Each [object in zones] is black": a Layer 5 effect over the
/// stack, so a spell is black there and its color wherever it goes.
fn painted_black(zones: ZoneSet) -> Arc<CardData> {
    let black = std::collections::HashSet::from([Color::Black]);
    CardDataBuilder::new("Fixture Pall")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeColor(ColorChange::Set(black), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredObjectsIn(ObjectFilter::All, zones),
        )))
        .build()
}

fn white_card() -> Arc<CardData> {
    CardDataBuilder::new("White Card").color(Color::White).card_type(CardType::Instant).build()
}

// ---------------------------------------------------------------------------
// 1. Countering (item 253, CR 603.10e)
// ---------------------------------------------------------------------------

// COVERS: ATOM-603.10e-001
/// "Whenever a spell is countered, you gain 2 life": a Counterspell counters
/// a spell, and the trigger fires.
#[test]
fn a_spell_being_countered_triggers_a_watcher() {
    let mut game = setup_two_player_game();
    let arm = TriggerEvent::IsCountered { spell: None, caster: None };
    let watching = put_on_battlefield(&mut game, watcher("Fixture Counter Watcher", arm), 0);
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 1);

    counter(&mut game, spell, 0);
    assert_eq!(waiting_from(&game, watching), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 22);
}

/// CR 603.10e looks back: the spell is read as it was on the stack. Black
/// there by an effect over the stack, it is a white card in the graveyard by
/// the time anything triggers, and "a black spell" is still countered.
#[test]
fn a_countered_spell_is_read_as_it_was_on_the_stack() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, painted_black(ZoneSet::of(Zone::Stack)), 0);
    let arm = TriggerEvent::IsCountered { spell: Some(ObjectFilter::ByColor(Color::Black)), caster: None };
    let watching = put_on_battlefield(&mut game, watcher("Fixture Black Spell Watcher", arm), 0);
    let spell = put_spell_on_stack(&mut game, white_card(), 1);

    counter(&mut game, spell, 0);
    assert_eq!(game.objects[&spell].zone, Zone::Graveyard);
    assert_eq!(waiting_from(&game, watching), 1, "black as it was countered");
}

/// Multani's Presence: your countered spell draws you a card.
#[test]
fn multanis_presence_draws_when_your_spell_is_countered() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    put_on_battlefield(&mut game, multanis_presence(), 0);
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 0);
    let before = hand_size(&game, 0);

    counter(&mut game, spell, 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 0), before + 1);
}

/// "A spell you've cast": an opponent's countered spell draws you nothing.
#[test]
fn multanis_presence_ignores_an_opponents_countered_spell() {
    let mut game = setup_two_player_game();
    let presence = put_on_battlefield(&mut game, multanis_presence(), 0);
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 1);

    counter(&mut game, spell, 0);
    assert_eq!(waiting_from(&game, presence), 0);
}

// RULING: Multani's Presence #1 - "A spell that doesn't resolve because its targets are illegal isn't countered, so this ability won't trigger. This is a change from previous rules."
#[test]
fn a_spell_whose_targets_are_illegal_is_not_countered() {
    let mut game = setup_two_player_game();
    let presence = put_on_battlefield(&mut game, multanis_presence(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = cast_at(&mut game, 0, lightning_bolt(), &[(ManaType::Red, 1)], &[ChoiceOption::Object(bears)]);
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object: bears }, &effect_ctx(&game, by)).unwrap();

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.objects[&bolt].zone, Zone::Graveyard, "it didn't resolve");
    assert!(game.events.held().iter().all(|r| !matches!(r.event, GameEvent::SpellCountered { .. })));
    assert_eq!(waiting_from(&game, presence), 0);
}

/// Who cast it, not who controls it: a spell of yours another player took
/// control of (CR 108.4), then countered, draws you the card and not them.
#[test]
fn a_stolen_spell_still_counts_as_the_one_you_cast() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    let yours = put_on_battlefield(&mut game, multanis_presence(), 0);
    let theirs = put_on_battlefield(&mut game, multanis_presence(), 1);
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 0);
    resolve_at(&mut game, 1, Primitive::GainControl(Duration::Indefinite), spell);
    assert_eq!(mtgsim::oracle::characteristics::get_effective_controller(&game, spell), Some(1), "player 1 controls it");

    counter(&mut game, spell, 1);
    assert_eq!((waiting_from(&game, yours), waiting_from(&game, theirs)), (1, 0));
}

// ---------------------------------------------------------------------------
// 2. Losing the game (CR 603.9, 603.10f)
// ---------------------------------------------------------------------------

/// "When a player loses the game, you gain 10 life."
fn loss_watcher() -> Arc<CardData> {
    gaining("Fixture Loss Watcher", loses_the_game(Whose::Each), 10)
}

/// `player` at 0 life, and the state-based check that makes them lose (CR
/// 704.5a), leaving any trigger waiting.
fn lose_at_zero_life(game: &mut GameState, player: PlayerId) {
    game.players[player].life_total = 0;
    game.check_state_based_actions(&test_dp()).unwrap();
    assert!(game.player_lost[player], "player {player} lost");
}

// COVERS: ATOM-603.9-001
/// At three seats, so the game goes on after the loss (CR 104.2a): an
/// opponent losing at 0 life triggers "when a player loses the game".
#[test]
fn a_player_losing_the_game_triggers_a_watcher() {
    let mut game = setup_game(3);
    let watching = put_on_battlefield(&mut game, loss_watcher(), 0);

    lose_at_zero_life(&mut game, 2);
    assert_eq!(waiting_from(&game, watching), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 30);
}

/// CR 603.10f looks back: a watcher the loser owned and another player
/// controlled leaves the game with its owner (CR 800.4a) in the event it
/// triggers on, and triggers for the player who controlled it.
#[test]
fn a_watcher_the_loser_owned_triggers_for_its_controller() {
    let mut game = setup_game(4);
    let watching = put_on_battlefield_under(&mut game, loss_watcher(), 3, 0);

    lose_at_zero_life(&mut game, 3);
    assert!(!game.objects.contains_key(&watching), "it left the game with its owner");
    assert_eq!(game.pending_triggers.len(), 1);
    assert_eq!(game.pending_triggers[0].controller, 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 30);
}

/// The loser's own watcher triggers too, looking back, and its trigger is
/// then kept off the stack, since the player who would control it has left
/// (CR 800.4d).
#[test]
fn the_losers_own_watcher_triggers_and_is_kept_off_the_stack() {
    let mut game = setup_game(4);
    put_on_battlefield(&mut game, loss_watcher(), 3);

    lose_at_zero_life(&mut game, 3);
    assert_eq!(game.pending_triggers.len(), 1, "it triggered off its frame");
    assert_eq!(game.pending_triggers[0].controller, 3);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty());
}

// ---------------------------------------------------------------------------
// 3. A mill kept in the library (item 131)
// ---------------------------------------------------------------------------

/// `player` mills `n` cards, as their own spell's instruction.
fn mill(game: &mut GameState, player: PlayerId, n: u64) {
    let source = put_in_graveyard(game, grizzly_bears(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let effect = Effect::Atom(Primitive::Mill(AmountExpr::Fixed(n)), EffectRecipient::Controller);
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
}

/// A zone-change arm on `subject` leaving a library by `act`.
fn from_library(subject: TriggerSubject, act: Option<ZoneChangeAct>, owner: Option<PlayerRef>, multiplicity: Multiplicity) -> TriggerEvent {
    let change = ZoneChangePattern { from: Some(Zone::Library), to: None, act };
    TriggerEvent::ZoneChange { subject, change, by: None, owner, multiplicity }
}

/// Three cards on top of `player`'s library with a Darksteel Colossus in the
/// middle, which shuffles itself back in instead of being milled into the
/// graveyard.
fn colossus_among_three(game: &mut GameState, player: PlayerId) -> ObjectId {
    put_in_library(game, card_of_type("Bottom Rock", CardType::Artifact), player);
    let colossus = put_in_library(game, darksteel_colossus(), player);
    put_in_library(game, card_of_type("Top Rock", CardType::Artifact), player);
    colossus
}

/// The owner's wrinkle: "whenever an opponent mills one or more cards, you
/// gain life equal to the number milled", over a mill of three with a
/// Darksteel Colossus among them. It was milled (CR 614.6, 701.17c), so the
/// number is 3. Before the record it read the two that reached the graveyard,
/// and before a zone change had an amount it refused to read any.
#[test]
fn the_number_milled_counts_a_colossus_shuffled_back() {
    let mut game = setup_two_player_game();
    let arm = from_library(TriggerSubject::Any, Some(ZoneChangeAct::Milled), Some(PlayerRef::Opponent), Multiplicity::OncePerEvent);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringAmount), EffectRecipient::Controller);
    let counter = CardDataBuilder::new("Fixture Mill Counter")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, gain)))
        .build();
    put_on_battlefield(&mut game, counter, 0);
    let colossus = colossus_among_three(&mut game, 1);

    mill(&mut game, 1, 3);
    assert_eq!(game.objects[&colossus].zone, Zone::Library, "shuffled back in");
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 23);
}

/// Zellix, Sanity Flayer's shape, "whenever a player mills one or more
/// creature cards": the Colossus is the only creature milled, and the record
/// carries it as the creature card it was (Jess Dunks, item 131).
#[test]
fn a_creature_card_kept_in_the_library_was_still_milled() {
    let mut game = setup_two_player_game();
    let creature_cards = TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature));
    let arm = from_library(creature_cards, Some(ZoneChangeAct::Milled), None, Multiplicity::OncePerEvent);
    let watching = put_on_battlefield(&mut game, watcher("Fixture Creature Mill Watcher", arm), 0);
    colossus_among_three(&mut game, 1);

    mill(&mut game, 1, 3);
    assert_eq!(waiting_from(&game, watching), 1);
}

/// A move within a zone is no departure (CR 400.7): "whenever a card leaves
/// your library" names no act, so it sees the two cards that left and not the
/// Colossus.
#[test]
fn an_arm_naming_no_act_ignores_a_card_kept_in_its_zone() {
    let mut game = setup_two_player_game();
    let arm = from_library(TriggerSubject::Any, None, Some(PlayerRef::You), Multiplicity::PerOccurrence);
    let watching = put_on_battlefield(&mut game, watcher("Fixture Library Leaver Watcher", arm), 1);
    colossus_among_three(&mut game, 1);

    mill(&mut game, 1, 3);
    assert_eq!(waiting_from(&game, watching), 2);
}
