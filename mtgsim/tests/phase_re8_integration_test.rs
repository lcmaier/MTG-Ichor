//! Phase RE-8 — the producers: discard (CR 701.9) and scry (CR 701.22).
//!
//! Two keyword actions that were stubs returning `Err`, and one replacement
//! effect scoped by what *caused* the event (CR 101.2's "by"). Three groups of
//! board:
//!
//! - **Discard** — who picks (CR 701.9b's two shapes), that N cards move as one
//!   batch (CR 603.2c's unit), and that CR 514.1's cleanup discard now does the
//!   same.
//! - **Scry** — CR 701.22a read literally: look at the top N, choose any number
//!   for the bottom, order each group. CR 701.22b's scry 0 is no event at all,
//!   and CR 701.22d's short library still scries.
//! - **`ReplacementDef::by`** — Nephalia Academy redirects a discard an
//!   opponent's spell caused, and leaves alone one caused by your own spell or
//!   by no spell at all.
//!
//! **What is not here**: the to-battlefield entry substitution and the five
//! cards that print it. Their clause is on a card in *hand* and `gather` has no
//! source that asks one, which is CR 113.6 and critical-path item 6a
//! (`cards::phase_re8_cards`'s module doc counts the population).

use std::sync::Arc;

use mtgsim::cards::phase_re8_cards::{
    eligeth_crossroads_augur, hymn_to_tourach, mind_rot, nephalia_academy, opt,
};
use mtgsim::cards::phase_re_cards::notion_thief;
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::{EventRecord, GameEvent};
use mtgsim::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game,
    test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    ObjectSet, AmountExpr, DiscardChooser, Effect, EffectRecipient, PlayerSet, Primitive,
};
use mtgsim::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::engine::targeting::{ChosenTargets};

/// The discard prompt, matched by kind alone (`ScriptedDecisionProvider`
/// compares discriminants).
const PICK_DISCARD: ChoiceKind = ChoiceKind::Discard { source: None };
/// The scry's "which go on the bottom" prompt.
const PICK_SCRY: ChoiceKind = ChoiceKind::Scry { source: None, n: 0 };
/// CR 701.22a's "in any order", either group.
const ORDER_SCRY: ChoiceKind = ChoiceKind::ScryOrder { source: None, bottom: false };
/// CR 616.1's choice between two applicable effects.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };
/// CR 616.1's "you may … instead" prompt.
const APPLY_OPTIONAL: ChoiceKind = ChoiceKind::ApplyOptionalReplacement {
    affected_object: None,
    source: ObjectId::UNASSIGNED,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities — a fixture's hand filler, or the source
/// object of a fixture resolution.
fn blank(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name).build()
}

/// A fixture sorcery whose whole text is one primitive aimed at a target
/// player. Invented, and named as one: it wears no printed card's name
/// (`engineering-practices.md` §3).
fn fixture_spell(name: &str, primitive: Primitive) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                primitive,
                EffectRecipient::Target(
                    mtgsim::types::effects::SelectionFilter::Player,
                    mtgsim::types::effects::TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

/// A fixture sorcery whose text is several primitives in order, all aimed at
/// one target player — CR 608.2c's "A, then B".
fn fixture_sequence(name: &str, primitives: Vec<Primitive>) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            // "**Target** player draws a card, then discards a card" — one
            // instance of "target" and several instructions (CR 115.3), which
            // is the shape Notion Thief's ruling is about. The first atom
            // declares the instance; the rest refer back.
            effect: Effect::Sequence(
                primitives
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let recipient = if i == 0 {
                            EffectRecipient::Target(
                                mtgsim::types::effects::SelectionFilter::Player,
                                mtgsim::types::effects::TargetCount::Exactly(1),
                            )
                        } else {
                            EffectRecipient::SameInstanceAs(0)
                        };
                        Effect::Atom(p, recipient)
                    })
                    .collect(),
            ),
        })
        .build()
}

/// Put `n` blank cards into `player`'s hand and return their ids, oldest first.
fn deal_hand(game: &mut GameState, player: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|i| put_in_hand(game, blank(&format!("Hand {i}")), player)).collect()
}

