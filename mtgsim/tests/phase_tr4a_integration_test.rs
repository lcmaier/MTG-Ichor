//! TR-4a — the frame's facts and their readers (`triggers-architecture.md`
//! §12, TR-4a).
//!
//! 1. `codebase-state.md` item 251: a "from anywhere" arm reads the card
//!    where it went, and only a look-back arm reads it as it left (CR 603.6c,
//!    603.10). Compost against a Clone and against Darkest Hour.
//! 2. Item 252: an Aura's "when enchanted creature dies" asks its host's
//!    departure what was attached to it (CR 603.10a). Demonic Vigor, and CR
//!    603.6e's Abduction.
//! 3. Item 168: a trigger of an object that left names the existence that
//!    left, and keeps its last known information from the start.
//! 4. Persist and undying (CR 702.79a, 702.93a): Kitchen Finks' seven
//!    rulings, each on both keywords, and Strangleroot Geist.
//! 5. The five leaves that read their source, and a delayed trigger's "if",
//!    off its last known information once it has left (CR 113.7a, 608.2h).
//! 6. CR 122.8 and 122.9: the counters it had, put anew.
//! 7. The owner last known information keeps (CR 108.3).

use std::sync::Arc;

use mtgsim::cards::authoring::{attacks, dies, leaves_the_battlefield, persist, triggered_ability, undying, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cv_cards;
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_re_cards::doubling_season;
use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::cards::phase_tr4a_cards::{compost, demonic_vigor, strangleroot_geist};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness, has_type};
use mtgsim::state::battlefield::CostChoices;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    effect_ctx, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield, put_on_battlefield_under,
    put_spell_on_stack, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp,
};
use mtgsim::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::costs::{AdditionalCost, Cost};
use mtgsim::types::history::CountIs;
use mtgsim::types::effects::{
    AmountExpr, Antecedent, ColorChange, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerGroup,
    PlayerRef, PlayerSet, Primitive, ReturnUnder, SelectionFilter, TargetCount, TypeChange,
};
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectOrPlayer, ObjectOrPlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{
    DamageRecipient, DelayedDuration, DelayedProvenance, DelayedTriggerTemplate, DelayedTurn, EventRole, Multiplicity, Referred,
    TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject, TriggerTurn,
};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::random::RandomDecisionProvider;

// ---------------------------------------------------------------------------
// Fixtures and helpers
// ---------------------------------------------------------------------------

/// A black creature card with no ability.
fn black_probe() -> Arc<CardData> {
    CardDataBuilder::new("Black Probe")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .build()
}

/// A white creature card with no ability.
fn white_probe() -> Arc<CardData> {
    CardDataBuilder::new("White Probe")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .build()
}

/// Darkest Hour — {B}, Enchantment: "All creatures are black." A fixture:
/// the card is not registered, and its one sentence is all this file needs.
fn darkest_hour() -> Arc<CardData> {
    let black = std::collections::HashSet::from([Color::Black]);
    CardDataBuilder::new("Darkest Hour")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("All creatures are black.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeColor(ColorChange::Set(black), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
        )))
        .build()
}

/// Abduction's trigger, ATOM-603.6e-002's board: "When enchanted creature
/// dies, return that card to the battlefield under its owner's control." A
/// fixture Aura with nothing else.
fn abduction_fixture() -> Arc<CardData> {
    let text = "When enchanted creature dies, return that card to the battlefield under its owner's control.";
    let returns = Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new(), tapped: false, attacking: None };
    CardDataBuilder::new("Abduction Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(dies(TriggerSubject::Host), Effect::Atom(returns, EffectRecipient::Triggering(EventRole::Object)))))
        .build()
}

/// Persist or undying, and the counter each reads and puts.
#[derive(Clone, Copy)]
struct Keyword {
    name: &'static str,
    ability: fn() -> AbilityDef,
    /// The kind it reads and returns with.
    counter: CounterType,
    /// The other kind.
    other: CounterType,
}

const PERSIST: Keyword =
    Keyword { name: "persist", ability: persist, counter: CounterType::MinusOneMinusOne, other: CounterType::PlusOnePlusOne };
const UNDYING: Keyword =
    Keyword { name: "undying", ability: undying, counter: CounterType::PlusOnePlusOne, other: CounterType::MinusOneMinusOne };
const BOTH: [Keyword; 2] = [PERSIST, UNDYING];

/// A 3/3 green creature with `keyword`, `instances` times over.
fn returner(keyword: Keyword, instances: usize) -> Arc<CardData> {
    let mut card = CardDataBuilder::new("Returner")
        .mana_cost(ManaCost::build(&[ManaType::Green], 2))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Ouphe))
        .power_toughness(3, 3);
    for _ in 0..instances {
        card = card.ability((keyword.ability)());
    }
    card.build()
}

fn destroy(game: &mut GameState, object: ObjectId) {
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object }, &effect_ctx(game, by)).unwrap();
}

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    let by = game.objects[&object].owner;
    game.execute_action(GameAction::AddCounters { subject: ObjectOrPlayerId::Object(object), counter, n, by }, &test_ctx()).unwrap();
}

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

fn counters(game: &GameState, object: ObjectId, counter: CounterType) -> u32 {
    game.battlefield.get(&object).map_or(0, |entry| entry.counter_count(counter))
}

fn zone(game: &GameState, object: ObjectId) -> Option<Zone> {
    game.objects.get(&object).map(|o| o.zone)
}

