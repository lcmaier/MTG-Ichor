//! Phase RC-2 integration tests: entering the battlefield as a replaceable event.
//!
//! RC-2 is the first RC phase that adds behaviour rather than removing it, so
//! these tests assert on two things at once, the way RB's do: the modified
//! outcome, **and** that the modification arrived through the pipeline. A
//! "enters tapped" test that passes because something wrote `tapped = true`
//! directly would be exactly the failure this phase is prone to
//! (`replacement-architecture.md` §10).

use std::sync::Arc;

use mtgsim::cards::phase_rc_cards::{chainbreaker, idyllic_beachfront};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    place_bare, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game,
    test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype};
use mtgsim::types::effects::{AffectedSet, CounterType, Effect};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EnterMods, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Every `PermanentEnteredBattlefield` in the log, as `(object, controller)`.
fn entries(game: &GameState) -> Vec<(ObjectId, usize)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::PermanentEnteredBattlefield { object_id, controller } => {
                Some((*object_id, *controller))
            }
            _ => None,
        })
        .collect()
}

/// A permanent whose only text is one entry-modifying static ability.
///
/// A fixture, not a card: it exists to vary one axis at a time, and every claim
/// it makes about a *registered* card is made again against
/// [`idyllic_beachfront`] or [`chainbreaker`] below.
fn enters_with(name: &str, power: i32, toughness: i32, mods: EnterMods) -> Arc<CardData> {
    CardDataBuilder::new(name)
        // One colored symbol and no generic: a generic cost would make
        // `cast_spell` ask how to allocate it, and this fixture exists to reach
        // the entry with no prompt in the way.
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(mtgsim::types::colors::Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(power, toughness)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(mods),
            ))),
        })
        .build()
}

/// Put `card` onto the battlefield the way the *game* does — a real zone change
/// through the chokepoint, so the CR 616.1 loop runs — rather than through
/// `put_on_battlefield`, which skips the pipeline entirely.
///
/// Reanimation's shape, and the cause is picked to be honest rather than
/// convenient: `ZoneChangeCause` has no catchall, and "return target creature
/// card from your graveyard to the battlefield" is a real effect with a real
/// cause. Every entry road in the engine converges on the same
/// `perform_action` arm, so which one a test takes changes nothing but the
/// label.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: usize) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
    id
}

// ---------------------------------------------------------------------------
// CR 110.5b — status on entry
// ---------------------------------------------------------------------------

// COVERS: ATOM-110.5b-001
#[test]
fn test_permanent_enters_untapped_by_default() {
    let mut game = setup_two_player_game();

    let bears = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);

    assert!(
        !game.battlefield.get(&bears).unwrap().tapped,
        "CR 110.5b: permanents enter untapped unless a spell or ability says otherwise"
    );
}

// COVERS: ATOM-110.5b-002
#[test]
fn test_tapland_enters_tapped() {
    let mut game = setup_two_player_game();

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).expect("the land drop is legal");

    assert!(
        game.battlefield.get(&land).unwrap().tapped,
        "Idyllic Beachfront says it enters tapped, so it does"
    );
}

/// The land is the CR 110.5b case that a game actually plays, so this is the
/// one that has to hold end to end: `play_land` never touches
/// `place_on_battlefield`, and the tapped status can only have come from the
/// `EnterBattlefield` proposal being rewritten.
#[test]
fn test_tapland_is_tapped_through_the_pipeline_not_by_hand() {
    let mut game = setup_two_player_game();

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    // The entry was announced exactly once, by the performer.
    assert_eq!(entries(&game), vec![(land, 0)]);
    // …and the land is a gather candidate now that it is a permanent, which is
    // what `register_static_effects` records.
    assert!(game.replacement_ability_sources.contains(&land));
}

/// CR 614.12's "as opposed to a general subset of permanents that includes it":
/// a `SourceOnly` entry replacement is the entering permanent's own, and it
/// reaches nothing else that enters afterwards.
#[test]
fn test_enters_tapped_does_not_leak_to_other_permanents() {
    let mut game = setup_two_player_game();

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();
    assert!(game.battlefield.get(&land).unwrap().tapped);

    let bears = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(
        !game.battlefield.get(&bears).unwrap().tapped,
        "the land's ability affects only the land"
    );
}

