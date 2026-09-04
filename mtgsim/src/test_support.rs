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
//! [`put_on_battlefield`] routes through [`GameState::place_on_battlefield`] with the
//! [`EnterMods`] the rules give it, so CR 306.5b's loyalty counters and
//! `register_static_effects` both fire and the arrival is announced.
//! [`place_bare`] inserts a [`BattlefieldEntity`] directly and does none of it —
//! which is what a fixture wants when the test counts the events its own action
//! emitted.
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
use crate::types::card_types::{ArtifactType, CardType, EnchantmentType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use crate::types::ids::{AbilityId, ObjectId, PlayerId, new_ability_id};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::engine::actions::ActionContext;
use crate::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer, Timestamp,
};
use crate::types::effects::Duration;
use crate::types::zones::Zone;
use crate::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Game setup
// ---------------------------------------------------------------------------

/// Create a minimal two-player game in precombat main phase with player 0 active.
pub fn setup_two_player_game() -> GameState {
    setup_game(2)
}

/// The N-player form of [`setup_two_player_game`]: `num_players` players, in
/// precombat main phase, player 0 active.
///
/// Anything that reads the *player set* rather than "me and the other one"
/// needs a board wider than two to be tested at all — CR 302.6's window spans
/// every opponent's turn, and three of them is a different answer from one.
pub fn setup_game(num_players: usize) -> GameState {
    let mut game = GameState::new(num_players, 20);
    game.phase = Phase::new(PhaseType::Precombat);
    game.active_player = 0;
    game
}

/// Advance `game` to the beginning of the next player's turn.
///
/// Walks the real phase machinery, so everything a turn transition records —
/// untaps, the CR 302.6 turn-start clock — is recorded the way a game records
/// it. Assigning `turn_number` directly does not.
pub fn pass_turn(game: &mut GameState) {
    let start = game.turn_number;
    for _ in 0..200 {
        game.advance_turn(&test_ctx()).expect("advancing a step");
        if game.turn_number > start {
            return;
        }
    }
    panic!("the turn never ended");
}

/// A no-op decision provider for tests that never reach a choice point.
pub fn test_dp() -> ScriptedDecisionProvider {
    ScriptedDecisionProvider::new()
}

/// A no-op [`ActionContext`] for tests that never reach a replacement choice.
///
/// `ActionContext` borrows its `DecisionProvider`, so a helper that returns one
/// has to own the provider somewhere. This leaks a fresh
/// `ScriptedDecisionProvider` per call — a few dozen bytes each, in a test
/// binary, behind the `test-support` feature that release builds turn off.
///
/// **Fresh rather than shared, on purpose.** A single shared provider would let
/// one test's scripted expectations leak into the next test that borrowed it.
/// Nothing reads `ctx.dp` yet (Phase RA threads it; Phase RB consults it), so
/// the hazard is not live — which is exactly why it is worth closing now,
/// before a test starts depending on the wrong answer.
///
/// If your test needs to script a choice, do not use this: build
/// `ActionContext::new(&dp)` against a provider you own and can enqueue onto.
pub fn test_ctx() -> ActionContext<'static> {
    ActionContext::new(Box::leak(Box::new(ScriptedDecisionProvider::new())))
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
            is_characteristic_defining: false,
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

/// An Aura named `name` that can only enchant a creature **you** control.
///
/// CR 303.4a's enchant restriction, with a `ByController(PlayerRef::You)` node
/// in it — which SBA 704.5n resolves against the Aura's own controller
/// (CR 109.5), independently of the enchanted creature's (CR 303.4e). Real
/// cards with this text include Aura of Silence's cousins and the whole
/// "Enchant creature you control" cycle (Ethereal Armor, Gryff's Boon, Angelic
/// Destiny).
pub fn aura_enchanting_your_creature(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .enchant_filter(SelectionFilter::Permanent(PermanentFilter::And(
            Box::new(PermanentFilter::ByType(CardType::Creature)),
            Box::new(PermanentFilter::ByController(PlayerRef::You)),
        )))
        .build()
}