fn make_token(game: &mut GameState, object: ObjectId) {
    game.objects.get_mut(&object).unwrap().is_token = true;
}

// ---------------------------------------------------------------------------
// 1. "From anywhere" reads the card where it went (item 251)
// ---------------------------------------------------------------------------

/// Compost's ruling: a black card reaching an opponent's graveyard from any
/// zone triggers it, and one reaching its own controller's does not.
// RULING: Compost #1 - "The ability triggers on a black card being discarded,
//   milled, or countered, as well as a black card going to the graveyard from
//   the battlefield."
#[test]
fn compost_sees_a_black_card_reach_a_graveyard_from_every_zone() {
    let mut game = setup_two_player_game();
    let compost = put_on_battlefield(&mut game, compost(), 0);
    let dies = put_on_battlefield(&mut game, black_probe(), 1);
    let discarded = put_in_hand(&mut game, black_probe(), 1);
    let milled = put_in_library(&mut game, black_probe(), 1);
    let countered = put_spell_on_stack(&mut game, black_probe(), 1);
    let yours = put_in_hand(&mut game, black_probe(), 0);

    destroy(&mut game, dies);
    assert_eq!(waiting_from(&game, compost), 1, "from the battlefield");
    game.change_zone(discarded, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, compost), 2, "discarded");
    game.change_zone(milled, Zone::Graveyard, ZoneChangeAct::Milled, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, compost), 3, "milled");
    game.change_zone(countered, Zone::Graveyard, ZoneChangeAct::Countered, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, compost), 4, "countered");
    game.change_zone(yours, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, compost), 4, "its controller's own graveyard is no opponent's");
}

/// Item 251: a Clone that copied a black creature is a blue card in the
/// graveyard, and "from anywhere" asks the card there (CR 603.6c, 603.10).
/// The black creature it copied still triggers Compost as it dies.
#[test]
fn compost_does_not_see_a_clone_that_copied_a_black_creature() {
    let mut game = setup_two_player_game();
    let compost = put_on_battlefield(&mut game, compost(), 0);
    let donor = put_on_battlefield(&mut game, black_probe(), 1);
    let clone = put_in_graveyard(&mut game, phase_cv_cards::clone(), 1);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: clone }, vec![ChoiceOption::Object(donor)]);
    game.change_zone(clone, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).unwrap();
    let chars = mtgsim::engine::layers::compute_characteristics(&game, clone).unwrap();
    assert!(chars.colors.contains(&Color::Black), "the Clone is a copy of the black creature");

    destroy(&mut game, clone);
    assert_eq!(waiting_from(&game, compost), 0, "it died blue");
    destroy(&mut game, donor);
    assert_eq!(waiting_from(&game, compost), 1, "the black creature it copied");
}

/// Item 251: Darkest Hour's creatures are black on the battlefield, and a
/// white one is a white card in the graveyard.
#[test]
fn compost_does_not_see_a_creature_darkest_hour_made_black() {
    let mut game = setup_two_player_game();
    let compost = put_on_battlefield(&mut game, compost(), 0);
    put_on_battlefield(&mut game, darkest_hour(), 0);
    let white = put_on_battlefield(&mut game, white_probe(), 1);
    let chars = mtgsim::engine::layers::compute_characteristics(&game, white).unwrap();
    assert!(chars.colors.contains(&Color::Black), "Darkest Hour made it black");

    destroy(&mut game, white);
    assert_eq!(waiting_from(&game, compost), 0, "it died white");
}

/// CR 108.2: a black token is not a black card, in the graveyard as on the
/// battlefield.
#[test]
fn compost_does_not_see_a_black_token() {
    let mut game = setup_two_player_game();
    let compost = put_on_battlefield(&mut game, compost(), 0);
    let token = put_on_battlefield(&mut game, black_probe(), 1);
    make_token(&mut game, token);

    destroy(&mut game, token);
    assert_eq!(waiting_from(&game, compost), 0);
}

/// Compost end to end: the trigger resolves and its controller may draw.
#[test]
fn compost_draws_its_controller_a_card_when_asked() {
    let mut game = setup_two_player_game();
    let compost = put_on_battlefield(&mut game, compost(), 0);
    put_in_library(&mut game, grizzly_bears(), 0);
    let dies = put_on_battlefield(&mut game, black_probe(), 1);
    destroy(&mut game, dies);
    let hand_before = game.players[0].hand.len();

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: compost }, vec![0]);
    resolve_all(&mut game, &dp);
    assert_eq!(game.players[0].hand.len(), hand_before + 1, "\"you may draw a card\", taken");
}

// ---------------------------------------------------------------------------
// 2. An Aura's "when enchanted creature dies" (item 252, CR 603.6e)
// ---------------------------------------------------------------------------

/// Item 252: the Aura stays on the battlefield, unattached, as its creature
/// dies, and asks the creature's departure what was attached to it. "That
/// card" is the one in its owner's graveyard, which goes to its owner's
/// hand; the Aura goes to its owner's graveyard by CR 704.5m.
#[test]
fn demonic_vigor_returns_the_creature_it_enchanted_to_its_owners_hand() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield_under(&mut game, grizzly_bears(), 1, 0);
    let vigor = put_on_battlefield(&mut game, demonic_vigor(), 0);
    assert!(game.attach(vigor, creature));
    assert_eq!(get_effective_power(&game, creature), Some(3), "enchanted creature gets +1/+1");

    destroy(&mut game, creature);
    assert_eq!(waiting_from(&game, vigor), 1, "when enchanted creature dies");
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, creature), Some(Zone::Hand));
    assert!(game.players[1].hand.contains(&creature), "its owner's hand");
    assert_eq!(zone(&game, vigor), Some(Zone::Graveyard), "CR 704.5m");
}

