//! RH-3: the record's new object, its applied replacements, and its moment
//! (`replacement-architecture.md` §2b.4–§2b.6).
//!
//! What a zone change's record holds and when it is read:
//! - the object the move made (CR 400.7e), which "it" and "them" find
//!   afterwards, and only in the zone it went to (CR 603.6c);
//! - the replacements that applied to it, in order (CR 614.5), which a "this
//!   way" and a "who exiled it" read;
//! - the instant each trigger reads it at: immediately after the event, before
//!   the riders its replacements queued (CR 603.4, 615.5).

use std::sync::Arc;

use mtgsim::cards::authoring::{another, dies, leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::engine::actions::{GameAction, ZoneChangeAct};
use mtgsim::engine::replacement::ReplacementInstanceId;
use mtgsim::events::cause::{EventCause, GeneratedBy};
use mtgsim::events::event::{DamageTarget, GameEvent, ResolutionStamp};
use mtgsim::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::controller_or_owner;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, effect_ctx, put_in_hand, put_in_library, put_on_battlefield, put_spell_on_stack,
    setup_two_player_game,
    static_ability, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, EnchantmentType, Subtype};
use mtgsim::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerFact, PlayerRef, PlayerSet,
    Primitive, ReturnUnder, SelectionFilter, TargetCount, TokenDef,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::restriction::SourceFilter;
use mtgsim::types::replacement::{EnterModsTemplate, EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::triggers::{EventRole, Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangePattern, ZoneSet};
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put every pending trigger on the stack.
fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

/// Place what triggered and resolve the stack down to empty.
fn resolve_everything(game: &mut GameState) {
    place(game, &test_dp());
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).expect("resolving");
        place(game, &test_dp());
    }
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// "If a creature you control would die, put it into its owner's graveyard,
/// then exile it." A replacement that leaves the death as it was, and whose
/// rider (CR 615.5) moves the creature again right after it died.
fn exiles_your_creatures_after_they_die() -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object: None,
        },
        ObjectSet::battlefield_filter(ObjectFilter::And(
            Box::new(a_creature()),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Graveyard }),
    )
    .with_then(Effect::Atom(
        Primitive::Exile,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    ));
    card_of_type_with(
        "Fixture: exiles your creatures right after they die",
        CardType::Enchantment,
        static_ability(Effect::Replacement(Box::new(replacement))),
    )
}

/// A card of one type with one ability.
fn card_of_type_with(name: &str, card_type: CardType, ability: mtgsim::objects::card_data::AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(card_type).ability(ability).build()
}

/// "When this creature dies, return it to the battlefield under its owner's
/// control."
fn returns_when_it_dies() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: returns when it dies",
        1,
        1,
        triggered_ability(
            "",
            whenever(
                dies(TriggerSubject::ThisObject),
                Effect::Atom(Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new(), tapped: false, attacking: None }, EffectRecipient::Triggering(EventRole::Object)),
            ),
        ),
    )
}

// ---------------------------------------------------------------------------
// The object the move made (CR 400.7e, 603.6c)
// ---------------------------------------------------------------------------

/// A rider moves the creature again before its dies trigger is dispatched.
/// "Return it" names the object its death made, in the graveyard, and CR
/// 603.6c looks for it only there: exiled since, it is a new object, and the
/// trigger does nothing. Read off the store at dispatch, "it" was the exiled
/// card, and it came back.
#[test]
fn a_rider_that_moves_the_dead_creature_again_leaves_return_it_nothing_to_find() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exiles_your_creatures_after_they_die(), 0);
    let returner = put_on_battlefield(&mut game, returns_when_it_dies(), 0);

    game.execute_action(GameAction::Destroy { object: returner }, &test_ctx()).unwrap();
    assert_eq!(game.get_object(returner).unwrap().zone, Zone::Exile, "the rider exiled it after it died");
    assert_eq!(game.pending_triggers.len(), 1, "its dies trigger");

    resolve_everything(&mut game);
    assert_eq!(
        game.get_object(returner).unwrap().zone,
        Zone::Exile,
        "CR 603.6c: the dies trigger looks for the card only in the graveyard, and the exiled card is a new object"
    );
}