/// Resolve `card`'s one spell ability for `caster`, targeting `target_player`.
///
/// The card itself is put in the **graveyard** — it needs to be a real object
/// with an owner for `ResolutionContext::source` to name, and the graveyard is
/// where a sorcery finishing resolution ends up (CR 608.2m). Not the hand:
/// half the boards below count a hand, and a source sitting in one is a card
/// the board does not have.
fn resolve_spell_at(
    game: &mut GameState,
    card: Arc<CardData>,
    caster: PlayerId,
    target_player: PlayerId,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let source = put_in_graveyard(game, card.clone(), caster);
    let effect = card.abilities[0].effect.clone();
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: caster,
        targets: ChosenTargets::one(vec![ResolvedTarget::Player(target_player)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&effect, &ctx, dp).expect("resolving");
    source
}

/// Resolve `card`'s one spell ability for `caster` with no target — Opt's
/// shape, where every instruction names the controller.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    caster: PlayerId,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let source = put_in_graveyard(game, card.clone(), caster);
    let effect = card.abilities[0].effect.clone();
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: caster,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&effect, &ctx, dp).expect("resolving");
    source
}

/// A started two-player game whose active player holds `hand` cards before the
/// draw step, optionally with a Nephalia Academy on the battlefield.
///
/// The cleanup discard has no entry point of its own — it is a turn-based
/// action — so these boards run a whole turn, which is how
/// `pre_phase3_integration_test` reaches it too. The decks are blank cards, so
/// the only decisions in the turn are the passes and the discard.
fn cleanup_game(hand: usize, academy: bool) -> (Game, ScriptedDecisionProvider) {
    let decklist: Vec<Arc<CardData>> = (0..30).map(|i| blank(&format!("Deck {i}"))).collect();
    let mut game = Game::new(GameConfig::test(), vec![decklist.clone(), decklist])
        .expect("game creation");
    let dp = ScriptedDecisionProvider::new();
    game.setup(&dp).expect("setup");
    let active = game.state.active_player;
    if academy {
        put_on_battlefield(&mut game.state, nephalia_academy(), active);
    }
    while game.state.players[active].hand.len() < hand {
        put_in_hand(&mut game.state, blank("Extra"), active);
    }
    dp.queue_empty_turn_passes();
    (game, dp)
}

/// Every `GameEvent::ZoneChange` with a discard's cause, in log order.
fn discards(game: &GameState) -> Vec<&EventRecord> {
    game.events
        .records()
        .iter()
        .filter(|r| {
            matches!(
                r.event,
                GameEvent::ZoneChange { cause: ZoneChangeCause::Discarded, .. }
            )
        })
        .collect()
}

/// The library, top card first.
fn library_top_down(game: &GameState, player: PlayerId) -> Vec<ObjectId> {
    game.players[player].library.iter().rev().copied().collect()
}

// ===========================================================================
// CR 701.9 — discard
// ===========================================================================

/// CR 701.9b's default: the affected player chooses, and both cards move.
#[test]
fn test_mind_rot_discards_two_of_the_targets_choice() {
    let mut game = setup_two_player_game();
    let hand = deal_hand(&mut game, 1, 4);
    let dp = test_dp();
    // The victim picks, not the caster — CR 701.9b's "the affected player".
    dp.expect_pick_n(PICK_DISCARD, vec![1, 3]);

    resolve_spell_at(&mut game, mind_rot(), 0, 1, &dp);

    assert_eq!(game.players[1].hand.len(), 2, "two of four left");
    assert!(game.players[1].hand.contains(&hand[0]) && game.players[1].hand.contains(&hand[2]));
    assert_eq!(game.players[1].graveyard, vec![hand[1], hand[3]]);
    assert!(dp.is_empty(), "exactly one prompt");
}

/// CR 603.2c's unit: "whenever one or more cards are discarded" must see a
/// two-card discard as **one** event, so both members carry one `BatchId`.
#[test]
fn test_a_discard_of_two_is_one_batch() {
    let mut game = setup_two_player_game();
    deal_hand(&mut game, 1, 4);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0, 1]);

    resolve_spell_at(&mut game, mind_rot(), 0, 1, &dp);

    let moved = discards(&game);
    assert_eq!(moved.len(), 2);
    assert_eq!(moved[0].batch(), moved[1].batch(), "one instruction, one batch");
    assert!(moved[0].batch().is_some());
}

