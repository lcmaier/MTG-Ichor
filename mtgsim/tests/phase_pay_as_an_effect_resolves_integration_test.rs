//! `pay-as-an-effect-resolves`: a cost paid as an effect resolves (CR
//! 118.12), "unless [a player] pays" (118.12a), "any player may" (101.4) and
//! ward (702.21a) — `choices-architecture.md`, decision 4.
//!
//! **Every board is cast or triggered, then resolved**: the payment is a
//! question a resolution asks, and only a resolution can ask it. Four groups:
//!
//! - **The payment** — offered only to a payer who can pay (CR 118.3), its
//!   answer read by "if you do", mana abilities activated as it resolves (CR
//!   605.3a), its parts in the order written (CR 608.2c), its records the
//!   resolution's own.
//! - **Who pays** — "its controller" (Mana Leak), "that player" (ward,
//!   Rhystic Study's shape), "any player" in APNAP order (Wild Might's shape).
//! - **How much** — an amount chosen as it is paid, which is X (CR 107.3f).
//! - **The cards** — Mana Leak, Waterfall Aerialist's ward and
//!   Contamination's upkeep, each with its rulings.

use std::sync::Arc;

use mtgsim::cards::authoring::{
    any_player, draw_cards, gets, may_pay, may_pay_any_amount_of, spell_ability, target_creature, that_target, triggered_ability, unless,
    whenever, you,
};
use mtgsim::cards::printings::{
    contamination, giant_growth, grizzly_bears, island, jagged_lightning, mana_leak, mountain, waterfall_aerialist,
};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer, ObjectSet};
use mtgsim::events::cause::EventCause;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_abilities, get_effective_power};
use mtgsim::oracle::x_values::XOffer;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::engine::actions::GameAction;
use mtgsim::test_support::{
    fill_library, lightning_bolt, put_in_hand, put_on_battlefield, set_active_player, setup_game, setup_two_player_game, test_ctx,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Condition, CostAnswer, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::TriggerEvent;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{PriorityAction, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Boards
// ---------------------------------------------------------------------------

/// A payment's question, matched by its kind alone.
const PAY: ChoiceKind = ChoiceKind::PayAsItResolves { source: ObjectId::UNASSIGNED, offer: Vec::new(), any_amount_of: None };

/// A mana window's question, matched by its kind alone.
fn window() -> ChoiceKind {
    ChoiceKind::ManaAbilityWindow { spell_or_ability_id: ObjectId::UNASSIGNED, remaining_cost: ManaCost::zero() }
}

fn amount() -> ChoiceKind {
    ChoiceKind::ChooseAmountToPay {
        source: ObjectId::UNASSIGNED,
        unit: ManaCost::zero(),
        values: Box::new(XOffer::Interval { at_least: 0, at_most: 0 }),
    }
}

fn generic(n: u8) -> Cost {
    Cost::Mana(ManaCost::build(&[], n))
}

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

/// `player`'s pool emptied, then filled with `pool`.
fn pool_of(game: &mut GameState, player: PlayerId, pool: &[(ManaType, u64)]) {
    for t in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
}

/// `player` casts `card` from hand with exactly `pool`, aiming at `aims`, one
/// pick per instance of "target" that has a choice: an instance with one
/// legal set takes it unasked.
fn cast(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], aims: &[Vec<ObjectId>]) -> ObjectId {
    let id = put_in_hand(game, card, player);
    pool_of(game, player, pool);
    let aim = ScriptedDecisionProvider::new();
    for picks in aims {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            picks.iter().map(|&o| ChoiceOption::Object(o)).collect(),
        );
    }
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    id
}

/// Place what triggered and resolve until the stack is empty, `script`
/// answering every question and a covered mana window declined for it.
/// Returns the script, to read what it was asked.
fn resolve_with(game: &mut GameState, script: ScriptedDecisionProvider) -> ScriptedDecisionProvider {
    let payer = ManaWindowStop::new(script);
    game.perform_sba_and_triggers(&payer).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&payer).unwrap();
        game.perform_sba_and_triggers(&payer).unwrap();
    }
    let script = payer.into_inner();
    assert!(script.is_empty(), "{} answers left unasked", script.remaining());
    script
}

