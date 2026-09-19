//! Phase RE-7 — leaving the game.
//!
//! CR 800.4a's four clauses inside the `PlayerLoses` performer, CR 800.4b and
//! 800.4d's refusals at the sites that would have created or moved the object,
//! CR 800.4c at the two moments a control-changing effect can end, and
//! CR 800.4m's turn that never begins.
//!
//! **Every board here has more than two seats**, and that is CR 800.1 rather
//! than a testing convention: "a multiplayer game is a game that begins with
//! more than two players", and CR 800.4's own first sentence says the section
//! is about what two-player games cannot do. The one two-player board is the
//! test that says so.
//!
//! The consumer is Act of Treason, both ways round — the rule's own second
//! example — plus a Mind-Control-shaped Aura for its first, and a permanent
//! entered under a controller who does not own it for Bribery's.

use std::sync::Arc;

use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::events::event::{DamageTarget, GameEvent, LossReason};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::game_state::{GameResult, GameState, Phase, PhaseType, StackEntry, StepType};
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield,
    put_on_battlefield_under, set_attacking, set_blocked_by, set_blocking, setup_game, test_ctx,
    test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, EnchantmentType, Subtype};
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
    TokenDef,
};
use mtgsim::types::colors::Color;
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::{ChosenTargets, TargetInstance};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// One state-based check; reports whether it did anything (CR 704.3).
fn sba(game: &mut GameState, dp: &dyn DecisionProvider) -> bool {
    game.check_state_based_actions(dp).expect("checking state-based actions")
}

/// Make `player` lose the game at the next check — CR 704.5a, the reason every
/// board here happens to use.
fn departs(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) {
    game.players[player].life_total = 0;
    assert!(sba(game, dp), "the loss is a performed state-based action");
    assert!(game.player_lost[player]);
}

/// Every `LeftTheGame` this game has recorded, in order.
fn departures(game: &GameState) -> Vec<(ObjectId, PlayerId, Zone)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::LeftTheGame { object_id, owner, from, .. } => {
                Some((*object_id, *owner, *from))
            }
            _ => None,
        })
        .collect()
}

/// Resolve `effect` for `controller` against `targets`, the way a spell would,
/// and return the fixture spell's id — which is the effect's source, so a
/// Layer 2 row it writes can be named.
fn resolve_with(
    game: &mut GameState,
    controller: PlayerId,
    effect: &Effect,
    targets: Vec<ResolvedTarget>,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let source = put_in_hand(game, CardDataBuilder::new("Fixture").build(), controller);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
    source
}

/// Act of Treason's first clause, resolved: `thief` gains control of `creature`
/// until end of turn. The Layer 2 row's source is the sorcery, which never
/// reaches the battlefield — so nothing but CR 800.4a's second clause can end
/// it early.
fn steal(game: &mut GameState, creature: ObjectId, thief: PlayerId) -> ObjectId {
    let effect = Effect::Atom(
        Primitive::GainControl(Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_with(game, thief, &effect, vec![ResolvedTarget::Object(creature)], &test_dp())
}

/// Aethersnatch's shape: a control-change effect with **no duration**, so
/// nothing in CR 514.2's cleanup ends it and CR 800.4a's second clause is the
/// only thing that can.
fn steal_indefinitely(game: &mut GameState, creature: ObjectId, thief: PlayerId) -> ObjectId {
    let effect = Effect::Atom(
        Primitive::GainControl(Duration::Indefinite),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_with(game, thief, &effect, vec![ResolvedTarget::Object(creature)], &test_dp())
}

/// Mind Control, in the shape CR 800.4a's first example needs: an Aura whose
/// static ability gives its controller control of the enchanted creature.
fn mind_control() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Mind Control")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GainControl(Duration::WhileSourceOnBattlefield),
                EffectRecipient::Host,
            ),
        })
        .build()
}

/// Put a card straight into the exile zone.
fn put_in_exile(game: &mut GameState, card: Arc<CardData>, owner: PlayerId) -> ObjectId {
    let obj = GameObject::new(card, owner, Zone::Exile);
    let id = game.add_object(obj);
    game.exile.push(id);
    id
}

