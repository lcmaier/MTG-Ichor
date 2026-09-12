//! Phase RD-4 — redirection and unpreventable damage.
//!
//! Two independent features that share one rule
//! (`replacement-architecture.md` §9, RD-4). **CR 614.9** moves where the damage
//! goes and re-checks the destination at the moment it applies; **CR 615.12**
//! makes a prevention effect apply and prevent nothing. What they share is
//! decision 7: an application that did nothing spends nothing.
//!
//! The card file's tests are the rulings pass. These are the rules' own — the
//! two restriction routes that have no printed consumer the engine can play,
//! CR 614.9's four ways for a destination to be illegal, and the dovetail board
//! where one event's two riders read two different numbers.

use std::sync::Arc;

use mtgsim::cards::phase_rd_cards::{
    angel_of_suffering, mending_hands, palisade_giant, pariah, pinpoint_avalanche,
    reflect_damage, reverse_damage,
};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use mtgsim::state::game_state::GameState;
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::state::restrictions::RegisteredRestriction;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_hand, put_on_battlefield, setup_game,
    setup_two_player_game, test_ctx, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{
    AmountRewrite, EventPattern, ReplacementDef, RetargetSpec, Rewrite, Uses,
};
use mtgsim::types::restriction::{
    ReplacementKindFilter, Restriction, RestrictionDef,
};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn marked(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

/// Every row's CR 615.7 count, in registry order.
fn counts(game: &GameState) -> Vec<u64> {
    game.replacement_effects
        .iter()
        .filter_map(|r| match r.def.uses {
            Uses::NextDamage(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// Deal `amount` from `source` to `target`, through the pipeline.
///
/// `unpreventable` is the proposer's, which is what CR 615.12's third shape is
/// — a fact about this event and about nothing else.
fn deal(
    game: &mut GameState,
    source: ObjectId,
    target: DamageTarget,
    amount: u64,
    unpreventable: bool,
    ctx: &ActionContext,
) {
    game.execute_action(
        GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable },
        ctx,
    )
    .unwrap();
}

/// Resolve `card`'s spell effect for `controller`, the way the stack would.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    resolve_spell_at(game, card, controller, Vec::new())
}

fn resolve_spell_at(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<mtgsim::engine::resolve::ResolvedTarget>,
) -> ObjectId {
    resolve_spell_with(game, card, controller, targets, &test_dp())
}

/// The same, with the resolution's own decision provider — CR 609.7a's source
/// choice is made here, and a board with two damage sources on it asks.
fn resolve_spell_with(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<mtgsim::engine::resolve::ResolvedTarget>,
    dp: &dyn mtgsim::ui::decision::DecisionProvider,
) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).unwrap();
    id
}

/// The source CR 609.7a wrote onto the one row in the registry.
fn chosen_damage_source(game: &GameState) -> Option<ObjectId> {
    match &game.replacement_effects.iter().next()?.def.pattern {
        EventPattern::DealDamage { source: Some(p), .. } => p.object,
        _ => None,
    }
}

/// A registry row inserted directly — the fixture shapes whose printed consumer
/// is a later PR, each named in its test.
fn fixture_row(game: &mut GameState, source: ObjectId, controller: PlayerId, def: ReplacementDef) {
    let turn = game.turn_number;
    game.replacement_effects.add(RegisteredReplacementEffect {
        id: 0,
        source,
        controller,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        targets: Vec::new(),
        def,
    });
}

/// CR 615.12's **first** printed shape as a fixture: "damage can't be prevented
/// this turn", a resolution's restriction with a duration.
///
/// Every card that prints it carries a second half the engine lacks —
/// Skullcrack's and Leyline of Punishment's "players can't gain life" needs
/// RE's `GainLife` pattern arm, and the other five have kicker, an adventure,
/// flashback, a `Conditional` or wither (§11 item 26). So the row is built
/// here, and Skullcrack lands it in a game after RE.
fn cant_be_prevented_this_turn(game: &mut GameState, source: ObjectId, controller: PlayerId) {
    let turn = game.turn_number;
    game.restrictions.add(RegisteredRestriction {
        id: 0,
        source,
        controller,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        def: RestrictionDef::new(Restriction::ApplyReplacement {
            kind: ReplacementKindFilter::Prevention,
            to_objects: AffectedSet::Filter { filter: ObjectFilter::All },
            to_players: PlayerSet::Everyone,
        }),
    });
}

/// CR 615.12's **second** printed shape as a fixture: a static ability's
/// "damage can't be prevented", swept off the source's effective ability list
/// rather than kept in a registry — so Humility strips it for free and it lasts
/// exactly while the source is on the battlefield.
///
/// **Leyline of Punishment entire, as of RE-3**, minus the one line that keeps
/// it unregistered. The card is three sentences: an opening-hand clause that is
/// §3.3 source 2's zone-reaching static and would be dead text under a real
/// card name, "players can't gain life", and "damage can't be prevented". RE-3
/// built the first of those two as a `Restriction::Event` with a `PlayerSet`
/// and landed it in a game on Skullcrack; this fixture is where the *static*
/// form of both halves is proved, which is what §9 asks of it. Everlasting
/// Torment is the other printed carrier, and it has wither.
fn leyline_fixture() -> Arc<CardData> {
    CardDataBuilder::new("Prevention Ban Probe")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Players can't gain life.\nDamage can't be prevented.")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::GainLife,
                affected: AffectedSet::NO_OBJECTS,
                affected_players: PlayerSet::Everyone,
                by: None,
            }))),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Restriction(Box::new(RestrictionDef::new(
                Restriction::ApplyReplacement {
                    kind: ReplacementKindFilter::Prevention,
                    to_objects: AffectedSet::Filter { filter: ObjectFilter::All },
                    to_players: PlayerSet::Everyone,
                },
            ))),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// The life half of the fixture above, which is Leyline of Punishment's