/// The payment questions `dp` was asked, by whom.
fn paid_asked(dp: &ScriptedDecisionProvider) -> Vec<PlayerId> {
    dp.asked_of().into_iter().filter(|(_, kind)| matches!(kind, ChoiceKind::PayAsItResolves { .. })).map(|(p, _)| p).collect()
}

fn hand(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
}

/// "Sorcery — you may pay {1}. If you do, draw a card": the shape of the
/// 460 "you may pay … if you do" cards (`choices-architecture.md` §3.2).
/// Invented: the printed ones are triggers or carry more text, and a sorcery
/// with the clause alone puts the payment first on the stack.
fn tithe_of_insight() -> Arc<CardData> {
    let draw = Effect::Conditional(Condition::CostAnswer(CostAnswer::Does), Box::new(draw_cards(you(), 1)));
    CardDataBuilder::new("Fixture Tithe of Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .card_type(CardType::Sorcery)
        .ability(spell_ability("", Effect::Sequence(vec![may_pay(PlayerRef::You, vec![generic(1)]), draw])))
        .build()
}

/// Rhystic Study's shape, ATOM-605.3a-002's board: "Whenever an opponent
/// casts a spell, you may draw a card unless that player pays {1}." A
/// fixture, not the card: the brief registers Mana Leak and one ward
/// creature, and the atom's board is what this test builds.
fn rhystic_study_shape() -> Arc<CardData> {
    let may_draw = Effect::Optional { chooser: PlayerRef::You, effect: Box::new(draw_cards(you(), 1)) };
    let cast = TriggerEvent::CastsSpell { caster: Some(PlayerRef::Opponent), spell: None };
    CardDataBuilder::new("Fixture Rhystic Study")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(cast, unless(mtgsim::cards::authoring::that_player(), vec![generic(1)], may_draw))))
        .build()
}

// ---------------------------------------------------------------------------
// The payment
// ---------------------------------------------------------------------------

/// "You may pay {1}. If you do, draw a card": taken, the cost is paid and
/// "if you do" reads `Does`.
#[test]
fn you_may_pay_pays_and_if_you_do_reads_it() {
    let mut game = stocked(2);
    let before = hand(&game, 0);
    cast(&mut game, 0, tithe_of_insight(), &[(ManaType::Blue, 1), (ManaType::Colorless, 1)], &[]);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    resolve_with(&mut game, dp);
    assert_eq!(hand(&game, 0), before + 1, "paid, and drew");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 0, "the {{1}} was paid");
}

/// Declined, nothing is paid, "if you do" reads `Doesnt`, and the mana stays.
#[test]
fn declining_reads_doesnt_and_pays_nothing() {
    let mut game = stocked(2);
    let before = hand(&game, 0);
    cast(&mut game, 0, tithe_of_insight(), &[(ManaType::Blue, 1), (ManaType::Colorless, 1)], &[]);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    resolve_with(&mut game, dp);
    assert_eq!(hand(&game, 0), before);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 1);
}

/// CR 118.3: a player with nothing to pay with is not asked, and doesn't.
#[test]
fn a_player_who_cannot_pay_is_not_asked() {
    let mut game = stocked(2);
    let before = hand(&game, 0);
    cast(&mut game, 0, tithe_of_insight(), &[(ManaType::Blue, 1)], &[]);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert_eq!(hand(&game, 0), before, "the {{1}} was never offered");
    assert!(paid_asked(&dp).is_empty());
}

/// ATOM-605.3a-002, Rhystic Study's board: the opponent has no mana in their
/// pool and an untapped land, and activates its mana ability while the
/// ability asking for {1} resolves. Paid, the Study's controller draws
/// nothing.
// COVERS: ATOM-605.3a-002
#[test]
fn mana_abilities_are_activated_while_the_payment_resolves() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, rhystic_study_shape(), 0);
    let land = put_on_battlefield(&mut game, mountain(), 1);
    set_active_player(&mut game, 1);
    cast(&mut game, 1, grizzly_bears(), &[(ManaType::Green, 2)], &[]);
    let before = hand(&game, 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    let tap = get_effective_abilities(&game, land)[0].id;
    dp.expect_choice(window(), vec![ChoiceOption::Action(PriorityAction::ActivateAbility(land, tap))]);
    let dp = resolve_with(&mut game, dp);

    assert_eq!(paid_asked(&dp), vec![1], "that player is asked");
    assert!(game.battlefield[&land].tapped, "the land paid");
    assert_eq!(hand(&game, 0), before, "paid, so the Study draws nothing");
}