/// CR 101.3 does as much as it can, and CR 102.2 asks nothing when the choice
/// is forced — a Mind Rot against a one-card hand takes it and prompts nobody.
#[test]
fn test_mind_rot_against_a_short_hand_takes_it_all_without_asking() {
    let mut game = setup_two_player_game();
    deal_hand(&mut game, 1, 1);
    let dp = test_dp();

    resolve_spell_at(&mut game, mind_rot(), 0, 1, &dp);

    assert!(game.players[1].hand.is_empty());
    assert_eq!(game.players[1].graveyard.len(), 1);
    assert!(dp.is_empty(), "a forced choice is not a choice (CR 102.2)");
}

/// An empty hand discards nothing and announces nothing.
#[test]
fn test_mind_rot_against_an_empty_hand_does_nothing() {
    let mut game = setup_two_player_game();
    let dp = test_dp();

    resolve_spell_at(&mut game, mind_rot(), 0, 1, &dp);

    assert!(discards(&game).is_empty());
    assert!(dp.is_empty());
}

// COVERS: ATOM-701.9b-001
/// CR 701.9b's second shape: "at random" picks for the player, and the
/// atom's own expected result is that no `DecisionProvider` is asked at all.
#[test]
fn test_hymn_to_tourach_discards_at_random_and_asks_nobody() {
    let mut game = setup_two_player_game();
    let hand = deal_hand(&mut game, 1, 3);
    let dp = test_dp();

    resolve_spell_at(&mut game, hymn_to_tourach(), 0, 1, &dp);

    assert_eq!(game.players[1].hand.len(), 1);
    assert_eq!(game.players[1].graveyard.len(), 2);
    // Whatever the RNG picked, it picked out of the hand.
    for id in &game.players[1].graveyard {
        assert!(hand.contains(id));
    }
    assert!(dp.is_empty(), "CR 701.9b's random discard gives the player no choice");
}

/// The randomness is the game's, not the process's: two identically seeded
/// games discard the same cards. `tests/determinism_test.rs` is the
/// end-to-end form; this is the mechanism, so a regression names itself.
#[test]
fn test_a_random_discard_is_reproducible_from_the_seed() {
    fn discarded_positions(seed: u64) -> Vec<usize> {
        let mut game = setup_two_player_game();
        game.reseed(seed);
        let hand = deal_hand(&mut game, 1, 5);
        let dp = test_dp();
        resolve_spell_at(&mut game, hymn_to_tourach(), 0, 1, &dp);
        game.players[1]
            .graveyard
            .iter()
            .map(|id| hand.iter().position(|h| h == id).expect("from the hand"))
            .collect()
    }

    assert_eq!(discarded_positions(7), discarded_positions(7));
    // Not a claim that every pair of seeds differs — only that the seed is
    // what is being read, which a constant answer would hide.
    let a = discarded_positions(7);
    assert!((0..40).any(|s| discarded_positions(s) != a), "the seed reaches the choice");
}

/// CR 514.1 is one turn-based action over "enough cards", so the cleanup
/// discard asks once and moves them as one batch — the same unit a Mind Rot
/// makes. Until RE-8 it was a loop that asked per card and made a batch per
/// card.
#[test]
fn test_a_cleanup_discard_of_three_is_one_prompt_and_one_batch() {
    let (mut game, dp) = cleanup_game(10, false);
    let active = game.state.active_player;
    let before = game.state.events.len();
    // Eleven after the draw step against a maximum of seven: four go, in one
    // prompt.
    dp.expect_pick_n(PICK_DISCARD, vec![0, 1, 2, 3]);
    game.run_turn(&dp).expect("turn");

    assert_eq!(game.state.players[active].hand.len(), 7);
    let moved: Vec<&EventRecord> = game
        .state
        .events
        .records_from(before)
        .iter()
        .filter(|r| {
            matches!(r.event, GameEvent::ZoneChange { cause: ZoneChangeCause::Discarded, .. })
        })
        .collect();
    assert_eq!(moved.len(), 4);
    assert_eq!(moved[0].batch(), moved[3].batch(), "CR 514.1 is one turn-based action");
    assert!(dp.is_empty(), "asked once for all four");
}

