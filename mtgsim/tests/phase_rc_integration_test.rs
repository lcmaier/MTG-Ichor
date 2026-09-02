//! Phase RC-2 integration tests: entering the battlefield as a replaceable event.
//!
//! RC-2 is the first RC phase that adds behaviour rather than removing it, so
//! these tests assert on two things at once, the way RB's do: the modified
//! outcome, **and** that the modification arrived through the pipeline. A
//! "enters tapped" test that passes because something wrote `tapped = true`
//! directly would be exactly the failure this phase is prone to
//! (`replacement-architecture.md` §10).

use std::sync::Arc;

use mtgsim::cards::phase_rc_cards::{
    adaptive_shimmerer, chainbreaker, idyllic_beachfront, root_maze,
};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    place_bare, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game,
    test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype};
use mtgsim::types::effects::{AffectedSet, CounterType, Effect, PermanentFilter};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EnterMods, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::zones::Zone;
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
                EventPattern::EnterBattlefield { cast: None },
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
/// **On a registered card**, which is the point rather than an incidental
/// choice: `engineering-practices.md` §3.3's sharpest finding is that a bespoke
/// fixture can cover an atom while the registered pool cannot build the same
/// scenario, and a 0/0 that enters with counters is exactly the board state
/// this claim needs. Adaptive Shimmerer is colorless, so `random_deck` puts it
/// in every deck.
#[test]
fn test_zero_toughness_creature_survives_because_counters_arrive_with_it() {
    let mut game = setup_two_player_game();

    let id = reanimate(&mut game, adaptive_shimmerer(), 0);

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
/// first rather than replacing it, and the order does not matter — provably,
/// which is why RC-4 stopped asking (`replacement-architecture.md` §11 item
/// 19). Both apply, one iteration each, with no prompt.
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
                EventPattern::EnterBattlefield { cast: None },
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
                EventPattern::EnterBattlefield { cast: None },
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::with_counters(CounterType::Charge, 2)),
            ))),
        })
        .build();

    // Two candidates, so CR 616.1 applies — and until RC-4 this was the first
    // prompt an entry ever produced. It no longer asks: every member is an
    // `EnterWith` over its own source, so no order can change the outcome, and
    // `pipeline::order_invariant_entry_bucket` says so. A provider with nothing
    // scripted is the witness — a prompt would fail this test.
    let id = put_in_graveyard(&mut game, data, 0);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
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

/// Orb of Dreams' shape: one static replacement over *every* permanent.
///
/// A fixture rather than the card, for [`enters_with`]'s reason — it varies one
/// axis, and the axis here is `AffectedSet::Filter` on the entering permanent's
/// own ability. Registered cards make the same claim from the other side:
/// `root_maze` is a filter on a permanent already on the battlefield.
fn orb_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Orb-shaped")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(mtgsim::types::colors::Color::Green)
        .card_type(CardType::Artifact)
        .rules_text("Permanents enter tapped.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                AffectedSet::Filter { filter: PermanentFilter::All },
                Rewrite::EnterWith(EnterMods::tapped()),
            ))),
        })
        .build()
}

// ---------------------------------------------------------------------------
// The gather source, and what strips it
// ---------------------------------------------------------------------------

/// CR 305.7 reaches the entry — RC-3's first consumer, and RC-2 asserted the
/// opposite here on purpose.
///
/// `gather` reads the entering permanent's *effective* ability list, so an
/// effect that takes the ability away takes the entry modification with it.
/// Blood Moon makes a nonbasic land a Mountain with no abilities but the
/// intrinsic one, so a tapland under Blood Moon enters **untapped** — the real
/// ruling, and now the engine's answer.
///
/// It needed one predicate: `effect_applies_to` gated a filter-scoped
/// `ContinuousEffect` on `game.battlefield` *membership*, and an entering
/// permanent is in the battlefield **zone** with no entry yet. Blood Moon is
/// one of CR 614.12 clause (3)'s "continuous effects that already exist and
/// would apply to the object", so the filter has to be allowed to match it.
// COVERS-PARTIAL: ATOM-614.12-001
#[test]
fn test_blood_moon_strips_an_entering_taplands_ability() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, mtgsim::cards::phase_ld_cards::blood_moon(), 1);

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert!(
        !game.battlefield.get(&land).unwrap().tapped,
        "CR 305.7 took the ability away before it could apply, so the land          enters untapped"
    );
    assert!(
        mtgsim::oracle::characteristics::get_effective_abilities(&game, land)
            .iter()
            .all(|a| !matches!(a.effect, Effect::Replacement(_))),
        "and it is still gone once the land is on the battlefield"
    );
}