/// static form of Skullcrack's first sentence (RE-3, §11 item 26).
///
/// **A static "can't" is a sweep, not a row**, and the difference is what this
/// asserts beyond the RE-3 file's registry-row boards: the prohibition is read
/// off the source's *effective* ability list on every proposal, so it starts
/// when the permanent enters and stops when it leaves, with nothing to expire.
#[test]
fn a_static_players_cant_gain_life_refuses_the_gain_while_its_source_is_there() {
    let mut game = setup_two_player_game();
    let leyline = put_on_battlefield(&mut game, leyline_fixture(), 0);
    let source = probe(&mut game, 0);
    let before = life(&game, 1);

    let gain = Effect::Atom(
        Primitive::GainLife(AmountExpr::Fixed(4)),
        EffectRecipient::Controller,
    );
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 1,
        targets: Vec::new(),
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&gain, &ctx, &ScriptedDecisionProvider::new()).unwrap();
    assert_eq!(life(&game, 1), before, "CR 101.2 refuses the proposal");

    // And it ends with the permanent rather than at cleanup — the sweep reads
    // a battlefield that no longer holds it.
    game.change_zone(leyline, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .unwrap();
    game.resolve_effect(&gain, &ctx, &ScriptedDecisionProvider::new()).unwrap();
    assert_eq!(life(&game, 1), before + 4, "the ability left with its source");
}

/// A source with no ability of its own — something for a fixture row to hang
/// off and something to deal damage.
fn probe(game: &mut GameState, controller: PlayerId) -> ObjectId {
    place_vanilla_creature(game, controller, 1, 1, &[])
}

// ---------------------------------------------------------------------------
// CR 614.9 — redirection
// ---------------------------------------------------------------------------

