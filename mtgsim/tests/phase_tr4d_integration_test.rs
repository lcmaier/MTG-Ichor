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
//! 4. Item 258: an effect finds what it moved in whatever public zone the
//!    move went to (CR 400.7j, 701.17c), and never in a hand or a library
//!    (CR 400.7e).
//! 5. Item 13: a change of control is announced by a sweep and looks back
//!    (CR 603.10d), so "you" is the player who lost control. Leaving the
//!    battlefield is losing control (Khârn the Betrayer's first ruling), a
//!    steal ending at cleanup triggers in a second cleanup step (CR 514.3a),
//!    and a player leaving takes back what they stole (CR 800.4a).
//! 6. Ray of Command's three rulings: its delayed trigger on losing control
//!    taps the creature, at cleanup or when a third player takes it.

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::authoring::{
    an_opponent_gains_control_of, at_beginning_of, loses_the_game, triggered_ability, whenever, you_lose_control_of, Whose,
};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_rb_cards::leyline_of_the_void;
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::cards::phase_tr4d_cards::{multanis_presence, ray_of_command};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer, ObjectSet};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    card_of_type, effect_ctx, fill_library, lightning_bolt, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    put_on_battlefield_under, put_spell_on_stack, registered, setup_game, setup_two_player_game, static_ability,
    test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    Antecedent, AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{new_ability_id, AbilityId, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{
    DelayedDuration, DelayedTriggerTemplate, DelayedTurn, EventRole, Multiplicity, TriggerEvent, TriggerSubject,
};
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
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(object)]),
        replaced_event: None,
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
    resolve_at(&mut game, 1, Primitive::GainControl { to: PlayerRef::You, duration: Duration::Indefinite }, spell);
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
    let ctx = ResolutionContext::untargeted(source, player, GeneratedBy::Spell);
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
    let arm = from_library(TriggerSubject::Filter(ObjectFilter::All), Some(ZoneChangeAct::Milled), Some(PlayerRef::Opponent), Multiplicity::OncePerEvent);
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
    let arm = from_library(TriggerSubject::Filter(ObjectFilter::All), None, Some(PlayerRef::You), Multiplicity::PerOccurrence);
    let watching = put_on_battlefield(&mut game, watcher("Fixture Library Leaver Watcher", arm), 1);
    colossus_among_three(&mut game, 1);

    mill(&mut game, 1, 3);
    assert_eq!(waiting_from(&game, watching), 2);
}

// ---------------------------------------------------------------------------
// 4. Finding what an effect moved (item 258)
// ---------------------------------------------------------------------------

/// An artifact whose `{T}` ability resolves `effect`.
fn totem(name: &str, effect: Effect) -> Arc<CardData> {
    let ability = AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: vec![Cost::TapSelf],
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    CardDataBuilder::new(name).card_type(CardType::Artifact).ability(ability).build()
}

/// "[Instruction]. At the beginning of the next end step, return that card to
/// its owner's hand": a delayed trigger referring to what the instruction
/// moved (CR 603.7c).
fn then_return_it_later(instruction: Effect) -> Effect {
    let def = Arc::new(whenever(
        at_beginning_of(StepType::End, Whose::Each),
        Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Object(Antecedent::Referred)),
    ));
    let template = DelayedTriggerTemplate { def, duration: DelayedDuration::Once, turn: DelayedTurn::NoneNamed, rules_text: "".into() };
    Effect::Sequence(vec![
        Effect::Remember(Box::new(instruction)),
        Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller),
    ])
}

/// What the one waiting delayed trigger refers to.
fn referred(game: &GameState) -> Vec<ObjectId> {
    assert_eq!(game.delayed_triggers.len(), 1);
    game.delayed_triggers[0].referred.objects.iter().map(|r| r.object.id).collect()
}

fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId) {
    game.activate_ability(player, source, 0, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

/// CR 701.17c: "an effect that refers to a milled card can find that card in
/// the zone it moved to from the library, as long as that zone is a public
/// zone". Leyline of the Void exiled both, and both are "that card".
#[test]
fn a_milled_card_a_replacement_exiled_is_still_found() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 1);
    let below = put_in_library(&mut game, grizzly_bears(), 0);
    let above = put_in_library(&mut game, grizzly_bears(), 0);
    let mill_two = Effect::Atom(Primitive::Mill(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    let source = put_on_battlefield(&mut game, totem("Fixture Mill Totem", then_return_it_later(mill_two)), 0);

    activate_and_resolve(&mut game, 0, source);
    assert_eq!((game.objects[&above].zone, game.objects[&below].zone), (Zone::Exile, Zone::Exile));
    assert_eq!(referred(&game), vec![above, below]);
}

/// A Darksteel Colossus milled and shuffled back is in a hidden zone, where
/// no effect can find it (CR 701.17c), though it was milled (item 131).
#[test]
fn a_milled_card_shuffled_back_is_not_found() {
    let mut game = setup_two_player_game();
    let colossus = colossus_among_three(&mut game, 0);
    let mill_three = Effect::Atom(Primitive::Mill(AmountExpr::Fixed(3)), EffectRecipient::Controller);
    let source = put_on_battlefield(&mut game, totem("Fixture Mill Totem", then_return_it_later(mill_three)), 0);

    activate_and_resolve(&mut game, 0, source);
    let found = referred(&game);
    assert_eq!(found.len(), 2);
    assert!(!found.contains(&colossus));
}

/// CR 400.7j finds a moved object only in a public zone: a creature returned
/// to its owner's hand is no "that card".
#[test]
fn a_card_put_into_a_hand_is_not_found() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let bounce = Effect::Atom(
        Primitive::ReturnToHand,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Creature)), TargetCount::Exactly(1)),
    );
    let source = put_on_battlefield(&mut game, totem("Fixture Bounce Totem", then_return_it_later(bounce)), 0);

    activate_and_resolve(&mut game, 0, source);
    assert_eq!(game.objects[&bears].zone, Zone::Hand);
    assert!(referred(&game).is_empty());
}

/// CR 400.7e: a zone change's trigger finds the object it moved only in a
/// public zone. "Whenever a creature is put into your hand from the
/// battlefield, exile it" exiles nothing.
#[test]
fn a_trigger_does_not_find_what_moved_into_a_hand() {
    let mut game = setup_two_player_game();
    let change = ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Hand), act: None };
    let arm = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
        change,
        by: None,
        owner: Some(PlayerRef::You),
        multiplicity: Multiplicity::PerOccurrence,
    };
    let exile_it = Effect::Atom(Primitive::Exile, EffectRecipient::Object(Antecedent::Triggering(EventRole::Object)));
    let card = CardDataBuilder::new("Fixture Bounce Watcher")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, exile_it)))
        .build();
    let watching = put_on_battlefield(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    let by = mtgsim::types::ids::new_object_id();
    let bounce = GameAction::ZoneChange { object: bears, from: Zone::Battlefield, to: Zone::Hand, act: ZoneChangeAct::Put };
    game.execute_action(bounce, &effect_ctx(&game, by)).unwrap();
    assert_eq!(waiting_from(&game, watching), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(game.objects[&bears].zone, Zone::Hand, "not found there");
}

// ---------------------------------------------------------------------------
// 5. A change of control (item 13, CR 603.10d)
// ---------------------------------------------------------------------------

/// **Fixture**, Khârn the Betrayer's Sigil of Corruption on a creature: "When
/// you lose control of this creature, draw two cards."
fn kharn_shaped() -> Arc<CardData> {
    let draw_two = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    CardDataBuilder::new("Fixture Betrayer")
        .card_type(CardType::Creature)
        .power_toughness(5, 5)
        .ability(triggered_ability("", whenever(you_lose_control_of(TriggerSubject::Object(Antecedent::ThisObject)), draw_two)))
        .build()
}

/// Act of Treason's resolution, `thief` taking `victim` until end of turn.
fn steal(game: &mut GameState, thief: PlayerId, victim: ObjectId) {
    let card = act_of_treason();
    // Where the sorcery is as it finishes resolving, out of every hand.
    let source = put_in_graveyard(game, card.clone(), thief);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: thief,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(victim)]),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
}