/// Demonic Vigor's ruling: a token is no card to return.
// RULING: Demonic Vigor #1 - "Demonic Vigor can enchant a token, but its last
//   ability won't return the token to your hand."
#[test]
fn demonic_vigor_triggers_for_a_token_and_returns_nothing() {
    let mut game = setup_two_player_game();
    let token = put_on_battlefield(&mut game, grizzly_bears(), 0);
    make_token(&mut game, token);
    let vigor = put_on_battlefield(&mut game, demonic_vigor(), 0);
    assert!(game.attach(vigor, token));

    destroy(&mut game, token);
    assert_eq!(waiting_from(&game, vigor), 1, "it triggers");
    resolve_all(&mut game, &test_dp());
    assert!(!game.objects.contains_key(&token), "the token ceased to exist (CR 704.5d)");
    assert!(game.players[0].hand.is_empty(), "nothing returned");
}

/// The Aura and its creature leave in one event: the Aura asks as it was,
/// off its own frame (CR 603.10a), and the creature's departure says the
/// Aura was attached to it.
#[test]
fn demonic_vigor_destroyed_beside_its_creature_still_returns_it() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let vigor = put_on_battlefield(&mut game, demonic_vigor(), 0);
    assert!(game.attach(vigor, creature));
    let by = mtgsim::types::ids::new_object_id();
    game.execute_actions(
        vec![GameAction::Destroy { object: creature }, GameAction::Destroy { object: vigor }],
        &effect_ctx(&game, by),
    )
    .unwrap();
    assert_eq!(waiting_from(&game, vigor), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, creature), Some(Zone::Hand));
}

/// Another creature dying is not the enchanted one.
#[test]
fn demonic_vigor_does_not_see_another_creature_die() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let vigor = put_on_battlefield(&mut game, demonic_vigor(), 0);
    assert!(game.attach(vigor, creature));

    destroy(&mut game, other);
    assert_eq!(waiting_from(&game, vigor), 0);
}

/// ATOM-603.6e-002: an Aura's trigger on its creature dying finds the card
/// the creature became, and returns it under its owner's control.
// COVERS: ATOM-603.6e-002
#[test]
fn an_aura_returns_the_card_its_creature_became_under_its_owners_control() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield_under(&mut game, grizzly_bears(), 1, 0);
    let abduction = put_on_battlefield(&mut game, abduction_fixture(), 0);
    assert!(game.attach(abduction, creature));
    let before = game.object_ref(creature).unwrap();

    destroy(&mut game, creature);
    resolve_all(&mut game, &test_dp());
    let after = game.object_ref(creature).unwrap();
    assert_eq!(zone(&game, creature), Some(Zone::Battlefield));
    assert_ne!(after, before, "the new object the card became (CR 400.7)");
    assert_eq!(mtgsim::oracle::characteristics::get_effective_controller(&game, creature), Some(1), "its owner's control");
}

// ---------------------------------------------------------------------------
// 3. A trigger names the existence that left (item 168)
// ---------------------------------------------------------------------------

/// Item 168: Blood Artist's own death triggers it as the object it was on
/// the battlefield, whose last known information the LKI store holds from
/// the move on, so the entry reads it with nothing handed over.
#[test]
fn blood_artists_own_death_triggers_as_the_creature_that_left_not_the_card_it_became() {
    let mut game = setup_two_player_game();
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let before = game.object_ref(artist).unwrap();

    destroy(&mut game, artist);
    assert_ne!(game.object_ref(artist), Some(before), "the card in the graveyard is a new object");
    let pending = game.pending_triggers.iter().find(|p| p.origin.source() == artist).expect("it triggered");
    assert_eq!(pending.origin.source_ref(), before, "the existence that left, not the card it became");
    let last_known = game.lki(pending.origin.source_ref()).map(|lki| lki.object());
    assert_eq!(last_known, Some(before), "its last known information, as it queued");
}

// ---------------------------------------------------------------------------
// 4. Persist and undying: Kitchen Finks' seven rulings, each on both
// ---------------------------------------------------------------------------

/// Kitchen Finks' first ruling: persist triggers for a permanent, so one
/// that stopped being a creature still returns. A fixture static ability
/// makes artifact creatures noncreature artifacts.
#[test]
fn persist_and_undying_still_return_a_permanent_that_stopped_being_a_creature() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let artifact = CardDataBuilder::new("Artifact Returner")
            .mana_cost(ManaCost::build(&[], 3))
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .ability((keyword.ability)())
            .build();
        let returner = put_on_battlefield(&mut game, artifact, 0);
        let not_creatures = TypeChange { remove_types: vec![CardType::Creature], ..TypeChange::NONE };
        let march = CardDataBuilder::new("Noncreature Fixture")
            .card_type(CardType::Enchantment)
            .ability(static_ability(Effect::Atom(
                Primitive::ChangeType(not_creatures, Duration::WhileSourceOnBattlefield),
                EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Artifact)),
            )))
            .build();
        put_on_battlefield(&mut game, march, 0);
        assert!(!has_type(&game, returner, CardType::Creature), "{}: no longer a creature", keyword.name);

        destroy(&mut game, returner);
        resolve_all(&mut game, &test_dp());
        assert_eq!(zone(&game, returner), Some(Zone::Battlefield), "{} still works", keyword.name);
        assert_eq!(counters(&game, returner, keyword.counter), 1);
    }
}