/// Put a card straight into the command zone.
fn put_in_command(game: &mut GameState, card: Arc<CardData>, owner: PlayerId) -> ObjectId {
    let obj = GameObject::new(card, owner, Zone::Command);
    let id = game.add_object(obj);
    game.command.push(id);
    id
}

/// A stack object with an owner and a controller that need not agree, and a
/// flag for whether it is a card.
///
/// Which is the shape CR 800.4a's third clause is written for: an ability
/// (`is_spell: false`) is not represented by a card, and a copy of a spell is
/// owned by whoever made it. No production path produces the split yet — an
/// activated ability carries its activator as owner — so this is what proves
/// the clause is wired rather than dead.
fn stack_object(
    game: &mut GameState,
    owner: PlayerId,
    controller: PlayerId,
    is_spell: bool,
) -> ObjectId {
    stack_object_with(game, owner, controller, is_spell, Effect::Sequence(Vec::new()), Vec::new())
}

/// [`stack_object`] with instructions and the targets CR 601.2c recorded.
fn stack_object_with(
    game: &mut GameState,
    owner: PlayerId,
    controller: PlayerId,
    is_spell: bool,
    effect: Effect,
    chosen_targets: Vec<ResolvedTarget>,
) -> ObjectId {
    let recipient = if chosen_targets.is_empty() {
        EffectRecipient::Implicit
    } else {
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1))
    };
    let obj = GameObject::new(CardDataBuilder::new("Fixture Stack Object").build(), owner, Zone::Stack);
    let id = game.add_object(obj);
    game.stack.push(id);
    game.set_stack_entry(StackEntry {
        object_id: id,
        controller,
        chosen_targets: vec![TargetInstance::new(recipient, chosen_targets)],
        chosen_modes: Vec::new(),
        x_value: None,
        effect,
        is_spell,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
        cast_from: if is_spell { Some(Zone::Hand) } else { None },
        ability_identity: None,
        trigger: None,
    });
    id
}

/// A 2/2 black Bear, for the token the departed player does not get.
fn bear_token() -> TokenDef {
    TokenDef {
        name: Some("Bear".to_string()),
        colors: vec![Color::Black],
        types: vec![CardType::Creature],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(2),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

/// Wind the turn on to the next unit — the drainer returns when one begins.
fn advance(game: &mut GameState) {
    game.advance_turn(&test_ctx()).expect("advancing the turn");
}

// ---------------------------------------------------------------------------
// CR 800.4a, first clause — every object they own leaves the game
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4a-001
//
// The atom's departure is a concession, which no harness offers (CR 104.3a is
// a *leave* and not a proposed loss); a state-based loss is the same departure
// for CR 800.4a's purposes. Its Mind Control half is the next test.
#[test]
fn every_object_a_departing_player_owns_leaves_the_game_from_every_zone() {
    let mut game = setup_game(4);
    let permanent = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let spell = stack_object(&mut game, 1, 1, true);
    let exiled = put_in_exile(&mut game, CardDataBuilder::new("Exiled").build(), 1);
    let commanded = put_in_command(&mut game, CardDataBuilder::new("Commanded").build(), 1);
    let dead = put_in_graveyard(&mut game, CardDataBuilder::new("Dead").build(), 1);
    let held = put_in_hand(&mut game, CardDataBuilder::new("Held").build(), 1);
    fill_library(&mut game, 1, 3);
    let library: Vec<ObjectId> = game.players[1].library.clone();

    // A bystander in each shared zone, to show the sweep is keyed on ownership.
    let others_permanent = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 2);
    let others_spell = stack_object(&mut game, 2, 2, true);

    departs(&mut game, 1, &test_dp());

    for id in [permanent, spell, exiled, commanded, dead, held].into_iter().chain(library) {
        assert!(!game.objects.contains_key(&id), "object {id} is still in the game");
    }
    assert!(game.players[1].hand.is_empty());
    assert!(game.players[1].library.is_empty());
    assert!(game.players[1].graveyard.is_empty());
    assert_eq!(game.battlefield.keys().copied().collect::<Vec<_>>(), vec![others_permanent]);
    assert_eq!(game.stack, vec![others_spell]);
    assert!(game.exile.is_empty());
    assert!(game.command.is_empty());

    // One event per object, and the board first: the log is the only witness to
    // an order, so this pins the one `owned_objects_leave` chose.
    let zones: Vec<Zone> = departures(&game).iter().map(|&(_, _, from)| from).collect();
    assert_eq!(
        zones,
        vec![
            Zone::Battlefield,
            Zone::Stack,
            Zone::Exile,
            Zone::Command,
            Zone::Graveyard,
            Zone::Hand,
            Zone::Library,
            Zone::Library,
            Zone::Library,
        ]
    );
    assert!(departures(&game).iter().all(|&(_, owner, _)| owner == 1));

    // CR 603.6c — "leaves-the-battlefield abilities trigger ... when a
    // phased-in permanent leaves the game because its owner leaves the game" —
    // so the CR 603.10a frame rides the event, and only for the permanent.
    let frames: Vec<(Zone, bool)> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::LeftTheGame { from, lki, .. } => Some((*from, lki.is_some())),
            _ => None,
        })
        .collect();
    assert_eq!(frames[0], (Zone::Battlefield, true), "the permanent carries its frame");
    assert!(
        frames[1..].iter().all(|&(_, has)| !has),
        "and nothing else does: no other zone has a permanent to look back at"
    );
    let lki = game.events.events().find_map(|e| match e {
        GameEvent::LeftTheGame { object_id, lki, .. } if *object_id == permanent => lki.as_ref(),
        _ => None,
    });
    assert_eq!(lki.expect("a frame").power, Some(2), "and it is the permanent as it was");
}