fn controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    mtgsim::oracle::characteristics::get_effective_controller(game, id)
}

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

/// Stolen by Act of Treason, the creature's owner lost control of it, and the
/// trigger looks back to the board before: "you" is the owner, who draws.
// RULING: Khârn the Betrayer #1 - another player gaining control of it is losing control.
#[test]
fn a_stolen_creature_triggers_for_the_player_who_lost_it() {
    let mut game = stocked(2);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 1);
    let (thiefs, owners) = (hand_size(&game, 0), hand_size(&game, 1));

    steal(&mut game, 0, betrayer);
    assert_eq!(controller(&game, betrayer), Some(0));
    assert_eq!(waiting_from(&game, betrayer), 1);
    assert_eq!(game.pending_triggers[0].controller, 1, "CR 603.3a, off the frame from before");
    resolve_all(&mut game, &test_dp());
    assert_eq!((hand_size(&game, 0), hand_size(&game, 1)), (thiefs, owners + 2));
}

/// CR 603.4's instant: Act of Treason's change is an event before its "untap
/// that creature" is, announced at the head of the untap's batch.
#[test]
fn the_change_is_announced_before_the_next_instruction() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.battlefield.get_mut(&bears).unwrap().tapped = true;
    let before = game.recorded_events().len();

    steal(&mut game, 0, bears);
    let order: Vec<&str> = game
        .recorded_events()
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ControlChanged { to: 0, lki } if lki.object().id == bears && lki.characteristics().controller == 1 => Some("control"),
            GameEvent::Untapped { object } if object.id == bears => Some("untap"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["control", "untap"]);
}

/// The steal ends at cleanup (CR 514.2), and the creature going back is a
/// change the sweep announces though no control-changing effect is left. The
/// thief lost control, so the thief draws, in the cleanup step, which grants
/// priority and repeats (CR 514.3a; Ogre Geargrabber's fourth ruling).
// RULING: Ogre Geargrabber #4 - the steal ends at cleanup, its trigger gets priority, and another cleanup step begins.
#[test]
fn a_steal_ending_at_cleanup_triggers_for_the_thief_in_a_second_cleanup() {
    let mut state = stocked(2);
    let betrayer = put_on_battlefield(&mut state, kharn_shaped(), 1);
    steal(&mut state, 0, betrayer);
    resolve_all(&mut state, &test_dp());
    state.set_turn_position(Phase::new(PhaseType::Postcombat));
    let (thiefs, before) = (hand_size(&state, 0), state.recorded_events().len());
    let mut game = Game { state, config: GameConfig::unrestricted() };

    game.resume_turn_at_priority(&test_dp()).unwrap();
    let state = &game.state;
    assert_eq!(controller(state, betrayer), Some(1), "it went back at cleanup");
    let records = state.recorded_events();
    let cleanups = records.records_from(before).iter().filter(|r| matches!(r.event, GameEvent::StepBegin { step: StepType::Cleanup, .. })).count();
    assert_eq!(cleanups, 2, "the trigger made the step grant priority, and another began");
    assert_eq!(hand_size(state, 0), thiefs + 2);
}

/// "You lose control of Khârn the Betrayer when it leaves the battlefield or
/// another player gains control of it" (its first ruling): dying is losing
/// control, and its controller as it left draws.
// RULING: Khârn the Betrayer #1 - leaving the battlefield is losing control of it.
// RULING: Gustha's Scepter #1 - its lose-control trigger fires as it leaves the battlefield.
#[test]
fn leaving_the_battlefield_is_losing_control() {
    let mut game = stocked(2);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 0);
    let before = hand_size(&game, 0);
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object: betrayer }, &effect_ctx(&game, by)).unwrap();

    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 0), before + 2);
}

