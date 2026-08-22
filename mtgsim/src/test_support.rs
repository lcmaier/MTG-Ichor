//! Shared test helpers, visible to both `src/` unit tests and `tests/` integration tests.
//!
//! # Why this lives in the library
//!
//! Integration tests in `mtgsim/tests/` link the crate as an external dependency, so
//! they cannot see `#[cfg(test)]` items in `src/`. Unit tests inside `src/` cannot see
//! `tests/common/mod.rs`. Nothing shared can live in either place, so the helpers had
//! forked — the same "place a vanilla creature" and "build a Forest" code existed in
//! several modules with subtly different bodies.
//!
//! The module is gated behind the `test-support` feature, which the crate turns on for
//! itself via a dev-dependency on itself (see `Cargo.toml`). `cargo test` and
//! `cargo build --all-targets` enable it; a plain `cargo build`/`cargo build --release`
//! does not, so none of this ships in a release artifact.
//!
//! # The two battlefield-placement idioms are NOT interchangeable
//!
//! [`put_on_battlefield`] routes through [`GameState::place_on_battlefield`], which fires
//! `init_etb_counters` and `register_static_effects`. [`place_bare`] inserts a
//! [`BattlefieldEntity`] directly and fires neither.
//!
//! Existing tests depend on the difference: the combat tests place vanilla creatures with
//! no static abilities and no ETB counters, and registering effects for them would put
//! rows in the continuous-effects registry that those tests do not expect. Do not
//! "simplify" these into one function — pick the one whose hooks the test actually wants.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::objects::object::GameObject;
use crate::state::battlefield::{AttackTarget, AttackingInfo, BattlefieldEntity, BlockingInfo};
use crate::state::game_state::{GameState, Phase, PhaseType};
use crate::types::card_types::{CardType, EnchantmentType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::{AbilityId, ObjectId, PlayerId, new_ability_id};
use crate::types::keywords::KeywordAbility;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::Zone;
use crate::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Game setup
// ---------------------------------------------------------------------------

/// Create a minimal two-player game in precombat main phase with player 0 active.
pub fn setup_two_player_game() -> GameState {
    let mut game = GameState::new(2, 20);
    game.phase = Phase::new(PhaseType::Precombat);
    game.active_player = 0;
    game
}

/// A no-op decision provider for tests that never reach a choice point.
pub fn test_dp() -> ScriptedDecisionProvider {
    ScriptedDecisionProvider::new()
}

// ---------------------------------------------------------------------------
// Card factories
//
// These are deliberately built inline rather than pulled from `CardRegistry`, so a
// test's fixture does not change under it when a real card definition is edited.
// ---------------------------------------------------------------------------

/// Basic Forest: `{T}: Add {G}`.
pub fn forest() -> Arc<CardData> {
    CardDataBuilder::new("Forest")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Forest))
        .mana_ability_single(ManaType::Green)
        .build()
}

/// Lightning Bolt: `{R}`, deal 3 damage to any target.
pub fn lightning_bolt() -> Arc<CardData> {
    CardDataBuilder::new("Lightning Bolt")
        .card_type(CardType::Instant)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::DealDamage(AmountExpr::Fixed(3)),
                EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Pacifism: `{1}{W}` Aura enchanting a creature.
pub fn pacifism() -> Arc<CardData> {
    CardDataBuilder::new("Pacifism")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .enchant_filter(SelectionFilter::Creature)
        .build()
}

/// A green "Test Creature" with the given P/T and keywords, and no other abilities.
pub fn vanilla_creature(power: i32, toughness: i32, keywords: &[KeywordAbility]) -> Arc<CardData> {
    let mut builder = CardDataBuilder::new("Test Creature")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(power, toughness);
    for kw in keywords {
        builder = builder.keyword(*kw);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------

/// Put a card into a player's hand and register it in the game.
pub fn put_in_hand(game: &mut GameState, card_data: Arc<CardData>, player: PlayerId) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Hand);
    let id = obj.id;
    game.add_object(obj);
    game.players[player].hand.push(id);
    id
}

/// Put any permanent onto the battlefield **with ETB hooks**.
///
/// Routes through [`GameState::place_on_battlefield`], so ETB counters and static-effect
/// registration both fire. Sets `entered_battlefield_turn = 0` so the permanent is not
/// summoning-sick — it mimics "has been here since before this turn".
///
/// See the module docs: this is not interchangeable with [`place_bare`].
pub fn put_on_battlefield(
    game: &mut GameState,
    card_data: Arc<CardData>,
    player: PlayerId,
) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);
    let entry = game.place_on_battlefield(id, player);
    entry.entered_battlefield_turn = 0;
    entry.controller_since_turn = 0;
    id
}