/// Not paid, the Study's controller may draw: a decline, then "you may".
#[test]
fn rhystic_study_shape_draws_when_that_player_declines() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, rhystic_study_shape(), 0);
    put_on_battlefield(&mut game, mountain(), 1);
    set_active_player(&mut game, 1);
    cast(&mut game, 1, grizzly_bears(), &[(ManaType::Green, 2)], &[]);
    let before = hand(&game, 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    dp.expect_take(ChoiceKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED });
    resolve_with(&mut game, dp);
    assert_eq!(hand(&game, 0), before + 1);
}

/// A window that ends with the cost unpaid is the payer declining: the mana
/// made stays in the pool (CR 106.4), and nothing is reversed.
#[test]
fn a_window_that_ends_unpaid_is_a_decline() {
    let mut game = stocked(2);
    let lands: Vec<ObjectId> = (0..3).map(|_| put_on_battlefield(&mut game, mountain(), 1)).collect();
    set_active_player(&mut game, 1);
    let bears = cast(&mut game, 1, grizzly_bears(), &[(ManaType::Green, 2)], &[]);
    cast(&mut game, 0, mana_leak(), &[(ManaType::Blue, 2)], &[]);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    let tap = get_effective_abilities(&game, lands[0])[0].id;
    dp.expect_choice(window(), vec![ChoiceOption::Action(PriorityAction::ActivateAbility(lands[0], tap))]);
    dp.expect_choice(window(), Vec::new());
    resolve_with(&mut game, dp);

    assert_eq!(game.objects[&bears].zone, Zone::Graveyard, "unpaid, so countered");
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Red), 1, "the mana made stays");
}

/// "You may sacrifice a creature and pay 2 life": paid in the order written
/// (CR 608.2c), never CR 601.2h's ranks, which would pay the life first; and
/// both records are the resolution's own, a cost a resolution asks for being
/// an effect's (Sigarda's Mogis ruling). Invented: a payment whose two parts
/// a cast would reorder, which no printed "you may" pairs this way.
#[test]
fn the_costs_are_paid_in_the_order_written_under_the_resolutions_cause() {
    let offering = CardDataBuilder::new("Fixture Blood Offering")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .card_type(CardType::Sorcery)
        .ability(spell_ability(
            "",
            Effect::Sequence(vec![
                may_pay(PlayerRef::You, vec![Cost::Sacrifice(ObjectFilter::ByType(CardType::Creature), 1), Cost::PayLife(2)]),
                Effect::Conditional(Condition::CostAnswer(CostAnswer::Does), Box::new(draw_cards(you(), 2))),
            ]),
        ))
        .build();
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spell = cast(&mut game, 0, offering, &[(ManaType::Black, 1)], &[]);
    let from = game.recorded_events().len();
    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    resolve_with(&mut game, dp);

    assert_eq!(game.objects[&bears].zone, Zone::Graveyard, "sacrificed");
    let records = game.recorded_events();
    let paid: Vec<(&str, &EventCause)> = records
        .records_from(from)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { act: mtgsim::engine::actions::ZoneChangeAct::Sacrificed, .. } => Some(("sacrifice", &r.stamp.cause)),
            GameEvent::LifeChanged { player_id: 0, .. } => Some(("life", &r.stamp.cause)),
            _ => None,
        })
        .collect();
    assert_eq!(paid.iter().map(|(what, _)| *what).collect::<Vec<_>>(), vec!["sacrifice", "life"]);
    for (what, cause) in paid {
        assert!(matches!(cause, EventCause::Effect { resolution, .. } if resolution.source == spell), "{what}: {cause:?}");
    }
}

// ---------------------------------------------------------------------------
// Who pays
// ---------------------------------------------------------------------------