/// The plain board: Pariah on your creature, damage aimed at you lands on it.
#[test]
fn pariah_puts_damage_aimed_at_you_onto_the_enchanted_creature() {
    let mut game = setup_two_player_game();
    let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
    let aura = put_on_battlefield(&mut game, pariah(), 0);
    assert!(game.attach(aura, host));
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 20, "the damage never reached the player");
    assert_eq!(marked(&game, host), 3, "it reached the enchanted creature");
}

/// CR 704.5m's window, which is where an Aura with no host lives: the enchanted
/// creature leaves, `change_zone` detaches every attachment and **leaves the
/// Aura on the battlefield** for the state-based action to find, and until a
/// player would receive priority (CR 704.3) it sits there enchanting nothing.
///
/// So a single resolution that destroys a creature and then damages its
/// controller reaches this board, and it is not exotic. What it exercises is
/// `retarget_destination` answering `None` — the *same* branch as
/// `pariah_attached_to_nothing_does_nothing` below, by a different route, which
/// is why neither of them can claim CR 614.9's "no longer on the battlefield"
/// leg. That one is
/// `a_registry_row_whose_source_has_left_the_battlefield_redirects_nothing`.
#[test]
fn pariah_whose_host_has_left_the_battlefield_does_nothing() {
    let mut game = setup_two_player_game();
    let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
    let aura = put_on_battlefield(&mut game, pariah(), 0);
    assert!(game.attach(aura, host));
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    game.change_zone(host, Zone::Graveyard, ZoneChangeCause::Destroyed, &ctx).unwrap();
    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17, "the damage was dealt as proposed");
}

/// CR 614.9's first clause, and the only board that reaches it: *"If one of
/// those permanents is **no longer on the battlefield** when the damage would
/// be redirected … the effect does nothing."*
///
/// An Aura cannot get here — `change_zone` detaches on the host's departure, so
/// a dangling `attached_to` never exists. A **registry row** can: it keeps the
/// `source` it was created with, and CR 608.2c's duration outlives the
/// permanent. So a resolution-created `ToEffectSource` redirect whose source
/// has died still names it, and `redirection_is_legal` is what says no.
///
/// The `Uses::Static` here is deliberate — the "not used up" half of the atom
/// is `a_once_redirect_to_a_player_who_has_left_the_game_keeps_its_row`, and
/// this one is only about the destination.
// COVERS-PARTIAL: ATOM-614.9-001 — the destination half; the atom's other half,
// "the shield is NOT used up", needs a `Uses::Once` redirect and is
// `a_once_redirect_to_a_player_who_has_left_the_game_keeps_its_row` below.
#[test]
fn a_registry_row_whose_source_has_left_the_battlefield_redirects_nothing() {
    let mut game = setup_two_player_game();
    let redirector = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
    fixture_row(
        &mut game,
        redirector,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Retarget(RetargetSpec::ToEffectSource),
        )
        .affecting_players(PlayerSet::You),
    );
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    // It works while the source is there — the control, so the test below is
    // about the destination and not about the row.
    deal(&mut game, source, DamageTarget::Player(0), 2, false, &ctx);
    assert_eq!(life(&game, 0), 20);
    assert_eq!(marked(&game, redirector), 2);

    game.change_zone(redirector, Zone::Graveyard, ZoneChangeCause::Destroyed, &ctx).unwrap();
    assert_eq!(game.replacement_effects.len(), 1, "the row outlives its source");

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17, "the damage was dealt as proposed");
}

/// The other route to a missing host: an Aura that entered and has not been
/// attached. `place_on_battlefield` writes `attached_to: None` and the
/// resolution attaches afterwards, so the two are separate steps here as they
/// are in CR 303.4c.
///
/// Same branch as the test above, kept beside it because the provenance
/// differs: one is an Aura that never had a host, the other an Aura whose host
/// left. A regression that broke only one of the two writers would show here.
#[test]
fn pariah_attached_to_nothing_does_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, pariah(), 0);
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17);
}