/// Kitchen Finks' second ruling: whether it had the counter is read off how
/// it last existed on the battlefield, at the trigger.
// COVERS-PARTIAL: ATOM-603.4-002
#[test]
fn persist_and_undying_skip_a_creature_that_died_with_their_counter_on_it() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let had = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        add_counters(&mut game, had, keyword.counter, 1);
        let had_none = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        add_counters(&mut game, had_none, keyword.other, 1);

        destroy(&mut game, had);
        assert_eq!(waiting_from(&game, had), 0, "{}: it had a {:?} counter", keyword.name, keyword.counter);
        destroy(&mut game, had_none);
        assert_eq!(waiting_from(&game, had_none), 1, "{}: the other kind is not asked", keyword.name);
        resolve_all(&mut game, &test_dp());
        assert_eq!(zone(&game, had), Some(Zone::Graveyard));
        assert_eq!(zone(&game, had_none), Some(Zone::Battlefield));
        assert_eq!(counters(&game, had_none, keyword.counter), 1, "{} returns it with one counter", keyword.name);
        assert_eq!(counters(&game, had_none, keyword.other), 0, "and nothing it had before (CR 400.7)");
    }
}

/// The recheck at resolution (CR 603.4) reads the same last known
/// information: the card is in the graveyard by then, with no counters, and
/// its frame still says it had none.
#[test]
fn persist_and_undying_recheck_on_resolution_reads_the_creature_not_the_graveyard_card() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let returner = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        destroy(&mut game, returner);
        game.perform_sba_and_triggers(&test_dp()).unwrap();
        assert_eq!(game.stack.len(), 1, "{} is on the stack", keyword.name);
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(zone(&game, returner), Some(Zone::Battlefield), "{}: the recheck held", keyword.name);
    }
}

/// Kitchen Finks' third ruling: two instances trigger separately, and the
/// second finds nothing to return.
#[test]
fn two_instances_of_persist_or_undying_both_trigger_and_return_it_once() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let returner = put_on_battlefield(&mut game, returner(keyword, 2), 0);
        destroy(&mut game, returner);
        assert_eq!(waiting_from(&game, returner), 2, "{}: each instance triggers", keyword.name);
        resolve_all(&mut game, &RandomDecisionProvider::seeded(7));
        assert_eq!(zone(&game, returner), Some(Zone::Battlefield));
        assert_eq!(counters(&game, returner, keyword.counter), 1, "{}: returned once", keyword.name);
    }
}

/// Kitchen Finks' fourth ruling: a token with the keyword triggers, and
/// cannot return.
#[test]
fn persist_and_undying_trigger_for_a_token_that_cannot_return() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let token = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        make_token(&mut game, token);
        destroy(&mut game, token);
        assert_eq!(waiting_from(&game, token), 1, "{}: it triggers", keyword.name);
        resolve_all(&mut game, &test_dp());
        assert!(!game.objects.contains_key(&token), "{}: the token ceased to exist", keyword.name);
    }
}

/// Kitchen Finks' fifth ruling: what returns is a new object, with no memory
/// of its previous existence — untapped, undamaged, unenchanted, newly
/// arrived, and carrying only the counter it returned with.
#[test]
fn persist_and_undying_return_a_new_object_carrying_only_its_counter() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let returner = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        let aura = put_on_battlefield(&mut game, holy_strength(), 0);
        assert!(game.attach(aura, returner));
        game.execute_action(GameAction::Tap { object: returner }, &test_ctx()).unwrap();
        let before = game.object_ref(returner).unwrap();

        destroy(&mut game, returner);
        resolve_all(&mut game, &test_dp());
        let entry = game.battlefield.get(&returner).expect("returned");
        assert_ne!(game.object_ref(returner), Some(before), "{}: a new object", keyword.name);
        assert!(!entry.tapped && entry.damage_marked == 0 && entry.attached_by.is_empty());
        assert_eq!(entry.entered_battlefield_turn, game.turn_number, "it arrived this turn");
        assert_eq!(zone(&game, aura), Some(Zone::Graveyard), "the Aura did not follow it");
        let base = if keyword.counter == CounterType::PlusOnePlusOne { 4 } else { 2 };
        assert_eq!(get_effective_toughness(&game, returner), Some(base), "3/3 and its one counter");
    }
}

/// Kitchen Finks' sixth ruling: the active player's triggers go on the stack
/// first, so the nonactive player's resolve first and their creature returns
/// first (CR 603.3b, 405.3).
#[test]
fn simultaneous_persist_or_undying_return_the_nonactive_players_creature_first() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let actives = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        let nonactives = put_on_battlefield(&mut game, returner(keyword, 1), 1);
        let by = mtgsim::types::ids::new_object_id();
        game.execute_actions(
            vec![GameAction::Destroy { object: actives }, GameAction::Destroy { object: nonactives }],
            &effect_ctx(&game, by),
        )
        .unwrap();
        resolve_all(&mut game, &test_dp());
        let arrived = |id: ObjectId| game.battlefield.get(&id).map(|entry| entry.timestamp).expect("returned");
        assert!(arrived(nonactives) < arrived(actives), "{}: one at a time, the nonactive player's first", keyword.name);
    }
}

