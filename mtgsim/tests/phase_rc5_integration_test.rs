//! Phase RC-5 integration tests: CR 614.13's auxiliary zone changes, and an
//! entry amount the board decides.
//!
//! **The acid tests are first, and they are first because of how they fail.**
//! Every rule this phase adds fails *silently* — a devoured creature counted
//! twice is 8 counters instead of 5, a Ghoul that exiles itself is a card in
//! the wrong zone, a Biomancer read through the wrong frame is 3 counters
//! instead of 2. None of them throws, and a test written after the code would
//! have been written to agree with whatever the code did. So §10's discipline
//! applies twice over here: assert on the *modified* outcome, and mutation-check
//! every assertion.
//!
//! Two of them — `test_614_13a_excludes_an_object_entering_simultaneously` and
//! `test_two_biomancers_entering_together_give_each_other_nothing` — build
//! their batch by calling `execute_actions` directly, because nothing in the
//! engine yet *produces* a batch with two entries in it
//! (`codebase-state.md` item 46). They prove the engine and are honest that
//! they do not prove the pool.

use std::sync::Arc;

use mtgsim::cards::phase_rc_cards::{master_biomancer, sutured_ghoul, thunder_thrash_elder};
use mtgsim::cards::phase_rs_cards::sigarda_host_of_herons;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::events::event::{BatchId, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_on_battlefield, setup_game, setup_two_player_game, static_ability,
    test_ctx, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, CounterType, Duration, Effect, EffectRecipient, PermanentFilter,
    PlayerRef, Primitive,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{
    AuxiliaryMove, EnterMods, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite,
};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Readers
// ---------------------------------------------------------------------------

fn counters(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield
        .get(&id)
        .map(|e| e.counter_count(kind))
        .unwrap_or(0)
}

/// Every `ZoneChange` since `start`, as `(object, from, to, cause)`.
fn zone_changes(
    game: &GameState,
    start: usize,
) -> Vec<(ObjectId, Zone, Zone, ZoneChangeCause)> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } => {
                Some((*object_id, *from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

/// A one-word kind for each record since `start`, paired with its batch id.
fn kinds_and_batches(game: &GameState, start: usize) -> Vec<(&'static str, Option<BatchId>)> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| {
            let kind = match &r.event {
                GameEvent::ZoneChange { .. } => "zone-change",
                GameEvent::PermanentEnteredBattlefield { .. } => "entered",
                GameEvent::CountersChanged { .. } => "counters",
                _ => return None,
            };
            Some((kind, r.batch()))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// An enchantment granting every entering creature a devour-`n` effect, the way
/// the plane card in CR 614.13b's example grants devour 5.
///
/// `AffectedSet::Filter`, so it is a second, *separate* replacement effect on
/// the same entry — which is the whole point of 614.13b: two effects, one
/// candidate, one choice.
fn grants_devour(name: &str, n: u32) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures entering have devour N.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Creature) },
            Rewrite::EnterAfterMoving(AuxiliaryMove {
                from: Zone::Battlefield,
                filter: PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                ),
                to: Zone::Graveyard,
                cause: ZoneChangeCause::Sacrificed,
                up_to: None,
                per_chosen: Some((CounterType::PlusOnePlusOne, n)),
            }),
        )))))
        .build()
}

/// "Creatures you control enter tapped" — a plain `EnterWith` to sit beside
/// Master Biomancer in one CR 616.1 bucket.
fn creatures_enter_tapped(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures you control enter tapped.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter {
                filter: PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                ),
            },
            Rewrite::EnterWith(EnterModsTemplate::tapped()),
        )))))
        .build()
}

/// A creature whose own entry replacement reads *its own* power — the shape
/// `order_invariant_entry_bucket`'s new premise exists to exclude.
fn enters_with_counters_equal_to_its_own_power(name: &str, p: i32, t: i32) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(p, t)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counter_amount(
                CounterType::PlusOnePlusOne,
                AmountExpr::SourcePower,
            )),
        )))))
        .build()
}