/// The same rule at the *counters* half of `EnterMods`, and the sharper board.
///
/// Humility takes every creature's abilities away in Layer 6 and sets base P/T
/// 1/1 in Layer 7b. Chainbreaker "enters with two -1/-1 counters", so with the
/// ability stripped before the entry it is a plain 1/1; with the counters it
/// would be 1/1 less two, and CR 704.5f would kill it the moment SBAs ran.
/// **Alive is the assertion**, which is the difference between an entry
/// modification that did not happen and one that happened and was undone.
#[test]
fn test_humility_strips_an_entering_permanents_enters_with_counters() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, mtgsim::cards::phase_lf_cards::humility(), 1);

    let scarecrow = reanimate(&mut game, chainbreaker(), 0);

    let entry = game.battlefield.get(&scarecrow).unwrap();
    assert_eq!(
        entry.counter_count(CounterType::MinusOneMinusOne),
        0,
        "Humility took the ability away before CR 122.6a could put counters on"
    );

    game.check_state_based_actions(&ScriptedDecisionProvider::new()).unwrap();
    assert!(
        game.battlefield.contains_key(&scarecrow),
        "a 1/1 with no counters survives CR 704.5f"
    );
}

/// CR 614.12's parenthesis: an entering permanent's own **filter-scoped**
/// replacement does not apply to itself.
///
/// > Such effects may come from the permanent itself if they affect only that
/// > permanent (as opposed to a general subset of permanents that includes it).
///
/// Orb of Dreams is the CR's own example — "Permanents enter tapped", and it
/// enters untapped. A `Filter` is matched by `set_affects` against any object
/// in any zone, so without the `SelfScope` check in `gather` the entering Orb
/// finds its own row and taps itself.
///
/// The fixture is Orb-shaped rather than Orb: `PermanentFilter::All` and
/// `EnterWith(tapped)` is the whole card as far as this rule can see, and the
/// second permanent is what proves the row is live rather than inert.
// COVERS: ATOM-614.12-003
#[test]
fn test_an_entering_permanents_own_filter_replacement_does_not_reach_itself() {
    let mut game = setup_two_player_game();

    let orb = reanimate(&mut game, orb_shaped(), 0);

    assert!(
        !game.battlefield.get(&orb).unwrap().tapped,
        "CR 614.12: its own ability is not one of the effects that already exist"
    );

    // ...and the row is not inert. The next permanent in enters tapped.
    let bear = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);

    assert!(
        game.battlefield.get(&bear).unwrap().tapped,
        "the same ability does reach a permanent entering after it"
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

/// The entry is the zone change (CR 614.1c): one proposal, one batch
/// (CR 603.2c), and the move is announced before the arrival — a permanent
/// is in the battlefield zone before it is a permanent, and the log says so
/// in that order.
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
    assert_eq!(batches[0], batches[1], "one event, one batch");
}

/// A token's entry, built by hand: no `from`, because the object was created
/// in the battlefield zone — the shape `Primitive::CreateToken` proposes
/// through `propose_entry`.
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
            from: None,
            controller: 0,
            mods: EnterMods::NONE,
            cause: None,
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

    let shimmerer = adaptive_shimmerer();
    assert_eq!(shimmerer.power, Some(0));
    assert_eq!(shimmerer.toughness, Some(0));
    assert!(shimmerer
        .keyword_flags
        .contains(&mtgsim::types::keywords::KeywordFlag::Flash));

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

// ---------------------------------------------------------------------------
// CR 616.1 from two registered cards — Root Maze (RC-3)
// ---------------------------------------------------------------------------

/// A filter-scoped replacement reaches an entering permanent, and always could.
///
/// Root Maze is on the battlefield, so it is not a look-ahead question: it is
/// one of CR 614.12 clause (3)'s effects that "already exist". `set_affects`
/// matches its `AffectedSet::Filter` through
/// `GameState::permanent_matches_filter`, which has no battlefield gate — the
/// path RC-2's "no `Filter` effect reaches an entry" claim missed, and the
/// reason this test passes against the pre-RC-3 tree too.
#[test]
fn test_root_maze_taps_an_entering_land() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);

    let forest = put_in_hand(&mut game, mtgsim::cards::basic_lands::forest(), 0);
    game.play_land(0, forest, Zone::Hand, &test_ctx()).unwrap();

    assert!(
        game.battlefield.get(&forest).unwrap().tapped,
        "an opponent's Root Maze taps a land entering under it"
    );
}