/// "When this Aura is put into a graveyard from the battlefield, return it to
/// its owner's hand", shaped like Rancor.
fn aura_returning_to_hand() -> Arc<CardData> {
    CardDataBuilder::new("Fixture: returns to hand from the graveyard")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Permanent(a_creature()))
        .ability(triggered_ability(
            "",
            whenever(
                dies(TriggerSubject::ThisObject),
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Triggering(EventRole::Object)),
            ),
        ))
        .build()
}

/// The Aura goes to the graveyard by CR 704.5m once its creature is gone, and
/// its own trigger finds the new object it became there (CR 400.7e).
// COVERS: ATOM-400.7e-001
#[test]
fn an_auras_own_graveyard_trigger_finds_the_aura_it_became() {
    let mut game = setup_two_player_game();
    let host = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, aura_returning_to_hand(), 0);
    game.execute_action(GameAction::Attach { attachment: aura, host }, &test_ctx()).unwrap();

    game.execute_action(GameAction::Destroy { object: host }, &test_ctx()).unwrap();
    resolve_everything(&mut game);

    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Hand, "CR 400.7e: the trigger finds the Aura in the graveyard");
}

/// "When this is put into a graveyard from anywhere, shuffle it into its
/// owner's library", shaped like Vigor's.
fn shuffles_in_from_anywhere() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: shuffles in from anywhere",
        2,
        2,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::ThisObject,
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    by: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                Effect::Atom(Primitive::ShuffleIntoLibrary, EffectRecipient::Triggering(EventRole::Object)),
            ),
        ),
    )
}

/// CR 400.7e's non-example: a "from anywhere" trigger is matched after the
/// move (CR 603.6c), against the card already in the graveyard, and needs no
/// look-back to find it.
// COVERS: ATOM-400.7e-002
#[test]
fn a_from_anywhere_trigger_finds_the_card_where_it_is() {
    let mut game = setup_two_player_game();
    let vigor = put_on_battlefield(&mut game, shuffles_in_from_anywhere(), 0);

    game.execute_action(GameAction::Destroy { object: vigor }, &test_ctx()).unwrap();
    resolve_everything(&mut game);

    assert_eq!(game.get_object(vigor).unwrap().zone, Zone::Library);
}

// ---------------------------------------------------------------------------
// "Them" (CR 603.2c)
// ---------------------------------------------------------------------------

/// "Whenever one or more cards are put into your graveyard from your library,
/// return them to your hand", shaped like Hedge Shredder's "put them onto the
/// battlefield".
fn returns_milled_cards() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: returns them",
        1,
        1,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    change: ZoneChangePattern { from: Some(Zone::Library), to: Some(Zone::Graveyard), act: None },
                    by: None,
                    owner: Some(PlayerRef::You),
                    multiplicity: Multiplicity::OncePerEvent,
                },
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Triggering(EventRole::Object)),
            ),
        ),
    )
}

/// "Them" is each card its own record says the mill made, found while it is
/// still that card: one exiled from the graveyard before the trigger resolves
/// is a new object, and stays in exile.
#[test]
fn them_finds_each_card_by_its_own_record() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, returns_milled_cards(), 0);
    let first = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    let second = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    let third = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);

    let milled: Vec<ObjectId> = vec![first, second, third];
    game.execute_actions(
        milled
            .iter()
            .map(|&object| GameAction::ZoneChange {
                object,
                from: Zone::Library,
                to: Zone::Graveyard,
                act: ZoneChangeAct::Milled,
            })
            .collect(),
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "CR 603.2c: one trigger for the one event");

    place(&mut game, &test_dp());
    game.change_zone(second, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(game.get_object(first).unwrap().zone, Zone::Hand);
    assert_eq!(game.get_object(third).unwrap().zone, Zone::Hand);
    assert_eq!(game.get_object(second).unwrap().zone, Zone::Exile, "a new object since the mill (CR 400.7)");
}