/// An Elf lord: "other Elf creatures you control get +1/+1", as a Layer 7c
/// registry row its own entry registers — Elvish Archdruid's first sentence.
fn elf_lord(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .power_toughness(2, 2)
        .rules_text("Other Elf creatures you control get +1/+1.")
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(PermanentFilter::And(
                Box::new(PermanentFilter::BySubtype(Subtype::Creature(CreatureType::Elf))),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// An enchantment granting every entering creature "exile any number of
/// creature cards from your graveyard", Sutured Ghoul's clause as a *second*
/// effect on someone else's entry.
///
/// The point of the pair is the **zone chain**: devour puts a creature into the
/// graveyard, and this one enumerates the graveyard. That is the one board on
/// which CR 614.13b is not redundant with the object having moved.
fn grants_graveyard_exile(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("As a creature enters, exile any number of creature cards from your graveyard.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Creature) },
            Rewrite::EnterAfterMoving(AuxiliaryMove {
                from: Zone::Graveyard,
                filter: PermanentFilter::ByType(CardType::Creature),
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
                up_to: None,
                per_chosen: None,
            }),
        )))))
        .build()
}

/// "Creatures you control can't be put into a graveyard from the battlefield" —
/// a `Rewrite::Prevent` on the sacrifice's own zone change.
///
/// **Engine-shaped rather than card-shaped, and deliberately so.** Nothing
/// printed stops a sacrifice's *move*: a sacrifice is not damage, so prevention
/// does not apply, and it is not a destruction, so regeneration and
/// indestructible do not either. The arm exists in the pipeline all the same,
/// and it is the only way to reach a chosen object whose move did not happen —
/// which is the case that separates "count what was chosen" from "count what
/// was sacrificed".
fn prevents_your_creatures_dying(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures you control can't be put into a graveyard from the battlefield.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::ZoneChange {
                from: Some(Zone::Battlefield),
                to: Some(Zone::Graveyard),
                cause: None,
                object: None,
            },
            AffectedSet::Filter { filter: PermanentFilter::ByController(PlayerRef::You) },
            Rewrite::Prevent,
        )))))
        .build()
}

/// "Creatures you control can't be sacrificed", with no source filter — the
/// unrestricted form of Sigarda's sentence, so it reaches your own abilities
/// too.
fn cant_sacrifice_your_creatures(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures you control can't be sacrificed.")
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(
            Restriction::Event {
                pattern: EventPattern::ZoneChange {
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Graveyard),
                    cause: Some(ZoneChangeCause::Sacrificed),
                    object: None,
                },
                affected: AffectedSet::Filter {
                    filter: PermanentFilter::ByController(PlayerRef::You),
                },
                by: None,
            },
        )))))
        .build()
}

/// A `+1/+1` anthem over your creatures — Glorious Anthem's shape, used to move
/// a Master Biomancer's power without touching the registry by hand.
fn anthem(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures you control get +1/+1.")
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// Reanimation's shape: the card starts in a graveyard and enters through the
/// chokepoint, so the CR 616.1 loop runs.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: usize) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
    id
}

/// The entry proposal a batch member is, built the way `propose_entry` builds
/// it. Used only by the two tests that need *two* entries in one batch, which
/// no production caller yet produces.
///
/// `EnterMods::NONE` is `default_enter_mods`' answer for every card here —
/// CR 306.5b's loyalty is the only seed it seeds, and none of these is a
/// planeswalker.
fn entry_of(game: &GameState, id: ObjectId, controller: usize) -> GameAction {
    GameAction::EnterBattlefield {
        object: id,
        from: Some(game.get_object(id).expect("in the store").zone),
        controller,
        mods: EnterMods::NONE,
        cause: Some(ZoneChangeCause::Returned),
    }
}

// ===========================================================================
// The acid tests
// ===========================================================================