/// Kitchen Finks' seventh ruling: a creature whose -1/-1 counters on top of
/// its +1/+1 counters kill it had both kinds as it left, since CR 704.5q's
/// annihilation is part of the same check, so neither keyword returns it.
// COVERS: COMP-9A-001
#[test]
fn persist_and_undying_skip_a_creature_that_died_with_both_counter_kinds() {
    for keyword in BOTH {
        let mut game = setup_two_player_game();
        let returner = put_on_battlefield(&mut game, returner(keyword, 1), 0);
        add_counters(&mut game, returner, CounterType::PlusOnePlusOne, 1);
        add_counters(&mut game, returner, CounterType::MinusOneMinusOne, 5);
        resolve_all(&mut game, &test_dp());
        assert_eq!(zone(&game, returner), Some(Zone::Graveyard), "{}: it had both as it left", keyword.name);
    }
}

/// Strangleroot Geist: undying returns it once with a +1/+1 counter, and
/// with haste it may attack the turn it returns. Dying with its counter, it
/// stays dead.
// COVERS: ATOM-702.93a-001
#[test]
fn strangleroot_geist_returns_once() {
    let mut game = setup_two_player_game();
    let geist = put_on_battlefield(&mut game, strangleroot_geist(), 0);
    destroy(&mut game, geist);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, geist), Some(Zone::Battlefield));
    assert_eq!((get_effective_power(&game, geist), get_effective_toughness(&game, geist)), (Some(3), Some(2)));
    assert!(mtgsim::oracle::legality::can_attack(&game, 0, geist).is_ok(), "haste");

    destroy(&mut game, geist);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, geist), Some(Zone::Graveyard), "it had a +1/+1 counter");
}

// ---------------------------------------------------------------------------
// 5. An intervening "if" about the creature itself, asked after it died
//
// Every test here uses one fixture: a creature with "When this creature dies,
// if [condition], you gain 1 life." The test sets the creature up — taps it,
// puts counters on it, marks it kicked — then destroys it and resolves the
// trigger. By then the creature is a card in the graveyard, untapped and with
// no counters, at both of the "if"'s checks: when it dies, and when the
// trigger resolves (CR 603.4). So the answer has to come from how the
// creature was on the battlefield (CR 113.7a). Its controller ending at 21
// life means both checks passed.
// ---------------------------------------------------------------------------

/// A 2/2 creature: "When this creature dies, if [condition], you gain 1 life."
fn creature_with_a_dies_trigger_if(condition: Condition) -> Arc<CardData> {
    let text = "When this creature dies, if [condition], you gain 1 life.";
    let def = TriggerDef {
        condition: TriggerCondition::Event(dies(TriggerSubject::ThisObject).into()),
        intervening_if: Some(condition),
        limit: None,
        effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
    };
    CardDataBuilder::new("Conditional Dier")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(text, def))
        .build()
}

/// Put [`creature_with_a_dies_trigger_if`] `condition` onto P0's
/// battlefield, call `set_up` on it, destroy it, and resolve everything.
/// True when P0 gained the life: the "if" held when the creature died and
/// again when its trigger resolved.
fn dies_trigger_gains_life(condition: Condition, set_up: impl Fn(&mut GameState, ObjectId)) -> bool {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, creature_with_a_dies_trigger_if(condition), 0);
    set_up(&mut game, creature);
    destroy(&mut game, creature);
    resolve_all(&mut game, &test_dp());
    game.players[0].life_total == 21
}

fn tap_it(game: &mut GameState, creature: ObjectId) {
    game.execute_action(GameAction::Tap { object: creature }, &test_ctx()).unwrap();
}

fn leave_it(_: &mut GameState, _: ObjectId) {}

/// "If it was tapped" is true of a creature that died tapped and false of
/// one that died untapped, and "if it was untapped" the reverse, though the
/// card in the graveyard is untapped either way.
#[test]
fn dies_trigger_if_tapped_sees_whether_the_creature_died_tapped() {
    assert!(dies_trigger_gains_life(Condition::SourceTapped, tap_it), "it died tapped");
    assert!(!dies_trigger_gains_life(Condition::SourceTapped, leave_it), "it died untapped");
    assert!(dies_trigger_gains_life(Condition::SourceUntapped, leave_it), "it died untapped");
    assert!(!dies_trigger_gains_life(Condition::SourceUntapped, tap_it), "it died tapped");
}

/// "If it had two or more charge counters" counts the counters the creature
/// died with; the card in the graveyard has none.
#[test]
fn dies_trigger_if_it_had_counters_counts_the_counters_it_died_with() {
    let two_or_more =
        Condition::CounterCount { object: Antecedent::ThisObject, kind: Some(CounterType::Charge), count: CountIs::AtLeast(2) };
    let with_two = |game: &mut GameState, creature: ObjectId| add_counters(game, creature, CounterType::Charge, 2);
    let with_one = |game: &mut GameState, creature: ObjectId| add_counters(game, creature, CounterType::Charge, 1);
    assert!(dies_trigger_gains_life(two_or_more.clone(), with_two), "it died with two");
    assert!(!dies_trigger_gains_life(two_or_more, with_one), "it died with one");
}