/// At four seats, the thief losing ends the steal (CR 800.4a), and the
/// creature goes back to its owner: the thief lost control, and the thief has
/// left, so the trigger is kept off the stack (CR 800.4d) and nobody draws.
#[test]
fn the_thief_leaving_returns_control_and_drops_the_thiefs_trigger() {
    let mut game = stocked(4);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 1);
    steal(&mut game, 0, betrayer);
    resolve_all(&mut game, &test_dp());
    let owners = hand_size(&game, 1);

    game.players[0].life_total = 0;
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(game.player_lost[0]);
    assert_eq!(controller(&game, betrayer), Some(1), "back with its owner");
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty(), "the thief's trigger was dropped");
    assert_eq!(hand_size(&game, 1), owners, "and not given to the owner");
}

// COVERS: ATOM-603.10d-001
/// "Whenever an opponent gains control of a permanent you own, draw a card":
/// an opponent steals one of your creatures, and the trigger looks back to
/// see you had the ability before the change.
#[test]
fn an_opponent_gaining_control_of_a_permanent_you_own_triggers() {
    let mut game = stocked(2);
    let arm = TriggerEvent::ControlChanges {
        subject: TriggerSubject::Filter(ObjectFilter::ByOwner(PlayerRef::You)),
        from: None,
        to: Some(PlayerRef::Opponent),
    };
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let card = CardDataBuilder::new("Fixture Owner's Watch").card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, draw))).build();
    let watching = put_on_battlefield(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    steal(&mut game, 1, bears);
    assert_eq!(waiting_from(&game, watching), 1);
}

/// Zidane, Tantalus Thief's first ruling: an opponent gaining control of it
/// and another permanent at once triggers its "whenever an opponent gains
/// control of a permanent from you" for each, itself included, off how it was
/// before.
// RULING: Zidane, Tantalus Thief #1 - stolen beside other permanents, it triggers once for each, itself included.
#[test]
fn a_watcher_stolen_with_another_permanent_triggers_for_each() {
    let mut game = stocked(2);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let any_permanent = TriggerSubject::Filter(ObjectFilter::All);
    let zidane = CardDataBuilder::new("Fixture Tantalus Thief")
        .card_type(CardType::Creature)
        .power_toughness(3, 3)
        .ability(triggered_ability("", whenever(an_opponent_gains_control_of(any_permanent), gain)))
        .build();
    let zidane = put_on_battlefield(&mut game, zidane, 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let mob_rule = Effect::Atom(
        Primitive::GainControl { to: PlayerRef::You, duration: Duration::UntilEndOfTurn },
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        )),
    );
    let source = put_in_hand(&mut game, grizzly_bears(), 1);
    let ctx = ResolutionContext::untargeted(source, 1, GeneratedBy::Spell);
    game.resolve_effect(&mob_rule, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(controller(&game, zidane), Some(1));
    resolve_all(&mut game, &test_dp());
    assert_eq!((life(&game, 0), life(&game, 1)), (22, 20), "two triggers, both the player's it was stolen from");
}

/// Decision 1's board: the first change on a board with no control-changing
/// effect. "Creatures you control have 'when you lose control of this, draw a
/// card'" gave the creature its trigger only while its owner controlled it;
/// stolen, it has it no more, and CR 603.10's "existence of those abilities
/// ... immediately prior to the event" still triggers it.
#[test]
fn the_first_steal_on_a_board_reads_the_frame_from_before() {
    let mut game = stocked(2);
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let mut granted = triggered_ability("", whenever(you_lose_control_of(TriggerSubject::Object(Antecedent::ThisObject)), draw));
    granted.id = new_ability_id();
    let granter = put_on_battlefield(&mut game, card_of_type("Fixture Banner", CardType::Enchantment), 0);
    let yours = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)));
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::WhileSourceOnBattlefield,
        affected_objects: ObjectSet::Filter { filter: yours, zones: ZoneSet::of(Zone::Battlefield) },
        ..registered(granter, Layer::Layer6Ability, 100, EffectModification::GrantAbility(Arc::new(granted)))
    });
    game.bump_layer_epoch();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let before = hand_size(&game, 0);
    assert!(!game.continuous_effects.summary().any_control_changing, "no steal in progress");

    steal(&mut game, 1, bears);
    assert_eq!(waiting_from(&game, bears), 1, "it had the ability just before");
    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 0), before + 1);
}

