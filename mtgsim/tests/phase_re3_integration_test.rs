//! Phase RE-3 — life.
//!
//! CR 119.3, 119.7, 119.10 and 120.3a's contained loss, against the six printed
//! cards in `cards::phase_re_cards`.
//!
//! **Every board here is about which of the three life events an effect
//! watches**, and the file's shape follows from that: a gain (CR 119.3), the
//! loss contained inside damage (CR 120.3a), and a loss that is neither damage
//! nor an effect (CR 119.4's payment). Ali from Cairo is the card that makes
//! the distinction observable, and its own ruling is the reason it is a
//! `LoseLife { cause: Some(Damage) }` rather than anything about damage:
//! *"this effect does not prevent damage, it prevents the damage from turning
//! into loss of life"*.

use std::sync::Arc;

use mtgsim::cards::phase_re_cards::{
    alhammarrets_archive, ali_from_cairo, rhox_faithmender, skullcrack, tainted_remedy,
    words_of_worship,
};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_hand, put_on_battlefield, setup_game, test_ctx,
    test_dp,
};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution. Built inline rather than pulled from the registry, so no real
/// card's behaviour leaks into a board that is only about life.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// Resolve an effect for `player`, the way a spell would, with `dp` answering
/// any CR 616.1 prompt.
fn resolve_for(game: &mut GameState, player: PlayerId, effect: &Effect, dp: &dyn DecisionProvider) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// "You gain `n` life", as a resolving spell says it.
fn gain_life(game: &mut GameState, player: PlayerId, n: u64, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::GainLife(AmountExpr::Fixed(n)),
        EffectRecipient::Controller,
    );
    resolve_for(game, player, &effect, dp);
}

/// Every `LifeChanged` this game has recorded for `player`, as `(old, new)`.
fn life_changes(game: &GameState, player: PlayerId) -> Vec<(i64, i64)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::LifeChanged { player_id, old, new, .. } if *player_id == player => {
                Some((*old, *new))
            }
            _ => None,
        })
        .collect()
}

/// "You lose `n` life", as a resolving spell says it — `LifeLossCause::Effect`,
/// which is the cause Ali from Cairo's first ruling is about.
fn lose_life(game: &mut GameState, player: PlayerId, n: u64, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::LoseLife(AmountExpr::Fixed(n)),
        EffectRecipient::Controller,
    );
    resolve_for(game, player, &effect, dp);
}

/// "Draw a card", as a resolving spell says it.
fn draw_one(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::Controller,
    );
    resolve_for(game, player, &effect, dp);
}

/// Resolve an effect whose source is a permanent already on the battlefield —
/// an activated ability's, so the registry row it creates names the right
/// source and CR 109.5's "you" resolves against the right controller.
fn resolve_from(
    game: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    effect: &Effect,
    targets: Vec<ResolvedTarget>,
    dp: &dyn DecisionProvider,
) {
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// Activate Words of Worship on the battlefield for `player`.
///
/// **The `{1}` is skipped**: every board here is about the row the activation
/// creates, and `Cost::Mana` is exercised where mana is. What is not skipped is
/// the source — the row names the permanent, so a second activation makes a
/// second row from the same source, which is the ruling's board.
fn activate_words(game: &mut GameState, words: ObjectId, player: PlayerId, dp: &dyn DecisionProvider) {
    let effect = words_of_worship().abilities[0].effect.clone();
    resolve_from(game, words, player, &effect, Vec::new(), dp);
}

/// Resolve Skullcrack, targeting `victim`. Its two restriction rows land before
/// its damage, which is CR 608.2c's printed order.
fn cast_skullcrack(game: &mut GameState, caster: PlayerId, victim: PlayerId) {
    let card = skullcrack();
    let effect = card.abilities[0].effect.clone();
    let source = put_in_hand(game, card, caster);
    resolve_from(
        game,
        source,
        caster,
        &effect,
        vec![ResolvedTarget::Player(victim)],
        &test_dp(),
    );
}

/// A 1/1 for `owner` to be a damage source.
fn source_for(game: &mut GameState, owner: PlayerId) -> ObjectId {
    place_vanilla_creature(game, owner, 1, 1, &[])
}

/// Deal `amount` damage to `victim`, which proposes CR 120.3a's contained loss.
fn bolt_player(game: &mut GameState, source: ObjectId, victim: PlayerId, amount: u64) {
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: DamageTarget::Player(victim),
            amount,
            is_combat: false,
            unpreventable: false,
        },
        &test_ctx(),
    )
    .unwrap();
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// The CR 616.1 prompt this file's boards produce — the subject is a player, so
/// there is no affected object.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };

// ---------------------------------------------------------------------------
// CR 614.5 over a player subject — the acid board
// ---------------------------------------------------------------------------

/// **Two Faithmenders quadruple, and nobody is asked which applies first.**
///
/// The ruling gives the arithmetic verbatim: *"if you control two Rhox
/// Faithmenders, life you gain will be multiplied by four. Three Rhox
/// Faithmenders will multiply any life gain by eight, and so on."*
///
/// **And the prompt is the first assertion, not the number.** Two multipliers
/// over one event commute — CR 616.1's choice has one outcome — which is the
/// same theorem `ordering_cannot_change_outcome` has carried for two Furnaces
/// of Rath since RD-1. It carried it as *"the pattern is
/// `EventPattern::DealDamage`"*, which is a list of shapes rather than the
/// property the theorem needs, so a `Multiplier` over a life gain fell through
/// to a real question. The provider is primed with nothing and asserts it was
/// never asked; `four_is_not_two_applications_of_one_faithmender` is the board
/// that separates 4 from the arithmetic that also gives 4.
#[test]
fn test_two_rhox_faithmenders_quadruple() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let before = game.players[0].life_total;

    let dp = ScriptedDecisionProvider::new();
    gain_life(&mut game, 0, 3, &dp);

    assert_eq!(
        game.players[0].life_total - before,
        12,
        "two doublers quadruple a gain of 3 (CR 614.5, one opportunity each)"
    );
    assert!(
        dp.is_empty(),
        "and two multipliers over one event commute, so CR 616.1 has nothing to ask"
    );
}

/// The same rule one copy further on, because the ruling states it: three
/// Faithmenders multiply by eight. Worth its own board — 2ⁿ is the claim, and
/// two copies alone are consistent with "doubled once per copy after the
/// first", which three copies separate from it.
#[test]
fn three_rhox_faithmenders_multiply_by_eight() {
    let mut game = setup_game(2);
    for _ in 0..3 {
        put_on_battlefield(&mut game, rhox_faithmender(), 0);
    }
    let before = game.players[0].life_total;

    let dp = ScriptedDecisionProvider::new();
    gain_life(&mut game, 0, 2, &dp);

    assert_eq!(game.players[0].life_total - before, 16);
    assert!(dp.is_empty(), "still one outcome with three, so still no prompt");
}

/// **One gain, not several** — the claim the life-total delta alone cannot
/// make.
///
/// CR 614.6 makes a modified event happen *in place of* the original, so a
/// doubled gain of 3 is one gain of 12 and not four gains of 3 or two of 6.
/// The distinction is invisible in the life total and will be the whole of
/// CR 119.9's "whenever a player gains life" trigger count when item 6 lands,
/// so it is asserted on the event stream now, while the board is cheap.
#[test]
fn four_is_not_two_applications_of_one_faithmender() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let before = game.players[0].life_total;

    gain_life(&mut game, 0, 3, &ScriptedDecisionProvider::new());

    assert_eq!(
        life_changes(&game, 0),
        vec![(before, before + 12)],
        "one life-gain event of 12, not a sequence of smaller ones"
    );
}

// ---------------------------------------------------------------------------
// CR 119.3 — the gain the pool already proposes
// ---------------------------------------------------------------------------