/// CR 614.13b, and it is the CR's own worked example: Runeclaw Bear may be
/// sacrificed to devour 3 **or** to devour 5, never to both, so the Elder
/// enters with 0, 3 or 5 counters and never with 8.
///
/// **Fails silently without the `chosen` set.** The Bear is in the graveyard by
/// the time the second devour asks, so the *candidate* filter would already
/// exclude it — which is exactly why this is written against the count and not
/// against the graveyard: if the two effects ever apply against one board, or a
/// destination stops falling out of the next filter, 8 is what comes out.
// COVERS: ATOM-614.13b-001
#[test]
fn test_the_same_creature_cannot_be_devoured_twice() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grants_devour("Jund", 5), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let dp = test_dp();
    // CR 616.1: two applicable effects on one entry, and the choice is real —
    // neither is an `EnterWith`, so nothing suppresses the prompt. Index 1 is
    // the Elder's own printed devour 3: `gather` splices the entering
    // permanent's own effects in *after* the battlefield sweep, because
    // CR 613.7's oldest-first puts the newest object last.
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![1]);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: bear, to: Zone::Graveyard },
        vec![0],
    );
    // The granted devour 5 applies next and finds nothing left to choose, so it
    // asks nothing at all.
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        3,
        "CR 614.13b: one Bear, chosen once — devour 3 gets it and devour 5 does not, \
         so the Elder is 3 counters and never 8"
    );
    assert_eq!(
        game.players[0].graveyard.iter().filter(|&&id| id == bear).count(),
        1,
        "the Bear went to the graveyard once"
    );
    assert!(dp.is_empty(), "every scripted prompt was consumed and no other was asked");
}

/// The same rule with four players, because APNAP with one nonactive player is
/// the same answer as no APNAP at all (§10).
///
/// The choice is one player's either way — CR 616.1's chooser is the affected
/// object's controller — so what the fourth player buys is the proof that the
/// other three are not asked and that `apnap_batch_order` does not reshuffle a
/// single-member batch into someone else's turn.
// COVERS-PARTIAL: ATOM-614.13b-001
#[test]
fn test_devour_choice_in_a_four_player_game_asks_only_the_entering_controller() {
    let mut game = setup_game(4);
    put_on_battlefield(&mut game, grants_devour("Jund", 5), 2);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 2);
    // An opponent's creature is not a candidate: CR 701.21a's "its controller
    // moves it" is the `ByController(You)` leaf on the filter.
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 3);

    let dp = ScriptedDecisionProvider::new();
    // Index 0 is the granted devour 5 — see the bucket-order note above.
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: mine, to: Zone::Graveyard },
        vec![0],
    );

    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 2);
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        5,
        "the granted devour 5 was chosen first and took the one legal candidate"
    );
    assert!(
        game.battlefield.contains_key(&theirs),
        "an opponent's creature is not a candidate for 'sacrifice any number of creatures'"
    );
    assert!(dp.is_empty());
}

/// CR 614.13a, first clause: a Sutured Ghoul returning **from its own
/// graveyard** may not exile itself.
///
/// Reachable with one entry and no help, and that is RC-4b's doing: the entry
/// is decided before the card moves, so the Ghoul is still sitting in the
/// graveyard its own replacement enumerates. Without
/// `EntrySelectionScope::entering` it is offered as a candidate and a
/// `pick_all` provider exiles it — a permanent that exiled itself as it entered.
// COVERS-PARTIAL: ATOM-614.13a-001
#[test]
fn test_sutured_ghoul_cannot_exile_itself_as_it_enters() {
    let mut game = setup_two_player_game();
    let food = put_in_graveyard(&mut game, vanilla_creature(3, 3, &[]), 0);
    let ghoul = put_in_graveyard(&mut game, sutured_ghoul(), 0);

    // Answers every prompt with index 0 and records what it was asked.
    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);
    game.change_zone(ghoul, Zone::Battlefield, ZoneChangeCause::Returned, &ctx)
        .expect("it enters");

    assert!(
        game.battlefield.contains_key(&ghoul),
        "CR 614.13a: the Ghoul is not a candidate for its own exile, so it is on the \
         battlefield rather than in exile"
    );
    assert!(
        game.exile.contains(&food),
        "the other creature card in the graveyard was the only candidate, and was exiled"
    );
    assert_eq!(
        dp.prompts(),
        1,
        "one prompt: the auxiliary choice. There is no CR 616.1 choice — one effect — \
         and no optional-replacement prompt, because 'any number' includes none"
    );
    assert!(
        dp.kinds()[0].starts_with("ChooseAuxiliaryZoneChange"),
        "and it was the auxiliary choice, not something else: {:?}",
        dp.kinds()
    );
}