/// A destination that is on the battlefield but is not a creature, planeswalker
/// or battle — CR 614.9's second clause. A Pariah on an animated artifact that
/// stops being a creature is the printed road here; the fixture takes the same
/// road with a `ToEffectSource` redirect off an enchantment.
#[test]
fn a_redirect_onto_a_noncreature_permanent_does_nothing() {
    let mut game = setup_two_player_game();
    let enchantment = put_on_battlefield(&mut game, leyline_fixture(), 0);
    fixture_row(
        &mut game,
        enchantment,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Retarget(RetargetSpec::ToEffectSource),
        )
        .affecting_players(PlayerSet::You),
    );
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17, "an enchantment is not a legal destination");
}

/// Palisade Giant's player half: "all damage that would be dealt to **you** …".
///
/// The half that needs no object filter at all — `set_affects` answers a player
/// subject off `PlayerSet` and never reaches the `Filter` — which is why it
/// passes on a tree where the object half does not.
#[test]
fn palisade_giant_takes_the_damage_aimed_at_you() {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, palisade_giant(), 0);
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 5, false, &ctx);

    assert_eq!(life(&game, 0), 20);
    assert_eq!(marked(&game, giant), 5);
}

/// Reflect Damage — "that damage is dealt to that source's controller instead".
///
/// The one redirect in the phase whose destination is read off the *damage's*
/// source rather than off the effect's, and the one whose destination is a
/// player.
#[test]
fn reflect_damage_sends_the_next_damage_back_to_its_sources_controller() {
    let mut game = setup_two_player_game();
    let source = probe(&mut game, 1);
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);
    resolve_spell(&mut game, reflect_damage(), 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: DamageTarget::Player(0),
            amount: 4,
            is_combat: false,
            unpreventable: false,
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(life(&game, 0), 20, "not the player it was aimed at");
    assert_eq!(life(&game, 1), 16, "the source's controller");
}

/// CR 614.9's last sentence: *"If damage would be redirected to or from a player
/// who has left the game, the effect does nothing."* — and decision 7's other
/// half, that a `Uses::Once` row which did nothing is still there.
///
/// A three-player board, because a two-player game ends when someone leaves.
/// CR 800.4a's removal of that player's objects is Phase 9's, so the source is
/// still on the battlefield here; the rule under test is what the *redirect*
/// does, and it is asked at application either way.
// COVERS: ATOM-614.9-001
#[test]
fn a_once_redirect_to_a_player_who_has_left_the_game_keeps_its_row() {
    let mut game = setup_game(3);
    let source = probe(&mut game, 2);
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);
    resolve_spell(&mut game, reflect_damage(), 0);
    assert_eq!(game.replacement_effects.len(), 1);

    game.player_lost[2] = true;
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: DamageTarget::Player(0),
            amount: 4,
            is_combat: false,
            unpreventable: false,
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(life(&game, 0), 16, "the damage was dealt as proposed");
    assert_eq!(life(&game, 2), 20, "and not to the player who has left");
    assert_eq!(
        game.replacement_effects.len(),
        1,
        "CR 609.7b — a redirect that did nothing is not used up"
    );
}

/// "… and **other permanents you control**" — the object half, and the first
/// card in the crate whose affected set needs `ObjectFilter::EachOther`.
#[test]
fn palisade_giant_takes_the_damage_aimed_at_your_other_permanents() {
    let mut game = setup_two_player_game();
    let giant = put_on_battlefield(&mut game, palisade_giant(), 0);
    let mine = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    let theirs = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Object(mine), 2, false, &ctx);
    deal(&mut game, source, DamageTarget::Object(theirs), 2, false, &ctx);

    assert_eq!(marked(&game, mine), 0, "a permanent its controller controls");
    assert_eq!(marked(&game, theirs), 2, "one they do not");
    assert_eq!(marked(&game, giant), 2);
}

// ---------------------------------------------------------------------------
// CR 615.12 — damage that can't be prevented
// ---------------------------------------------------------------------------