/// CR 603.6c's frame is the *effective* one, not the printed card — the same
/// claim `perform_zone_change`'s LKI makes, on the one event that is not a zone
/// change. A departing player's creature under somebody else's Aura leaves the
/// game as the creature the board had made it.
#[test]
fn the_frame_a_permanent_leaves_the_game_with_is_the_one_the_board_made() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let pump = Effect::Atom(
        Primitive::ModifyPowerToughness(
            AmountExpr::Fixed(3),
            AmountExpr::Fixed(3),
            Duration::UntilEndOfTurn,
        ),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_with(&mut game, 0, &pump, vec![ResolvedTarget::Object(bears)], &test_dp());

    departs(&mut game, 1, &test_dp());

    let lki = game.events.events().find_map(|e| match e {
        GameEvent::LeftTheGame { object_id, lki, .. } if *object_id == bears => lki.as_ref(),
        _ => None,
    });
    let lki = lki.expect("a frame");
    assert_eq!((lki.power, lki.toughness), (Some(5), Some(5)));
}

/// CR 800.4a's first example, second half: *"If, instead, Bianca leaves the
/// game, so does Assault Griffin, and Mind Control is put into Alex's
/// graveyard."* Ownership is absolute — the creature leaves whoever controls
/// it — and the Aura, left attached to nothing, is CR 704.5m's from there.
#[test]
fn a_creature_leaves_the_game_with_its_owner_even_while_someone_else_controls_it() {
    let mut game = setup_game(4);
    let griffin = put_on_battlefield(&mut game, vanilla_creature(3, 1, &[]), 1);
    let aura = put_on_battlefield(&mut game, mind_control(), 0);
    game.attach(aura, griffin);
    assert_eq!(get_effective_controller(&game, griffin), Some(0), "P0 has stolen it");

    departs(&mut game, 1, &test_dp());

    assert!(!game.objects.contains_key(&griffin), "CR 800.4a: its owner left");
    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Battlefield, "P0 still owns the Aura");
    assert_eq!(game.battlefield[&aura].attached_to, None, "its host is gone");

    // 704.5m finishes the example on the next check.
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// CR 800.4a, second clause — the control they were given ends
// ---------------------------------------------------------------------------

/// CR 800.4a's first example, first half: *"If Alex leaves the game, so does
/// Mind Control, and Assault Griffin reverts to Bianca's control."* The Aura
/// leaves by clause 1, and its row goes with its source — so clause 2 has
/// nothing left to do here, and that is the point of the pair.
#[test]
fn an_aura_leaving_with_its_owner_hands_the_enchanted_creature_back() {
    let mut game = setup_game(4);
    let griffin = put_on_battlefield(&mut game, vanilla_creature(3, 1, &[]), 1);
    let aura = put_on_battlefield(&mut game, mind_control(), 0);
    game.attach(aura, griffin);
    assert_eq!(get_effective_controller(&game, griffin), Some(0));

    departs(&mut game, 0, &test_dp());

    assert!(!game.objects.contains_key(&aura));
    assert_eq!(game.get_object(griffin).unwrap().zone, Zone::Battlefield);
    assert_eq!(
        get_effective_controller(&game, griffin),
        Some(1),
        "CR 800.4a: the creature reverts to its owner rather than being exiled"
    );
}