/// CR 614.13a, second clause: nothing entering *at the same time* may be
/// chosen — the rule's own example, Sutured Ghoul and Runeclaw Bear returning
/// from a graveyard together.
///
/// **Built at the `execute_actions` boundary on purpose.** No registered effect
/// puts two permanents onto the battlefield as one event
/// (`codebase-state.md` item 46), so this proves the engine decides the rule
/// correctly and makes no claim that a game can reach it. The producer is named
/// and sized in item 46.
// COVERS: ATOM-614.13a-001
#[test]
fn test_614_13a_excludes_an_object_entering_simultaneously() {
    let mut game = setup_two_player_game();
    // The atom's third card: in the graveyard, staying there, and so the one
    // legal choice. Without it the prompt is never asked at all and the test
    // would pass for the wrong reason — "nothing was exiled" is true of a rule
    // that works and of a rule that was never consulted.
    let bystander = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bear = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);
    let ghoul = put_in_graveyard(&mut game, sutured_ghoul(), 0);

    let dp = RecordingDecisionProvider::picking(0);
    let ctx = ActionContext::new(&dp);
    let batch = vec![entry_of(&game, ghoul, 0), entry_of(&game, bear, 0)];
    game.execute_actions(batch, &ctx).expect("both enter");

    assert!(
        game.battlefield.contains_key(&bear),
        "CR 614.13a: the Bear is entering at the same time as the Ghoul, so the Ghoul's \
         replacement may not choose it — it is on the battlefield, not in exile"
    );
    assert!(game.battlefield.contains_key(&ghoul), "and the Ghoul entered too");
    assert_eq!(
        game.exile,
        vec![bystander],
        "the only legal choice was the card that was in the graveyard and staying there"
    );
    assert_eq!(dp.prompts(), 1, "one prompt, and it offered one option");
}

/// §5b: two Master Biomancers entering as one event give each other nothing.
///
/// Every batch member's look-ahead reads the pre-batch board, so neither
/// Biomancer is on the battlefield when the other's replacements are gathered.
/// **RC-4b is what makes this true** — an entry is a phase-1 proposal, decided
/// before phase 2 performs any of them — and this is the test that says so;
/// RC-5's re-size found the restructuring already paid for
/// (`replacement-architecture.md` §9, RC-5 piece 2).
///
/// At the `execute_actions` boundary, for the reason the test above is.
#[test]
fn test_two_biomancers_entering_together_give_each_other_nothing() {
    let mut game = setup_two_player_game();
    let a = put_in_graveyard(&mut game, master_biomancer(), 0);
    let b = put_in_graveyard(&mut game, master_biomancer(), 0);

    let dp = test_dp();
    let batch = vec![entry_of(&game, a, 0), entry_of(&game, b, 0)];
    game.execute_actions(batch, &ActionContext::new(&dp)).expect("both enter");

    for (id, other) in [(a, b), (b, a)] {
        assert_eq!(
            counters(&game, id, CounterType::PlusOnePlusOne),
            0,
            "CR 614.12 / §5b: {:?} was not on the battlefield when {:?}'s entry was \
             decided, so its 'each other creature you control' gave nothing",
            other,
            id
        );
    }
}