/// The rule's own board, with the per-event flag as its cause: a CR 615.7 count
/// applies, prevents nothing, and is not reduced.
// COVERS: ATOM-615.12-001
#[test]
fn a_count_applied_to_unpreventable_damage_prevents_nothing_and_is_not_reduced() {
    let mut game = setup_two_player_game();
    let source = probe(&mut game, 1);
    let shield = probe(&mut game, 0);
    fixture_row(
        &mut game,
        shield,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3),
    );
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 4, true, &ctx);

    assert_eq!(life(&game, 0), 16, "all four, none of it prevented");
    assert_eq!(counts(&game), vec![3], "the count was not touched");
}

/// The same rule at a larger amount, which is the composed atom's board.
// COVERS: COMP-615-UNPREVENTABLE-SHIELD-001
#[test]
fn a_three_damage_count_under_five_unpreventable_damage_survives_intact() {
    let mut game = setup_two_player_game();
    let source = probe(&mut game, 1);
    let shield = probe(&mut game, 0);
    fixture_row(
        &mut game,
        shield,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3),
    );
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 5, true, &ctx);

    assert_eq!(life(&game, 0), 15);
    assert_eq!(counts(&game), vec![3]);
}

/// *"Prevent the next 3 damage … If damage is prevented this way, gain that much
/// life."* Under unpreventable damage the prevention is applied, prevents 0, and
/// the rider **runs** with a 0 — which is why "if damage is prevented this way"
/// needs no `Effect::Conditional`: `GainLife(0)` is a CR 119.10 non-event.
// COVERS: ATOM-615.12-002
#[test]
fn a_riders_prevented_amount_is_zero_under_unpreventable_damage() {
    let mut game = setup_two_player_game();
    let source = probe(&mut game, 1);
    let shield = probe(&mut game, 0);
    fixture_row(
        &mut game,
        shield,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3)
        .with_then(Effect::Atom(
            Primitive::GainLife(AmountExpr::DamagePrevented),
            EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        )),
    );
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, true, &ctx);

    assert_eq!(life(&game, 0), 17, "three dealt, none prevented, none gained");
    assert_eq!(counts(&game), vec![3]);
}

/// CR 615.12a — *"A prevention effect is applied to any particular unpreventable
/// damage event just once. It won't invoke itself repeatedly."*
///
/// The termination argument is CR 614.5's applied set, which the loop writes
/// before `apply_rewrite` runs: an effect that changed nothing is still in it.
/// The observable is that this test returns at all, and that two prevention
/// effects produce exactly one prompt and then no more.
// COVERS: ATOM-615.12a-001
#[test]
fn two_prevention_effects_on_unpreventable_damage_are_each_applied_once() {
    let mut game = setup_two_player_game();
    let source = probe(&mut game, 1);
    let shield = probe(&mut game, 0);
    for _ in 0..2 {
        fixture_row(
            &mut game,
            shield,
            0,
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
            )
            .affecting_players(PlayerSet::You),
        );
    }
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);

    deal(&mut game, source, DamageTarget::Player(0), 3, true, &ctx);

    assert_eq!(life(&game, 0), 17, "neither prevented anything");
    assert_eq!(dp.prompts(), 1, "two candidates, one CR 616.1 choice, then none");
}

/// CR 615.12's first printed shape, as a registry row: "damage can't be
/// prevented this turn". The consult is at the site the prevention arm applies,
/// so a whole-event `Prevent` leaves the event alone.
#[test]
fn a_cant_be_prevented_row_makes_a_whole_event_prevention_prevent_nothing() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = probe(&mut game, 1);
    let banner = probe(&mut game, 1);
    cant_be_prevented_this_turn(&mut game, banner, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17, "the Angel's Prevent prevented nothing");
    assert_eq!(game.players[0].graveyard.len(), 6, "and its rider still milled 2x3");
    assert_eq!(marked(&game, angel), 0);
}