/// A permanent spell resolving is the other road onto the battlefield, and it
/// is the one where CR 110.2b's default controller has to survive the
/// resolution. Both halves in one test, because the entry proposal carries them
/// together.
#[test]
fn test_resolving_permanent_spell_enters_tapped_under_its_caster() {
    let mut game = setup_two_player_game();

    let id = put_in_hand(&mut game, enters_with("Slow Bear", 2, 2, EnterMods::tapped()), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.cast_spell(0, id, &test_dp()).expect("it is castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let entry = game.battlefield.get(&id).expect("it is a permanent now");
    assert!(entry.tapped, "the spell said it enters tapped");
    assert_eq!(entry.controller, 0, "CR 110.2b — the player who put it on the stack");
    assert_eq!(entries(&game), vec![(id, 0)]);
}

// ---------------------------------------------------------------------------
// CR 122.6a — counters on entry
// ---------------------------------------------------------------------------

// COVERS: ATOM-122.6a-001
#[test]
fn test_enters_with_counters_under_its_controller() {
    let mut game = setup_two_player_game();

    // Chainbreaker is a 3/3 that enters with two -1/-1 counters, so the board
    // sees a 1/1 and the counters are on the permanent its controller controls.
    let id = reanimate(&mut game, chainbreaker(), 1);

    let entry = game.battlefield.get(&id).unwrap();
    assert_eq!(entry.counter_count(CounterType::MinusOneMinusOne), 2);
    assert_eq!(
        entry.controller, 1,
        "CR 122.6a — with no player named, the permanent's controller puts them on"
    );
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, id),
        Some(1),
        "3/3 with two -1/-1 counters"
    );
}

/// The ordering claim, and the only test in the suite that can fail if the
/// performer puts counters on *after* the entry is visible.
///
/// Faithful Watchdog ({G}{W}, 0/0, "This creature enters with three +1/+1
/// counters on it") is deliberately a fixture rather than a registered card:
/// `random_deck` filters by colour, so it would reach about one deck in
/// sixteen, and [`chainbreaker`] is what carries CR 122.6a into a fuzz game.
/// The atom it sharpens is covered by the registered card above.
#[test]
fn test_zero_toughness_creature_survives_because_counters_arrive_with_it() {
    let mut game = setup_two_player_game();

    let watchdog = enters_with(
        "Faithful Watchdog",
        0,
        0,
        EnterMods::with_counters(CounterType::PlusOnePlusOne, 3),
    );
    let id = reanimate(&mut game, watchdog, 0);

    let performed = game
        .check_state_based_actions(&ScriptedDecisionProvider::new())
        .expect("SBAs run");
    assert!(!performed, "CR 704.5f has nothing to do: it is a 3/3, not a 0/0");
    assert!(game.battlefield.contains_key(&id));
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_toughness(&game, id),
        Some(3)
    );
}

/// CR 306.5b's loyalty is the rules' own entry replacement — "a planeswalker has
/// the intrinsic ability 'This permanent enters with a number of loyalty
/// counters on it equal to its printed loyalty number.' This ability creates a
/// replacement effect (see rule 614.1c)" — and RC-2 moved it from a direct write
/// inside the performer onto the proposal, where a counter doubler will be able
/// to reach it.
///
/// A planeswalker *spell*, resolving from the stack, because that is the road
/// the rule is written about.
// COVERS: ATOM-209.1-001, ATOM-306.5b-001
#[test]
fn test_planeswalker_spell_enters_with_its_printed_loyalty() {
    let mut game = setup_two_player_game();

    let pw = CardDataBuilder::new("Jace, the Mind Sculptor")
        // One colored symbol, no generic: see `enters_with`.
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(mtgsim::types::colors::Color::Blue)
        .card_type(CardType::Planeswalker)
        .loyalty(4)
        .build();

    let id = put_in_hand(&mut game, pw, 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    game.cast_spell(0, id, &test_dp()).expect("it is castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert_eq!(
        game.battlefield.get(&id).unwrap().counter_count(CounterType::Loyalty),
        4
    );
}

// ---------------------------------------------------------------------------
// CR 616.1f — accumulation across the loop
// ---------------------------------------------------------------------------

/// Two entry-modifying abilities on one permanent, which is the shape
/// `Rewrite::EnterWith` exists for: the second application merges into the
/// first rather than replacing it, and the order does not matter.
///
/// **No printed card can build this yet** — the whole population needs {X}, a
/// condition or a trigger — so it is a fixture, and
/// `cards/phase_rc_cards.rs` records that finding rather than hiding it.
#[test]
fn test_two_entry_replacements_accumulate() {
    let mut game = setup_two_player_game();

    let data = CardDataBuilder::new("Twice-Modified Bear")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::tapped()),
            ))),
        })
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::with_counters(CounterType::Charge, 2)),
            ))),
        })
        .build();

    // Two candidates, so CR 616.1 has a genuine choice to offer — and this is
    // the **first prompt an entry has ever produced**. One prompt, not two: the
    // 616.1f re-gather finds a single candidate left and the pipeline never
    // asks with fewer than two.
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: None },
        vec![0],
    );

    let id = put_in_graveyard(&mut game, data, 0);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    let entry = game.battlefield.get(&id).unwrap();
    assert!(entry.tapped, "the first ability still applied");
    assert_eq!(
        entry.counter_count(CounterType::Charge),
        2,
        "and so did the second — `EnterWith` merges, it does not overwrite"
    );
}