/// Notion Thief's ruling, which is about **one** instruction and not two:
///
/// > If an opponent is instructed to draw a card **then discard a card**, and
/// > Notion Thief causes you to draw a card instead, that opponent still
/// > discards a card. The same is true of any other actions that opponent is
/// > instructed to do.
///
/// So the board is a single resolution that draws and then discards — a
/// looting effect — and the claim is that replacing the *draw* leaves the rest
/// of the instruction alone. Two separate spells would prove nothing: nobody
/// doubts that a later Mind Rot still discards.
///
/// RE-2 asserted this against a fixture because `Primitive::Discard` did not
/// exist; the fixture is still a fixture here — no printed card makes an
/// *opponent* loot — but both halves are real primitives now, and the discard
/// is the one that could not be written before.
#[test]
fn test_a_looting_instruction_whose_draw_the_thief_stole_still_discards() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, notion_thief(), 0);
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    let hand = deal_hand(&mut game, 1, 3);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);

    // "Target player draws a card, then discards a card", resolved by the
    // Thief's controller at the opponent — one resolution, two instructions.
    let looting = fixture_sequence(
        "Fixture Loot",
        vec![
            Primitive::DrawCards(AmountExpr::Fixed(1)),
            Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
        ],
    );
    resolve_spell_at(&mut game, looting, 0, 1, &dp);

    assert_eq!(game.players[0].hand.len(), 1, "the Thief drew instead");
    assert_eq!(
        game.players[1].hand.len(),
        2,
        "the opponent drew nothing and still discarded one of the three they had"
    );
    assert_eq!(game.players[1].graveyard, vec![hand[0]]);
    assert!(dp.is_empty());
}

/// The control board for the one above: with no Notion Thief, the same
/// instruction draws *and* discards, so the hand size is unchanged and the
/// discard is still one card.
#[test]
fn test_a_looting_instruction_draws_then_discards() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 5);
    deal_hand(&mut game, 1, 3);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);

    let looting = fixture_sequence(
        "Fixture Loot",
        vec![
            Primitive::DrawCards(AmountExpr::Fixed(1)),
            Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
        ],
    );
    resolve_spell_at(&mut game, looting, 0, 1, &dp);

    assert_eq!(game.players[1].hand.len(), 3, "drew one, discarded one");
    assert_eq!(game.players[1].graveyard.len(), 1);
}

// ===========================================================================
// CR 701.22 — scry
// ===========================================================================

/// Opt keeps the card it looked at: the library is unchanged and the draw
/// takes that same card. One prompt, and never an ordering one — a group of
/// one has no order to choose.
#[test]
fn test_opt_scrys_one_and_keeps_it_then_draws_it() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![]); // none to the bottom

    resolve_spell(&mut game, opt(), 0, &dp);

    assert_eq!(game.players[0].hand.len(), 1);
    assert_eq!(game.players[0].hand[0], before[0], "the card looked at is the card drawn");
    assert_eq!(library_top_down(&game, 0), before[1..].to_vec());
    assert!(dp.is_empty(), "scry 1 asks once and never orders");
}

/// Opt's other answer: the looked-at card goes to the bottom, so the draw
/// takes what was second.
#[test]
fn test_opt_can_bottom_the_card_it_looked_at() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![0]);

    resolve_spell(&mut game, opt(), 0, &dp);

    assert_eq!(game.players[0].hand[0], before[1], "the second card is drawn");
    let after = library_top_down(&game, 0);
    assert_eq!(*after.last().expect("non-empty"), before[0], "and the first is on the bottom");
    assert_eq!(after.len(), 4);
}

// COVERS: ATOM-701.22a-001
/// CR 701.22a whole, on the atom's own board: five cards in the library, scry
/// 3, one to the bottom and the other two kept on top **reordered**.
///
/// A fixture rather than a printed card: no registered card scries more than
/// one, and the ordering is what this atom is about — so it is built with a
/// fixture and said so (`engineering-practices.md` §4, "the CR is the
/// customer; a printed card is the test").
#[test]
fn test_scry_three_bottoms_one_and_reorders_the_rest() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();
    // Of the top three, the middle one goes to the bottom …
    dp.expect_pick_n(PICK_SCRY, vec![1]);
    // … and the other two are put back the other way round.
    dp.expect_ordering(ORDER_SCRY, vec![1, 0]);

    let scry3 = fixture_spell("Fixture Scry Three", Primitive::Scry(AmountExpr::Fixed(3)));
    resolve_spell_at(&mut game, scry3, 0, 0, &dp);

    let after = library_top_down(&game, 0);
    assert_eq!(after.len(), 5, "a scry moves no card between zones");
    assert_eq!(after[0], before[2], "reordered: the third card is now on top");
    assert_eq!(after[1], before[0]);
    assert_eq!(after[2], before[3], "the cards never looked at did not move");
    assert_eq!(after[3], before[4]);
    assert_eq!(after[4], before[1], "and the chosen one is on the bottom");
    assert!(dp.is_empty(), "two prompts: the split, then one group's order");
}

