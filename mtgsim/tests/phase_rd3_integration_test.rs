//! Phase RD-3 — sources.
//!
//! One thing changes here and every test below is about it
//! (`replacement-architecture.md` §9, RD-3): **a replacement effect can
//! constrain the damage's *source*.** CR 609.7a's chosen object and
//! CR 609.7b/c's rechecked property are two fields on one `SourcePattern`, and
//! CR 510.2's combat flag is beside them.
//!
//! The card file's tests are the rulings pass; these are the rules' own — the
//! CR 615.10 example verbatim, the simultaneity the example needs, and the two
//! things RD-2's build predicted about a count meeting a predicate that stops
//! matching (`plans/handoffs/rd.md`, note 2).

use std::sync::Arc;

use mtgsim::cards::phase_rd_cards::{
    daunting_defender, fog, guardian_seraph, pyroclasm, torbran_thane_of_red_fell,
};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::events::event::{BatchId, DamageTarget, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::test_support::{
    place_vanilla_creature, put_in_hand, put_on_battlefield, registered, set_attacking,
    set_blocked_by, set_blocking, setup_two_player_game, test_ctx, test_dp,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{AffectedSet, Duration, ObjectFilter, PlayerSet};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{
    AmountRewrite, EventPattern, ReplacementDef, Rewrite, SourcePattern, Uses,
};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s spell effect for `controller`, the way the stack would.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: Vec::new(),
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
}

/// A red creature — the colour half of a `SourcePattern` needs a board.
fn red_creature(power: i32, keywords: &[KeywordFlag]) -> Arc<CardData> {
    let mut builder = CardDataBuilder::new("Red Probe")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .power_toughness(power, power);
    for k in keywords {
        builder = builder.keyword_flag(*k);
    }
    builder.build()
}

/// A Cleric creature, so Daunting Defender's filter has more than itself.
fn cleric(power: i32) -> Arc<CardData> {
    CardDataBuilder::new("Cleric Probe")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(power, power)
        .build()
}

/// A registry row inserted directly — the fixture shapes whose printed
/// consumer is a later PR, each named in its test.
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

/// Repaint `id` — CR 609.7b's "the source's properties no longer match", which
/// no registered card can do to a creature yet.
fn make_blue(game: &mut GameState, id: ObjectId) {
    let ts = game.allocate_timestamp();
    let mut blue = std::collections::HashSet::new();
    blue.insert(Color::Blue);
    game.continuous_effects.add(registered(
        id,
        Layer::Layer5Color,
        ts,
        EffectModification::SetColors(blue),
    ));
}

fn marked(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
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

/// The `BatchId` of every damage event since `from`.
fn damage_batches(game: &GameState, from: usize) -> Vec<Option<BatchId>> {
    game.events
        .records_from(from)
        .iter()
        .filter(|r| matches!(r.event, GameEvent::DamageDealt { .. }))
        .map(|r| r.batch())
        .collect()
}

// ---------------------------------------------------------------------------
// CR 615.10 — the rule's own example
// ---------------------------------------------------------------------------

/// > 615.10 Example: Daunting Defender says "If a source would deal damage to a
/// > Cleric creature you control, prevent 1 of that damage." Pyroclasm says
/// > "Pyroclasm deals 2 damage to each creature." Pyroclasm will deal 1 damage
/// > to each Cleric creature controlled by Daunting Defender's controller. It
/// > will deal 2 damage to each other creature.
///
/// The Defender is itself a Cleric, so the board is three Clerics — it and two
/// more — plus a non-Cleric of its controller's and one an opponent controls.
/// **One prevention effect, applied separately to each applicable event**,
/// which is what makes 615.10 different from CR 615.7's shared count.
// COVERS: ATOM-615.10-001
#[test]
fn daunting_defender_takes_one_off_each_clerics_share_of_a_pyroclasm() {
    let mut game = setup_two_player_game();
    let defender = put_on_battlefield(&mut game, daunting_defender(), 0);
    let cleric_two = put_on_battlefield(&mut game, cleric(3), 0);
    let not_a_cleric = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    let theirs = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
    let their_cleric = put_on_battlefield(&mut game, cleric(3), 1);

    resolve_spell(&mut game, pyroclasm(), 1);

    assert_eq!(marked(&game, defender), 1, "a Cleric its controller controls");
    assert_eq!(marked(&game, cleric_two), 1, "and the other one");
    assert_eq!(marked(&game, not_a_cleric), 2, "each other creature");
    assert_eq!(marked(&game, theirs), 2);
    assert_eq!(
        marked(&game, their_cleric),
        2,
        "a Cleric, but not one Daunting Defender's controller controls"
    );
}

/// "Pyroclasm deals 2 damage to each creature" is **one** event, not one per
/// creature. CR 704.3's simultaneity, CR 615.7's "two or more applicable
/// sources at the same time" and CR 603.2c's "one or more" all read the batch,
/// and none of them is reachable from a loop of one-member batches.
#[test]
fn pyroclasm_deals_its_damage_to_every_creature_as_one_event() {
    let mut game = setup_two_player_game();
    place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    place_vanilla_creature(&mut game, 1, 3, 3, &[]);

    let before = game.events.len();
    resolve_spell(&mut game, pyroclasm(), 1);

    let batches = damage_batches(&game, before);
    assert_eq!(batches.len(), 3, "three creatures, three damage events");
    assert!(
        batches.windows(2).all(|w| w[0] == w[1]),
        "and one batch id between them: {:?}",
        batches
    );
}

// ---------------------------------------------------------------------------
// CR 615.10 on a player, and CR 615.7's contrast
// ---------------------------------------------------------------------------

/// The same rule with the subject a player: Guardian Seraph's 1 comes off
/// **each** applicable event, so two attackers in one combat damage step lose
/// a point each rather than sharing one.
///
/// That is the line between CR 615.10 and CR 615.7, and it is why a static
/// partial prevention has no `Uses::NextDamage`: a count would be shared and
/// would prompt for an allocation.
#[test]
fn guardian_seraph_takes_one_off_each_of_two_simultaneous_attackers() {
    let mut game = setup_two_player_game();
    game.active_player = 1;
    put_on_battlefield(&mut game, guardian_seraph(), 0);
    let first = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
    let second = place_vanilla_creature(&mut game, 1, 4, 4, &[]);
    set_attacking(&mut game, first, 0);
    set_attacking(&mut game, second, 0);

    let dp = RecordingDecisionProvider::picking(0);
    game.process_combat_damage(&dp, false).unwrap();

    assert_eq!(life(&game, 0), 20 - 2 - 3, "1 off each event");
    assert_eq!(dp.prompts(), 0, "one candidate per member, so nothing is asked");
}

// ---------------------------------------------------------------------------
// A count meeting a source predicate (`plans/handoffs/rd.md`, note 2)
// ---------------------------------------------------------------------------

/// **(i)** A CR 615.7 count with a source predicate, partly spent, then met by
/// the same source after it has stopped matching: the remainder stays.
///
/// The count lives on the row and is only ever reached through an application,
/// so a source that fails `pattern_watches` yields no candidate and there is
/// nothing to spend — CR 609.7b's "if for any reason the shield prevents no
/// damage ... the shield isn't used up", from the side where some of it
/// already has been.
///
/// The def is a fixture: no printed card in RD-3 pairs `Uses::NextDamage` with
/// a `SourcePattern` (Harm's Way does, and is RD-5's gate). The test exists so
/// a later refactor cannot fold the predicate into the row.
#[test]
fn a_partly_spent_count_keeps_its_remainder_when_the_source_stops_matching() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, red_creature(1, &[]), 1);
    let shield = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
    fixture_row(
        &mut game,
        shield,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage {
                source: Some(SourcePattern::matching(ObjectFilter::ByColor(Color::Red))),
                combat: None,
            },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3),
    );

    let bolt = |game: &mut GameState, amount: u64| {
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(0),
                amount,
                is_combat: false,
                unpreventable: false
            },
            &test_ctx(),
        )
        .unwrap();
    };

    bolt(&mut game, 1);
    assert_eq!(life(&game, 0), 20, "prevented");
    assert_eq!(counts(&game), vec![2], "and spent by exactly that much");

    make_blue(&mut game, source);
    bolt(&mut game, 3);
    assert_eq!(life(&game, 0), 17, "no longer red, so nothing is prevented");
    assert_eq!(counts(&game), vec![2], "and the remainder is untouched");
}