/// CR 800.4a's second example, verbatim: *"Alex casts Act of Treason ...
/// targeting Bianca's Runeclaw Bears. If Alex leaves the game, Act of Treason's
/// change-of-control effect ends and Runeclaw Bears reverts to Bianca's
/// control."*
///
/// The row this ends is the residual clause 1 cannot reach: its source is a
/// sorcery in a graveyard, so no `remove_by_source` will ever come for it.
#[test]
fn the_thief_leaving_ends_the_steal_and_the_creature_goes_home() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    steal(&mut game, bears, 0);
    assert_eq!(get_effective_controller(&game, bears), Some(0));

    departs(&mut game, 0, &test_dp());

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(1));
    assert!(
        !game.continuous_effects.iter().any(|e| e.controller == 0),
        "the row ended rather than being left pointing at a seat the game does not have"
    );
}

/// A control-change effect with **no duration** — Aethersnatch's — ends here
/// and nowhere else, which is the case CR 800.4a's second clause exists for.
/// Nothing in CR 514.2's cleanup would touch it, so if the departure did not
/// end it the creature would stay stolen for the rest of the game.
#[test]
fn a_control_effect_with_no_duration_ends_when_the_player_it_favours_leaves() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    steal_indefinitely(&mut game, bears, 0);
    assert_eq!(get_effective_controller(&game, bears), Some(0));

    // A cleanup first, to show the duration really is the thing that does not
    // end it.
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });
    advance(&mut game);
    assert_eq!(game.phase.step, Some(StepType::Cleanup));
    assert_eq!(get_effective_controller(&game, bears), Some(0), "no duration, nothing ended");

    departs(&mut game, 0, &test_dp());

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(1));
}

/// The other half of the same rule, and the one a judge answer turns on: when
/// the *default* controller leaves and the no-duration effect survives them,
/// the thief keeps the creature. CR 800.4c never fires, because the effect
/// that gives control never ends.
#[test]
fn a_no_duration_effect_keeps_the_creature_when_its_default_controller_leaves() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 1, 0);
    steal_indefinitely(&mut game, bears, 2);

    departs(&mut game, 0, &test_dp());
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });
    advance(&mut game);
    assert_eq!(game.phase.step, Some(StepType::Cleanup));

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(2), "P2's effect never ended");
}

/// The same board with the *owner* leaving: the creature goes with them, and
/// the thief keeps nothing (CR 800.4a's first clause again, through a
/// resolution's row rather than an Aura's).
#[test]
fn the_owner_leaving_takes_the_stolen_creature_with_them() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    steal(&mut game, bears, 0);

    departs(&mut game, 1, &test_dp());

    assert!(!game.objects.contains_key(&bears));
    assert!(game.battlefield.is_empty());
}

// ---------------------------------------------------------------------------
// CR 800.4a, third clause — stack objects not represented by cards
// ---------------------------------------------------------------------------

/// The residual clause 1 leaves: an object on the stack that is not a card and
/// that the departing player controls without owning.
///
/// Its printed customer is a copy of a spell (CR 707.10), which CV-4 will
/// produce — `copy-effects-architecture.md` calls it "`is_copy`'s first
/// writer"; today no production path separates an ability's owner from its
/// controller, so clause 1 takes every one. What this pins is that the clause
/// is wired — and that it announces nothing, because the object was never the
/// departing player's to leave with.
#[test]
fn a_stack_object_that_is_not_a_card_ceases_to_exist_for_its_controller() {
    let mut game = setup_game(4);
    let ability = stack_object(&mut game, 2, 1, false);
    let card_spell = stack_object(&mut game, 2, 1, true);

    departs(&mut game, 1, &test_dp());

    assert!(!game.objects.contains_key(&ability), "CR 800.4a: it ceased to exist");
    assert!(
        !departures(&game).iter().any(|&(id, _, _)| id == ability),
        "ceasing to exist is not leaving the game, and P2 still owns it"
    );
    assert_eq!(
        game.get_object(card_spell).unwrap().zone,
        Zone::Exile,
        "a spell *is* represented by a card, so it takes the fourth clause instead"
    );
}