/// Both groups get their own ordering prompt when both hold two or more —
/// CR 701.22a says "in any order" twice, once per pile.
#[test]
fn test_scry_four_orders_both_groups() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 6);
    let before = library_top_down(&game, 0);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![0, 1]);
    // Top group first (the cards staying), then the bottom group.
    dp.expect_ordering(ORDER_SCRY, vec![1, 0]);
    dp.expect_ordering(ORDER_SCRY, vec![1, 0]);

    let scry4 = fixture_spell("Fixture Scry Four", Primitive::Scry(AmountExpr::Fixed(4)));
    resolve_spell_at(&mut game, scry4, 0, 0, &dp);

    let after = library_top_down(&game, 0);
    assert_eq!(after[0], before[3], "the kept pair, reversed");
    assert_eq!(after[1], before[2]);
    assert_eq!(after[4], before[1], "the bottomed pair, reversed");
    assert_eq!(after[5], before[0]);
    assert!(dp.is_empty(), "three prompts: the split and one order per pile");
}

// COVERS-PARTIAL: ATOM-701.22b-001
// The atom's board has a "whenever you scry" trigger on the battlefield and
// asserts it does not fire. Triggered abilities are critical-path item 6 and
// nothing fires today, so what is proved here is the half the rule is written
// about — *"no scry event occurs"*: nothing is looked at, nobody is asked, and
// the log holds no `Scried` line for a trigger to read. The trigger half is
// claimable the day critical-path item 6 lands, against this same board.
/// CR 701.22b — a scry 0 is not a small scry, it is no scry.
#[test]
fn test_scry_zero_is_no_event_at_all() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();

    let scry0 = fixture_spell("Fixture Scry Zero", Primitive::Scry(AmountExpr::Fixed(0)));
    resolve_spell_at(&mut game, scry0, 0, 0, &dp);

    assert_eq!(library_top_down(&game, 0), before, "nothing moved");
    assert!(dp.is_empty(), "nothing was looked at, so nobody was asked");
    assert!(
        !game.events.events().any(|e| matches!(e, GameEvent::Scried { .. })),
        "CR 701.22b: no scry event occurs"
    );
}

/// CR 701.22d — "even if some or all of those actions were impossible". A
/// scry 3 against a one-card library looks at one card and is still a scry,
/// so the event is announced with the instruction's number.
#[test]
fn test_a_short_library_still_scrys() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 1);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![]);

    let scry3 = fixture_spell("Fixture Scry Three", Primitive::Scry(AmountExpr::Fixed(3)));
    resolve_spell_at(&mut game, scry3, 0, 0, &dp);

    assert_eq!(game.players[0].library.len(), 1);
    assert!(
        game.events.events().any(|e| matches!(e, GameEvent::Scried { n: 3, .. })),
        "the event carries the instruction's number, not the count looked at"
    );
}

/// An empty library scries with nothing to look at, and still announces one
/// (CR 701.22d).
#[test]
fn test_an_empty_library_still_scrys_and_asks_nobody() {
    let mut game = setup_two_player_game();
    let dp = test_dp();

    let scry2 = fixture_spell("Fixture Scry Two", Primitive::Scry(AmountExpr::Fixed(2)));
    resolve_spell_at(&mut game, scry2, 0, 0, &dp);

    assert!(dp.is_empty());
    assert!(game.events.events().any(|e| matches!(e, GameEvent::Scried { n: 2, .. })));
}

/// Elrond, Master of Healing's ruling is that its trigger "cares about the
/// number of cards you **actually** looked at. For example, if you were
/// supposed to scry 3 but only had two cards in your library, X would be 2."
/// So the event carries both numbers, and this is the board where they differ.
///
/// The trigger itself is critical-path item 6's; what is asserted here is that
/// the fact it will read is *in the log*, because a moment later the performer
/// has rewritten the library and it is unrecoverable.
#[test]
fn test_a_scry_announces_what_was_actually_looked_at() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 2);
    let dp = test_dp();
    // Two cards there, not three: one prompt for the split and — with both
    // staying on top — one for their order.
    dp.expect_pick_n(PICK_SCRY, vec![]);
    dp.expect_ordering(ORDER_SCRY, vec![0, 1]);

    let scry3 = fixture_spell("Fixture Scry Three", Primitive::Scry(AmountExpr::Fixed(3)));
    resolve_spell_at(&mut game, scry3, 0, 0, &dp);

    let scried: Vec<(u64, u64)> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::Scried { n, looked_at, .. } => Some((*n, *looked_at)),
            _ => None,
        })
        .collect();
    assert_eq!(scried, vec![(3, 2)], "Elrond's X is 2 where CR 615.5's \"that many\" is 3");
}