/// A green "Test Creature" with the given P/T and keywords, and no other abilities.
pub fn vanilla_creature(power: i32, toughness: i32, keywords: &[KeywordFlag]) -> Arc<CardData> {
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

/// A `DecisionProvider` that records the `ChoiceKind` of every prompt and
/// answers each one with a nominated index.
///
/// **The complement of `ScriptedDecisionProvider`, not a replacement for it.**
/// A scripted provider with an empty queue panics on the first prompt, which is
/// the sharpest possible assertion that a path asks *nothing* — but it can
/// only say "none", never "exactly one, and it was this kind". Tests that own a
/// new decision site need the second half: RS-1 wanted it and grew a private
/// `CountingDp`, CV-1 wanted it and grew a private `RecordingDp`, and a third
/// copy is where a helper stops being premature.
///
/// Recording the *kind* rather than a count is the part worth sharing. A test
/// that asserts "one prompt" passes just as well when the prompt was the wrong
/// one.
pub struct RecordingDecisionProvider {
    pick: usize,
    all: bool,
    seen: std::cell::RefCell<Vec<String>>,
}

impl RecordingDecisionProvider {
    /// Answer every `pick_n` with `index`, clamped to the options offered.
    pub fn picking(index: usize) -> Self {
        RecordingDecisionProvider {
            pick: index,
            all: false,
            seen: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Answer every `pick_n` by taking **everything offered**, up to the bound.
    ///
    /// **The one that catches an over-permissive candidate list**, which
    /// [`Self::picking`] structurally cannot: a rule that should have removed an
    /// option leaves it somewhere in the list, and a provider that always takes
    /// index 0 only notices when the wrongly-offered option happens to be
    /// first. RC-5's CR 614.13a tests were killed by their mutation until a
    /// third candidate was added in front of the excluded one, at which point
    /// they passed with the rule deleted (`phase_rc5_integration_test`).
    pub fn picking_all() -> Self {
        RecordingDecisionProvider {
            pick: 0,
            all: true,
            seen: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// The `ChoiceKind`s seen so far, `Debug`-formatted, in prompt order.
    ///
    /// A `String` rather than the `ChoiceKind` itself because `ChoiceKind` is
    /// not `PartialEq` (it carries `EffectRecipient`, which carries filters) and
    /// making it so for a test helper would be the tail wagging the dog.
    /// `starts_with("ChooseCopySource")` is the idiom.
    pub fn kinds(&self) -> Vec<String> {
        self.seen.borrow().clone()
    }

    /// How many prompts have been asked.
    pub fn prompts(&self) -> usize {
        self.seen.borrow().len()
    }
}

impl DecisionProvider for RecordingDecisionProvider {
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        ctx: &crate::ui::choice_types::ChoiceContext,
        options: &[crate::ui::choice_types::ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        self.seen.borrow_mut().push(format!("{:?}", ctx.kind));
        if self.all {
            return (0..options.len().min(_bounds.1)).collect();
        }
        vec![self.pick.min(options.len().saturating_sub(1))]
    }

    fn pick_number(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &crate::ui::choice_types::ChoiceContext,
        min: u64,
        _max: u64,
    ) -> u64 {
        self.seen.borrow_mut().push("pick_number".to_string());
        min
    }

    fn allocate(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &crate::ui::choice_types::ChoiceContext,
        total: u64,
        buckets: &[crate::ui::choice_types::ChoiceOption],
        _per_bucket_mins: &[u64],
        _per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.seen.borrow_mut().push("allocate".to_string());
        let mut out = vec![0; buckets.len()];
        if !out.is_empty() {
            out[0] = total;
        }
        out
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &crate::ui::choice_types::ChoiceContext,
        items: &[crate::ui::choice_types::ChoiceOption],
    ) -> Vec<usize> {
        self.seen.borrow_mut().push("choose_ordering".to_string());
        (0..items.len()).collect()
    }
}

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
/// Routes through [`GameState::place_on_battlefield`] with `default_enter_mods`, so
/// CR 306.5b's counters and static-effect registration both fire. Sets
/// `entered_battlefield_turn = 0` so the permanent is not summoning-sick — it mimics
/// "has been here since before this turn".
///
/// **Not the production path.** A permanent really entering goes through
/// `GameState::propose_entry`, so a CR 614.1c replacement can modify how — this
/// helper skips the pipeline and puts the permanent down as the rules alone
/// would have it.
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
    let mods = game.default_enter_mods(id, player);
    let entry = game.place_on_battlefield(id, player, &mods);
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
    keywords: &[KeywordFlag],
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

/// Attach `attachment` to `host`, writing both sides of the link.
///
/// Writes the state directly rather than replaying an Aura's ETB or an
/// Equipment's equip ability, for the same reason the combat helpers write
/// combat state directly: the tests that want this are asking what happens to
/// an *already attached* permanent, and there is no equip ability to replay.
///
/// Both directions matter — `cleanup_zone_state` walks `attached_by` to detach
/// a departing host, and SBA 704.5m/n reads `attached_to`. Writing one and not
/// the other produces a board no real game can reach, which is why this goes
/// through `GameState::attach` rather than the two fields: it is also the
/// writer that bumps the layer epoch, and a test that wrote the fields itself
/// would be the stale-memo bug in a fixture.
pub fn attach(game: &mut GameState, attachment: ObjectId, host: ObjectId) {
    game.attach(attachment, host);
}

/// A `{1}` Equipment named `name`, with no equip ability and no granted bonus.
///
/// Equip (CR 702.6) is not implemented, and the control-independence tests do
/// not need it: CR 301.5d is about whose permanent the Equipment *is*, which is
/// a fact about the Equipment rather than about anything it grants.
pub fn equipment(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 1))
        .build()
}

// ---------------------------------------------------------------------------
// Continuous-effect rows
//
// These exist so that adding a field to `ContinuousEffect` breaks one function
// instead of thirty struct literals scattered across four test modules. That is not
// hypothetical: `AbilityDef` gained `is_characteristic_defining` in the CDA phase and
// broke every inline copy of the Lightning Bolt builder at once.
// ---------------------------------------------------------------------------

/// One registry row applying to `id` alone, via [`AffectedSet::Fixed`].
///
/// The row is `EffectOrigin::Resolution` and `Duration::UntilEndOfTurn` — a pump-spell
/// shaped effect, not a static ability. Tests that need a static ability's origin (so the
/// CR 613.7a existence check has something to ask about) must build their own row.
pub fn registered(
    id: ObjectId,
    layer: Layer,
    timestamp: Timestamp,
    modification: EffectModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: 0, // assigned by the registry on add()
        source: id,
        origin: EffectOrigin::Resolution,
        layer,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected: AffectedSet::Fixed(vec![id]),
        modification,
    }
}

/// As [`registered`], but selecting the source through [`AffectedSet::SourceOnly`].
///
/// `SourceOnly` and `Fixed(vec![source])` agree in `effect_applies_to` when the source is
/// the only member, so the two are interchangeable *today*. They are kept as separate
/// constructors anyway: they are different enum variants, and a test that was written
/// against one should not silently start exercising the other.
pub fn registered_source_only(
    source: ObjectId,
    layer: Layer,
    timestamp: Timestamp,
    modification: EffectModification,
) -> ContinuousEffect {
    ContinuousEffect {
        affected: AffectedSet::SourceOnly,
        ..registered(source, layer, timestamp, modification)
    }
}

// ---------------------------------------------------------------------------
// Layer 6 helpers
// ---------------------------------------------------------------------------

/// A static [`AbilityDef`] carrying `effect`, with a fresh id and the CDA flag
/// clear.
///
/// The id is what Layer 6 tests care about: `EffectModification::LoseAbility`
/// removes by id, and CR 113.10b makes "every instance with this id" the
/// question. Clone the returned def to put the *same* ability on an object
/// twice — a fresh call would give a second id and prove nothing.
pub fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
    }
}