/// "If it was kicked" reads the cost decisions the permanent kept from its
/// spell (CR 707.10) after it died.
#[test]
fn dies_trigger_if_kicked_sees_that_the_creature_was_kicked_after_it_died() {
    let kicked = |game: &mut GameState, creature: ObjectId| {
        game.battlefield.get_mut(&creature).unwrap().cost_choices =
            CostChoices { additional: vec![AdditionalCost::Kicker(Vec::new())], alternative: None };
    };
    assert!(dies_trigger_gains_life(Condition::SpellWasKicked, kicked), "it was kicked");
    assert!(!dies_trigger_gains_life(Condition::SpellWasKicked, leave_it), "it was not");
}

/// An Aura's "When this Aura leaves the battlefield, if it enchanted a
/// creature, you gain 1 life": the Aura in the graveyard is attached to
/// nothing, so "what it enchanted" is read off how it last existed. Its host
/// is still on the battlefield, and is read there.
#[test]
fn aura_leaves_trigger_if_it_enchanted_a_creature_reads_its_last_host() {
    for (host_card, enchanted_a_creature) in [(grizzly_bears(), true), (mtgsim::test_support::forest(), false)] {
        let mut game = setup_two_player_game();
        let text = "When this Aura leaves the battlefield, if it enchanted a creature, you gain 1 life.";
        let def = TriggerDef {
            condition: TriggerCondition::Event(leaves_the_battlefield(TriggerSubject::ThisObject).into()),
            intervening_if: Some(Condition::HostMatches(ObjectFilter::ByType(CardType::Creature))),
            limit: None,
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        };
        let aura_card = CardDataBuilder::new("Leaving Aura")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(EnchantmentType::Aura))
            .enchant_filter(SelectionFilter::Permanent(ObjectFilter::All))
            .ability(triggered_ability(text, def))
            .build();
        let host = put_on_battlefield(&mut game, host_card, 0);
        let aura = put_on_battlefield(&mut game, aura_card, 0);
        assert!(game.attach(aura, host));
        destroy(&mut game, aura);
        resolve_all(&mut game, &test_dp());
        assert_eq!(game.players[0].life_total == 21, enchanted_a_creature);
    }
}

/// A delayed trigger, "When you next gain life, if this creature was tapped,
/// you lose 5 life", made by a tapped creature. The creature is exiled and
/// returned untapped: that is a new object (CR 400.7), so the "if" must read
/// the tapped creature that made the trigger, as it last existed, and not
/// the untapped one now on the battlefield.
#[test]
fn delayed_trigger_if_its_source_was_tapped_reads_the_creature_that_made_it() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, grizzly_bears(), 0);
    tap_it(&mut game, source);
    let def = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::GainsLife {
            player: Some(PlayerRef::You),
            multiplicity: Multiplicity::PerOccurrence,
        }),
        intervening_if: Some(Condition::SourceTapped),
        limit: None,
        effect: Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(5)), EffectRecipient::Controller),
    };
    let template = DelayedTriggerTemplate {
        def: Arc::new(def),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "When you next gain life, if this creature was tapped, you lose 5 life.".into(),
    };
    let provenance = DelayedProvenance {
        source: game.object_ref(source).unwrap(),
        source_card: grizzly_bears(),
        controller: 0,
        created_by: None,
        x_value: None,
        turn: TriggerTurn::Any,
        resolution: None,
        cause: mtgsim::events::cause::EventCause::Harness,
    };
    game.create_delayed_trigger(&template, provenance, Referred::default()).unwrap();
    game.change_zone(source, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.change_zone(source, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert!(!game.battlefield[&source].tapped, "the new object is untapped");

    game.execute_action(GameAction::GainLife { player: 0, amount: 1, source }, &test_ctx()).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "it read the tapped creature that made it");
}

// ---------------------------------------------------------------------------
// 6. The counters it had (CR 122.8, 122.9)
// ---------------------------------------------------------------------------

/// "Put a number of +1/+1 counters equal to the number [this] had on target
/// creature."
fn put_as_many_as_it_had() -> Effect {
    Effect::Atom(
        Primitive::AddCounters {
            counter: CounterType::PlusOnePlusOne,
            amount: AmountExpr::SourceCounters(CounterType::PlusOnePlusOne),
            by: PlayerRef::You,
        },
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    )
}

/// ATOM-122.8-001: a dies trigger puts as many counters as its source had,
/// new ones, so Doubling Season doubles them.
// COVERS: ATOM-122.8-001
#[test]
fn a_dies_trigger_puts_as_many_counters_as_its_source_had() {
    let mut game = setup_two_player_game();
    let text = "When this creature dies, put a number of +1/+1 counters equal to the number it had on target creature.";
    let a_card = CardDataBuilder::new("Counter Giver")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(triggered_ability(text, whenever(dies(TriggerSubject::ThisObject), put_as_many_as_it_had())))
        .build();
    let a = put_on_battlefield(&mut game, a_card, 0);
    add_counters(&mut game, a, CounterType::PlusOnePlusOne, 3);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, doubling_season(), 0);

    destroy(&mut game, a);
    resolve_all(&mut game, &test_dp());
    assert_eq!(counters(&game, b, CounterType::PlusOnePlusOne), 6, "three new counters, put and so doubled");
    let put_on_b = game.recorded_events().events().any(|event| {
        matches!(event, GameEvent::CountersPutOn { subject: ObjectOrPlayer::Object(object), n: 6, .. } if object.id == b)
    });
    assert!(put_on_b, "put, not moved: A's ceased to exist as it left (CR 122.2)");
}