/// CR 614.5 bounds the loop: an `EnterWith` produces an event its own pattern
/// still watches, so only the applied set stops it from firing forever.
#[test]
fn test_one_entry_replacement_applies_once() {
    let mut game = setup_two_player_game();

    let id = reanimate(
        &mut game,
        enters_with(
            "Charge Bear",
            2,
            2,
            EnterMods::with_counters(CounterType::Charge, 1),
        ),
        0,
    );

    assert_eq!(
        game.battlefield.get(&id).unwrap().counter_count(CounterType::Charge),
        1,
        "CR 614.5 — one opportunity, so one counter"
    );
}

// ---------------------------------------------------------------------------
// The gather source, and what strips it
// ---------------------------------------------------------------------------

/// **A known-wrong answer, asserted so that RC-3 flips it.**
///
/// `gather` reads the entering permanent's *effective* ability list, so an
/// effect that takes the ability away should take the entry modification with
/// it — Blood Moon makes a nonbasic land a Mountain with no abilities but the
/// intrinsic one (CR 305.7), so a tapland under Blood Moon enters **untapped**.
/// That is the real ruling and it is not what happens here.
///
/// The reason is one line: Blood Moon's row is an `AffectedSet::Filter`, and
/// `effect_applies_to` returns `false` for any filter effect against an object
/// that is not on the battlefield (`compute.rs`, the gate). An entering
/// permanent is never on the battlefield yet, so **no `Filter` effect reaches
/// an entry at all** — Dress Down and every Clone included.
///
/// Removing that gate is Phase **RC-3**, deliberately separated because the
/// line sits in the hottest path in the engine and its deliverable is a
/// measurement. This test is the flag on it: when RC-3 lands, the assertion
/// below inverts and this doc comment goes away.
#[test]
fn test_blood_moon_does_not_yet_strip_an_entering_taplands_ability() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, mtgsim::cards::phase_ld_cards::blood_moon(), 1);

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert!(
        game.battlefield.get(&land).unwrap().tapped,
        "RC-3 has not landed: the CR 305.7 strip cannot reach an entering permanent"
    );
    // The strip is real the moment it *is* on the battlefield, which is what
    // localises the gap to the entry rather than to Blood Moon.
    assert!(
        mtgsim::oracle::characteristics::get_effective_abilities(&game, land)
            .iter()
            .all(|a| !matches!(a.effect, Effect::Replacement(_))),
        "on the battlefield, CR 305.7 has taken the ability away"
    );
}

/// The gate leg. `replacement_ability_sources` is written by
/// `register_static_effects`, which runs *inside* the performer — so at the
/// moment the entry is proposed the entering permanent is in no gather hint
/// set, and only the explicit entering-object source finds it.
///
/// Partial on CR 614.12's second clause — "continuous effects from the
/// permanent's own static abilities that would apply to it once it's on the
/// battlefield". The clause holds here for a `SourceOnly` ability; the atom's
/// own scenario is a token copy of Voice of All, which needs copy effects
/// (Phase CV) and a choice made before entry (RC-4).
// COVERS-PARTIAL: ATOM-614.12-002
#[test]
fn test_entering_permanent_is_gathered_before_it_is_a_source() {
    let mut game = setup_two_player_game();

    assert!(
        game.replacement_ability_sources.is_empty(),
        "nothing on the board is a replacement source"
    );
    assert!(game.replacement_effects.is_empty(), "and the registry is empty");

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert!(
        game.battlefield.get(&land).unwrap().tapped,
        "found with every fast-path gate closed, which is what the gate leg is for"
    );
}

// ---------------------------------------------------------------------------
// The chokepoint
// ---------------------------------------------------------------------------