/// §5b's second worked example, and the asymmetry it is about: an Elf lord
/// entering under Master Biomancer gets **2** counters, not 3.
///
/// Biomancer's power is read off the *real board*, where the lord's own anthem
/// is not applying — the lord is not a permanent yet, and CR 604.3 makes a
/// static ability function on the battlefield. Clause (2) of the frame puts the
/// lord's anthem into the *lord's* frame and into no other object's, which is
/// what `EntryFrame::frame_of` answering only for the entering object gives.
///
/// **Three is what a frame-for-everyone would produce**, and 3 is what this
/// asserts against.
#[test]
fn test_an_elf_lord_entering_under_biomancer_gets_two_counters_not_three() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, biomancer),
        Some(2),
        "the Biomancer is a 2/4 on the board as the lord's entry is decided"
    );

    let lord = reanimate(&mut game, elf_lord("Elvish Lord"), 0);

    assert_eq!(
        counters(&game, lord, CounterType::PlusOnePlusOne),
        2,
        "CR 614.12 / §5b: the entering lord's own anthem is in its own frame and in \
         nothing else's, so the Biomancer is read as a 2/4 and not as a 3/5"
    );
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, biomancer),
        Some(3),
        "and once the lord is a permanent the anthem does reach the Biomancer — the \
         two answers really are different, which is what makes the assertion above \
         mean something"
    );
}

// ===========================================================================
// CR 614.13 — the mechanism
// ===========================================================================

/// The headline: devour 3 with two creatures sacrificed is a 1/1 that arrives
/// as a 7/7, and the sacrifices are in the log *before* the entry.
// COVERS: ATOM-614.13-001
#[test]
fn test_devour_sacrifices_happen_before_the_entry_and_set_its_counters() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let second = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0, 1],
    );

    let mark = game.events.len();
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        6,
        "devour 3, two creatures sacrificed: three times that many counters"
    );
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, elder),
        Some(7),
        "CR 122.6a's counters are on before anything can observe the permanent, so a \
         1/1 is never seen as a 1/1"
    );

    let moves = zone_changes(&game, mark);
    assert_eq!(
        moves,
        vec![
            (first, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::Sacrificed),
            (second, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::Sacrificed),
            (elder, Zone::Graveyard, Zone::Battlefield, ZoneChangeCause::Returned),
        ],
        "CR 614.13: the auxiliary moves happen while the effect is applied, which is \
         before the entry they modify — and they are sacrifices, not destructions"
    );
}

/// The sacrifices are their own event, not part of the entry's.
///
/// §4.2's default is that a nested `execute_actions` joins the enclosing batch,
/// because CR 120.3f makes lifelink's gain a *result of* the damage. CR 614.13's
/// moves are the converse — performed while the entry is still being decided —
/// so a CR 603.2c "whenever one or more creatures die" trigger must not see them
/// and the entry as one event. Nothing reads a batch id yet
/// (critical-path item 6 is the customer), which is why this is asserted here
/// rather than left to be discovered there.
#[test]
fn test_the_auxiliary_moves_are_their_own_batch() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0],
    );

    let mark = game.events.len();
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    let seen = kinds_and_batches(&game, mark);
    let sacrifice_batch = seen[0].1.expect("the sacrifice was performed in a batch");
    let entry_batch = seen
        .iter()
        .find(|(kind, _)| *kind == "entered")
        .and_then(|(_, b)| *b)
        .expect("the entry was performed in a batch");
    assert_ne!(
        sacrifice_batch, entry_batch,
        "CR 614.13's moves are not a result of the entry, so they carry their own \
         batch id: {:?}",
        seen
    );
}

/// "Any number" includes none, and declining is a count rather than a second
/// prompt. So there is exactly one prompt and no
/// `ApplyOptionalReplacement` — devour is not a `ReplacementDef::optional`,
/// and a decline here does not spend CR 614.5's one opportunity.
#[test]
fn test_devour_declined_enters_with_no_counters_and_asks_once() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![],
    );
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 0);
    assert!(game.battlefield.contains_key(&bear), "nothing was sacrificed");
    assert!(dp.is_empty(), "one prompt, and no optional-replacement prompt beside it");
}