/// Put a land onto the battlefield for a player (from a factory function).
pub fn put_land_on_battlefield(
    game: &mut GameState,
    land_fn: fn() -> Arc<CardData>,
    player: PlayerId,
) -> ObjectId {
    put_on_battlefield(game, land_fn(), player)
}

/// Put a permanent onto the battlefield **without ETB hooks**, by inserting a
/// [`BattlefieldEntity`] directly. No ETB counters, no static-effect registration.
///
/// `entered_battlefield_turn` is 0, so the permanent is not summoning-sick.
///
/// See the module docs: this is not interchangeable with [`put_on_battlefield`].
pub fn place_bare(game: &mut GameState, card_data: Arc<CardData>, owner: PlayerId) -> ObjectId {
    let obj = GameObject::new(card_data, owner, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);
    let ts = game.allocate_timestamp();
    let entry = BattlefieldEntity::new(id, owner, ts, 0);
    game.battlefield.insert(id, entry);
    id
}

/// Place a vanilla creature on the battlefield without ETB hooks. Convenience wrapper
/// over [`place_bare`] + [`vanilla_creature`], which is the shape the combat tests want.
pub fn place_vanilla_creature(
    game: &mut GameState,
    owner: PlayerId,
    power: i32,
    toughness: i32,
    keywords: &[KeywordAbility],
) -> ObjectId {
    place_bare(game, vanilla_creature(power, toughness, keywords), owner)
}

/// Place a Forest on the battlefield without ETB hooks, returning its object id and the
/// id of its mana ability.
pub fn place_forest(game: &mut GameState, player: PlayerId) -> (ObjectId, AbilityId) {
    let data = forest();
    let ability_id = data.abilities[0].id;
    (place_bare(game, data, player), ability_id)
}

// ---------------------------------------------------------------------------
// Libraries
// ---------------------------------------------------------------------------

/// Give a player some dummy cards in their library (for draw effects).
pub fn fill_library(game: &mut GameState, player: PlayerId, count: usize) {
    for _ in 0..count {
        let card = CardDataBuilder::new("Dummy Card").build();
        let obj = GameObject::new(card, player, Zone::Library);
        let id = obj.id;
        game.add_object(obj);
        game.players[player].library.push(id);
    }
}

/// Give every player enough Forests in their library not to deck out during draw steps.
pub fn stock_libraries(game: &mut GameState, cards_per_player: usize) {
    let num_players = game.num_players();
    for pid in 0..num_players {
        for _ in 0..cards_per_player {
            let obj = GameObject::in_library(forest(), pid);
            let id = game.add_object(obj);
            game.players[pid].library.push(id);
        }
    }
}

// ---------------------------------------------------------------------------
// Combat state
//
// These write combat state directly rather than going through the combat engine, so a
// test can set up a specific board without replaying a whole declare-attackers step.
// ---------------------------------------------------------------------------

/// Mark a permanent as attacking a player.
pub fn set_attacking(game: &mut GameState, id: ObjectId, target_player: PlayerId) {
    if let Some(entry) = game.battlefield.get_mut(&id) {
        entry.attacking = Some(AttackingInfo {
            target: AttackTarget::Player(target_player),
            is_blocked: false,
            blocked_by: Vec::new(),
        });
    }
}

/// Mark an attacker as blocked by the given blockers. No-op if it is not attacking.
pub fn set_blocked_by(game: &mut GameState, attacker: ObjectId, blockers: Vec<ObjectId>) {
    if let Some(entry) = game.battlefield.get_mut(&attacker) {
        if let Some(ref mut info) = entry.attacking {
            info.is_blocked = true;
            info.blocked_by = blockers;
        }
    }
}

/// Mark a permanent as blocking the given attackers.
pub fn set_blocking(game: &mut GameState, blocker: ObjectId, blocking: Vec<ObjectId>) {
    if let Some(entry) = game.battlefield.get_mut(&blocker) {
        entry.blocking = Some(BlockingInfo { blocking });
    }
}