// ---------------------------------------------------------------------------
// The replacements that applied (CR 614.5, 614.6)
// ---------------------------------------------------------------------------

/// "If you would discard this card, discard it into exile", the replacement
/// half of madness (CR 702.35a), functioning from its owner's hand (CR
/// 113.6b).
fn discards_into_exile() -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Hand), to: Some(Zone::Graveyard), act: Some(ZoneChangeAct::Discarded) },
            object: None,
        },
        ObjectSet::SourceOnly,
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    );
    let mut ability = static_ability(Effect::Conditional(
        Condition::SourceInZone(ZoneSet::HAND),
        Box::new(Effect::Replacement(Box::new(replacement))),
    ));
    ability.id = new_ability_id();
    creature_with_ability("Fixture: madness", 2, 2, ability)
}

/// "When this card is exiled this way" asks whether the card's own madness
/// applied to the discard that exiled it (CR 702.35a): the record keeps it,
/// with the act it kept (CR 614.6) and who it acted for.
#[test]
fn a_madness_discard_keeps_its_own_replacement_on_the_record() {
    let mut game = setup_two_player_game();
    let card = put_in_hand(&mut game, discards_into_exile(), 0);
    let before = game.object_ref(card).unwrap();
    let madness = game.get_object(card).unwrap().card_data.abilities[0].id;

    game.change_zone(card, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();

    let recorded = game.recorded_events();
    let record = recorded
        .records()
        .iter()
        .rev()
        .find(|r| matches!(r.event, GameEvent::ZoneChange { ref departed, .. } if departed.object().id == card))
        .expect("the discard's record")
        .clone();
    let GameEvent::ZoneChange { to, act, applied, .. } = &record.event else { unreachable!() };
    assert_eq!((*to, *act), (Zone::Exile, ZoneChangeAct::Discarded), "discarded into exile");
    let applied = applied.as_deref().expect("a replacement applied");
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].instance, ReplacementInstanceId::StaticAbility(before, madness), "this card's own madness");
    assert!(applied[0].redirected, "it changed where the card went");
    assert_eq!(
        applied[0].by,
        EventCause::Effect {
            resolution: ResolutionStamp { source: card, controller: 0 },
            generated_by: GeneratedBy::StaticAbility,
        }
    );
}

/// "Whenever a spell or ability you control exiles one or more permanents",
/// shaped like Hero of Bretagard's battlefield half.
fn exile_watcher() -> Arc<CardData> {
    card_of_type_with(
        "Fixture: exiles watcher",
        CardType::Enchantment,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Exile), act: None },
                    by: Some(SourceFilter::ControlledBy(PlayerRef::You)),
                    owner: None,
                    multiplicity: Multiplicity::OncePerEvent,
                },
                Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            ),
        ),
    )
}

/// Player 0 watches; `rest_in_peace` is whose Rest in Peace is out, if any;
/// `exiler` is whose effect removes a creature, by destroying it or, with no
/// Rest in Peace, by exiling it. The count of the watcher's triggers.
fn exiles_seen(rest_in_peace: Option<PlayerId>, exiler: PlayerId) -> usize {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exile_watcher(), 0);
    if let Some(player) = rest_in_peace {
        put_on_battlefield(&mut game, mtgsim::cards::phase_rb_cards::rest_in_peace(), player);
    }
    let spell = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), exiler);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.pending_triggers.clear();
    let ctx = effect_ctx(&game, spell);
    match rest_in_peace {
        Some(_) => game.execute_action(GameAction::Destroy { object: victim }, &ctx).unwrap(),
        None => game.change_zone(victim, Zone::Exile, ZoneChangeAct::Put, &ctx).unwrap(),
    }
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    game.pending_triggers.len()
}