/// No candidates, no prompt. A `ScriptedDecisionProvider` with an empty queue
/// panics on the first prompt, which is the sharpest assertion that this path
/// asks nothing.
#[test]
fn test_devour_with_no_creatures_asks_nothing() {
    let mut game = setup_two_player_game();
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 0);
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, elder),
        Some(1),
        "a devour creature with nothing to devour is its printed self"
    );
}

/// CR 101.2 filters the *candidates*, not the moves.
///
/// A creature that can't be sacrificed is removed from the list rather than
/// offered and then refused — offering it would let a player make a choice the
/// rules do not allow, and the Elder would count a creature that never went
/// anywhere. This is `sacrifice_of_choice`'s axis-1 shape, asked of the event
/// the choice would produce.
#[test]
fn test_a_cant_be_sacrificed_effect_removes_devour_candidates() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, cant_sacrifice_your_creatures("Sanctuary"), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    // Empty queue: any prompt at all is a failure.
    let dp = test_dp();
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(game.battlefield.contains_key(&bear), "the Bear could not be sacrificed");
    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        0,
        "CR 101.2 wins over CR 614.13's choice, and the Elder counts nothing"
    );
}

/// Sigarda, Host of Herons does **not** stop your own devour, and that is the
/// card rather than a gap.
///
/// "Spells and abilities **your opponents control** can't cause you to sacrifice
/// permanents" is a `SourceFilter::ControlledBy(Opponent)` on the restriction,
/// and devour is your own creature's ability. So this is the test that the
/// `cause` threaded into the candidate filter is the *effect's* controller and
/// not something convenient: get that wrong and Sigarda silently switches devour
/// off for its own controller.
#[test]
fn test_sigarda_does_not_stop_your_own_devour() {
    let mut game = setup_two_player_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    // Both are candidates, in `battlefield_ids_ordered` order — Sigarda entered
    // first, so she is index 0, and she is not even protected from herself.
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0, 1],
    );
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        !game.battlefield.contains_key(&sigarda) && !game.battlefield.contains_key(&bear),
        "your own ability may sacrifice both — Sigarda's restriction names spells and          abilities your *opponents* control"
    );
    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 6);
}

/// An opponent's devour creature cannot eat your board: the candidate filter is
/// `ByController(You)`, resolved against the *effect's* controller.
#[test]
fn test_devour_cannot_choose_a_creature_you_do_not_control() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(game.battlefield.contains_key(&theirs));
    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 0);
}

/// The moves go through the whole CR 614 pipeline with fresh applied sets, so a
/// replacement on the sacrifice still applies — and a creature whose death was
/// *redirected* was still sacrificed, so it still counts.
///
/// CR 122.1h's finality counter is the shape: "if this permanent would be put
/// into a graveyard from the battlefield, exile it instead".
#[test]
fn test_a_devoured_creature_with_a_finality_counter_is_exiled_and_still_counts() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.add_counters(bear, CounterType::Finality, 1);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0],
    );
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        game.exile.contains(&bear),
        "CR 122.1h replaced the sacrifice's destination — the nested batch is a real \
         trip through the pipeline, not a direct write"
    );
    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        3,
        "a redirected move still happened, so the creature was still sacrificed and \
         devour still counts it"
    );
}

// ===========================================================================
// CR 614.1c/d — an amount the board decides
// ===========================================================================

/// Master Biomancer's headline, and the mutation that would break it: a 2/4
/// gives two counters, and a Biomancer pumped to 5 gives five.
#[test]
fn test_biomancer_gives_counters_equal_to_its_current_power() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);

    let first = reanimate(&mut game, vanilla_creature(1, 1, &[]), 0);
    assert_eq!(counters(&game, first, CounterType::PlusOnePlusOne), 2);

    // An anthem makes the Biomancer a 3/5. The amount is read off the *effective*
    // characteristics, so the next creature gets three counters and not the
    // printed two.
    put_on_battlefield(&mut game, anthem("Glorious-ish Anthem"), 0);
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, biomancer),
        Some(3),
        "the anthem moved the Biomancer's power"
    );

    let second = reanimate(&mut game, vanilla_creature(1, 1, &[]), 0);
    assert_eq!(
        counters(&game, second, CounterType::PlusOnePlusOne),
        3,
        "'equal to this creature's power' is the layer walk's answer, not the printed 2"
    );
}