/// An activated ability on the stack carries its activator as its owner, so it
/// leaves the game by clause 1 — which is why clause 3 finds nothing today.
#[test]
fn an_ability_the_departing_player_activated_leaves_the_game_by_the_first_clause() {
    let mut game = setup_game(4);
    let ability = stack_object(&mut game, 1, 1, false);

    departs(&mut game, 1, &test_dp());

    assert!(!game.objects.contains_key(&ability));
    assert_eq!(
        departures(&game).iter().filter(|&&(id, _, _)| id == ability).count(),
        1,
        "announced once, as an object its owner took with them"
    );
}

// ---------------------------------------------------------------------------
// CR 800.4a, fourth clause — what they still control is exiled
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4a-002
//
// The atom's board is Bribery's, which CR 800.4a's third example writes out:
// the Serra Angel is owned by Bianca and entered under Alex's control, so
// clause 1 leaves it alone and clause 2 has no row to end.
#[test]
fn a_permanent_the_departing_player_controls_but_does_not_own_is_exiled() {
    let mut game = setup_game(4);
    let angel = put_on_battlefield_under(&mut game, vanilla_creature(4, 4, &[]), 1, 0);
    assert_eq!(get_effective_controller(&game, angel), Some(0));

    departs(&mut game, 0, &test_dp());

    assert_eq!(game.get_object(angel).unwrap().zone, Zone::Exile);
    let causes: Vec<ZoneChangeCause> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, cause, .. } if *object_id == angel => Some(*cause),
            _ => None,
        })
        .collect();
    assert_eq!(causes, vec![ZoneChangeCause::ControllerLeftTheGame]);
}

/// The fourth clause does not fire while somebody still in the game controls
/// the object — which is the whole of CR 800.4c's "there is no other effect
/// giving control of that object to another player in the game", asked as the
/// layer walk's own answer rather than as a second test.
#[test]
fn a_permanent_someone_still_playing_controls_is_not_exiled() {
    let mut game = setup_game(4);
    let angel = put_on_battlefield_under(&mut game, vanilla_creature(4, 4, &[]), 1, 0);
    steal(&mut game, angel, 2);

    departs(&mut game, 0, &test_dp());

    assert_eq!(game.get_object(angel).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, angel), Some(2));
}

// ---------------------------------------------------------------------------
// CR 800.4c — the same predicate when a control effect ends later
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4c-001
//
// The atom's board said the creature's *owner* leaves, which cannot reach this
// rule: CR 800.4a's first clause takes an object whose owner leaves, whoever
// controls it, and the CR's own Bribery example says so. The board here is
// session-1's Gonti scenario and the corrected atom's: the default controller
// leaves while a third player holds the creature on a duration.
#[test]
fn a_control_effect_ending_with_its_default_controller_gone_exiles_the_object() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 1, 0);
    steal(&mut game, bears, 2);

    departs(&mut game, 0, &test_dp());
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "P2 still has it");

    // Cleanup: Act of Treason's row expires, and there is no player left who
    // could control the creature.
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });
    advance(&mut game);
    assert_eq!(game.phase.step, Some(StepType::Cleanup));

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Exile);
}

/// The same board with the default controller still playing: the creature goes
/// back to them, which is CR 613.1b and not CR 800.4c.
#[test]
fn a_control_effect_ending_with_its_default_controller_playing_hands_the_object_back() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 1, 0);
    steal(&mut game, bears, 2);

    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) });
    advance(&mut game);
    assert_eq!(game.phase.step, Some(StepType::Cleanup));

    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(0));
}