/// CR 615.12's second printed shape: a static ability's restriction, swept off
/// the source's effective ability list rather than kept in a row.
///
/// The same board as above and the same answer, which is the point — the two
/// routes are two *sources* of one `Restriction`, and `is_prohibited` unions
/// them.
#[test]
fn a_static_cant_be_prevented_ability_makes_a_prevention_prevent_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_fixture(), 1);
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 17);
    assert_eq!(game.players[0].graveyard.len(), 6);
}

/// And the control: with the fixture gone, the same board prevents all of it.
#[test]
fn without_the_restriction_the_same_board_prevents_the_damage() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = probe(&mut game, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 20);
    assert_eq!(game.players[0].graveyard.len(), 6);
}

/// A CR 615.7 count under the *restriction* rather than the flag — the second
/// route reaching the second consult site.
#[test]
fn a_cant_be_prevented_row_leaves_a_count_intact() {
    let mut game = setup_two_player_game();
    let target = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
    resolve_spell_at(
        &mut game,
        mending_hands(),
        0,
        vec![mtgsim::engine::resolve::ResolvedTarget::Object(target)],
    );
    assert_eq!(counts(&game), vec![4]);
    let source = probe(&mut game, 1);
    let banner = probe(&mut game, 1);
    cant_be_prevented_this_turn(&mut game, banner, 1);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Object(target), 2, false, &ctx);

    assert_eq!(marked(&game, target), 2);
    assert_eq!(counts(&game), vec![4], "CR 615.12's last sentence");
}

// ---------------------------------------------------------------------------
// The dovetail — one event, two riders, two different numbers (decision 4)
// ---------------------------------------------------------------------------

/// Angel of Suffering mills "twice that many", where "that many" is the damage
/// that *would have been* dealt; Reverse Damage gains life "equal to the damage
/// prevented this way". Under damage that can't be prevented both riders run,
/// and the two numbers come apart: 6 milled, 0 gained.
///
/// Angel's own ruling is the first half — *"if the damage can't be prevented for
/// some reason, you'll still mill twice that many"* — and CR 615.12's middle
/// sentence is the second.
#[test]
fn under_a_cant_be_prevented_row_the_angel_still_mills_and_reverse_damage_gains_nothing() {
    let mut game = setup_two_player_game();
    // First on the battlefield, so it is the first candidate CR 609.7a offers.
    let bolt_source = probe(&mut game, 1);
    let angel = put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    cant_be_prevented_this_turn(&mut game, angel, 1);

    // Reverse Damage names its source at resolution — RD-3's chosen source.
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);
    resolve_spell_with(
        &mut game,
        reverse_damage(),
        0,
        vec![mtgsim::engine::resolve::ResolvedTarget::Player(0)],
        &dp,
    );
    assert_eq!(chosen_damage_source(&game), Some(bolt_source), "the source under test");

    game.execute_action(
        GameAction::DealDamage {
            source: bolt_source,
            target: DamageTarget::Player(0),
            amount: 3,
            is_combat: false,
            unpreventable: false,
        },
        &ctx,
    )
    .unwrap();

    assert_eq!(life(&game, 0), 17, "three dealt, and no life gained on top");
    assert_eq!(game.players[0].graveyard.len(), 6, "ReplacedAmount x 2");
    assert_eq!(
        game.replacement_effects.len(),
        1,
        "Reverse Damage's row prevented nothing, so it is not used up"
    );
}

/// The second dovetail board: Pinpoint Avalanche into a shield counter.
///
/// CR 122.1c's prevention half applies, prevents nothing, and its rider removes
/// a counter anyway — Disciplined Duelist's ruling, which is CR 615.12's middle
/// sentence on a counter rather than on a spell.
#[test]
fn pinpoint_avalanche_into_a_shield_counter_deals_the_damage_and_removes_the_counter() {
    let mut game = setup_two_player_game();
    let target = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
    game.add_counters(target, mtgsim::types::effects::CounterType::Shield, 1);

    resolve_spell_at(
        &mut game,
        pinpoint_avalanche(),
        0,
        vec![mtgsim::engine::resolve::ResolvedTarget::Object(target)],
    );

    assert_eq!(marked(&game, target), 4, "the damage can't be prevented");
    assert_eq!(
        game.battlefield[&target].counter_count(mtgsim::types::effects::CounterType::Shield),
        0,
        "and the rider removed the counter anyway"
    );
}