/// ATOM-122.9-001: "Sacrifice this: put a number of +1/+1 counters equal to
/// the number it had on target creature" — the sacrifice is the cost, and the
/// counters are read off how it last existed.
// COVERS: ATOM-122.9-001
#[test]
fn a_sacrificed_source_puts_as_many_counters_as_it_had() {
    let mut game = setup_two_player_game();
    let a_card = CardDataBuilder::new("Sacrificial Giver")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(AbilityDef {
            rules_text: "Sacrifice this creature: Put a number of +1/+1 counters equal to the number it had on target creature.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::SacrificeSelf],
            effect: put_as_many_as_it_had(),
        })
        .build();
    let a = put_on_battlefield(&mut game, a_card, 0);
    add_counters(&mut game, a, CounterType::PlusOnePlusOne, 2);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: a },
        vec![ChoiceOption::Object(b)],
    );
    game.activate_ability(0, a, 0, &dp).unwrap();
    assert_eq!(zone(&game, a), Some(Zone::Graveyard), "sacrificed to pay the cost");
    resolve_all(&mut game, &test_dp());
    assert_eq!(counters(&game, b, CounterType::PlusOnePlusOne), 2);
}

// ---------------------------------------------------------------------------
// 7. The owner last known information keeps (CR 108.3)
// ---------------------------------------------------------------------------

/// A permanent whose owner left the game left with them (CR 800.4a), and the
/// store has lost it; "a creature an opponent owns" is still asked of it, off
/// its last known information.
#[test]
fn a_leaves_trigger_reads_the_owner_of_an_object_that_left_the_game() {
    let mut game = setup_game(3);
    let text = "Whenever a creature an opponent owns leaves the battlefield, you gain 1 life.";
    let theirs = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByOwner(PlayerRef::Opponent)),
    );
    let watcher = CardDataBuilder::new("Owner Watcher")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(text, whenever(leaves_the_battlefield(theirs), Effect::Atom(
            Primitive::GainLife(AmountExpr::Fixed(1)),
            EffectRecipient::Controller,
        ))))
        .build();
    let watcher = put_on_battlefield(&mut game, watcher, 0);
    let leaving = put_on_battlefield(&mut game, grizzly_bears(), 2);

    game.execute_action(GameAction::PlayerLoses { player: 2 }, &test_ctx()).unwrap();
    assert!(!game.objects.contains_key(&leaving), "it left the game with its owner");
    assert_eq!(waiting_from(&game, watcher), 1, "an opponent owned it");
}

/// A delayed trigger made after its source is gone reads "its owner" off the
/// last known information its resolution kept: a token its owner's opponent
/// controlled, which ceased to exist before its ability resolved. The
/// controller would be the wrong player.
#[test]
fn a_delayed_trigger_made_after_its_source_is_gone_knows_its_owner() {
    let mut game = setup_two_player_game();
    let owner_gains = TriggerEvent::GainsLife { player: Some(PlayerRef::Owner), multiplicity: Multiplicity::PerOccurrence };
    let template = DelayedTriggerTemplate {
        def: Arc::new(whenever(owner_gains, Effect::Sequence(Vec::new()))),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "When its owner next gains life, nothing.".into(),
    };
    let text = "When this creature dies, when its owner next gains life, nothing.";
    let card = CardDataBuilder::new("Delayed Maker")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(triggered_ability(
            text,
            whenever(dies(TriggerSubject::ThisObject), Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Implicit)),
        ))
        .build();
    let token = put_on_battlefield_under(&mut game, card, 0, 1);
    make_token(&mut game, token);

    destroy(&mut game, token);
    resolve_all(&mut game, &test_dp());
    assert!(!game.objects.contains_key(&token), "the token ceased to exist before its trigger resolved");
    assert_eq!(game.delayed_triggers.len(), 1);
    assert_eq!(game.delayed_triggers[0].owner, 0, "its owner, though its controller was P1");
    game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: token }, &test_ctx()).unwrap();
    assert!(game.delayed_triggers.is_empty(), "P0's gain is its owner's");
}

// ---------------------------------------------------------------------------
// 8. The model pass's three disagreements, each now one reading
// ---------------------------------------------------------------------------

/// CR 603.4 at the trigger is asked of each occurrence, as it is at
/// resolution: two attackers declared together are two triggers, each with
/// its own "it", and only the one without a counter triggers.
#[test]
fn attack_trigger_if_it_had_no_counters_checks_each_attacker_separately() {
    let mut game = setup_two_player_game();
    let text = "Whenever a creature you control attacks, if it had no +1/+1 counters on it, you gain 1 life.";
    let def = TriggerDef {
        condition: TriggerCondition::Event(attacks(TriggerSubject::Filter(ObjectFilter::ByController(PlayerRef::You))).into()),
        intervening_if: Some(Condition::CounterCount {
            object: Antecedent::Triggering(EventRole::Attacker),
            kind: Some(CounterType::PlusOnePlusOne),
            count: CountIs::AtMost(0),
        }),
        limit: None,
        effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
    };
    let watcher = CardDataBuilder::new("Attack Watcher").card_type(CardType::Enchantment).ability(triggered_ability(text, def)).build();
    put_on_battlefield(&mut game, watcher, 0);
    let plain = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let countered = put_on_battlefield(&mut game, grizzly_bears(), 0);
    add_counters(&mut game, countered, CounterType::PlusOnePlusOne, 1);
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 1]);
    game.process_declare_attackers(&dp).unwrap();
    let subjects: Vec<_> = game.pending_triggers.iter().map(|t| t.binding.triggering_object(EventRole::Attacker).map(|o| o.id)).collect();
    assert_eq!(subjects, vec![Some(plain)], "one trigger, for the attacker with no counter");
}