/// Who exiled a creature Rest in Peace sent to exile is the resolving spell's
/// controller, whoever controls the Rest in Peace: the spell "is causing
/// everything to happen, even if some of it gets replaced" (judges, 2026-10-13,
/// reversing #242's review; `replacement-architecture.md` §2b.4).
#[test]
fn who_exiled_a_redirected_move_is_the_resolving_spells_controller() {
    assert_eq!(exiles_seen(None, 0), 1, "your own exile");
    assert_eq!(exiles_seen(None, 1), 0, "an opponent's exile");
    assert_eq!(exiles_seen(Some(1), 0), 1, "your destroy, which an opponent's Rest in Peace exiled");
    assert_eq!(exiles_seen(Some(0), 1), 0, "an opponent's destroy, which your Rest in Peace exiled");
}

/// The record grew by what it holds and not by a byte (§2b.10's pricing).
/// Its widest payload is a targeting's, which names its spell or ability
/// and that ability's source each by existence.
#[test]
fn the_event_keeps_its_size() {
    assert_eq!(std::mem::size_of::<GameEvent>(), 80);
}

// ---------------------------------------------------------------------------
// The moment: immediately after the event, before its riders (CR 603.4, 615.5)
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// The triggers `source` has waiting to be put on the stack.
fn pending_from(game: &GameState, source: ObjectId) -> Vec<PlayerId> {
    game.pending_triggers.iter().filter(|t| t.origin.source() == source).map(|t| t.controller).collect()
}

/// A 2/2 creature token with `abilities`.
fn token_with(abilities: Vec<AbilityDef>) -> TokenDef {
    TokenDef {
        name: Some("Fixture Token".into()),
        colors: Vec::new(),
        types: vec![CardType::Creature],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(2),
        keyword_flags: Vec::new(),
        abilities,
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// "If a creature an opponent controls would die, exile it instead and
/// create [token]": Kalitas's shape on an enchantment, so its controller can
/// control no creature.
fn exiler_making(token: TokenDef) -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object: None,
        },
        ObjectSet::battlefield_filter(ObjectFilter::And(
            Box::new(a_creature()),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        )),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    )
    .with_then(Effect::Atom(Primitive::CreateToken(token, AmountExpr::Fixed(1)), EffectRecipient::Controller));
    card_of_type_with(
        "Fixture: exiles and makes a token",
        CardType::Enchantment,
        static_ability(Effect::Replacement(Box::new(replacement))),
    )
}

/// "If a source would deal damage to a creature, prevent that damage, then
/// [verb] that creature": a rider (CR 615.5) acting on the creature right
/// after the event it rode on.
fn prevents_then(verb: Primitive) -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::DealDamage { source: None, combat: None },
        ObjectSet::battlefield_filter(a_creature()),
        Rewrite::Prevent,
    )
    .with_then(Effect::Atom(
        verb,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    ));
    card_of_type_with(
        "Fixture: prevents damage to a creature, then acts on it",
        CardType::Enchantment,
        static_ability(Effect::Replacement(Box::new(replacement))),
    )
}

/// One damage from `source` to `creature`, proposed with `others` as one
/// event.
fn with_one_damage(others: Vec<GameAction>, source: ObjectId, creature: ObjectId) -> Vec<GameAction> {
    let mut batch = others;
    batch.push(GameAction::DealDamage {
        source,
        source_lki: None,
        target: DamageTarget::Object(creature),
        amount: 1,
        is_combat: false,
        unpreventable: false,
    });
    batch
}

/// A permanent a rider made did not exist when the event it rides on
/// happened, so it does not look back at that event (CR 603.10a): its
/// leaves-the-battlefield ability sees nothing. Read at the window's close,
/// the token saw the death before it.
#[test]
fn a_permanent_a_rider_makes_does_not_look_back_at_the_event_before_it() {
    let mut game = setup_two_player_game();
    let watches = triggered_ability("", whenever(leaves_the_battlefield(a_creature()), gain_one()));
    put_on_battlefield(&mut game, exiler_making(token_with(vec![watches])), 0);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_action(GameAction::Destroy { object: victim }, &test_ctx()).unwrap();
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert_eq!(game.pending_triggers.len(), 0, "the token was made after the creature left");
}