/// A creature named `name` with the given P/T carrying `ability` as printed
/// text, and no keywords.
pub fn creature_with_ability(name: &str, power: i32, toughness: i32, ability: AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(power, toughness)
        .ability(ability)
        .build()
}

// ---------------------------------------------------------------------------
// CDA-phase helpers
// ---------------------------------------------------------------------------

/// A card with a name and a single card type, and nothing else.
///
/// Tarmogoyf counts card types in graveyards, so its tests need cheap one-type cards.
pub fn card_of_type(name: &str, card_type: CardType) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(card_type).build()
}

/// Put a card straight into a player's graveyard.
pub fn put_in_graveyard(
    game: &mut GameState,
    card_data: Arc<CardData>,
    player: PlayerId,
) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Graveyard);
    let id = obj.id;
    game.add_object(obj);
    game.players[player].graveyard.push(id);
    id
}

/// Put a permanent onto the battlefield with ETB hooks, entering **this** turn — so it
/// is summoning-sick.
///
/// [`put_on_battlefield`] backdates entry to turn 0 instead. `GameState::new` starts at
/// turn 1, so the two really do differ; pick the one whose summoning-sickness the test
/// wants rather than assuming they are the same function with different arity.
pub fn put_on_battlefield_this_turn(
    game: &mut GameState,
    card_data: Arc<CardData>,
    player: PlayerId,
) -> ObjectId {
    let obj = GameObject::new(card_data, player, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);
    let mods = game.default_enter_mods(id, player);
    game.place_on_battlefield(id, player, &mods);
    id
}