/// The control for the board above: without the flag, the counter prevents the
/// damage and is spent doing it.
#[test]
fn a_shield_counter_still_prevents_preventable_damage() {
    let mut game = setup_two_player_game();
    let target = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
    game.add_counters(target, mtgsim::types::effects::CounterType::Shield, 1);
    let source = probe(&mut game, 0);
    let ctx = test_ctx();

    deal(&mut game, source, DamageTarget::Object(target), 4, false, &ctx);

    assert_eq!(marked(&game, target), 0);
    assert_eq!(
        game.battlefield[&target].counter_count(mtgsim::types::effects::CounterType::Shield),
        0
    );
}

/// Redirection and prevention on one board, in one loop: the redirect moves the
/// event onto a creature, and a shield counter on **that creature** is gathered
/// against the rewritten event and applies.
///
/// The board that makes the group's subject and the member's subject come apart
/// — a `RemoveCountersFromAffected` reading the group key would have nothing to
/// take a counter from, because the key is a player.
#[test]
fn a_redirect_hands_the_event_to_the_destinations_own_shield_counter() {
    let mut game = setup_two_player_game();
    let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
    game.add_counters(host, mtgsim::types::effects::CounterType::Shield, 1);
    let aura = put_on_battlefield(&mut game, pariah(), 0);
    assert!(game.attach(aura, host));
    let source = probe(&mut game, 1);
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);

    deal(&mut game, source, DamageTarget::Player(0), 3, false, &ctx);

    assert_eq!(life(&game, 0), 20, "redirected off the player");
    assert_eq!(marked(&game, host), 0, "and prevented on the creature");
    assert_eq!(
        game.battlefield[&host].counter_count(mtgsim::types::effects::CounterType::Shield),
        0,
        "the counter was spent by the prevention it created"
    );
}

/// `unpreventable` travels with the damage, exactly as `is_combat` does —
/// CR 614.9 moves "the same damage", and a redirect that dropped the flag would
/// hand a prevention effect on the *destination* a free save.
///
/// Not vacuous just because the field is copied in one line: nothing else here
/// reads it after a rewrite, so a `Retarget` arm that forgot it would leave
/// every other test in this file green. The combat-damage twin of this
/// assertion is in the card file's rulings pass.
#[test]
fn a_redirect_carries_the_unpreventable_flag_onto_the_destination() {
    let mut game = setup_two_player_game();
    let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
    let aura = put_on_battlefield(&mut game, pariah(), 0);
    assert!(game.attach(aura, host));
    game.add_counters(host, mtgsim::types::effects::CounterType::Shield, 1);
    let source = probe(&mut game, 1);
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);

    deal(&mut game, source, DamageTarget::Player(0), 3, true, &ctx);

    assert_eq!(life(&game, 0), 20, "redirected off the player");
    assert_eq!(marked(&game, host), 3, "and the shield counter prevented none of it");
    assert_eq!(
        game.battlefield[&host].counter_count(mtgsim::types::effects::CounterType::Shield),
        0,
        "CR 615.12's middle sentence — the rider ran anyway"
    );
}

/// A vanilla creature is a source with no abilities — the shape every fixture
/// above leans on, asserted once so a change to `vanilla_creature` cannot make
/// the whole file vacuous.
#[test]
fn the_probe_source_has_no_replacement_ability_of_its_own() {
    let card = vanilla_creature(1, 1, &[]);
    assert!(card.abilities.is_empty());
}