/// The same newcomer's ordinary arm (CR 603.6a): what it asks is the board
/// immediately after the event, where it was not.
#[test]
fn a_permanent_a_rider_makes_does_not_see_the_event_before_it() {
    let mut game = setup_two_player_game();
    let watches = triggered_ability(
        "",
        whenever(
            TriggerEvent::ZoneChange {
                subject: TriggerSubject::Any,
                change: ZoneChangePattern { from: None, to: Some(Zone::Exile), act: None },
                by: None,
                owner: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
            gain_one(),
        ),
    );
    put_on_battlefield(&mut game, exiler_making(token_with(vec![watches])), 0);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_action(GameAction::Destroy { object: victim }, &test_ctx()).unwrap();
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert_eq!(game.pending_triggers.len(), 0, "the token was made after the creature was exiled");
}

/// A permanent that left in the event is not there when its rider destroys
/// another creature, so its last known information answers only the event it
/// left in (CR 603.10a). Read at the window's close, it saw the later death.
#[test]
fn a_permanent_that_left_does_not_look_back_at_a_death_after_it() {
    let mut game = setup_two_player_game();
    let watcher = put_on_battlefield(
        &mut game,
        creature_with_ability("Fixture: another dies", 1, 1, triggered_ability("", whenever(dies(another(a_creature())), gain_one()))),
        0,
    );
    put_on_battlefield(&mut game, prevents_then(Primitive::Destroy), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_actions(with_one_damage(vec![GameAction::Destroy { object: watcher }], source, later), &test_ctx())
        .unwrap();
    assert_eq!(game.get_object(later).unwrap().zone, Zone::Graveyard, "the rider destroyed it");
    assert!(pending_from(&game, watcher).is_empty(), "it had left before the rider's death");
}

/// Decision 5's reading of CR 615.5's "immediately afterward": a rider's
/// records are an event after the one it rides on, so "whenever one or more
/// creatures die" sees two (`replacement-architecture.md` §2b.11's open
/// question).
#[test]
fn one_or_more_sees_an_event_and_its_rider_as_two_events() {
    let mut game = setup_two_player_game();
    let watcher = put_on_battlefield(
        &mut game,
        card_of_type_with(
            "Fixture: one or more die",
            CardType::Enchantment,
            triggered_ability("", whenever(dies(a_creature()).once_per_event(), gain_one())),
        ),
        0,
    );
    put_on_battlefield(&mut game, prevents_then(Primitive::Destroy), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let first = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let second = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_actions(with_one_damage(vec![GameAction::Destroy { object: first }], source, second), &test_ctx())
        .unwrap();
    assert_eq!(game.get_object(second).unwrap().zone, Zone::Graveyard);
    assert_eq!(pending_from(&game, watcher).len(), 2, "the death, then the rider's");
}

/// CR 603.4 checks an intervening "if" when the trigger event occurs, before
/// the rider's Zombie enters: player 0 controlled no creature then. Read at
/// the window's close, the Zombie made the "if" true.
#[test]
fn the_intervening_if_is_checked_when_the_event_occurs() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exiler_making(token_with(Vec::new())), 0);
    let mut def = whenever(
        leaves_the_battlefield(ObjectFilter::And(
            Box::new(a_creature()),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        )),
        gain_one(),
    );
    def.intervening_if = Some(Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(a_creature()) });
    let watcher = put_on_battlefield(
        &mut game,
        card_of_type_with("Fixture: if you control a creature", CardType::Enchantment, triggered_ability("", def)),
        0,
    );
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_action(GameAction::Destroy { object: victim }, &test_ctx()).unwrap();
    assert_eq!(game.battlefield.len(), 3, "the exiler, the watcher and the Zombie");
    assert!(pending_from(&game, watcher).is_empty(), "no creature was player 0's when the creature left");
}