/// **(ii)** Two simultaneous sources, one of which the predicate rejects.
///
/// CR 615.7's allocation exists only among "two or more **applicable**
/// sources", and applicability is per member — so the rejected source is not a
/// bucket, nobody is asked, and its damage is dealt in full while the
/// admitted one is prevented against the whole count.
#[test]
fn a_count_over_one_applicable_source_of_two_asks_nobody() {
    let mut game = setup_two_player_game();
    let red = put_on_battlefield(&mut game, red_creature(1, &[]), 1);
    let colorless = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
    let shield = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
    fixture_row(
        &mut game,
        shield,
        0,
        ReplacementDef::new(
            EventPattern::DealDamage {
                source: Some(SourcePattern::matching(ObjectFilter::ByColor(Color::Red))),
                combat: None,
            },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3),
    );

    let dp = RecordingDecisionProvider::picking(0);
    game.execute_actions(
        vec![
            GameAction::DealDamage {
                source: red,
                target: DamageTarget::Player(0),
                amount: 2,
                is_combat: false,
                unpreventable: false
            },
            GameAction::DealDamage {
                source: colorless,
                target: DamageTarget::Player(0),
                amount: 4,
                is_combat: false,
                unpreventable: false
            },
        ],
        &ActionContext::new(&dp),
    )
    .unwrap();

    assert_eq!(dp.prompts(), 0, "one applicable source is not CR 615.7's choice");
    assert_eq!(life(&game, 0), 16, "the red 2 prevented, the other 4 dealt");
    assert_eq!(counts(&game), vec![1], "and 2 of 3 spent");
}