/// A token is a permanent (CR 111.1), so its entry is proposed like any other
/// and its own entry-modifying ability applies to it.
#[test]
fn test_token_entry_goes_through_the_pipeline() {
    let mut game = setup_two_player_game();

    // `place_bare` is the fixture idiom and announces nothing, so any entry
    // event below belongs to the proposal under test.
    let witness = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);
    assert!(entries(&game).is_empty());

    let id = reanimate(&mut game, chainbreaker(), 0);

    assert_eq!(entries(&game), vec![(id, 0)]);
    assert!(!game.battlefield.get(&witness).unwrap().tapped);
}

/// Every entry is one `EnterBattlefield` and one announcement — the invariant
/// that used to be "one performer, one emitter" spread over three call sites.
#[test]
fn test_entry_is_announced_exactly_once_per_permanent() {
    let mut game = setup_two_player_game();

    let a = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = reanimate(&mut game, chainbreaker(), 1);
    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert_eq!(entries(&game), vec![(a, 0), (b, 1), (land, 0)]);
}

/// The entry rides inside the zone change's batch (CR 603.2c), and it is
/// announced after it — a permanent is in the battlefield zone before it is a
/// permanent, and the log has to say so in that order.
#[test]
fn test_zone_change_is_announced_before_the_entry() {
    let mut game = setup_two_player_game();

    let id = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);

    let kinds: Vec<&'static str> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, to: Zone::Battlefield, .. } if *object_id == id => {
                Some("zone-change")
            }
            GameEvent::PermanentEnteredBattlefield { object_id, .. } if *object_id == id => {
                Some("entered")
            }
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["zone-change", "entered"]);

    let batches: Vec<_> = game
        .events
        .records()
        .iter()
        .filter(|r| {
            matches!(
                r.event,
                GameEvent::ZoneChange { to: Zone::Battlefield, .. }
                    | GameEvent::PermanentEnteredBattlefield { .. }
            )
        })
        .map(|r| r.stamp.batch)
        .collect();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], batches[1], "a nested call joins the enclosing batch");
}

/// The performer refuses to run off the chokepoint's own path: proposing an
/// entry for something that is not in the battlefield zone is a caller bug, and
/// `EnterBattlefield` is only ever proposed from inside the `ZoneChange` arm.
#[test]
fn test_direct_enter_battlefield_action_still_performs() {
    let mut game = setup_two_player_game();

    // The one shape a caller may build by hand: a token, created directly on
    // the battlefield, which `Primitive::CreateToken` does through
    // `propose_entry`.
    let data = enters_with("Token-ish", 2, 2, EnterMods::tapped());
    let obj = mtgsim::objects::object::GameObject::new(data, 0, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);

    game.execute_action(
        GameAction::EnterBattlefield {
            object: id,
            controller: 0,
            mods: EnterMods::NONE,
        },
        &ActionContext::new(&test_dp()),
    )
    .expect("it performs");

    assert!(
        game.battlefield.get(&id).unwrap().tapped,
        "its own ability was gathered and applied"
    );
}

// ---------------------------------------------------------------------------
// Card fidelity
// ---------------------------------------------------------------------------

/// Both RC-2 cards, checked against their printed text rather than against the
/// engine's opinion of it.
///
/// Partial on CR 614.1c's boundary: the in-set member ("this permanent enters
/// with …" is a replacement effect) is what both cards are, and the out-of-set
/// member ("when this permanent enters, put counters on it" is a *triggered*
/// ability) cannot be built until item 6 gives the engine triggers at all.
// COVERS-PARTIAL: BOUNDARY-DEF-614.1c-001
#[test]
fn test_rc2_cards_read_as_printed() {
    let land = idyllic_beachfront();
    assert!(land.types.contains(&CardType::Land));
    assert!(land.subtypes.contains(&Subtype::Land(LandType::Plains)));
    assert!(land.subtypes.contains(&Subtype::Land(LandType::Island)));
    assert!(
        !land.supertypes.contains(&mtgsim::types::card_types::Supertype::Basic),
        "nonbasic, which is what CR 305.7 needs to matter"
    );
    assert_eq!(
        land.abilities
            .iter()
            .filter(|a| a.ability_type == AbilityType::Mana)
            .count(),
        2
    );

    let scarecrow = chainbreaker();
    assert!(scarecrow.types.contains(&CardType::Artifact));
    assert!(scarecrow.types.contains(&CardType::Creature));
    assert_eq!(scarecrow.power, Some(3));
    assert_eq!(scarecrow.toughness, Some(3));
    assert_eq!(
        scarecrow
            .abilities
            .iter()
            .filter(|a| a.ability_type == AbilityType::Activated)
            .count(),
        1
    );
}