/// Player 1 casts Grizzly Bears with exactly its cost and `spare` left over,
/// and player 0 casts Mana Leak at it.
fn mana_leak_at_bears(spare: u64) -> (GameState, ObjectId, ObjectId) {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let bears = cast(&mut game, 1, grizzly_bears(), &[(ManaType::Green, 2)], &[]);
    game.players[1].mana_pool.add(ManaType::Colorless, spare);
    // The one spell it can target.
    let leak = cast(&mut game, 0, mana_leak(), &[(ManaType::Blue, 2)], &[]);
    (game, bears, leak)
}

/// "Its controller" is the target spell's: player 1 is asked, pays {3}, and
/// the Bears resolve.
#[test]
fn mana_leak_asks_the_target_spells_controller_who_pays() {
    let (mut game, bears, leak) = mana_leak_at_bears(3);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    let dp = resolve_with(&mut game, dp);
    assert_eq!(paid_asked(&dp), vec![1]);
    assert!(game.battlefield.contains_key(&bears), "paid, so the Bears resolved");
    assert_eq!(game.objects[&leak].zone, Zone::Graveyard);
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Colorless), 0);
}

#[test]
fn mana_leak_counters_a_spell_whose_controller_declines() {
    let (mut game, bears, _) = mana_leak_at_bears(3);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    resolve_with(&mut game, dp);
    assert_eq!(game.objects[&bears].zone, Zone::Graveyard, "countered");
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Colorless), 3, "nothing paid");
}

/// A tapped-out player is not asked, and their spell is countered.
#[test]
fn mana_leak_counters_a_tapped_out_players_spell_unasked() {
    let (mut game, bears, _) = mana_leak_at_bears(2);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert!(paid_asked(&dp).is_empty(), "two mana cannot pay {{3}}");
    assert_eq!(game.objects[&bears].zone, Zone::Graveyard);
}

/// ATOM-702.21a-001: ward triggers above the Bolt, and its controller pays
/// {2} or the Bolt is countered — both branches.
// COVERS: ATOM-702.21a-001
#[test]
fn ward_counters_the_spell_unless_that_player_pays() {
    for pays in [true, false] {
        let mut game = stocked(2);
        let aerialist = put_on_battlefield(&mut game, waterfall_aerialist(), 0);
        set_active_player(&mut game, 1);
        let bolt = cast(&mut game, 1, lightning_bolt(), &[(ManaType::Red, 1)], &[vec![aerialist]]);
        game.players[1].mana_pool.add(ManaType::Colorless, 2);
        let dp = ScriptedDecisionProvider::new();
        if pays {
            dp.expect_take(PAY);
        } else {
            dp.expect_choice(PAY, Vec::new());
        }
        let dp = resolve_with(&mut game, dp);
        assert_eq!(paid_asked(&dp), vec![1], "that player, the Bolt's controller, is asked");
        assert_eq!(game.objects[&bolt].zone, Zone::Graveyard);
        assert_eq!(game.battlefield.contains_key(&aerialist), !pays, "paid {pays}: the Bolt resolved {pays}");
    }
}

/// ATOM-702.21a-002: its controller's own spell does not trigger ward, and no
/// one is asked to pay.
// COVERS: ATOM-702.21a-002
#[test]
fn ward_ignores_its_controllers_own_spell() {
    let mut game = stocked(2);
    let aerialist = put_on_battlefield(&mut game, waterfall_aerialist(), 0);
    cast(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[]);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert!(paid_asked(&dp).is_empty());
    assert_eq!(get_effective_power(&game, aerialist), Some(6));
}