/// **The board the phase exists for**, and the one no fixture sets up:
/// lifelink's contained `GainLife` (CR 120.3f) has been a proposal since RB
/// with nothing watching it, and Rhox Faithmender has lifelink of its own.
///
/// The damage is 1 and the life gained is 2 — the doubler applies to the
/// *gain*, not to the damage, which is the same boundary Bloodletter of
/// Aclazotz's ruling draws from the other side.
#[test]
fn rhox_faithmender_doubles_the_life_its_own_lifelink_gains() {
    let mut game = setup_game(2);
    let rhox = put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let before = life(&game, 0);
    let victim_before = life(&game, 1);

    bolt_player(&mut game, rhox, 1, 1);

    assert_eq!(life(&game, 0) - before, 2, "lifelink gained 1, doubled to 2");
    assert_eq!(
        victim_before - life(&game, 1),
        1,
        "and the damage is untouched: the doubler watches the gain"
    );
}

// COVERS: ATOM-119.10-001
//
// The atom's board is a player with a life-gain replacement and an effect that
// causes 0 life gain, and its expected result is that the replacement does not
// apply. Before this phase there was no replacement that *could* apply, so the
// atom had no watcher to prove was not offered; Rhox Faithmender is one, and
// `never_happens` drops the proposal ahead of `gather`.
#[test]
fn a_gain_of_zero_is_no_event_and_a_doubler_never_sees_it() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let before = life(&game, 0);

    gain_life(&mut game, 0, 0, &test_dp());

    assert_eq!(life(&game, 0), before, "CR 119.10 — no life gain event occurred");
    assert!(
        life_changes(&game, 0).is_empty(),
        "and nothing was performed for the doubler to double"
    );
}

// ---------------------------------------------------------------------------
// CR 614.1a — a gain becomes a loss (Tainted Remedy)
// ---------------------------------------------------------------------------

/// The plain substitution, and the first customer of
/// `TemplateAmount::ReplacedAmount`: "that much" is the gain's own number.
#[test]
fn tainted_remedy_turns_an_opponents_gain_into_a_loss_of_the_same_size() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, tainted_remedy(), 0);
    let before = life(&game, 1);

    gain_life(&mut game, 1, 4, &test_dp());

    assert_eq!(life(&game, 1), before - 4, "gained nothing, lost four");
    assert_eq!(
        life_changes(&game, 1),
        vec![(before, before - 4)],
        "one event, and it is the loss: CR 614.6 puts the modified event in the original's place"
    );
}

/// "If an **opponent** would gain life" — the controller's own gain is not
/// watched, which is `PlayerSet::Opponents` resolved against CR 109.5's "you".
#[test]
fn tainted_remedy_leaves_its_own_controllers_gain_alone() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, tainted_remedy(), 0);
    let before = life(&game, 0);

    gain_life(&mut game, 0, 4, &test_dp());

    assert_eq!(life(&game, 0), before + 4);
}

/// **Its second ruling:** *"having more than one Tainted Remedy on the
/// battlefield doesn't have any noticeable effect on life gain. Once the effect
/// of one Tainted Remedy applies, there is no life gain for the others to apply
/// to."*
///
/// CR 614.7 by the same road the ruling describes: after the first applies, the
/// event is a `LoseLife`, which no `EventPattern::GainLife` watches — so the
/// second produces no candidate at the re-gather rather than being filtered out
/// of one, and there is no second application to double the loss.
///
/// **CR 616.1 still asks which one, and that is right.** Both are applicable at
/// the first iteration, and the rule's question is "choose one to apply", not
/// "choose one if it matters". `ordering_cannot_change_outcome` is a proof
/// obligation rather than a requirement, and this shape is deliberately not one
/// of its three: a fourth semantics-assuming shortcut would have to carry its
/// own expiry conditions for a board no printed ruling calls a choice and no
/// pooled card reaches. So the ruling is asserted the stronger way — the prompt
/// happens, and both answers are the same three life.
#[test]
fn a_second_tainted_remedy_has_no_gain_left_to_apply_to() {
    for chosen in [0usize, 1usize] {
        let mut game = setup_game(2);
        put_on_battlefield(&mut game, tainted_remedy(), 0);
        put_on_battlefield(&mut game, tainted_remedy(), 0);
        let before = life(&game, 1);

        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![chosen]);
        gain_life(&mut game, 1, 3, &dp);

        assert!(dp.is_empty(), "one prompt, not two: the second has nothing left to watch");
        assert_eq!(
            life(&game, 1),
            before - 3,
            "lost three once, whichever Remedy was picked"
        );
    }
}