/// A spell an effect stole on the stack (CR 108.4) arrives its new
/// controller's (CR 400.7a's row): nothing changed control, and nothing is
/// announced.
#[test]
fn a_permanent_that_arrives_stolen_announces_no_change() {
    let mut game = stocked(2);
    let spell = put_spell_on_stack(&mut game, grizzly_bears(), 0);
    resolve_at(&mut game, 1, Primitive::GainControl { to: PlayerRef::You, duration: Duration::Indefinite }, spell);
    let before = game.recorded_events().len();

    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(controller(&game, spell), Some(1));
    let announced = game.recorded_events().records_from(before).iter().filter(|r| matches!(r.event, GameEvent::ControlChanged { .. })).count();
    assert_eq!(announced, 0);
}

/// **Fixture**, a one-sided Humility: "Creatures your opponents control lose
/// all abilities."
fn silencer() -> Arc<CardData> {
    let theirs = EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
    ));
    CardDataBuilder::new("Fixture Silencer")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield), theirs)))
        .build()
}

/// CR 603.10 reads "the existence of those abilities ... immediately prior to
/// the event". The thief's one-sided Humility left the owner's creature no
/// abilities before the steal; stolen, it has its trigger again, and still
/// does not trigger.
#[test]
fn a_trigger_the_creature_lacked_before_the_steal_does_not_trigger() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, silencer(), 0);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 1);
    let owners = hand_size(&game, 1);

    steal(&mut game, 0, betrayer);
    assert_eq!(waiting_from(&game, betrayer), 0, "it had no trigger just before");
    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 1), owners);
}

/// The same board with a trigger that names no player, "whenever control of
/// this creature changes": read live, the stolen creature has it and would
/// trigger, so only its list from before (CR 603.10) keeps it from triggering.
/// The "lose control" board above refuses on its "you" as well.
#[test]
fn a_trigger_on_any_change_the_creature_lacked_before_does_not_trigger() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, silencer(), 0);
    let draw_two = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    let any_change = TriggerEvent::ControlChanges { subject: TriggerSubject::Object(Antecedent::ThisObject), from: None, to: None };
    let watcher = CardDataBuilder::new("Fixture Turncoat")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability("", whenever(any_change, draw_two)))
        .build();
    let turncoat = put_on_battlefield(&mut game, watcher, 1);

    steal(&mut game, 0, turncoat);
    assert_eq!(waiting_from(&game, turncoat), 0, "it had no trigger just before");
}

/// The other side: the owner's one-sided Humility takes the creature's
/// trigger away once the thief controls it, and the trigger it had just
/// before the steal still triggers, for the owner.
#[test]
fn a_trigger_the_steal_takes_away_still_triggers() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, silencer(), 1);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 1);
    let owners = hand_size(&game, 1);

    steal(&mut game, 0, betrayer);
    assert_eq!(waiting_from(&game, betrayer), 1, "it had the trigger just before");
    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 1), owners + 2);
}

/// A change of one permanent's controller can change another's abilities:
/// "creatures you control have ..." follows its source to the thief, and the
/// creatures left behind lose what it gave them. Each had the trigger just
/// before the steal, so each triggers (CR 603.10), the granter and the Bears
/// alike.
#[test]
fn a_grant_the_steal_moves_still_triggers_the_creatures_left_behind() {
    let mut game = stocked(2);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let watcher = triggered_ability("", whenever(an_opponent_gains_control_of(TriggerSubject::Filter(ObjectFilter::All)), gain));
    let yours = EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    ));
    let banner_bearer = CardDataBuilder::new("Fixture Banner-Bearer")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(watcher), Duration::WhileSourceOnBattlefield),
            yours,
        )))
        .build();
    let bearer = put_on_battlefield(&mut game, banner_bearer, 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);

    steal(&mut game, 1, bearer);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 22, "the Bears and the granter, each off its list from before");
}