// ---------------------------------------------------------------------------
// CR 800.4b and 800.4d — the refusals
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4b-001
// COVERS-PARTIAL: ATOM-800.4d-001
//
// The partial is the atom's second sentence, a delayed triggered ability that
// is not put onto the stack: there are no triggered abilities to refuse until
// CR 603. Its first sentence is this test — CR 111.2 makes a token's owner the
// player who controls the effect that created it, so 800.4b's token clause and
// 800.4d's creation clause name one player here.
#[test]
fn no_token_is_created_under_the_control_of_a_player_who_has_left() {
    let mut game = setup_game(4);
    departs(&mut game, 1, &test_dp());
    let before = game.objects.len();

    let effect = Effect::Atom(
        Primitive::CreateToken(bear_token(), AmountExpr::Fixed(2)),
        EffectRecipient::Controller,
    );
    resolve_with(&mut game, 1, &effect, vec![], &test_dp());

    assert!(game.battlefield.is_empty(), "no token is created");
    assert_eq!(
        game.objects.len(),
        before + 1,
        "and nothing but the fixture spell was added to the game"
    );
}

/// CR 800.4b's first sentence — "if an object would change to the control of a
/// player who has left the game, it doesn't".
///
/// The row is written and kept: CR 800.4a's second clause ends the rows in the
/// *departed* player's favor that existed when they left, and this one did
/// not. What the rule denies is the change, which is Layer 2's answer.
#[test]
fn an_object_does_not_change_to_the_control_of_a_player_who_has_left() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 2);
    departs(&mut game, 1, &test_dp());

    steal(&mut game, bears, 1);

    assert_eq!(
        get_effective_controller(&game, bears),
        Some(2),
        "the creature stays with its owner"
    );
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "and is not exiled");
}

// ---------------------------------------------------------------------------
// CR 800.4e — combat damage
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4e-001
#[test]
fn combat_damage_is_not_assigned_to_a_player_who_has_left() {
    let mut game = setup_game(4);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    set_attacking(&mut game, attacker, 1);
    departs(&mut game, 1, &test_dp());

    let assignments =
        mtgsim::engine::combat::resolution::assign_combat_damage(&game, &test_dp(), 0, false);

    assert!(assignments.is_empty(), "CR 800.4e: that damage isn't assigned");
}

/// CR 800.4e stops the *player's* share and nothing else. A trampler whose
/// defending player has left still assigns to its blocker: the rule is about
/// the assignment to that player, and RE-7's `retain` is over the finished
/// list rather than over the attacker.
#[test]
fn a_tramplers_blocker_still_takes_damage_when_the_defending_player_has_left() {
    let mut game = setup_game(4);
    let attacker = put_on_battlefield(
        &mut game,
        vanilla_creature(5, 5, &[mtgsim::types::keywords::KeywordFlag::Trample]),
        0,
    );
    let blocker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 2);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);
    departs(&mut game, 1, &test_dp());

    // The attacker's controller is still asked how to divide — CR 702.19b's
    // "at least lethal to each blocker" is their choice and does not stop
    // being one because the spill has nowhere to go.
    let dp = RecordingDecisionProvider::picking(0);
    let assignments =
        mtgsim::engine::combat::resolution::assign_combat_damage(&game, &dp, 0, false);

    let from_attacker: Vec<&DamageTarget> = assignments
        .iter()
        .filter(|a| a.source == attacker)
        .map(|a| &a.target)
        .collect();
    assert_eq!(
        from_attacker,
        vec![&DamageTarget::Object(blocker)],
        "the blocker's share stands and the player's is not assigned"
    );
    assert!(
        assignments.iter().all(|a| !matches!(a.target, DamageTarget::Player(_))),
        "CR 800.4e: nothing is assigned to a player who has left"
    );
}

/// What actually happens when a player leaves after blockers are declared:
/// their blockers leave the game with them (CR 800.4a), and CR 510.1c makes an
/// attacker that was blocked and has no blockers left deal no damage at all —
/// which is a *different* rule from 800.4e and is why that one needs the board
/// above to be seen.
#[test]
fn a_blocked_attacker_whose_blockers_left_the_game_assigns_nothing() {
    let mut game = setup_game(4);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let blocker = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 2);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);

    departs(&mut game, 2, &test_dp());

    assert!(!game.objects.contains_key(&blocker), "the blocker left with its owner");
    let assignments =
        mtgsim::engine::combat::resolution::assign_combat_damage(&game, &test_dp(), 0, false);
    assert!(
        assignments.is_empty(),
        "CR 510.1c: blocked, no blockers remaining, no trample"
    );
}