/// Why the count is carried on the event rather than recomputed from the
/// library's length later: **Opt empties the library it just looked at.**
///
/// A scry moves no card between zones, so `min(n, library.len())` looks like
/// a derivation that would always work. It does not: "Scry 1. Draw a card."
/// against a one-card library looks at that card and then draws it, so a
/// reader arriving afterwards sees an empty library and derives 0 where the
/// answer is 1. Elrond, Master of Healing's X would be wrong on a card that
/// is in `PERFORMANCE_POOL`.
#[test]
fn test_the_count_looked_at_survives_the_library_it_counted() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 1);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![]);

    resolve_spell(&mut game, opt(), 0, &dp);

    assert!(game.players[0].library.is_empty(), "the scried card was then drawn");
    assert!(
        game.events
            .events()
            .any(|e| matches!(e, GameEvent::Scried { n: 1, looked_at: 1, .. })),
        "the event still says one card was looked at, which no later read could recover"
    );
}

/// The two numbers agree whenever the library is long enough, which is every
/// other board in this file.
#[test]
fn test_a_scry_with_cards_to_spare_looked_at_all_of_them() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 9);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![]);
    dp.expect_ordering(ORDER_SCRY, vec![0, 1, 2]);

    let scry3 = fixture_spell("Fixture Scry Three", Primitive::Scry(AmountExpr::Fixed(3)));
    resolve_spell_at(&mut game, scry3, 0, 0, &dp);

    assert!(game
        .events
        .events()
        .any(|e| matches!(e, GameEvent::Scried { n: 3, looked_at: 3, .. })));
}

// ===========================================================================
// Kenessos' shape — `Rewrite::Amount` over a scry
// ===========================================================================

/// Kenessos, Priest of Thassa — "if you would scry a number of cards, scry
/// that many cards plus one instead" — is the second of the two printed
/// "would scry" clauses and the only arithmetic one.
///
/// **A fixture, and the card is named rather than worn.** Kenessos' other
/// ability looks at the top card of its controller's library and acts on what
/// it is, which is `backlog.md` §2.9's information model, so the card cannot
/// be registered whole — and a fixture must not wear a real card's name while
/// behaving differently (`engineering-practices.md` §3). The arm is built
/// because the CR states the event and a printed card wants it.
fn scry_plus_one() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Scry Doubler")
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(
                ReplacementDef::new(
                    EventPattern::Scry,
                    ObjectSet::NO_OBJECTS,
                    Rewrite::Amount(AmountRewrite::Plus(1)),
                )
                .affecting_players(PlayerSet::You),
            )),
        })
        .build()
}

/// A scry 1 under Kenessos' shape looks at two cards, and the event says so.
#[test]
fn test_an_amount_rewrite_over_a_scry_makes_it_bigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, scry_plus_one(), 0);
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();
    // Two cards looked at rather than one, so both prompts are real.
    dp.expect_pick_n(PICK_SCRY, vec![0]);

    resolve_spell(&mut game, opt(), 0, &dp);

    assert!(
        game.events
            .events()
            .any(|e| matches!(e, GameEvent::Scried { n: 2, looked_at: 2, .. })),
        "scry 1 plus one is scry 2"
    );
    // The first card went to the bottom, the second stayed on top and was
    // then drawn by Opt's second instruction.
    assert_eq!(game.players[0].hand[0], before[1]);
    assert_eq!(*library_top_down(&game, 0).last().expect("non-empty"), before[0]);
    assert!(dp.is_empty());
}

/// CR 701.22b ahead of the pipeline again: a scry 0 is no event, so there is
/// nothing for an arithmetic rewrite to add one to. "Scry 0" does not become
/// "scry 1".
#[test]
fn test_an_amount_rewrite_has_no_scry_zero_to_enlarge() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, scry_plus_one(), 0);
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();

    let scry0 = fixture_spell("Fixture Scry Zero", Primitive::Scry(AmountExpr::Fixed(0)));
    resolve_spell_at(&mut game, scry0, 0, 0, &dp);

    assert_eq!(library_top_down(&game, 0), before, "nothing was looked at");
    assert!(!game.events.events().any(|e| matches!(e, GameEvent::Scried { .. })));
    assert!(dp.is_empty());
}