/// **Tainted Remedy's own ordering ruling, both branches, with its numbers.**
///
/// *"If a player who controls Alhammarret's Archive would gain 3 life while
/// Tainted Remedy is on the battlefield, that player may choose to have the 3
/// life become doubled to 6 life and then lose 6 life. The player may also
/// choose to apply Tainted Remedy first, turning 'gain 3 life' into 'lose 3
/// life.' Alhammarret's Archive would then not apply."*
///
/// This is the board CR 616.1 exists for and the one
/// `ordering_cannot_change_outcome` must **not** suppress: a multiplier beside
/// a kind-changing substitution is not a bucket of multipliers, and the two
/// orders differ by 3 life. The chooser is the player who would gain — the
/// event's subject — which is the Archive's controller and not the Remedy's.
#[test]
fn the_gaining_player_chooses_between_the_archive_and_tainted_remedy() {
    let doubled_first = {
        let mut game = setup_game(2);
        put_on_battlefield(&mut game, alhammarrets_archive(), 1);
        put_on_battlefield(&mut game, tainted_remedy(), 0);
        let before = life(&game, 1);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![0]);
        gain_life(&mut game, 1, 3, &dp);
        assert!(dp.is_empty(), "one prompt");
        before - life(&game, 1)
    };
    let remedied_first = {
        let mut game = setup_game(2);
        put_on_battlefield(&mut game, alhammarrets_archive(), 1);
        put_on_battlefield(&mut game, tainted_remedy(), 0);
        let before = life(&game, 1);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![1]);
        gain_life(&mut game, 1, 3, &dp);
        assert!(dp.is_empty(), "one prompt");
        before - life(&game, 1)
    };

    assert_eq!(doubled_first, 6, "doubled to 6, then lose 6");
    assert_eq!(remedied_first, 3, "lose 3, and the Archive has no gain to double");
}

// ---------------------------------------------------------------------------
// CR 614.1a — a draw becomes a gain (Words of Worship)
// ---------------------------------------------------------------------------

/// The kind-changing substitution RE-2's pattern and RE-3's template meet on,
/// and the first `Uses::Once` row over a draw.
#[test]
fn words_of_worship_turns_the_next_draw_into_five_life() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    let before = life(&game, 0);
    let library = game.players[0].library.len();

    activate_words(&mut game, words, 0, &test_dp());
    draw_one(&mut game, 0, &test_dp());

    assert_eq!(life(&game, 0), before + 5);
    assert_eq!(game.players[0].library.len(), library, "and no card was drawn");
}

/// "The **next** time" — `Uses::Once`, so the row is spent by the draw it
/// replaced and the one after it is an ordinary draw.
#[test]
fn words_of_worship_is_spent_by_the_draw_it_replaces() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);

    activate_words(&mut game, words, 0, &test_dp());
    draw_one(&mut game, 0, &test_dp());
    let library = game.players[0].library.len();
    let before = life(&game, 0);

    draw_one(&mut game, 0, &test_dp());

    assert_eq!(game.players[0].library.len(), library - 1, "the second draw is a draw");
    assert_eq!(life(&game, 0), before);
}

/// **Its one ruling:** *"If multiple Words have been used prior to drawing a
/// card, then you can choose which one to apply (and use up) each time you draw
/// a card."*
///
/// Two rows, one player, and the choice is real because each application spends
/// a different row: the first draw takes one row and gains 5, the second takes
/// the other. What the ruling is *about* is that the player picks which — so
/// the assertion is the prompt and its effect on which row survives, not the
/// life total, which is 5 either way.
#[test]
fn two_words_rows_are_a_choice_and_each_draw_uses_one_up() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    let before = life(&game, 0);

    activate_words(&mut game, words, 0, &test_dp());
    activate_words(&mut game, words, 0, &test_dp());

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(PICK_REPLACEMENT, vec![1]);
    draw_one(&mut game, 0, &dp);
    assert!(dp.is_empty(), "two rows, so CR 616.1 asks which one applies");
    assert_eq!(life(&game, 0), before + 5, "one row applied, not both");

    // The other row is still there, and the next draw finds it alone.
    draw_one(&mut game, 0, &test_dp());
    assert_eq!(life(&game, 0), before + 10);

    let library = game.players[0].library.len();
    draw_one(&mut game, 0, &test_dp());
    assert_eq!(game.players[0].library.len(), library - 1, "both rows spent");
}