/// The same attacker against a defending player still in the game, so the test
/// above is about the rule and not about the fixture.
#[test]
fn combat_damage_is_assigned_to_a_player_still_in_the_game() {
    let mut game = setup_game(4);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    set_attacking(&mut game, attacker, 1);

    let assignments =
        mtgsim::engine::combat::resolution::assign_combat_damage(&game, &test_dp(), 0, false);

    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].target, DamageTarget::Player(1));
    assert_eq!(assignments[0].amount, 3);
}

// ---------------------------------------------------------------------------
// CR 800.4m — the turn that would have begun
// ---------------------------------------------------------------------------

// COVERS: ATOM-800.4m-001
#[test]
fn an_until_your_next_turn_effect_lasts_until_the_departed_players_turn_would_have_begun() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    // P1's "until your next turn" pump, written on P0's turn.
    let effect = Effect::Atom(
        Primitive::ModifyPowerToughness(
            AmountExpr::Fixed(1),
            AmountExpr::Fixed(1),
            Duration::UntilYourNextTurn,
        ),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_with(&mut game, 1, &effect, vec![ResolvedTarget::Object(bears)], &test_dp());
    assert_eq!(game.continuous_effects.len(), 1);

    departs(&mut game, 1, &test_dp());
    assert_eq!(
        game.continuous_effects.len(),
        1,
        "CR 800.4m: it does not expire immediately"
    );

    // P1's turn would have been the next one; the rotation passes their seat on
    // the way to P2's.
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::Cleanup) });
    advance(&mut game);

    assert_eq!(game.active_player, 2, "CR 800.4k: P1's turn did not begin");
    assert_eq!(
        game.continuous_effects.len(),
        0,
        "CR 800.4m: it lasted until that turn would have begun, and no longer"
    );
}

/// The other half of the same sentence — "nor last indefinitely" — is the test
/// above's last assertion; this is the control that says the expiry is the
/// departed player's own and not everyone's.
#[test]
fn a_departed_seat_expires_only_its_own_until_your_next_turn_rows() {
    let mut game = setup_game(4);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let pump = Effect::Atom(
        Primitive::ModifyPowerToughness(
            AmountExpr::Fixed(1),
            AmountExpr::Fixed(1),
            Duration::UntilYourNextTurn,
        ),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve_with(&mut game, 1, &pump, vec![ResolvedTarget::Object(bears)], &test_dp());
    // P3's, whose own turn is two seats further on.
    resolve_with(&mut game, 3, &pump, vec![ResolvedTarget::Object(bears)], &test_dp());

    departs(&mut game, 1, &test_dp());
    game.set_turn_position(Phase { phase_type: PhaseType::Ending, step: Some(StepType::Cleanup) });
    advance(&mut game);

    assert_eq!(game.active_player, 2);
    assert_eq!(game.continuous_effects.len(), 1, "P3's row is still live");
    assert_eq!(game.continuous_effects.iter().next().unwrap().controller, 3);
}

// ---------------------------------------------------------------------------
// CR 704.3 meets CR 800.4a — the batch performs its losses last
// ---------------------------------------------------------------------------

/// A creature the departing player owns and that is dying in the same
/// state-based check: the `Destroy` was decided against a board the creature is
/// on, so it performs first and the departure takes the card out of the
/// graveyard a moment later.
///
/// Both events are in the log, in that order, and neither performer was asked
/// about a board it could not find.
#[test]
fn a_creature_dying_in_the_same_check_is_destroyed_and_then_leaves_the_game() {
    let mut game = setup_game(4);
    let doomed = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.battlefield.get_mut(&doomed).unwrap().damage_marked = 2;

    departs(&mut game, 1, &test_dp());

    let order: Vec<&'static str> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, to: Zone::Graveyard, .. } if *object_id == doomed => {
                Some("died")
            }
            GameEvent::LeftTheGame { object_id, .. } if *object_id == doomed => Some("left"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["died", "left"]);
    assert!(!game.objects.contains_key(&doomed));
}

/// Two players losing in one check: each takes their own objects, and the
/// batch settles the result once (CR 104.2a, RE-6's).
#[test]
fn two_players_leaving_in_one_check_each_take_their_own_objects() {
    let mut game = setup_game(4);
    let ones = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let twos = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 2);
    let threes = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 3);
    game.players[1].life_total = 0;
    game.players[2].life_total = 0;

    assert!(sba(&mut game, &test_dp()));

    assert!(!game.objects.contains_key(&ones));
    assert!(!game.objects.contains_key(&twos));
    assert_eq!(game.get_object(threes).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.result, None, "two players remain");
    assert_eq!(departures(&game).len(), 2);
}