/// Eligeth beside Kenessos' shape is a real CR 616.1 choice, and the two
/// orders differ: add one then draw two, or draw one and leave nothing to add
/// to. The affected player chooses — here, the scrying player.
#[test]
fn test_a_scry_doubler_beside_eligeth_is_a_real_choice() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, scry_plus_one(), 0);
    put_on_battlefield(&mut game, eligeth_crossroads_augur(), 0);
    fill_library(&mut game, 0, 9);
    let dp = test_dp();
    // Index 0 is the first candidate in battlefield timestamp order — the
    // enchantment, which entered first. Applying it first makes the scry 2,
    // and Eligeth then draws that many.
    dp.expect_pick_n(PICK_REPLACEMENT, vec![0]);

    let scry1 = fixture_spell("Fixture Scry One", Primitive::Scry(AmountExpr::Fixed(1)));
    resolve_spell_at(&mut game, scry1, 0, 0, &dp);

    assert_eq!(game.players[0].hand.len(), 2, "plus one, then draw that many");
    assert!(!game.events.events().any(|e| matches!(e, GameEvent::Scried { .. })));
    assert!(dp.is_empty());
}

// ===========================================================================
// Eligeth — §10's acid test with a producer that is not a draw
// ===========================================================================

/// §10's acid test. Opt under Eligeth draws two and the log holds no scry:
/// the first of Opt's two instructions is replaced whole (CR 614.1a), the
/// second is untouched (§4.1a's instruction split), and the substitute's count
/// is the scry's own number read through `TemplateAmount::ReplacedAmount`.
#[test]
fn test_opt_with_eligeth_draws_two_and_never_scrys() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eligeth_crossroads_augur(), 0);
    fill_library(&mut game, 0, 5);
    let before = library_top_down(&game, 0);
    let dp = test_dp();

    resolve_spell(&mut game, opt(), 0, &dp);

    assert_eq!(game.players[0].hand.len(), 2, "scry 1 became draw 1, and Opt still draws");
    assert_eq!(game.players[0].hand[0], before[0]);
    assert_eq!(game.players[0].hand[1], before[1]);
    assert!(
        !game.events.events().any(|e| matches!(e, GameEvent::Scried { .. })),
        "CR 614.6: the replaced event never happened"
    );
    assert!(dp.is_empty(), "no scry prompt, because there was no scry");
}

/// "That many" is the *replaced event's* amount and not a constant: a scry 3
/// under Eligeth draws three.
#[test]
fn test_eligeth_draws_the_scrys_own_number() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eligeth_crossroads_augur(), 0);
    fill_library(&mut game, 0, 10);
    let dp = test_dp();

    let scry3 = fixture_spell("Fixture Scry Three", Primitive::Scry(AmountExpr::Fixed(3)));
    resolve_spell_at(&mut game, scry3, 0, 0, &dp);

    assert_eq!(game.players[0].hand.len(), 3);
    assert!(dp.is_empty());
}

/// Eligeth is "if **you** would scry", so an opponent's scry is untouched —
/// `PlayerSet::You` on the def, asked of the event's subject.
#[test]
fn test_eligeth_leaves_an_opponents_scry_alone() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eligeth_crossroads_augur(), 0);
    fill_library(&mut game, 1, 5);
    let dp = test_dp();
    dp.expect_pick_n(PICK_SCRY, vec![]);

    let scry1 = fixture_spell("Fixture Scry One", Primitive::Scry(AmountExpr::Fixed(1)));
    resolve_spell_at(&mut game, scry1, 1, 1, &dp);

    assert!(game.players[1].hand.is_empty(), "the opponent scried rather than drew");
    assert!(game.events.events().any(|e| matches!(e, GameEvent::Scried { .. })));
}

/// CR 701.22b ahead of the pipeline: a scry 0 is no event, so there is nothing
/// for Eligeth to replace and no card is drawn.
#[test]
fn test_eligeth_has_no_scry_zero_to_replace() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, eligeth_crossroads_augur(), 0);
    fill_library(&mut game, 0, 5);
    let dp = test_dp();

    let scry0 = fixture_spell("Fixture Scry Zero", Primitive::Scry(AmountExpr::Fixed(0)));
    resolve_spell_at(&mut game, scry0, 0, 0, &dp);

    assert!(game.players[0].hand.is_empty(), "no event, so no replacement");
    assert!(dp.is_empty());
}