// ---------------------------------------------------------------------------
// 6. Ray of Command
// ---------------------------------------------------------------------------

/// Ray of Command cast by player 0 at `creature`, the one creature an
/// opponent controls, so no prompt asks for it; resolved.
fn ray_at(game: &mut GameState, creature: ObjectId) {
    cast_at(game, 0, ray_of_command(), &[(ManaType::Blue, 4)], &[]);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(controller(game, creature), Some(0));
}

fn tapped(game: &GameState, id: ObjectId) -> bool {
    game.battlefield[&id].tapped
}

// RULING: Ray of Command #3 - "The creature returns to the opponent when the "until end of turn" effect wears off during the cleanup step. It taps during the Cleanup step (if it is not already tapped), so any abilities triggered off it tapping happen at that time."
#[test]
fn ray_of_command_taps_the_creature_in_the_cleanup_step() {
    let mut state = stocked(2);
    let bears = put_on_battlefield(&mut state, grizzly_bears(), 1);
    ray_at(&mut state, bears);
    state.set_turn_position(Phase::new(PhaseType::Postcombat));
    let before = state.recorded_events().len();
    let mut game = Game { state, config: GameConfig::unrestricted() };

    game.resume_turn_at_priority(&test_dp()).unwrap();
    let state = &game.state;
    assert_eq!(controller(state, bears), Some(1), "back with its owner");
    // Its owner's untap step, which the driver also ran, untaps it again, so
    // the stream says where it was tapped: between the two cleanup steps.
    let order: Vec<&str> = state
        .recorded_events()
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::StepBegin { step: StepType::Cleanup, .. } => Some("cleanup"),
            GameEvent::ControlChanged { lki, .. } if lki.object().id == bears => Some("back"),
            GameEvent::Tapped { object } if object.id == bears => Some("tapped"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["cleanup", "back", "tapped", "cleanup"]);
}

// RULING: Ray of Command #1 - "You tap the creature when you lose control of it for any reason — because Ray of Command's effect ends, or because a spell or ability causes another player to gain control of it."
#[test]
fn ray_of_command_taps_the_creature_another_player_takes() {
    let mut game = stocked(3);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    ray_at(&mut game, bears);

    steal(&mut game, 2, bears);
    resolve_all(&mut game, &test_dp());
    assert_eq!(controller(&game, bears), Some(2));
    assert!(tapped(&game, bears), "Act of Treason untapped it, and Ray's trigger tapped it after");
}

/// Ray of Command untaps the creature, steals it and then grants it haste,
/// and nothing announces the change between the steal and the haste. The
/// change still looks back to the board before the steal (CR 603.10d), so the
/// owner is the one who lost control, and draws.
#[test]
fn ray_of_command_steals_from_a_player_who_then_lost_control() {
    let mut game = stocked(2);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 1);
    let owners = hand_size(&game, 1);

    ray_at(&mut game, betrayer);
    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 1), owners + 2, "CR 603.3a, off the frame from before the steal");
}

/// **Fixture**, Control Magic's static: "You control enchanted creature."
fn steal_aura(name: &str) -> Arc<CardData> {
    use mtgsim::types::card_types::{EnchantmentType, Subtype};
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .ability(static_ability(Effect::Atom(Primitive::GainControl { to: PlayerRef::You, duration: Duration::WhileSourceOnBattlefield }, EffectRecipient::Object(Antecedent::ThisObjectsHost))))
        .build()
}