/// Waterfall Aerialist's ruling: a spell targeting two ward creatures
/// triggers both, and is countered unless both are paid for.
// RULING: Waterfall Aerialist #1 - "If a player casts a spell that targets multiple permanents their opponent controls with ward, each of those ward abilities will trigger. If that player doesn't pay for all of them, the spell will be countered."
#[test]
fn a_spell_targeting_two_ward_creatures_pays_for_each() {
    for pays_both in [true, false] {
        let mut game = stocked(2);
        let first = put_on_battlefield(&mut game, waterfall_aerialist(), 0);
        let second = put_on_battlefield(&mut game, waterfall_aerialist(), 0);
        set_active_player(&mut game, 1);
        // Jagged Lightning's two targets are the two creatures there are.
        let shock = cast(&mut game, 1, jagged_lightning(), &[(ManaType::Red, 5)], &[]);
        game.players[1].mana_pool.add(ManaType::Colorless, 4);
        let dp = ScriptedDecisionProvider::new();
        // The two triggers are one tier of one player's, and no order of
        // them changes the game: placed unasked.
        dp.expect_take(PAY);
        if pays_both {
            dp.expect_take(PAY);
        } else {
            dp.expect_choice(PAY, Vec::new());
        }
        let dp = resolve_with(&mut game, dp);
        assert_eq!(paid_asked(&dp), vec![1, 1], "two triggers, each asked");
        assert_eq!(game.objects[&shock].zone, Zone::Graveyard);
        let survived = [first, second].iter().filter(|id| game.battlefield.contains_key(id)).count();
        assert_eq!(survived, if pays_both { 0 } else { 2 }, "paid for both: {pays_both}");
    }
}

/// Wild Might's shape: "Target creature gets +1/+1 until end of turn. That
/// creature gets an additional +4/+4 until end of turn unless any player
/// pays {2}." A fixture, not the card, as Rhystic Study's is: the brief's
/// cards are Mana Leak and one ward creature.
fn wild_might_shape() -> Arc<CardData> {
    let more = gets(that_target(0), 4, 4).until_end_of_turn();
    CardDataBuilder::new("Fixture Wild Might")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "",
            Effect::Sequence(vec![gets(target_creature(), 1, 1).until_end_of_turn(), unless(any_player(), vec![generic(2)], more)]),
        ))
        .build()
}

/// "Any player" is asked in APNAP order (CR 101.4), the active player first,
/// until one pays: the first declines, the second pays, and the creature gets
/// +1/+1 alone. Played with each player active, so the order is the turn's
/// and not the seats'.
#[test]
fn any_player_is_asked_in_apnap_order_until_one_pays() {
    for active in [0, 1] {
        let mut game = stocked(2);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), active);
        set_active_player(&mut game, active);
        cast(&mut game, active, wild_might_shape(), &[(ManaType::Green, 1)], &[]);
        game.players[0].mana_pool.add(ManaType::Colorless, 2);
        game.players[1].mana_pool.add(ManaType::Colorless, 2);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_choice(PAY, Vec::new());
        dp.expect_take(PAY);
        let dp = resolve_with(&mut game, dp);
        let other = 1 - active;
        assert_eq!(paid_asked(&dp), vec![active, other], "the active player first");
        assert_eq!(get_effective_power(&game, bears), Some(3), "paid, so +1/+1 alone");
        assert_eq!(game.players[other].mana_pool.amount(ManaType::Colorless), 0, "the second player paid");
    }
}

/// The active player, who can't pay, is passed over unasked, and the next
/// player is asked.
#[test]
fn any_player_passes_over_one_who_cannot_pay() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    cast(&mut game, 0, wild_might_shape(), &[(ManaType::Green, 1)], &[]);
    game.players[1].mana_pool.add(ManaType::Colorless, 2);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    let dp = resolve_with(&mut game, dp);
    assert_eq!(paid_asked(&dp), vec![1], "player 0 is not asked");
    assert_eq!(get_effective_power(&game, bears), Some(7));
}

/// Nobody who can pay: nobody is asked, and the creature gets +5/+5.
#[test]
fn any_player_with_no_payer_does_the_rest() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    cast(&mut game, 0, wild_might_shape(), &[(ManaType::Green, 1)], &[]);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert!(paid_asked(&dp).is_empty());
    assert_eq!(get_effective_power(&game, bears), Some(7));
}

// ---------------------------------------------------------------------------
// How much
// ---------------------------------------------------------------------------

/// Elenda and Azor's shape, the mana its own: "You may pay {X}{U}. If you
/// do, draw X cards." X is in no cost, so it is chosen as the payment is
/// made (CR 107.3f), from what the payer can pay.
fn paid_insight() -> Arc<CardData> {
    let draw = Effect::Conditional(Condition::CostAnswer(CostAnswer::Does), Box::new(draw_cards(you(), AmountExpr::X)));
    let pay = Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(Effect::Atom(
            Primitive::Pay { costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Blue], 0))], any_amount_of: Some(ManaCost::build(&[], 1)) },
            EffectRecipient::Implicit,
        )),
    };
    CardDataBuilder::new("Fixture Paid Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .card_type(CardType::Sorcery)
        .ability(spell_ability("", Effect::Sequence(vec![pay, draw])))
        .build()
}

