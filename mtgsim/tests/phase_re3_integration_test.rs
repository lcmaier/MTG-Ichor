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

use mtgsim::cards::phase_re_cards::rhox_faithmender;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_game};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::PlayerId;
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