/// Calming Verse's shape, cast by player 1 at a creature two steal Auras
/// enchant, player 1's and then player 2's: "Destroy all enchantments you
/// don't control. Then ... destroy all enchantments you control." Each
/// instruction ends one Aura, so control changes twice in one resolution,
/// as two events: player 2 loses it, then player 1 (CR 608.2c), and each
/// draws off the frame from before their own loss.
#[test]
fn two_control_changes_in_one_resolution_are_two_events() {
    let mut game = stocked(3);
    let betrayer = put_on_battlefield(&mut game, kharn_shaped(), 0);
    for thief in [1, 2] {
        let aura = put_on_battlefield(&mut game, steal_aura("Fixture Control Magic"), thief);
        game.execute_action(GameAction::Attach { attachment: aura, host: betrayer }, &effect_ctx(&game, aura)).unwrap();
        resolve_all(&mut game, &test_dp());
    }
    assert_eq!(controller(&game, betrayer), Some(2));
    let hands = |game: &GameState| (hand_size(game, 0), hand_size(game, 1), hand_size(game, 2));
    let before = hands(&game);

    let enchantments = |whose: PlayerRef| {
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Enchantment)),
            Box::new(ObjectFilter::ByController(whose)),
        ))
    };
    let verse = Effect::Sequence(vec![
        Effect::Atom(Primitive::Destroy, enchantments(PlayerRef::Opponent)),
        Effect::Atom(Primitive::Destroy, enchantments(PlayerRef::You)),
    ]);
    let source = put_in_graveyard(&mut game, card_of_type("Fixture Calming Verse", CardType::Sorcery), 1);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 1,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(Vec::new()),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&verse, &ctx, &test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());

    assert_eq!(controller(&game, betrayer), Some(0), "back with its owner");
    assert_eq!(hands(&game), (before.0, before.1 + 2, before.2 + 2), "players 2 and 1 each lost control");
}

/// **Fixture**: "Creatures you control have 'When you lose control of this
/// creature, draw two cards.'"
fn gives_the_betrayers_trigger() -> Arc<CardData> {
    let draw_two = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    let betrayal = triggered_ability("", whenever(you_lose_control_of(TriggerSubject::Object(Antecedent::ThisObject)), draw_two));
    CardDataBuilder::new("Fixture Sigil of Betrayal")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(betrayal), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// One resolution steals the enchantment that gives its controller's
/// creatures "when you lose control of this, draw two", then one of those
/// creatures, with no batch between. The first change is an event as it
/// happens (CR 603.2), so the creature had lost the ability before it
/// changed hands, and its owner draws nothing (CR 608.2c, 603.10d). Merged
/// into one event, both changes looked back to the board before the first.
// COVERS-PARTIAL: ATOM-603.10d-001
#[test]
fn a_second_steal_looks_back_to_the_board_the_first_left() {
    let mut game = stocked(2);
    let sigil = put_on_battlefield(&mut game, gives_the_betrayers_trigger(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let hands = |game: &GameState| (hand_size(game, 0), hand_size(game, 1));
    let before = hands(&game);

    let an_opponents = |kind: CardType| {
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(kind)),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        ))
    };
    let thefts = Effect::Sequence(vec![
        Effect::Atom(Primitive::GainControl { to: PlayerRef::You, duration: Duration::UntilEndOfTurn }, an_opponents(CardType::Enchantment)),
        Effect::Atom(Primitive::GainControl { to: PlayerRef::You, duration: Duration::UntilEndOfTurn }, an_opponents(CardType::Creature)),
    ]);
    let source = put_in_graveyard(&mut game, card_of_type("Fixture Double Theft", CardType::Sorcery), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(Vec::new()),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&thefts, &ctx, &test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());

    assert_eq!((controller(&game, sigil), controller(&game, bears)), (Some(0), Some(0)), "both stolen");
    assert_eq!(hands(&game), before, "the Bears had no trigger left when it changed hands");
}

// RULING: Ray of Command #2 - "You may target an untapped creature with Ray of Command."
#[test]
fn ray_of_command_may_target_an_untapped_creature() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert!(!tapped(&game, bears));

    ray_at(&mut game, bears);
    assert!(!tapped(&game, bears), "the untap does nothing, and the steal still happens");
    assert_eq!(game.delayed_triggers.len(), 1);
}