/// A kicked spell's delayed trigger reads the spell as it left the stack
/// (CR 707.10, 113.7a): its entry was taken as it resolved, and the
/// resolving object holds its cost decisions until it is put away.
#[test]
fn delayed_trigger_if_its_spell_was_kicked_holds_after_the_spell_resolved() {
    let mut game = setup_two_player_game();
    let def = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::GainsLife {
            player: Some(PlayerRef::You),
            multiplicity: Multiplicity::PerOccurrence,
        }),
        intervening_if: Some(Condition::SpellWasKicked),
        limit: None,
        effect: Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(5)), EffectRecipient::Controller),
    };
    let template = DelayedTriggerTemplate {
        def: Arc::new(def),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "When you next gain life, if this spell was kicked, you lose 5 life.".into(),
    };
    let spell = CardDataBuilder::new("Kicked Spell").card_type(CardType::Instant).build();
    let id = put_spell_on_stack(&mut game, spell, 0);
    let entry = game.stack_entries.get_mut(&id).unwrap();
    entry.effect = Arc::new(Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller));
    entry.additional_costs_paid = vec![AdditionalCost::Kicker(Vec::new())];
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, id), Some(Zone::Graveyard));

    game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: id }, &test_ctx()).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "it was kicked");
}

/// "Its owner" in a gone source's condition is its last known information's
/// (CR 108.3): a token Aura's leaves trigger, rechecked after CR 704.5d
/// removed the token, asks whether its host is controlled by the Aura's
/// owner. The store has lost the Aura; its owner has not changed.
#[test]
fn token_aura_leaves_trigger_if_its_owner_controls_the_host_holds_after_the_token_is_gone() {
    let mut game = setup_two_player_game();
    let text = "When this Aura leaves the battlefield, if enchanted creature is controlled by this Aura's owner, you gain 1 life.";
    let def = TriggerDef {
        condition: TriggerCondition::Event(leaves_the_battlefield(TriggerSubject::ThisObject).into()),
        intervening_if: Some(Condition::HostMatches(ObjectFilter::ByController(PlayerRef::Owner))),
        limit: None,
        effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
    };
    let aura_card = CardDataBuilder::new("Token Aura")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .ability(triggered_ability(text, def))
        .build();
    let host = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let aura = put_on_battlefield(&mut game, aura_card, 0);
    make_token(&mut game, aura);
    assert!(game.attach(aura, host));

    destroy(&mut game, aura);
    resolve_all(&mut game, &test_dp());
    assert!(!game.objects.contains_key(&aura), "the token ceased to exist before its trigger resolved");
    assert_eq!(game.players[0].life_total, 21, "its owner, P0, controls the creature");
}

// ---------------------------------------------------------------------------
// 9. "Enchanted creature" names a permanent at the event
// ---------------------------------------------------------------------------

/// Spirit Link's trigger, as a fixture: "Enchant creature. Whenever
/// enchanted creature deals damage, you gain that much life."
fn spirit_link_fixture() -> Arc<CardData> {
    let text = "Whenever enchanted creature deals damage, you gain that much life.";
    let deals_damage = TriggerEvent::DamageDealt {
        source: TriggerSubject::Host,
        recipient: DamageRecipient::Any,
        combat: None,
        excess: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    CardDataBuilder::new("Spirit Link Fixture")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(deals_damage, Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringAmount), EffectRecipient::Controller)),
        ))
        .build()
}

/// A creature sets up "When you next gain life, this creature deals 2 damage
/// to each opponent", then is exiled and returned: the creature that comes
/// back is a new object (CR 400.7), and Spirit Link is put on it. When the
/// damage is dealt, the old creature deals it, as it last existed (CR
/// 608.2h). The creature Spirit Link enchants dealt no damage, so Spirit
/// Link does not trigger, though both creatures carry one id in this engine.
#[test]
fn enchanted_creature_deals_damage_does_not_see_the_creature_it_was_before_a_flicker() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let deals_2 = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
    );
    let template = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            TriggerEvent::GainsLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence },
            deals_2,
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "When you next gain life, this creature deals 2 damage to each opponent.".into(),
    };
    let provenance = DelayedProvenance {
        source: game.object_ref(creature).unwrap(),
        source_card: grizzly_bears(),
        controller: 0,
        created_by: None,
        x_value: None,
        turn: TriggerTurn::Any,
        resolution: None,
        cause: mtgsim::events::cause::EventCause::Harness,
    };
    game.create_delayed_trigger(&template, provenance, Referred::default()).unwrap();
    game.change_zone(creature, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.change_zone(creature, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    let link = put_on_battlefield(&mut game, spirit_link_fixture(), 0);
    assert!(game.attach(link, creature), "on the creature that came back");

    game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: creature }, &test_ctx()).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(game.players[1].life_total, 18, "the old creature dealt its 2 damage");
    assert_eq!(game.players[0].life_total, 21, "only the 1 life that set it off: Spirit Link did not trigger");
}