// COVERS-PARTIAL: ATOM-616.2-001
//
// The atom's chain is gain -> draw -> return-from-graveyard, and its second leg
// has no arm: nothing prints "if you would draw a card, return a card from your
// graveyard instead", and `GameActionTemplate::ZoneChangeTo` keeps the event's
// object, which a draw has none of. What this board builds is the atom's claim
// over the two legs the printed pool has — a draw substituted for a gain, and a
// gain doubler that could not have applied to the draw becoming applicable only
// because of it. CR 616.2 in the direction the corpus wrote it, one card short.
#[test]
fn a_doubler_becomes_applicable_only_after_a_draw_has_become_a_gain() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let before = life(&game, 0);

    activate_words(&mut game, words, 0, &test_dp());
    let dp = ScriptedDecisionProvider::new();
    draw_one(&mut game, 0, &dp);

    assert_eq!(
        life(&game, 0),
        before + 10,
        "CR 616.2 — the Faithmender watches no draw, and became applicable only \
         once the draw was a gain"
    );
    assert!(
        dp.is_empty(),
        "and no prompt: at the first iteration only Words watches a draw, and at \
         the second only the Faithmender watches a gain"
    );
}

// ---------------------------------------------------------------------------
// CR 120.3a's contained loss — Ali from Cairo
// ---------------------------------------------------------------------------

/// The card's whole text, with the numbers it is about: at 3 life, 10 damage
/// leaves you at 1.
#[test]
fn ali_from_cairo_clamps_a_lethal_damage_loss_to_one() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 3;
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 10);

    assert_eq!(life(&game, 0), 1);
}