/// The amounts offered run from none to what the pool and the lands make
/// beside the {U}: a pool of {U} and two Islands pays X up to 2. The payer
/// picks 2, taps both Islands as the window asks, and draws two.
#[test]
fn an_amount_is_chosen_as_it_is_paid_and_is_x() {
    let mut game = stocked(2);
    let islands: Vec<ObjectId> = (0..2).map(|_| put_on_battlefield(&mut game, island(), 0)).collect();
    cast(&mut game, 0, paid_insight(), &[(ManaType::Blue, 1)], &[]);
    pool_of(&mut game, 0, &[(ManaType::Blue, 1)]);
    let before = hand(&game, 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    dp.expect_number(amount(), 2);
    for &land in &islands {
        let tap = get_effective_abilities(&game, land)[0].id;
        dp.expect_choice(window(), vec![ChoiceOption::Action(PriorityAction::ActivateAbility(land, tap))]);
    }
    let dp = resolve_with(&mut game, dp);

    let offered = dp.asked().into_iter().find_map(|kind| match kind {
        ChoiceKind::ChooseAmountToPay { values, .. } => Some(*values),
        _ => None,
    });
    assert_eq!(offered, Some(XOffer::Interval { at_least: 0, at_most: 2 }));
    assert_eq!(hand(&game, 0), before + 2, "X is 2");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Blue), 0, "{{2}}{{U}} paid");
    assert!(islands.iter().all(|land| game.battlefield[land].tapped));
}

/// Leyline Tyrant's payment: "When this creature dies, you may pay any
/// amount of {R}." Its "when you do" waits for the record of mana spent
/// (`mana-architecture.md`'s decision 4) and its other ability for
/// `mana-kept-and-lost`, which registers the card.
fn leyline_tyrants_payment() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Leyline Tyrant's Payment")
        .card_type(CardType::Creature)
        .power_toughness(4, 4)
        .ability(triggered_ability(
            "",
            whenever(mtgsim::cards::authoring::dies(mtgsim::types::triggers::TriggerSubject::ThisObject), may_pay_any_amount_of(PlayerRef::You, red())),
        ))
        .build()
}

fn red() -> ManaCost {
    ManaCost::build(&[ManaType::Red], 0)
}

/// Only red pays "any amount of {R}": a pool of {R}{R}{G} and a Mountain
/// offer up to three, the Forest beside them nothing. Three paid, the
/// Mountain is tapped and the {G} is left.
#[test]
fn any_amount_of_red_is_paid_in_red() {
    let mut game = stocked(2);
    let tyrant = put_on_battlefield(&mut game, leyline_tyrants_payment(), 0);
    let land = put_on_battlefield(&mut game, mountain(), 0);
    put_on_battlefield(&mut game, mtgsim::test_support::forest(), 0);
    pool_of(&mut game, 0, &[(ManaType::Red, 2), (ManaType::Green, 1)]);
    game.change_zone(tyrant, Zone::Graveyard, mtgsim::engine::actions::ZoneChangeAct::Destroyed, &mtgsim::test_support::test_ctx()).unwrap();

    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    dp.expect_number(amount(), 3);
    let tap = get_effective_abilities(&game, land)[0].id;
    dp.expect_choice(window(), vec![ChoiceOption::Action(PriorityAction::ActivateAbility(land, tap))]);
    let dp = resolve_with(&mut game, dp);

    let offered = dp.asked().into_iter().find_map(|kind| match kind {
        ChoiceKind::ChooseAmountToPay { values, unit, .. } => Some((*values, unit)),
        _ => None,
    });
    assert_eq!(offered, Some((XOffer::Interval { at_least: 0, at_most: 3 }, red())));
    assert!(game.battlefield[&land].tapped);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0, "three red paid");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1, "the green paid nothing");
}