/// "Creatures enter tapped, then return that creature to its owner's hand."
fn enters_tapped_then_bounces() -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::battlefield_filter(a_creature()),
        Rewrite::EnterWith(EnterModsTemplate::tapped()),
    )
    .with_then(Effect::Atom(
        Primitive::ReturnToHand,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    ));
    card_of_type_with(
        "Fixture: enters tapped, then bounces",
        CardType::Enchantment,
        static_ability(Effect::Replacement(Box::new(replacement))),
    )
}

/// "Whenever a creature enters, if it was cast" reads the permanent as it
/// entered (CR 400.7d): a rider bouncing it right after takes nothing back.
/// Read at the window's close, the permanent was gone, and so was its cast.
#[test]
fn was_it_cast_reads_the_permanent_as_it_entered() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, enters_tapped_then_bounces(), 0);
    let watcher = put_on_battlefield(
        &mut game,
        card_of_type_with(
            "Fixture: a cast creature enters",
            CardType::Enchantment,
            triggered_ability(
                "",
                whenever(
                    TriggerEvent::EntersBattlefield {
                        subject: TriggerSubject::Filter(a_creature()),
                        controller: None,
                        from: None,
                        was_cast: Some(true),
                        multiplicity: Multiplicity::PerOccurrence,
                    },
                    gain_one(),
                ),
            ),
        ),
        0,
    );
    let bear = put_spell_on_stack(&mut game, vanilla_creature(2, 2, &[]), 0);

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Hand, "the rider bounced it");
    assert_eq!(pending_from(&game, watcher).len(), 1, "it was cast, and it entered");
}

/// CR 603.2f asks visibility of the object as the event left it: in the
/// graveyard, public. A rider returning it to a hand right after hides it
/// only afterwards. Read at the window's close, the card was in a hand.
#[test]
fn a_source_hidden_after_the_event_is_seen_as_the_event_left_it() {
    let mut game = setup_two_player_game();
    let returner = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object: None,
        },
        ObjectSet::battlefield_filter(a_creature()),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Graveyard }),
    )
    .with_then(Effect::Atom(
        Primitive::ReturnToHand,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    ));
    put_on_battlefield(
        &mut game,
        card_of_type_with(
            "Fixture: dying creatures go to a hand right after",
            CardType::Enchantment,
            static_ability(Effect::Replacement(Box::new(returner))),
        ),
        0,
    );
    let card = put_on_battlefield(&mut game, gains_when_put_into_a_graveyard(), 0);

    game.execute_action(GameAction::Destroy { object: card }, &test_ctx()).unwrap();
    assert_eq!(game.get_object(card).unwrap().zone, Zone::Hand, "the rider returned it");
    assert_eq!(pending_from(&game, card).len(), 1, "it was in a graveyard, public, when it triggered");
}

/// "When this is put into a graveyard from anywhere, you gain 1 life."
fn gains_when_put_into_a_graveyard() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: gains from anywhere",
        2,
        2,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::ThisObject,
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    by: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            ),
        ),
    )
}

/// CR 603.3a: a triggered ability's controller is its source's when it
/// triggered. A rider that steals the source right after the event steals it
/// too late. Read at the window's close, the thief controlled the trigger.
#[test]
fn a_trigger_is_controlled_by_its_sources_controller_when_it_triggered() {
    let mut game = setup_two_player_game();
    let watcher = put_on_battlefield(
        &mut game,
        creature_with_ability("Fixture: a creature dies", 1, 1, triggered_ability("", whenever(dies(a_creature()), gain_one()))),
        0,
    );
    put_on_battlefield(&mut game, prevents_then(Primitive::GainControl(Duration::UntilEndOfTurn)), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    game.execute_actions(with_one_damage(vec![GameAction::Destroy { object: victim }], source, watcher), &test_ctx())
        .unwrap();
    assert_eq!(controller_or_owner(&game, watcher), Some(1), "the rider stole it (CR 613.1b)");
    assert_eq!(pending_from(&game, watcher), vec![0], "player 0 controlled it when the creature died");
}