/// CR 608.2m — "if a spell or ability leaves the stack while resolving, it will
/// continue to resolve fully". Its owner leaving during its own resolution is
/// how it leaves here, and CR 608.2n's graveyard trip then has no card to make.
#[test]
fn a_spell_whose_owner_leaves_during_its_own_resolution_makes_no_graveyard_trip() {
    let mut game = setup_game(4);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 2);
    let spell = stack_object_with(
        &mut game,
        1,
        1,
        true,
        Effect::Atom(
            Primitive::LoseGame,
            EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        ),
        vec![ResolvedTarget::Player(1)],
    );

    game.resolve_top_of_stack(&test_dp()).expect("the resolution completes");

    assert!(game.player_lost[1]);
    assert!(!game.objects.contains_key(&spell), "it left the game with its owner");
    assert!(game.players[1].graveyard.is_empty());
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Battlefield);
}

// ---------------------------------------------------------------------------
// CR 800.4a beside CR 903.10a — what a departure does not undo
// ---------------------------------------------------------------------------

/// **What this guards is a dangling key, not a rule.** CR 903.10a's tally is a
/// `HashMap<ObjectId, u32>` on each damaged player, keyed by the *commander
/// object* — and CR 800.4a has just deleted that object from the game. So the
/// board asks whether CR 704.6c's check still reads the tallies of a commander
/// that no longer exists, which it does only because the check reads
/// `.values()` and never looks the key up. A future reader that resolves the
/// key to a card — "you lost to Bianca's Gonti" in a log line, say — breaks
/// here and nowhere else.
///
/// The composite's own framing ("no further commander damage can accumulate
/// from it") is true by construction and would not be worth a test on its own:
/// a player who has left cannot come back, and CR 800.4a took their commander
/// with them.
// COVERS-PARTIAL: COMP-800-PLAYER-LEAVES-COMMANDER-001
//
// The partial is the word *Commander*: there is no constructor for a Commander
// game yet (`codebase-state.md`, "Before Commander" items 2 and 3), so the
// board below is a four-player game with the CR 903.10a tallies written
// directly rather than dealt.
#[test]
fn commander_damage_already_dealt_survives_its_dealer_leaving_the_game() {
    let mut game = setup_game(4);
    let commander = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 0);
    game.objects.get_mut(&commander).unwrap().is_commander = true;
    game.players[1].commander_damage_taken.insert(commander, 15);
    game.players[2].commander_damage_taken.insert(commander, 10);

    departs(&mut game, 0, &test_dp());

    assert!(!game.objects.contains_key(&commander), "CR 800.4a: the commander left too");
    assert_eq!(game.players[1].commander_damage_taken.get(&commander), Some(&15));
    assert_eq!(game.players[2].commander_damage_taken.get(&commander), Some(&10));
    assert!(!sba(&mut game, &test_dp()), "and neither player is at CR 704.6c's 21");
    assert!(game.in_game(1) && game.in_game(2));
}

// ---------------------------------------------------------------------------
// CR 800.1 — the scope
// ---------------------------------------------------------------------------

/// CR 800.1 — "a multiplayer game is a game that begins with more than two
/// players" — is the gate on all of the above, and CR 800.4's own first
/// sentence says why: a two-player game cannot continue after a player leaves,
/// so there is nothing for these rules to be about. The loser's board is left
/// exactly as the game ended with it.
#[test]
fn a_two_player_game_leaves_the_departed_players_objects_where_they_are() {
    let mut game = setup_game(2);
    let permanent = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let held = put_in_hand(&mut game, CardDataBuilder::new("Held").build(), 1);

    departs(&mut game, 1, &test_dp());

    assert_eq!(game.result, Some(GameResult::Winner(0)));
    assert_eq!(game.get_object(permanent).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.get_object(held).unwrap().zone, Zone::Hand);
    assert!(departures(&game).is_empty());
    assert_eq!(
        game.events
            .events()
            .filter(|e| matches!(e, GameEvent::PlayerLost { reason: LossReason::LifeReachedZero, .. }))
            .count(),
        1
    );
}