/// **Its third ruling, and the reason the pattern is the loss rather than the
/// damage:** *"the full damage is dealt (and abilities that trigger on damage
/// being dealt still trigger), but the full loss of life is not applied."*
///
/// So `DamageDealt` still says 10 — which is what item 6's CR 603.2c triggers
/// will read — and only the contained `LoseLife` is clamped.
#[test]
fn the_full_damage_is_still_dealt_and_only_the_loss_is_clamped() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 3;
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 10);

    let dealt: Vec<u64> = game
        .events
        .events()
        .filter_map(|e| match e {
            GameEvent::DamageDealt { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(dealt, vec![10], "the damage event carries the full amount");
    assert_eq!(life_changes(&game, 0), vec![(3, 1)], "only the loss is clamped");
}

/// **Its first ruling:** *"this effect does not apply to effects which reduce
/// your life without doing damage."* A `Primitive::LoseLife` is
/// `LifeLossCause::Effect`, which the pattern does not match, so the loss goes
/// through and the player is at -7.
#[test]
fn ali_from_cairo_does_not_clamp_a_loss_that_is_not_damage() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 3;

    lose_life(&mut game, 0, 10, &test_dp());

    assert_eq!(life(&game, 0), -7, "CR 119.6 will lose the game for this at the next SBA check");
}

/// **The `LifeLossCause::Cost` arm, which nothing has watched until now, and
/// the answer is structural twice over.**
///
/// A life payment is not damage, so the pattern does not match it; and CR 119.4
/// refuses a payment larger than the life total *before* any replacement is
/// asked, so a payment that would go below 1 is either legal down to exactly 0
/// or not a payment at all. Ali from Cairo is not a floor on your life total —
/// it is a modification of one kind of loss — and paying 3 at 3 life leaves you
/// at 0 with him on the battlefield.
#[test]
fn ali_from_cairo_does_not_clamp_a_life_payment() {
    let mut game = setup_game(2);
    let source = put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 3;

    let ctx = test_ctx();
    let plan = game
        .plan_payment(&[Cost::PayLife(3)], 0, source, &ctx)
        .expect("CR 119.4 allows paying down to exactly 0");
    game.pay_costs(&plan, 0, source, &ctx).unwrap();

    assert_eq!(life(&game, 0), 0, "a payment is not damage, and the clamp is about damage");
}

/// **Its second ruling:** *"the ability works up until Ali enters the
/// graveyard, so if he takes lethal damage or is destroyed at the same time you
/// take damage, the ability helps you."*
///
/// CR 704.3's decide-then-perform is what makes that true rather than something
/// coded: a batch decides every member against one board, so an Earthquake that
/// is lethal to a 0/1 and to a player at 3 is decided while Ali is still on the
/// battlefield. The clamp applies, and only afterwards does the SBA sweep find
/// him dead.
#[test]
fn ali_helps_on_the_earthquake_that_kills_him() {
    let mut game = setup_game(2);
    let ali = put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 3;
    let source = source_for(&mut game, 1);

    // One batch: the same source, the same amount, a player and Ali.
    game.execute_actions(
        vec![
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(0),
                amount: 10,
                is_combat: false,
                unpreventable: false,
            },
            GameAction::DealDamage {
                source,
                target: DamageTarget::Object(ali),
                amount: 10,
                is_combat: false,
                unpreventable: false,
            },
        ],
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(life(&game, 0), 1, "decided against the board Ali was still on");
}

/// A clamp from a total already at the floor takes the whole loss, and the
/// event is still proposed — it is the *performer's* local 0-guard that makes
/// it silent, not CR 614.7a.
///
/// **`never_happens` deliberately has no `LoseLife` arm.** CR 119.10 is written
/// about life *gain* and the CR has no counterpart for loss, so RE-2 kept
/// `DrawCards { n: 0 }` out on the same reasoning and RE-3 keeps this out. A
/// watcher of a 0 loss would therefore see one, which is what the rules say and
/// what no printed card yet asks about.
#[test]
fn a_clamp_from_the_floor_removes_the_whole_loss() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, ali_from_cairo(), 0);
    game.players[0].life_total = 1;
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 5);

    assert_eq!(life(&game, 0), 1, "a floor never hands life back, and never takes more");
    assert!(
        life_changes(&game, 0).is_empty(),
        "the loss is performed as 0, which `perform_action` makes silent"
    );
}

// ---------------------------------------------------------------------------
// CR 101.2 ahead of the pipeline — Skullcrack
// ---------------------------------------------------------------------------

// COVERS: ATOM-119.7-004
//
// CR 119.7's last clause, verbatim: "a replacement effect that would replace a
// life gain event affecting that player won't do anything." The atom's second
// effect is "instead draw that many cards" and Tainted Remedy's is "loses that
// much life"; the claim is the same and it is the whole claim — there is no
// event to replace, so the replacement does not apply and produces nothing.
// Skullcrack's own ruling says it in the card's words: "effects that would
// replace gaining life with another effect won't apply because it's impossible
// for players to gain life."
#[test]
fn under_skullcrack_a_gain_replacement_has_no_event_to_replace() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, tainted_remedy(), 0);
    cast_skullcrack(&mut game, 0, 1);
    let before = life(&game, 1);

    let dp = ScriptedDecisionProvider::new();
    gain_life(&mut game, 1, 3, &dp);

    assert_eq!(life(&game, 1), before, "no life gained, and none lost either");
    assert!(dp.is_empty(), "CR 101.2 is asked before `gather`, so nothing was offered");
}

/// **Skullcrack's second ruling:** *"spells and abilities that would cause a
/// player to gain life or that would prevent damage still resolve, but the
/// life-gain and damage-prevention parts have no effect."* The resolution is
/// not an error — the proposal is refused.
#[test]
fn a_life_gain_spell_still_resolves_under_skullcrack_and_gains_nothing() {
    let mut game = setup_game(2);
    cast_skullcrack(&mut game, 0, 1);
    let before = life(&game, 1);
    // Skullcrack's own 3 damage is a life change of its own, so the assertion
    // below counts only what the gain spell did.
    let changes_before = life_changes(&game, 1).len();

    gain_life(&mut game, 1, 7, &test_dp());

    assert_eq!(life(&game, 1), before);
    assert_eq!(
        life_changes(&game, 1).len(),
        changes_before,
        "the spell resolved and performed nothing"
    );
}

