//! TR-4d — control, countering, loss, and the records' missing facts
//! (`triggers-architecture.md` §12, TR-4d).
//!
//! 1. Item 253: a countered spell's record carries the spell as it was on the
//!    stack, and a trigger on a spell being countered looks back at it (CR
//!    603.10e). Multani's Presence's "a spell you've cast" reads who cast it.

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::authoring::{triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_tr4d_cards::multanis_presence;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, fill_library, lightning_bolt, put_in_hand, put_on_battlefield, put_spell_on_stack, setup_two_player_game,
    static_ability, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::TriggerEvent;
use mtgsim::types::zones::{Zone, ZoneSet};
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
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(2)), EffectRecipient::Controller);
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