/// A part that is the same at every amount and cannot be paid makes the
/// whole payment one the payer cannot make (CR 118.3), and nothing is asked.
#[test]
fn an_amount_beside_an_unpayable_part_is_not_offered() {
    let mut game = stocked(2);
    cast(&mut game, 0, paid_insight(), &[(ManaType::Blue, 1)], &[]);
    pool_of(&mut game, 0, &[(ManaType::Red, 3)]);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert!(paid_asked(&dp).is_empty(), "no {{U}} to pay");
}

/// An amount of none, once taken, is a payment: the {U} is paid and "if you
/// do" reads it, drawing X = 0 cards.
#[test]
fn an_amount_of_none_taken_is_paid() {
    let mut game = stocked(2);
    cast(&mut game, 0, paid_insight(), &[(ManaType::Blue, 1)], &[]);
    pool_of(&mut game, 0, &[(ManaType::Blue, 1), (ManaType::Colorless, 1)]);
    let before = hand(&game, 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    dp.expect_number(amount(), 0);
    resolve_with(&mut game, dp);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Blue), 0, "the {{U}} was paid");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 1);
    assert_eq!(hand(&game, 0), before);
}

/// "Any player may pay {X}. If a player does, draw X cards": invented, to
/// put an amount beside "any player". The active player picks 2 and lets the
/// window end unpaid, which leaves no X behind; the next pays 1, and X is 1.
#[test]
fn an_amount_left_unpaid_leaves_no_x_for_the_next_payer() {
    let draw = Effect::Conditional(Condition::CostAnswer(CostAnswer::Does), Box::new(draw_cards(you(), AmountExpr::X)));
    let pay = Effect::Optional {
        chooser: any_player(),
        effect: Box::new(Effect::Atom(Primitive::Pay { costs: Vec::new(), any_amount_of: Some(ManaCost::build(&[], 1)) }, EffectRecipient::Implicit)),
    };
    let shared_insight = CardDataBuilder::new("Fixture Shared Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .card_type(CardType::Sorcery)
        .ability(spell_ability("", Effect::Sequence(vec![pay, draw])))
        .build();
    let mut game = stocked(2);
    let islands: Vec<ObjectId> = (0..2).map(|_| put_on_battlefield(&mut game, island(), 0)).collect();
    cast(&mut game, 0, shared_insight, &[(ManaType::Blue, 1)], &[]);
    game.players[1].mana_pool.add(ManaType::Colorless, 1);
    let before = hand(&game, 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    dp.expect_number(amount(), 2);
    let tap = get_effective_abilities(&game, islands[0])[0].id;
    dp.expect_choice(window(), vec![ChoiceOption::Action(PriorityAction::ActivateAbility(islands[0], tap))]);
    dp.expect_choice(window(), Vec::new());
    dp.expect_take(PAY);
    dp.expect_number(amount(), 1);
    let dp = resolve_with(&mut game, dp);

    assert_eq!(paid_asked(&dp), vec![0, 1]);
    assert_eq!(hand(&game, 0), before + 1, "X is the 1 player 1 paid");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Blue), 1, "player 0's mana stays");
}

// ---------------------------------------------------------------------------
// Contamination
// ---------------------------------------------------------------------------

/// Player 0's next upkeep, Contamination's trigger waiting: about a dozen
/// steps from a precombat main, and 40 a bound past any turn's.
fn to_player_0s_upkeep(game: &mut GameState) {
    for _ in 0..40 {
        game.advance_turn(&ScriptedDecisionProvider::new()).expect("advancing");
        if game.active_player == 0 && game.phase.step == Some(StepType::Upkeep) {
            return;
        }
    }
    panic!("player 0's upkeep never began");
}