// ---------------------------------------------------------------------------
// Fog — the combat flag over a whole damage step
// ---------------------------------------------------------------------------

/// Fog over a real combat damage step: every member of the batch is dropped,
/// attacker and blocker alike, because "all combat damage" is `Filter { All }`
/// beside `PlayerSet::Everyone` and CR 510.2 makes the whole step one event.
#[test]
fn fog_empties_a_combat_damage_step() {
    let mut game = setup_two_player_game();
    game.active_player = 1;
    let attacker = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
    let blocker = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let unblocked = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    set_attacking(&mut game, attacker, 0);
    set_attacking(&mut game, unblocked, 0);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);
    resolve_spell(&mut game, fog(), 0);

    let before = game.events.len();
    game.process_combat_damage(&test_dp(), false).unwrap();

    assert_eq!(marked(&game, attacker), 0);
    assert_eq!(marked(&game, blocker), 0);
    assert_eq!(life(&game, 0), 20);
    assert!(damage_batches(&game, before).is_empty(), "no damage was dealt at all");
}

// ---------------------------------------------------------------------------
// Torbran — divide before adding
// ---------------------------------------------------------------------------

/// Torbran's ruling: "If damage dealt by a source you control is being divided
/// or assigned among multiple permanents an opponent controls or among an
/// opponent and one or more permanents they control, divide the original
/// amount before adding 2. For example, if you attack with a 5/5 red creature
/// with trample and your opponent blocks with a 2/2 creature, you can assign 2
/// damage to the blocker and 3 damage to the defending player. These amounts
/// are then modified to 4 and 5, respectively."
///
/// Structural rather than coded, exactly as Furnace of Rath's is:
/// `assign_combat_damage` divides before anything is proposed, so the pipeline
/// never sees the undivided 5 — and `Plus` lands on each half separately,
/// which is the whole difference between +2 once and +2 twice.
#[test]
fn torbran_divides_before_adding_two() {
    let mut game = setup_two_player_game();
    game.active_player = 0;
    put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
    let trampler = put_on_battlefield(&mut game, red_creature(5, &[KeywordFlag::Trample]), 0);
    let blocker = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    set_attacking(&mut game, trampler, 1);
    set_blocked_by(&mut game, trampler, vec![blocker]);
    set_blocking(&mut game, blocker, vec![trampler]);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(
        ChoiceKind::AssignTrampleDamage {
            attacker_id: trampler,
            defending_target: DamageTarget::Player(1),
        },
        vec![2, 3],
    );
    let assignments = assign_combat_damage(&game, &dp, 0, false);
    game.apply_combat_damage(assignments, &ActionContext::new(&dp)).unwrap();

    assert_eq!(marked(&game, blocker), 4, "2 assigned, then +2");
    assert_eq!(life(&game, 1), 20 - 5, "3 assigned, then +2");
    assert_eq!(
        marked(&game, trampler),
        2,
        "the blocker is not an opponent's from Torbran's controller's side"
    );
}