// ===========================================================================
// CR 101.2's "by" — Nephalia Academy
// ===========================================================================

/// The whole of Nephalia Academy: an opponent's spell causes the discard, the
/// controller accepts the replacement, and the card goes to the top of the
/// library rather than the graveyard.
#[test]
fn test_nephalia_academy_redirects_an_opponents_discard_to_the_library() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, nephalia_academy(), 1);
    fill_library(&mut game, 1, 3);
    let hand = deal_hand(&mut game, 1, 2);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);
    dp.expect_pick_n(APPLY_OPTIONAL, vec![0]); // "you may" — yes

    let discard_one = fixture_spell(
        "Fixture Discard One",
        Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
    );
    resolve_spell_at(&mut game, discard_one, 0, 1, &dp);

    assert!(game.players[1].graveyard.is_empty(), "it did not reach the graveyard");
    assert_eq!(library_top_down(&game, 1)[0], hand[0], "on top of the library instead");
    assert!(dp.is_empty());
}

/// "You **may**" — declining leaves the discard where it was going.
#[test]
fn test_nephalia_academy_can_be_declined() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, nephalia_academy(), 1);
    fill_library(&mut game, 1, 3);
    let hand = deal_hand(&mut game, 1, 2);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);
    dp.expect_pick_n(APPLY_OPTIONAL, vec![]); // "you may" — no

    let discard_one = fixture_spell(
        "Fixture Discard One",
        Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
    );
    resolve_spell_at(&mut game, discard_one, 0, 1, &dp);

    assert_eq!(game.players[1].graveyard, vec![hand[0]]);
    assert!(dp.is_empty());
}

/// CR 101.2's "by" doing its whole job: the same discard, caused by the
/// Academy's *own* controller's spell, is not redirected — and the effect is
/// never offered, so there is no optional prompt at all.
#[test]
fn test_nephalia_academy_ignores_a_discard_you_caused_yourself() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, nephalia_academy(), 1);
    fill_library(&mut game, 1, 3);
    let hand = deal_hand(&mut game, 1, 2);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);

    let discard_one = fixture_spell(
        "Fixture Discard One",
        Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
    );
    // Player 1 casts it at themselves: the cause is their own spell. (The
    // spell itself is in that graveyard too, so this asks about the card.)
    resolve_spell_at(&mut game, discard_one, 1, 1, &dp);

    assert!(game.players[1].graveyard.contains(&hand[0]));
    assert!(!game.players[1].library.contains(&hand[0]), "not redirected");
    assert!(dp.is_empty(), "the effect was never applicable, so it was never offered");
}

/// A turn-based action has no controller, so no `SourceFilter` matches it —
/// which is why Nephalia Academy does not redirect CR 514.1's cleanup
/// discard, and why Library of Leng has to print "you have no maximum hand
/// size" in a separate sentence.
#[test]
fn test_nephalia_academy_ignores_the_cleanup_discard() {
    let (mut game, dp) = cleanup_game(7, true);
    let active = game.state.active_player;
    let graveyard_before = game.state.players[active].graveyard.len();

    // Eight after the draw step: one goes.
    dp.expect_pick_n(PICK_DISCARD, vec![0]);
    game.run_turn(&dp).expect("turn");

    assert_eq!(
        game.state.players[active].graveyard.len(),
        graveyard_before + 1,
        "CR 514.1 is nobody's spell, so the Academy's `by` does not match it"
    );
    assert!(dp.is_empty(), "and it was not even offered");
}

/// The `by` clause is about the *cause*, not about who is discarding: an
/// opponent's spell against a player with no Academy is unaffected, which is
/// the control board for the two above.
#[test]
fn test_without_the_academy_an_opponents_discard_goes_to_the_graveyard() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 3);
    let hand = deal_hand(&mut game, 1, 2);
    let dp = test_dp();
    dp.expect_pick_n(PICK_DISCARD, vec![0]);

    let discard_one = fixture_spell(
        "Fixture Discard One",
        Primitive::Discard(AmountExpr::Fixed(1), DiscardChooser::Affected),
    );
    resolve_spell_at(&mut game, discard_one, 0, 1, &dp);

    assert_eq!(game.players[1].graveyard, vec![hand[0]]);
}