/// The Biomancer does not give itself counters, and it needs no "other" clause
/// to say so: gather source 1 sweeps the battlefield, where an entering object
/// is not, and source 1a admits only `AffectedSet::SourceOnly`.
#[test]
fn test_a_biomancer_entering_alone_gives_itself_nothing() {
    let mut game = setup_two_player_game();
    let biomancer = reanimate(&mut game, master_biomancer(), 0);
    assert_eq!(counters(&game, biomancer, CounterType::PlusOnePlusOne), 0);
}

/// An opponent's Biomancer gives your creatures nothing: "creature **you**
/// control" is resolved against the effect's controller.
#[test]
fn test_an_opponents_biomancer_gives_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 1);

    let mine = reanimate(&mut game, vanilla_creature(1, 1, &[]), 0);
    assert_eq!(counters(&game, mine, CounterType::PlusOnePlusOne), 0);
}

// ===========================================================================
// §11 item 19 / item 47 — the order-invariance predicate, revisited
// ===========================================================================

/// A Biomancer beside a plain `EnterWith` still suppresses CR 616.1's prompt.
///
/// The premise the dynamic amount forced is *exact*, not conservative: an
/// amount is order-invariant when it is `Fixed` **or** its source is not the
/// entering object, because `EntryFrame::frame_of` answers only for the
/// entering object. A conservative "all amounts `Fixed`" would prompt here, and
/// every fuzz game with a Biomancer would start asking a question with one
/// outcome.
#[test]
fn test_a_biomancer_bucket_is_still_order_invariant() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    put_on_battlefield(&mut game, creatures_enter_tapped("Kismet-ish"), 0);

    // Empty queue: a CR 616.1 prompt here is a failure.
    let dp = test_dp();
    let id = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(counters(&game, id, CounterType::PlusOnePlusOne), 2, "both applied");
    assert!(game.battlefield.get(&id).unwrap().tapped, "and so did the other one");
}

/// The other side of the same premise: an amount that reads the *entering*
/// object's own frame is order-dependent, so the prompt is real.
///
/// Two counters applied in the other order give a different number, which is
/// why `order_invariant_entry_bucket` must not suppress this bucket.
#[test]
fn test_a_self_read_amount_forces_the_ordering_prompt() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, creatures_enter_tapped("Kismet-ish"), 0);

    let dp = test_dp();
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);

    let id = put_in_graveyard(
        &mut game,
        enters_with_counters_equal_to_its_own_power("Narcissus", 2, 2),
        0,
    );
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        dp.is_empty(),
        "CR 616.1's prompt was asked: a `SourcePower` amount on the entering object's \
         own effect reads the frame, so the two applications do not commute"
    );
    assert_eq!(counters(&game, id, CounterType::PlusOnePlusOne), 2);
}

// ===========================================================================
// The exclusion sets are batch-scoped, not game-scoped
// ===========================================================================

/// A second entry in a *later* batch starts with empty exclusion sets: CR
/// 614.13b is about "replacement effects that modify how one or more permanents
/// enter", which is one simultaneous entry event, not the whole game.
#[test]
fn test_the_exclusion_sets_do_not_leak_between_batches() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let cub = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    let dp = test_dp();
    let first = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: first, to: Zone::Graveyard },
        vec![0],
    );
    game.change_zone(first, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    // The second Elder is a different event, so the first Elder itself is a
    // legal candidate for it — nothing about the earlier batch survives.
    let second = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: second, to: Zone::Graveyard },
        vec![0, 1],
    );
    game.change_zone(second, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert_eq!(counters(&game, second, CounterType::PlusOnePlusOne), 6);
    assert!(
        !game.battlefield.contains_key(&bear) || !game.battlefield.contains_key(&cub),
        "the second Elder ate two of the three creatures available to it"
    );
    assert!(dp.is_empty());
}