/// Contamination's ruling: the choice is made as the trigger resolves, and
/// sacrificing a creature keeps Contamination; declining sacrifices it.
// RULING: Contamination #3 - "You choose whether to sacrifice a creature or not on resolution. If not, then you sacrifice Contamination. You can choose to not sacrifice even if you no longer control Contamination on resolution."
#[test]
fn contamination_asks_as_its_trigger_resolves() {
    for sacrifices in [true, false] {
        let mut game = stocked(2);
        let contamination = put_on_battlefield(&mut game, contamination(), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
        to_player_0s_upkeep(&mut game);
        let dp = ScriptedDecisionProvider::new();
        if sacrifices {
            dp.expect_take(PAY);
        } else {
            dp.expect_choice(PAY, Vec::new());
        }
        resolve_with(&mut game, dp);
        assert_eq!(game.battlefield.contains_key(&bears), !sacrifices, "the creature paid: {sacrifices}");
        assert_eq!(game.battlefield.contains_key(&contamination), sacrifices, "Contamination stays: {sacrifices}");
    }
}

/// With two creatures, which one pays is asked as the trigger resolves, and
/// the prompt names the trigger, not Contamination.
#[test]
fn contamination_asks_which_creature_pays() {
    let mut game = stocked(2);
    let contamination = put_on_battlefield(&mut game, contamination(), 0);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    to_player_0s_upkeep(&mut game);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_take(PAY);
    dp.expect_choice(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: ObjectId::UNASSIGNED, count: 1 },
        vec![ChoiceOption::Object(second)],
    );
    let dp = resolve_with(&mut game, dp);
    assert!(game.battlefield.contains_key(&first) && !game.battlefield.contains_key(&second));
    assert!(game.battlefield.contains_key(&contamination));
    let named = dp.asked().into_iter().find_map(|kind| match kind {
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, .. } => Some(spell_or_ability_id),
        _ => None,
    });
    assert!(named.is_some_and(|id| id != contamination), "the resolving trigger, {named:?}");
}

/// With no creature to sacrifice nobody is asked, and Contamination goes.
#[test]
fn contamination_with_no_creature_is_sacrificed_unasked() {
    let mut game = stocked(2);
    let contamination = put_on_battlefield(&mut game, contamination(), 0);
    to_player_0s_upkeep(&mut game);
    let dp = ScriptedDecisionProvider::new();
    let dp = resolve_with(&mut game, dp);
    assert!(paid_asked(&dp).is_empty());
    assert_eq!(game.objects[&contamination].zone, Zone::Graveyard);
}

/// The ruling's last sentence: Contamination destroyed before its trigger
/// resolves, its controller is still asked and may decline, and then nothing
/// is sacrificed: "this enchantment" was the permanent, and the card in the
/// graveyard is a new object (CR 400.7).
// RULING: Contamination #3 - "You choose whether to sacrifice a creature or not on resolution. If not, then you sacrifice Contamination. You can choose to not sacrifice even if you no longer control Contamination on resolution."
#[test]
fn contamination_gone_before_its_trigger_resolves_may_be_declined() {
    let mut game = stocked(2);
    let contamination = put_on_battlefield(&mut game, contamination(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    to_player_0s_upkeep(&mut game);
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();
    assert_eq!(game.stack.len(), 1, "the upkeep trigger waits");
    game.execute_action(GameAction::Destroy { object: contamination }, &test_ctx()).unwrap();

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    let dp = resolve_with(&mut game, dp);
    assert_eq!(paid_asked(&dp), vec![0], "asked all the same");
    assert!(game.battlefield.contains_key(&bears), "declined, and the creature stays");
    assert_eq!(game.objects[&contamination].zone, Zone::Graveyard);
}

/// The same answer when another player controls Contamination as its
/// trigger resolves. It is the same object (a change of control is no zone
/// change, CR 400.7), so "this enchantment" names it, and player 0 can't
/// sacrifice what they don't control (CR 701.21a).
#[test]
fn contamination_lost_to_an_opponent_is_not_sacrificed_on_a_decline() {
    let mut game = stocked(2);
    let contamination = put_on_battlefield(&mut game, contamination(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    to_player_0s_upkeep(&mut game);
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: EffectId::UNASSIGNED,
        source: contamination,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer2Control,
        duration: Duration::UntilEndOfTurn,
        controller: 1,
        created_on_turn: game.turn_number,
        timestamp,
        affected_objects: ObjectSet::Fixed(vec![contamination]),
        modification: EffectModification::SetController(PlayerRef::Player(1)),
    });
    game.settle_announced_controllers();

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(PAY, Vec::new());
    let dp = resolve_with(&mut game, dp);
    assert_eq!(paid_asked(&dp), vec![0], "the trigger's controller is asked");
    assert!(game.battlefield.contains_key(&contamination), "player 0 controls it no longer, and sacrifices nothing");
    assert!(game.battlefield.contains_key(&bears));
}