/// **Leyline of Punishment's ruling, which is Words of Worship's board**:
/// *"effects that replace an event with gaining life (like Words of Worship's)
/// will end up replacing the event with nothing."*
///
/// It falls out of the order of the two checks rather than being coded: the
/// draw is replaced by a `GainLife`, CR 101.2 refuses that proposal ahead of
/// the pipeline, and the draw is gone. The card is Skullcrack because Leyline
/// is deliberately unregistered; the static form of the same row is a fixture
/// in `phase_rd4_integration_test.rs`.
#[test]
fn words_of_worship_under_skullcrack_replaces_the_draw_with_nothing() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    cast_skullcrack(&mut game, 0, 1);
    let before = life(&game, 0);
    let library = game.players[0].library.len();

    activate_words(&mut game, words, 0, &test_dp());
    draw_one(&mut game, 0, &test_dp());

    assert_eq!(game.players[0].library.len(), library, "the draw did not happen");
    assert_eq!(life(&game, 0), before, "and neither did the gain it was replaced with");
}

/// The whole card in one board: both restrictions land, and the 3 damage is
/// dealt after them — so a lifelinker that is dealt damage this turn gains
/// nothing, which is what "this turn" is for.
#[test]
fn skullcrack_stops_life_gain_for_everyone_including_its_controller() {
    let mut game = setup_game(2);
    let rhox = put_on_battlefield(&mut game, rhox_faithmender(), 0);
    let victim_before = life(&game, 1);
    let caster_before = life(&game, 0);

    cast_skullcrack(&mut game, 0, 1);
    assert_eq!(life(&game, 1), victim_before - 3, "and the damage is dealt");

    bolt_player(&mut game, rhox, 1, 1);
    assert_eq!(
        life(&game, 0),
        caster_before,
        "lifelink gained nothing: `PlayerSet::Everyone` includes the controller"
    );
}

/// **Ali from Cairo is not a prevention effect, so Skullcrack does not switch
/// him off** — the card's own ruling, tested against the sentence that would
/// have made it wrong.
///
/// `ReplacementDef::is_prevention` tests the pattern for damage first, and
/// `LoseLife` is not damage. This is the assertion that keeps that ordering
/// honest: both of Skullcrack's rows are in the registry, and the clamp still
/// applies.
#[test]
fn skullcrack_does_not_turn_off_ali_from_cairo() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, ali_from_cairo(), 0);
    cast_skullcrack(&mut game, 1, 0);
    // Skullcrack's own 3 damage took 20 to 17; set the total afterwards so the
    // board is the card's and the number is the test's.
    game.players[0].life_total = 3;
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 10);

    assert_eq!(
        life(&game, 0),
        1,
        "\"damage can't be prevented\" has nothing to say to an effect that \
         prevents the damage from turning into loss of life"
    );
}

// ---------------------------------------------------------------------------
// Alhammarret's Archive — both halves on one permanent
// ---------------------------------------------------------------------------

/// One legendary artifact carrying RE-2's draw doubler and RE-3's gain doubler,
/// which is the reason §9 makes RE-2 -> RE-3 a hard order. Its rulings are Rhox
/// Faithmender's and Teferi's Ageless Insight's; this asserts only that the two
/// abilities coexist on one object and each watches its own event.
#[test]
fn alhammarrets_archive_doubles_a_gain_and_a_draw_from_one_permanent() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    put_on_battlefield(&mut game, alhammarrets_archive(), 0);
    let before = life(&game, 0);
    let library = game.players[0].library.len();

    gain_life(&mut game, 0, 3, &test_dp());
    assert_eq!(life(&game, 0), before + 6);

    draw_one(&mut game, 0, &test_dp());
    assert_eq!(game.players[0].library.len(), library - 2, "and the draw doubled too");
}