// ===========================================================================
// The three the mutation pass added
// ===========================================================================

/// CR 614.13b where it is **not** redundant: two effects on one entry whose
/// zones chain.
///
/// Devour puts the Bear into the graveyard; the granted "exile any number of
/// creature cards from your graveyard" then enumerates that graveyard and would
/// find it there. Without `EntrySelectionScope::chosen` the same object is
/// chosen to change zones twice while applying replacement effects to one
/// entry, which is exactly the sentence 614.13b is.
///
/// **This is the board the first draft of these tests could not build.** With
/// both effects reading the battlefield, a sacrificed creature stops matching
/// the next filter on its own and the rule looks like bookkeeping; a mutation
/// pass is what found that out.
///
/// **Asserted as the absence of a third prompt**, which is the sharpest form
/// available: with the Bear excluded there is no candidate left in the
/// graveyard — the Elder itself is excluded by 614.13a — so the second effect
/// asks nothing, and a `ScriptedDecisionProvider` with an empty queue panics
/// the moment it is asked anything. It also catches the exclusion sets being
/// lost across the *nested* batch the sacrifice runs in, which is why they are
/// recorded before the moves rather than after.
// COVERS: ATOM-614.13b-001
#[test]
fn test_a_devoured_creature_cannot_then_be_exiled_by_the_next_effect() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grants_graveyard_exile("Grave Chain"), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    // Devour first (index 1 — the entering permanent's own effect is spliced in
    // after the sweep), so the Bear is in the graveyard when the second effect
    // asks.
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![1]);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0],
    );

    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        game.players[0].graveyard.contains(&bear),
        "CR 614.13b: the Bear was already chosen to change zones while applying an entry          replacement, so the next one may not choose it — it stays in the graveyard"
    );
    assert!(game.exile.is_empty(), "and nothing was exiled");
    assert!(
        dp.is_empty(),
        "two prompts, not three: with the Bear excluded by 614.13b and the Elder by          614.13a, the second effect had no candidate and asked nothing"
    );
}

/// The exclusion sets belong to one batch: a *later* entry may choose what an
/// earlier one did.
///
/// CR 614.13b scopes itself to "replacement effects that modify how one or more
/// permanents enter the battlefield" — one simultaneous entry event. A second
/// entry, later, is a different event, and the Bear the Elder devoured is a
/// perfectly good thing for a Sutured Ghoul to exile.
#[test]
fn test_a_later_entry_may_choose_what_an_earlier_one_chose() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0],
    );
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");
    assert!(game.players[0].graveyard.contains(&bear));

    let ghoul = put_in_graveyard(&mut game, sutured_ghoul(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: ghoul, to: Zone::Exile },
        vec![0],
    );
    game.change_zone(ghoul, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        game.exile.contains(&bear),
        "a new batch starts with empty exclusion sets — the Bear was excluded from the \
         Elder's entry, not from every entry for the rest of the game"
    );
    assert!(dp.is_empty());
}

/// Devour counts what was *sacrificed*, and a move that did not happen is not a
/// sacrifice (CR 701.21a: "its controller moves it from the battlefield to that
/// player's graveyard").
///
/// The chosen creature is still on the battlefield afterward, and the Elder is
/// a 1/1: it may not count a creature it did not spend.
#[test]
fn test_a_prevented_sacrifice_is_not_counted() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, prevents_your_creatures_dying("Sanctum"), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    dp.expect_pick_n(
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, to: Zone::Graveyard },
        vec![0],
    );
    game.change_zone(elder, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("it enters");

    assert!(
        game.battlefield.contains_key(&bear),
        "CR 614.6: the move did not happen, so the Bear is where it was"
    );
    assert_eq!(
        counters(&game, elder, CounterType::PlusOnePlusOne),
        0,
        "and it was never sacrificed, so devour counts nothing — the count is what the \
         nested batch performed, not what the prompt returned"
    );
}