/// **Two registered cards, one entry, and no prompt — §11 item 19 landed.**
///
/// Root Maze's filter and Idyllic Beachfront's own `SourceOnly` ability both
/// rewrite the same `EnterBattlefield` into `EnterWith(tapped)`. CR 616.1 makes
/// the land's controller choose which applies first, and RC-3 proved both that
/// the choice was asked and that no assertion could tell the two orders apart:
/// `EnterMods::merge` is `|=` and `+`, and neither rewrite reads the other's
/// output. RC-4 turned that theorem into `pipeline::order_invariant_entry_bucket`,
/// the pipeline no longer asks, and the CLI harness stops paying a decision
/// round-trip on every land drop under Root Maze.
///
/// The branch is not dead. The same board with Containment Priest on it asks,
/// because an `Instead` beside an `EnterWith` is a bucket whose order is a
/// different event log — `phase_rc4_integration_test`.
// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn test_two_commuting_entry_replacements_do_not_ask() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);

    let land = put_in_hand(&mut game, idyllic_beachfront(), 0);
    // A provider with nothing scripted: a prompt here fails the test.
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert!(
        game.battlefield.get(&land).unwrap().tapped,
        "both applied, and it does not matter which went first"
    );
}

/// Root Maze matches **artifacts** too, which is what puts it on a board with
/// Chainbreaker: one entry, one filter-scoped status rewrite and one
/// `SourceOnly` counters rewrite, and they are not the same half of `EnterMods`.
///
/// CR 616.1 still applies — two effects, one event — but the two rewrites
/// touch different fields of `EnterMods`, so this is the accumulation case
/// (CR 616.1f) rather than the commuting one above, and since RC-4 it is not a
/// prompt either: both are `EnterWith`s over filters no `EnterMods` field can
/// move (`pipeline::filter_is_mods_invariant`).
#[test]
fn test_root_maze_and_chainbreaker_modify_one_entry_together() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);

    let scarecrow = put_in_graveyard(&mut game, chainbreaker(), 0);
    game.change_zone(scarecrow, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .unwrap();

    let entry = game.battlefield.get(&scarecrow).unwrap();
    assert!(entry.tapped, "Root Maze's half");
    assert_eq!(
        entry.counter_count(CounterType::MinusOneMinusOne),
        2,
        "and Chainbreaker's, on the same entry"
    );
}

// ---------------------------------------------------------------------------
// CR 110.2b across a resolution — the read RC-3's gate makes reachable
// ---------------------------------------------------------------------------

/// Kismet's shape: "permanents **your opponents** control enter tapped".
///
/// The half of the four Root-Maze-family cards that RC-3 could not register
/// until `base_controller` grew its resolving arm — see the test below and
/// `root_maze`'s doc comment.
fn kismet_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Kismet-shaped")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(mtgsim::types::colors::Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("Permanents your opponents control enter tapped.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                AffectedSet::Filter {
                    filter: PermanentFilter::ByController(
                        mtgsim::types::effects::PlayerRef::Opponent,
                    ),
                },
                Rewrite::EnterWith(EnterMods::tapped()),
            ))),
        })
        .build()
}

/// A permanent spell resolving is controlled by whoever cast it, **not by its
/// owner**, and a filter that asks about controllers has to see that.
///
/// `resolve_top_of_stack` takes the `StackEntry` before it resolves anything,
/// so for the whole resolution `base_controller`'s first two probes miss. The
/// owner fallback answered, which is right for a land drop and wrong here:
/// CR 110.2b says the controller is the player who put the spell onto the
/// stack, and `GameState::resolving` is where that value lives across exactly
/// this window.
///
/// P1's Kismet-shaped enchantment scopes to its opponents. P0 casts a creature
/// **owned by P1**, so owner and controller disagree and the two answers are
/// opposite: as P0's permanent it enters tapped, as P1's it would not.
#[test]
fn test_a_spell_cast_by_a_non_owner_enters_under_its_caster_for_a_filter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, kismet_shaped(), 1);

    // No replacement ability of its own: the Kismet-shaped filter must be the
    // *only* candidate, or CR 616.1 asks a question this test is not about.
    let bear = CardDataBuilder::new("Borrowed Bear")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(mtgsim::types::colors::Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .build();
    let id = put_in_hand(&mut game, bear, 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.cast_spell(0, id, &test_dp()).expect("it is castable");

    // Owner and controller pulled apart by hand, and **after** the cast on
    // purpose: `check_cast_legality` refuses "another player's spell", so no
    // registered card can reach this board and the disagreement CR 110.2b
    // describes is unreachable end to end today. The `StackEntry` written by
    // the ordinary cast above still says P0, which is the whole point — the
    // resolution below is not doctored.
    game.get_object_mut(id).unwrap().owner = 1;
    assert_eq!(game.stack_entries[&id].controller, 0, "the cast was ordinary");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let entry = game.battlefield.get(&id).expect("it is a permanent now");
    assert_eq!(entry.controller, 0, "CR 110.2b — the player who put it on the stack");
    assert!(
        entry.tapped,
        "it entered as P0's permanent, and P0 is the enchantment controller's opponent"
    );
}
