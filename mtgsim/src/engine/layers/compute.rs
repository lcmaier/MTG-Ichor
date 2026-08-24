//! Core computation: `compute_characteristics(game, id)`.
//!
//! Walks the continuous effect registry in layer order and produces
//! `EffectiveCharacteristics` for a given object. All oracle queries
//! route through this function.
//!
//! Reads base characteristics from CardData, then applies all continuous
//! effects in layer order (1→2→3→4→5→6→7b→7c→7d).

use std::collections::HashMap;

use crate::engine::layers::types::*;
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::{ObjectId, PlayerId};

/// The layers, in application order (CR 613.1). Index into this array is the
/// "layer ceiling" used by the frame cache: ceiling `n` means layers
/// `LAYER_ORDER[..n]` have been applied, i.e. the frame as of the end of
/// layer `n - 1`.
///
/// This mirrors the `Layer` enum exactly, and the enum is **not** the CR's full
/// list. One sublayer split is still missing, tracked as Deferred Migrations
/// item 10:
///
/// - **1a / 1b.** CR 613.2 splits layer 1 into face-down effects (1a) and copy
///   effects (1b); `Layer1Copy` collapses them, even though
///   `layers-architecture.md` §7 specifies both and says Phase LA ships them.
///   The order matters — a Clone copying a face-down creature must copy the
///   2/2 colorless characteristics, not the printed card (CR 707.2).
///
/// Not reachable today: nothing produces a layer 1 effect. Splitting it later
/// just lengthens this array — the ceiling is an index into it, computed at
/// runtime, so nothing else moves.
const LAYER_ORDER: [Layer; 10] = [
    Layer::Layer1Copy,
    Layer::Layer2Control,
    Layer::Layer3Text,
    Layer::Layer4Type,
    Layer::Layer5Color,
    Layer::Layer6Ability,
    Layer::Layer7aCdaPT,
    Layer::Layer7bSetPT,
    Layer::Layer7cModifyPT,
    Layer::Layer7dSwitchPT,
];

/// Memo for one top-level `compute_characteristics` call
/// (`layers-architecture.md` §5.2).
///
/// Deciding whether a CR 613.7a effect still exists means asking whether some
/// *other* object still has the static ability that generates it, which is a
/// characteristics query of its own. §5.2's answer is to answer it at a lower
/// **layer ceiling**: at layer index `i` we need the source's frame as of the
/// end of layer `i - 1`, which is ceiling `i`.
///
/// That is also the termination argument. Computing an object at ceiling `C`
/// only ever requests ceilings `< C`, so the recursion strictly descends and
/// bottoms out at ceiling 0, which applies no effects at all. There is no
/// fixpoint to iterate and nothing to cap.
///
/// Discarded when the top-level call returns, so it never has to be
/// invalidated.
pub(super) type FrameCache = HashMap<(ObjectId, usize), EffectiveCharacteristics>;

/// Compute the effective characteristics of a game object after applying
/// all active continuous effects in layer order.
///
/// Returns `None` if the object doesn't exist.
pub fn compute_characteristics(game: &GameState, id: ObjectId) -> Option<EffectiveCharacteristics> {
    let mut cache = FrameCache::new();
    compute_to_ceiling(game, id, LAYER_ORDER.len(), &mut cache)
}

/// `compute_characteristics` with layers `LAYER_ORDER[ceiling..]` left unapplied.
pub(super) fn compute_to_ceiling(
    game: &GameState,
    id: ObjectId,
    ceiling: usize,
    cache: &mut FrameCache,
) -> Option<EffectiveCharacteristics> {
    // Only sub-computations are worth memoizing. The top-level frame is
    // requested exactly once per call, so caching it would be a pure clone.
    let memoize = ceiling < LAYER_ORDER.len();
    if memoize {
        if let Some(cached) = cache.get(&(id, ceiling)) {
            return Some(cached.clone());
        }
    }

    let obj = game.objects.get(&id)?;
    let card = &obj.card_data;

    // Start from printed (base) characteristics
    let chars_controller = base_controller(game, id).unwrap_or(obj.owner);
    let mut chars = EffectiveCharacteristics {
        name: card.name.clone(),
        mana_cost: card.mana_cost.clone(),
        colors: card.colors.clone(),
        types: card.types.clone(),
        subtypes: card.subtypes.clone(),
        supertypes: card.supertypes.clone(),
        keywords: card.keywords.clone(),
        abilities: card.abilities.clone(),
        power: card.power,
        toughness: card.toughness,
        controller: chars_controller,
        // CR 302.6's "continuously since their most recent turn began". Seeded
        // from the battlefield entry because that is what a control change
        // *outside* Layer 2 writes (entering the battlefield, today the only
        // one); Layer 2 overwrites it when it actually changes the controller.
        control_since_turn: game
            .battlefield
            .get(&id)
            .map(|entry| entry.controller_since_turn)
            .unwrap_or(0),
    };

    // Walk layers in order, applying all effects (registered + counters)
    apply_effects(game, id, &mut chars, ceiling, cache);

    if memoize {
        cache.insert((id, ceiling), chars.clone());
    }
    Some(chars)
}

/// CR 613.7a — does the static ability that generates `effect` still exist?
///
/// A continuous effect from a static ability applies only while its source
/// actually has that ability. Registry membership does not answer this: the
/// effect is registered when the permanent enters the battlefield, but CR 305.7
/// (Blood Moon) and Layer 6 ability removal can take the ability away later
/// without touching the registry. So the question is re-asked at every layer,
/// against the source's frame as of the end of the previous layer.
///
/// `EffectOrigin::Resolution` effects (CR 613.7b) always exist: a resolution
/// already happened and cannot be taken back, so there is no ability to go
/// looking for.
///
/// Existence is not the same as surviving, and only existence is decided here.
/// An instant that grants first strike until end of turn creates an effect that
/// exists for the turn no matter what — but Humility, applying later in layer 6,
/// still clears the keyword it granted. That is ordering inside a layer, which
/// `effects_in_layer` handles (timestamp today, CR 613.8 eventually), not a
/// question about whether the effect is there to apply.
fn static_ability_still_exists(
    game: &GameState,
    effect: &ContinuousEffect,
    layer_index: usize,
    cache: &mut FrameCache,
) -> bool {
    let ability_id = match effect.origin {
        EffectOrigin::Resolution => return true,
        EffectOrigin::StaticAbility { ability } => ability,
    };

    match compute_to_ceiling(game, effect.source, layer_index, cache) {
        Some(source_frame) => source_frame.abilities.iter().any(|a| a.id == ability_id),
        // Source is gone from the object store entirely.
        None => false,
    }
}

/// Apply all continuous effects in layer order (rule 613).
///
/// Walks registered effects by layer, and also applies counter-derived
/// P/T modifications in layer 7c (rule 613.4c) alongside other modifiers.
fn apply_effects(
    game: &GameState,
    id: ObjectId,
    chars: &mut EffectiveCharacteristics,
    ceiling: usize,
    cache: &mut FrameCache,
) {
    let has_registered = !game.continuous_effects.is_empty();
    // Looked up once and reused by layers 6 and 7c. Both need it every call,
    // and a second `HashMap` probe per `compute_characteristics` is not free —
    // this function is the inner loop of mana enumeration and targeting.
    let bf_entry = game.battlefield.get(&id);
    let on_battlefield = bf_entry.is_some();

    // CR 613.6 — "if an effect starts to apply in one layer, it will continue
    // to be applied to the same set of objects in each other applicable layer".
    //
    // `effect_applies_to` reads `chars`, which earlier layers have already
    // mutated, so re-filtering from scratch at every layer is wrong: March of
    // the Machines' Layer 4 part makes a noncreature artifact a creature, and
    // its Layer 7b part then finds nothing matching "noncreature artifact".
    // Once a CR-level effect has started applying to this object, membership
    // here short-circuits the filter for the rest of the walk.
    //
    // Keyed by `EffectGroup`, not `EffectId`: the two halves of March of the
    // Machines are two registry rows, and it is the *effect* that started
    // applying, not the row.
    let mut started: std::collections::HashSet<EffectGroup> = std::collections::HashSet::new();

    // ...but only when some CR-level effect actually occupies more than one
    // row. For a single-row group the mark is written after its only row
    // applies and never read, so maintaining it is pure cost — two SipHashes
    // over a pair of UUIDs, per effect, per layer. That was 70% of the layer
    // walk on a static-heavy board before this check existed.
    let track_started = game.continuous_effects.summary().any_multi_row_group;

    // Fast path: nothing to apply.
    //
    // A CDA is not in the registry and does not need the battlefield (CR 604.3
    // — CDAs function in all zones), so it gets its own term. Reading the
    // printed list is exact here: with an empty registry and no battlefield
    // entry, nothing in the walk can add an ability.
    if !has_registered && !on_battlefield && !crate::engine::layers::cda::has_any_cda(chars) {
        return;
    }

    // Keyword counters (CR 122.1b) are a *second* source of layer 6 effects, and
    // CR 613.7c timestamps them, so they interleave with that layer's registry
    // rows by timestamp rather than following them. Humility with a later
    // timestamp than a flying counter really does strip that flying.
    //
    // Built once per call rather than per layer, and empty for every permanent
    // carrying no keyword counter — which is nearly all of them. Held descending
    // so `pop()` yields the earliest.
    let mut pending_counters = collect_keyword_counters(bf_entry);

    for (layer_index, &layer) in LAYER_ORDER.iter().enumerate() {
        if layer_index >= ceiling {
            break;
        }

        // CR 613.3 — "apply effects from characteristic-defining abilities
        // first, then all other effects in timestamp order". Intrinsic before
        // the registry slice is that sentence. Only three layers can hold a
        // CDA (CR 604.3a(1)), so the other seven skip the scan entirely.
        if crate::engine::layers::cda::CDA_LAYERS.contains(&layer) {
            crate::engine::layers::cda::apply_intrinsic_cdas(
                game, chars, id, layer, layer_index, cache,
            );
        }

        // Apply registered effects in this layer
        if has_registered {
            let effects = game.continuous_effects.effects_in_layer(layer);
            for effect in effects {
                // Every keyword counter older than this row goes first (CR
                // 613.7). Done before the filter and existence checks below,
                // because those `continue` and would skip the drain.
                if !pending_counters.is_empty() && layer == Layer::Layer6Ability {
                    drain_keyword_counters_before(
                        &mut pending_counters, effect.timestamp, chars,
                    );
                }

                let already_applying = track_started && started.contains(&effect.group());
                if !already_applying {
                    if !effect_applies_to(effect, id, chars, game, layer_index, cache) {
                        continue;
                    }
                    // CR 613.7a. Only asked before the effect starts applying:
                    // once it has, CR 613.6 keeps it applying for the rest of
                    // this walk even if a later layer removes the ability.
                    if !static_ability_still_exists(game, effect, layer_index, cache) {
                        continue;
                    }
                    if track_started {
                        started.insert(effect.group());
                    }
                }
                apply_modification(
                    &effect.modification, chars, id, game, layer_index, cache, Some(effect),
                );
            }
        }

        // Whatever is left is later than every row in the layer.
        if !pending_counters.is_empty() && layer == Layer::Layer6Ability {
            drain_keyword_counters_before(&mut pending_counters, Timestamp::MAX, chars);
        }

        // Apply counter P/T in layer 7c (rule 613.4c).
        //
        // Not interleaved by timestamp the way layer 6's keyword counters are,
        // and it does not need to be: every layer 7c effect is an addition to
        // power and toughness, so the layer's result is order-independent. That
        // is a property of the layer, not a shortcut — if a non-commutative 7c
        // effect ever exists, this has to become a merge like the one above.
        if layer == Layer::Layer7cModifyPT {
            if let Some(entry) = bf_entry {
                let plus = entry.counter_count(CounterType::PlusOnePlusOne) as i32;
                if plus != 0 {
                    if let Some(ref mut p) = chars.power { *p += plus; }
                    if let Some(ref mut t) = chars.toughness { *t += plus; }
                }
                let minus = entry.counter_count(CounterType::MinusOneMinusOne) as i32;
                if minus != 0 {
                    if let Some(ref mut p) = chars.power { *p -= minus; }
                    if let Some(ref mut t) = chars.toughness { *t -= minus; }
                }
                // TODO: handle other P/T-modifying counter types (+2/+2, +0/+1, etc.)
                // when they are added to CounterType.
            }
        }
    }
}

/// The keyword counters on `entry` (CR 122.1b), as `(timestamp, keyword)` sorted
/// **descending** so `pop()` yields the earliest.
///
/// Returns empty for every layer but 6, and for every permanent with no keyword
/// counter — which is the overwhelming majority, and an empty `Vec` does not
/// allocate.
fn collect_keyword_counters(
    entry: Option<&crate::state::battlefield::BattlefieldEntity>,
) -> Vec<(Timestamp, crate::types::keywords::KeywordFlag)> {
    let Some(entry) = entry else {
        return Vec::new();
    };
    // The overwhelmingly common case, and worth its own exit: no counters at
    // all means no iteration and no `Vec`.
    if entry.counters.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<(Timestamp, crate::types::keywords::KeywordFlag)> = entry
        .counters
        .iter()
        .filter(|(_, stack)| stack.count > 0)
        .filter_map(|(counter_type, stack)| {
            counter_type.keyword_granted().map(|kw| (stack.timestamp, kw))
        })
        .collect();

    // Descending, and by keyword on a tie. The tie is reachable — two kinds of
    // keyword counter put on in one action share a timestamp under CR 613.7c —
    // and `HashMap` iteration order is per-process, so without the second key
    // the order would differ between runs of the binary. It cannot change the
    // answer here (both are inserts into a set), but a `Vec` whose order is
    // nondeterministic is a trap for whoever extends this. `KeywordFlag` derives
    // `Ord` for exactly this, which is why the tiebreak costs nothing.
    out.sort_unstable_by(|a, b| b.cmp(a));
    out
}

/// Apply every pending keyword counter with a timestamp earlier than `limit`.
fn drain_keyword_counters_before(
    pending: &mut Vec<(Timestamp, crate::types::keywords::KeywordFlag)>,
    limit: Timestamp,
    chars: &mut EffectiveCharacteristics,
) {
    while let Some(&(timestamp, keyword)) = pending.last() {
        if timestamp >= limit {
            return;
        }
        chars.keywords.insert(keyword);
        pending.pop();
    }
}

/// The effective controller of `id` as of the end of layer `layer_index - 1`.
///
/// `None` only when `id` is not in the object store at all.
///
/// The gate is exact rather than a heuristic. Layer 2 is the only thing that
/// writes `chars.controller` after `compute_to_ceiling` seeds it, so when the
/// registry holds no `SetController` row the seed *is* the answer and reading
/// the field skips a whole sub-walk.
///
/// It was worth ~4% when `FilterPlayers::you()` was its only caller. The Layer 2
/// phase put 20 more behind it and re-measured on boards where the flag is
/// actually true: **+28%**, and the phase itself cost ~+4% rather than the +7%
/// that was predicted. `RegistryScopeSummary::any_control_changing` carries the
/// numbers, and the note on why the sharper per-object gate was built, measured
/// and discarded.
///
/// The ungated arm asks at `layer_index`, never at the full ceiling
/// (`layers-architecture.md` §5.2) — that descent is the termination argument,
/// and `test_self_stripping_land_terminates_and_is_stable` is its canary.
fn effective_controller(
    game: &GameState,
    id: ObjectId,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<PlayerId> {
    if !game.continuous_effects.summary().any_control_changing {
        return base_controller(game, id);
    }
    compute_to_ceiling(game, id, layer_index, cache).map(|frame| frame.controller)
}

/// The controller an object has before Layer 2 touches it — CR 110.2's default.
///
/// The arms are CR 108.4's sentence in order: a permanent reads
/// `BattlefieldEntity`, a spell reads its `StackEntry`, and a card in a hand or
/// graveyard has no controller at all — owner is what this reports for it,
/// because `EffectiveCharacteristics.controller` is not an `Option`.
///
/// **The single definition of the pre-Layer-2 seed.** Both
/// `any_control_changing` gates return this instead of walking, and they are
/// only exact while they and `compute_to_ceiling`'s seed agree — which they
/// used to do by having the same body written out three times.
pub(crate) fn base_controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    // Battlefield first, so the common case is one probe rather than three.
    if let Some(entry) = game.battlefield.get(&id) {
        return Some(entry.controller);
    }
    if let Some(entry) = game.stack_entries.get(&id) {
        return Some(entry.controller);
    }
    game.objects.get(&id).map(|obj| obj.owner)
}

/// The players a continuous effect's `PermanentFilter` can name — resolved
/// lazily, at most once per filter tree.
///
/// Laziness is load-bearing, not tidiness. A filter with no `ByController` node
/// (Cloudspire Mesa's bare "creatures have flying") must cost what it cost
/// before this refactor, and `And(ByType(Creature), ByController(You))` must
/// cost nothing extra for a land, because `&&` never reaches the second arm.
///
/// The two origins get their "you" from different rules, and the difference is
/// not a shortcut:
///
/// - **`StaticAbility` (CR 613.7a).** CR 109.5: "For a static ability, this is
///   the *current* controller of the object it's on." So ask the source for its
///   effective controller, the same way `static_ability_still_exists` asks it
///   for its effective ability list — `compute_to_ceiling` at `layer_index`,
///   never at the full ceiling (`layers-architecture.md` §5.2).
///
///   It is tempting to call that walk free on the grounds that the existence
///   check makes the identical request a few lines below and would hit the
///   frame cache. **It is not.** The existence check runs only for effects that
///   already *matched*; this one runs for every object the filter is about to
///   reject, which is most of the board. Resolving it eagerly *and* ungated
///   measured 749 ms/game against a 73.0 baseline on `fuzz_games --games 200
///   --seed 12345`.
///
///   Of the two fixes, **this one is the load-bearing half**: lazy-and-ungated
///   is 78.1 ms/game, lazy-and-gated 74.8. So laziness is what has to survive a
///   refactor here; `effective_controller`'s gate is a further ~4%.
///
/// - **`Resolution` (CR 613.7b).** "You" was fixed when the spell or ability
///   resolved — CR 611.2c, "the set of objects it affects is determined when
///   that continuous effect begins" — and `effect.controller` is that player.
///   The source permanent may since have changed hands, or left the
///   battlefield entirely, without moving the effect's allegiance.
///
/// `effect.controller` is also the fallback when the source object is gone from
/// the store. A `StaticAbility` effect in that state is about to be retired by
/// `static_ability_still_exists` anyway, so the value only has to be defined,
/// not meaningful.
///
/// # A Layer 2 effect asking about its own controller
///
/// Layer 2 lands next, and "permanents you control" on a Layer 2 effect reaches
/// here with `layer_index == 1` — the frame as of the end of Layer 1, i.e.
/// *before* any control-changing effect has applied. That reads
/// `BattlefieldEntity.controller`, CR 110.2's default controller.
///
/// That is the right answer whenever the source is not itself under a
/// control-changing effect, and it is the CR's own fallback when it is. Two
/// Layer 2 effects where applying one changes what the other applies to are
/// dependent under CR 613.8a — same layer, and the answer to "what it applies
/// to" changes — so 613.8b orders them, and a mutual pair is a dependency
/// *loop*, which 613.8b resolves by ignoring dependency and applying in
/// timestamp order. Timestamp order over a pre-layer frame is what we already
/// do. The exact version needs the frame this reads to be a partially-applied
/// layer, which is `codebase-state.md` item 8 step 4's board-wide sequential
/// pass — the same missing piece a granted Layer 6 static ability already
/// waits on (`resolve::register_granted_static_effects`), and scheduled with
/// 613.8 for that reason. Nothing here needs redoing when it arrives; only the
/// ceiling this asks at.
struct FilterPlayers<'a, 'c> {
    effect: &'a ContinuousEffect,
    game: &'a GameState,
    layer_index: usize,
    cache: &'c mut FrameCache,
    you: Option<PlayerId>,
    owner: Option<PlayerId>,
}

impl FilterPlayers<'_, '_> {
    /// CR 109.5's "you".
    fn you(&mut self) -> PlayerId {
        if let Some(you) = self.you {
            return you;
        }
        let you = match self.effect.origin {
            EffectOrigin::Resolution => self.effect.controller,
            EffectOrigin::StaticAbility { .. } => effective_controller(
                self.game,
                self.effect.source,
                self.layer_index,
                self.cache,
            )
            .unwrap_or(self.effect.controller),
        };
        self.you = Some(you);
        you
    }

    /// The source object's owner (CR 108.3 / 110.2).
    fn owner(&mut self) -> PlayerId {
        if let Some(owner) = self.owner {
            return owner;
        }
        let owner = self
            .game
            .objects
            .get(&self.effect.source)
            .map(|obj| obj.owner)
            .unwrap_or(self.effect.controller);
        self.owner = Some(owner);
        owner
    }
}

/// Resolve a Layer 2 `SetController`'s `PlayerRef` to the player who ends up
/// controlling `object_id` (CR 613.1b).
///
/// `You` routes through the same `FilterPlayers` a filter's `ByController`
/// uses, so CR 109.5 and CR 611.2c have one implementation between them.
///
/// `Owner` is the owner of the object being *moved*, not of the effect's
/// source. Homeward Path's "each player gains control of all creatures they
/// own" hands each creature to its own owner, which is the opposite of what the
/// same variant means inside a `PermanentFilter`, where it describes the source.
///
/// # Which player identities may stay symbolic in a row
///
/// The walk is a pure read: it cannot prompt, and it runs many times per game
/// state. So a `PlayerRef` survives into the registry only when it must be
/// **re-derived** on every walk — `You`, because CR 109.5 makes a static
/// ability's "you" the source's *current* controller, and `Owner`, which is
/// fixed but free to recompute (CR 108.3).
///
/// Everything else is settled **when the effect is created** and stored as
/// `Player(pid)`. That covers every card whose new controller is chosen or
/// computed rather than named:
///
/// - "An opponent gains control" (Akroan Horse, Fateful Handoff, Rainbow Vale
///   — 9 cards) does not target, and its ruling is explicit that in a
///   multiplayer game *you choose the opponent as the ability resolves*.
/// - "That player gains control" (Risky Move), "choose a player at random"
///   (Scrambleverse), an auction winner (Illicit Auction) — all resolution-time
///   computations over game state.
///
/// None of those need a new `PlayerRef` variant, and none are boxed out by this
/// function; they need the *lowering* to make the choice, which is the piece
/// that does not exist yet (`codebase-state.md` item 13).
///
/// `Opponent` therefore resolves here only in a two-player game, where CR 102.2
/// leaves nothing to choose. Above two players it means the resolution step
/// skipped a choice it owed, so it asserts rather than inventing one.
fn resolve_set_controller(
    player_ref: &crate::types::effects::PlayerRef,
    object_id: ObjectId,
    effect: &ContinuousEffect,
    game: &GameState,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<PlayerId> {
    use crate::types::effects::PlayerRef;

    let mut players = FilterPlayers {
        effect,
        game,
        layer_index,
        cache,
        you: None,
        owner: None,
    };

    match player_ref {
        PlayerRef::You => Some(players.you()),
        PlayerRef::Player(pid) => Some(*pid),
        PlayerRef::Owner => game.objects.get(&object_id).map(|obj| obj.owner),
        PlayerRef::Opponent => {
            let you = players.you();
            let mut opponents = (0..game.num_players()).filter(|&pid| pid != you);
            match (opponents.next(), opponents.next()) {
                // CR 102.2 — exactly one opponent, so nothing was ever chosen.
                (Some(only), None) => Some(only),
                _ => {
                    debug_assert!(
                        false,
                        "SetController(PlayerRef::Opponent) with {} players. Which                          opponent is a choice the effect's controller makes as it                          resolves, so the row should already carry                          PlayerRef::Player(..); reaching the layer walk means the                          lowering skipped it.",
                        game.num_players()
                    );
                    None
                }
            }
        }
    }
}

/// Check whether a continuous effect applies to the given object.
fn effect_applies_to(
    effect: &ContinuousEffect,
    id: ObjectId,
    chars: &EffectiveCharacteristics,
    game: &GameState,
    layer_index: usize,
    cache: &mut FrameCache,
) -> bool {
    match &effect.affected {
        AffectedSet::SourceOnly => effect.source == id,
        AffectedSet::Fixed(ids) => ids.contains(&id),
        AffectedSet::Filter { filter } => {
            // Object must be on the battlefield for filter-based effects.
            // Checked before anything else so a non-permanent costs no frame
            // computation for the source.
            if !game.battlefield.contains_key(&id) {
                return false;
            }
            let mut players = FilterPlayers {
                effect,
                game,
                layer_index,
                cache,
                you: None,
                owner: None,
            };
            permanent_matches_filter(filter, chars, &mut players)
        }
    }
}

/// Check if a permanent's current characteristics match a filter.
///
/// `players` resolves the `PlayerRef` in a `ByController` node. Controller is
/// matched here rather than beside the filter, which is how the snapshot bug
/// hid: `ByController` used to return `true` unconditionally and defer to a
/// field on `AffectedSet::Filter`, so the two halves of one question lived in
/// two places and only one of them was re-asked during the walk.
fn permanent_matches_filter(
    filter: &crate::types::effects::PermanentFilter,
    chars: &EffectiveCharacteristics,
    players: &mut FilterPlayers<'_, '_>,
) -> bool {
    use crate::types::effects::{PermanentFilter, PlayerRef};
    match filter {
        PermanentFilter::All => true,
        PermanentFilter::ByType(t) => chars.types.contains(t),
        PermanentFilter::BySubtype(s) => chars.subtypes.contains(s),
        PermanentFilter::BySupertype(s) => chars.supertypes.contains(s),
        PermanentFilter::ByColor(c) => chars.colors.contains(c),
        // `chars.controller` is the *effective* controller of the object being
        // tested — Layer 2 will write it, and this comparison then costs
        // nothing to keep correct.
        //
        // Every variant resolves; none of them asserts. `Opponent` is a
        // predicate rather than an id on purpose: CR 102.2 makes it exactly one
        // player in a two-player game, but CR 102.3 makes "your opponents" a
        // set in multiplayer, and "controlled by someone who isn't you" is the
        // same answer in both without the type having to lie.
        PermanentFilter::ByController(player_ref) => match player_ref {
            PlayerRef::You => chars.controller == players.you(),
            PlayerRef::Opponent => chars.controller != players.you(),
            PlayerRef::Owner => chars.controller == players.owner(),
            PlayerRef::Player(pid) => chars.controller == *pid,
        },
        PermanentFilter::PowerLE(n) => {
            chars.power.map(|p| p <= *n).unwrap_or(false)
        }
        PermanentFilter::And(a, b) => {
            permanent_matches_filter(a, chars, players)
                && permanent_matches_filter(b, chars, players)
        }
        PermanentFilter::Not(inner) => !permanent_matches_filter(inner, chars, players),
    }
}

/// Resolve one side of a P/T modification against the frame so far.
///
/// `None` means the expression has no meaning in a static context — `Variable`
/// is CR 107.3's X, chosen as a spell is cast, and the `Target*`/`DamageDealt`
/// arms read a resolution that already happened. A continuous effect asking for
/// one of those is a card-authoring error, so it asserts in debug and declines
/// to apply in release rather than inventing a number.
///
/// Evaluated fresh at every layer: that is the point of `PtValue::Dynamic`.
pub(super) fn evaluate_pt_value(
    value: &PtValue,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<i32> {
    match value {
        PtValue::Fixed(n) => Some(*n),
        PtValue::Dynamic(expr) => evaluate_amount(expr, game, chars, layer_index, cache),
    }
}

/// Evaluate a card-definition amount inside the layer walk.
///
/// Anything reading *another* object goes through `compute_to_ceiling` at the
/// current `layer_index`, never through `card_data`. Both halves of that matter:
/// it keeps the layer-system invariant (effective characteristics, not printed
/// ones), and it preserves §5.2's termination argument — a request at ceiling
/// `layer_index` is strictly below the ceiling of the walk that made it, so the
/// recursion descends. A Tarmogoyf in a graveyard counting itself bottoms out
/// for exactly that reason.
fn evaluate_amount(
    expr: &crate::types::effects::AmountExpr,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<i32> {
    use crate::types::effects::{AmountExpr, Selector};

    match expr {
        AmountExpr::Fixed(n) => Some(*n as i32),

        // CR 202.3b — an object with no mana cost has mana value 0.
        AmountExpr::AffectedManaValue => {
            Some(chars.mana_cost.as_ref().map(|c| c.mana_value()).unwrap_or(0) as i32)
        }

        AmountExpr::Plus(inner, n) => {
            evaluate_amount(inner, game, chars, layer_index, cache).map(|v| v + *n as i32)
        }

        // Card *types*, not cards: ten artifact creatures in a graveyard are
        // still two types.
        AmountExpr::CardTypesAmong(selector) => match selector {
            Selector::CardsInGraveyard(None) => {
                let mut types: std::collections::HashSet<crate::types::card_types::CardType> =
                    std::collections::HashSet::new();
                for player in &game.players {
                    for card_id in &player.graveyard {
                        if let Some(card) = compute_to_ceiling(game, *card_id, layer_index, cache) {
                            types.extend(card.types.iter().copied());
                        }
                    }
                }
                Some(types.len() as i32)
            }
            other => {
                debug_assert!(
                    false,
                    "CardTypesAmong({:?}) has no evaluator yet (on '{}')",
                    other, chars.name
                );
                None
            }
        },

        // CR 107.3's X is chosen as a spell is cast, and the Target*/DamageDealt
        // arms read a resolution that already happened. A continuous effect
        // asking for one of those is a card-authoring error: assert in debug,
        // decline to apply in release, never invent a number.
        other => {
            debug_assert!(
                false,
                "continuous effect on '{}' carries {:?}, which has no static-context evaluator",
                chars.name, other
            );
            None
        }
    }
}

/// Apply a single effect modification to the characteristics frame.
///
/// `object_id` is the object being computed. Layer 4's subtype arms need it to
/// derive stable ids for intrinsic mana abilities (CR 305.6) — see
/// `land_types::intrinsic_mana_ability`.
/// `origin` is the registry row this modification came from, or `None` for an
/// intrinsic CDA application (`layers::cda`), which has no row.
///
/// Only `SetController` reads it, because it is the only modification that does
/// not carry its own answer. `AddType(Creature)` says what to do; "the
/// controller becomes *you*" does not say who "you" is, and CR 109.5 answers
/// that from the ability's source and the row's origin. The row also supplies
/// `created_on_turn`, which is when the new controller's CR 302.6 clock starts.
///
/// CDAs never reach that arm — CR 613.4a lists 7a as their only P/T sublayer
/// and none live in Layer 2 — so it asserts instead of guessing.
pub(super) fn apply_modification(
    modification: &EffectModification,
    chars: &mut EffectiveCharacteristics,
    object_id: ObjectId,
    game: &GameState,
    layer_index: usize,
    cache: &mut FrameCache,
    origin: Option<&ContinuousEffect>,
) {
    match modification {
        // Layer 2
        EffectModification::SetController(player_ref) => {
            let Some(effect) = origin else {
                debug_assert!(
                    false,
                    "SetController reached `apply_modification` with no registry                      row. CR 613.4a puts no characteristic-defining ability in                      Layer 2, so the only caller that passes `None` cannot                      produce this modification."
                );
                return;
            };
            let Some(new_controller) =
                resolve_set_controller(player_ref, object_id, effect, game, layer_index, cache)
            else {
                return;
            };
            // CR 302.6 asks whether control has been *continuous*, so the clock
            // only restarts when control actually moves. Act of Treason legally
            // targets a creature you already control; gaining control of
            // something you control changes nothing, and resetting the epoch
            // here would invent summoning sickness the CR does not give. (Act
            // of Treason grants haste, so it would hide the bug; a card that
            // gains control without haste would not.)
            if chars.controller != new_controller {
                chars.controller = new_controller;
                chars.control_since_turn = effect.created_on_turn;
            }
        }

        // Layer 4
        EffectModification::AddType(t) => { chars.types.insert(*t); }
        EffectModification::RemoveType(t) => { chars.types.remove(t); }
        EffectModification::SetTypes(types) => { chars.types = types.clone(); }
        // CR 305.6/305.7 land semantics live in `land_types` — see the module
        // docs there for why this is not a Layer 6 concern.
        EffectModification::AddSubtype(s) => {
            crate::engine::layers::land_types::apply_add_subtype(chars, s, object_id);
        }
        EffectModification::RemoveSubtype(s) => { chars.subtypes.remove(s); }
        EffectModification::SetSubtypes(subtypes) => {
            crate::engine::layers::land_types::apply_set_subtypes(chars, subtypes, object_id);
        }
        EffectModification::AddSupertype(s) => { chars.supertypes.insert(*s); }
        EffectModification::RemoveSupertype(s) => { chars.supertypes.remove(s); }
        EffectModification::SetSupertypes(supertypes) => { chars.supertypes = supertypes.clone(); }

        // Layer 5
        EffectModification::AddColor(c) => { chars.colors.insert(*c); }
        EffectModification::SetColors(colors) => { chars.colors = colors.clone(); }
        EffectModification::RemoveAllColors => { chars.colors.clear(); }

        // Layer 6
        EffectModification::GrantKeywordFlag(kw) => { chars.keywords.insert(*kw); }
        // CR 113.10b — "effects that remove an ability remove all instances of
        // it". For a keyword flag that is structural: a `HashSet` never held
        // more than one.
        EffectModification::RemoveKeywordFlag(kw) => { chars.keywords.remove(kw); }
        EffectModification::GrantAbility(def) => {
            // CR 604.3a(2) — an ability that reached an object by being granted
            // is never a characteristic-defining ability, however its text
            // reads. The flag on `AbilityDef` asserts only the four criteria
            // that are properties of the text; provenance is maintained by
            // whoever writes the ability onto an object, and this is that
            // place. Copy (Layer 1) and text-changing (Layer 3) effects hand
            // the def over whole and keep the flag, which is the *other* half
            // of 604.3a(2) and equally deliberate.
            //
            // Not clearing it would let a granted Tarmogoyf ability define P/T
            // at Layer 7a, which is exactly what 604.3a(2) forbids.
            let mut granted = (**def).clone();
            granted.is_characteristic_defining = false;
            chars.abilities.push(granted);
        }
        // CR 113.10b again, and here it is *not* structural: `abilities` is a
        // `Vec` and the same ability can genuinely appear twice — printed on
        // the card and granted on top of it. `retain`, never "remove the first
        // match".
        EffectModification::LoseAbility(ability_id) => {
            chars.abilities.retain(|a| a.id != *ability_id);
        }
        EffectModification::LoseAllAbilities => {
            chars.keywords.clear();
            chars.abilities.clear();
        }

        // Layer 7b
        EffectModification::SetPowerToughness { power, toughness } => {
            // Evaluated before mutating: `AffectedManaValue` reads `chars`, and
            // setting power first would let it observe a half-applied frame.
            let p = evaluate_pt_value(power, game, chars, layer_index, cache);
            let t = evaluate_pt_value(toughness, game, chars, layer_index, cache);
            if let (Some(p), Some(t)) = (p, t) {
                chars.power = Some(p);
                chars.toughness = Some(t);
            }
        }

        // Layer 7c
        EffectModification::ModifyPowerToughness { power, toughness } => {
            let dp = evaluate_pt_value(power, game, chars, layer_index, cache);
            let dt = evaluate_pt_value(toughness, game, chars, layer_index, cache);
            if let Some(dp) = dp {
                if let Some(ref mut p) = chars.power {
                    *p += dp;
                }
            }
            if let Some(dt) = dt {
                if let Some(ref mut t) = chars.toughness {
                    *t += dt;
                }
            }
        }

        // Layer 7d
        EffectModification::SwitchPowerToughness => {
            let old_power = chars.power;
            chars.power = chars.toughness;
            chars.toughness = old_power;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::effects::PlayerRef;
    use crate::types::keywords::KeywordFlag;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::test_support::registered;

    #[test]
    fn test_base_characteristics_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Grizzly Bears");
        assert_eq!(chars.power, Some(2));
        assert_eq!(chars.toughness, Some(2));
        assert!(chars.types.contains(&CardType::Creature));
        assert!(chars.colors.contains(&Color::Green));
        assert_eq!(chars.controller, 0);
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_pump_power_only() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a +3/+0 effect
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(0) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(2));
    }

    // COVERS: ATOM-122.1a-001
    #[test]
    fn test_counters_modify_pt() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);
        game.add_counters(id, CounterType::PlusOnePlusOne, 2);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(4));
        assert_eq!(chars.toughness, Some(4));
    }

    // COVERS-PARTIAL: ATOM-122.1a-002
    #[test]
    fn test_counters_plus_and_minus() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Big Creature")
            .card_type(CardType::Creature)
            .power_toughness(5, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);
        game.add_counters(id, CounterType::PlusOnePlusOne, 3);
        game.add_counters(id, CounterType::MinusOneMinusOne, 1);

        let chars = compute_characteristics(&game, id).unwrap();
        // Net: +3 -1 = +2
        assert_eq!(chars.power, Some(7));
        assert_eq!(chars.toughness, Some(7));
    }

    #[test]
    fn test_nonexistent_object_returns_none() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(compute_characteristics(&game, fake_id).is_none());
    }

    #[test]
    fn test_non_battlefield_object_no_modifiers() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(Color::Red)
            .build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Lightning Bolt");
        assert!(chars.types.contains(&CardType::Instant));
        assert!(chars.colors.contains(&Color::Red));
        // Not on battlefield, so controller defaults to owner
        assert_eq!(chars.controller, 0);
    }

    #[test]
    fn test_keywords_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .power_toughness(4, 4)
            .keyword(KeywordFlag::Flying)
            .keyword(KeywordFlag::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordFlag::Flying));
        assert!(chars.keywords.contains(&KeywordFlag::Vigilance));
        assert!(!chars.keywords.contains(&KeywordFlag::Trample));
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_modifies_pt() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a +3/+3 effect targeting this creature
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_registered_effect_grants_keyword() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "gains flying" effect
        let effect = registered(
            id,
            Layer::Layer6Ability,
            1,
            EffectModification::GrantKeywordFlag(KeywordFlag::Flying),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordFlag::Flying));
    }

    #[test]
    fn test_filter_based_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        let mut game = GameState::new(2, 20);

        // Two creatures controlled by player 0
        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        let giant_data = CardDataBuilder::new("Hill Giant")
            .card_type(CardType::Creature)
            .power_toughness(3, 3)
            .build();
        let giant = GameObject::new(giant_data, 0, Zone::Battlefield);
        let giant_id = giant.id;
        game.add_object(giant);
        game.place_on_battlefield(giant_id, 0);

        // Register an anthem: "Creatures you control get +1/+1"
        let anthem_source = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: anthem_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Filter {
                filter: PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                ),
            },
            modification: EffectModification::ModifyPowerToughness { power: PtValue::Fixed(1), toughness: PtValue::Fixed(1) },
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert_eq!(bears_chars.power, Some(3));
        assert_eq!(bears_chars.toughness, Some(3));

        let giant_chars = compute_characteristics(&game, giant_id).unwrap();
        assert_eq!(giant_chars.power, Some(4));
        assert_eq!(giant_chars.toughness, Some(4));
    }

    #[test]
    fn test_set_colors_replaces_base_colors() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes blue" effect (SetColors)
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        assert_eq!(chars.colors.len(), 1);
    }

    #[test]
    fn test_add_color_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register an "also red" effect (AddColor)
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::AddColor(Color::Red),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Green));
        assert!(chars.colors.contains(&Color::Red));
        assert_eq!(chars.colors.len(), 2);
    }

    #[test]
    fn test_remove_all_colors_makes_colorless() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes colorless" effect
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::RemoveAllColors,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.is_empty());
    }

    #[test]
    fn test_color_change_independent_of_pt() {

        // Color change (L5) should not affect P/T (L7) and vice versa
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L5: becomes blue
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let color_effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(color_effect);

        // L7c: +3/+3
        let pt_effect = registered(
            id,
            Layer::Layer7cModifyPT,
            game.allocate_timestamp(),
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(pt_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Color should be blue (not green)
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        // P/T should be 5/5 (2+3)
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_filter_based_color_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Static ability: "Creatures you control are also red"
        let mut game = GameState::new(2, 20);

        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        // Opponent's creature should NOT be affected
        let opp_data = CardDataBuilder::new("Savannah Lions")
            .card_type(CardType::Creature)
            .color(Color::White)
            .power_toughness(2, 1)
            .build();
        let opp = GameObject::new(opp_data, 1, Zone::Battlefield);
        let opp_id = opp.id;
        game.add_object(opp);
        game.place_on_battlefield(opp_id, 1);

        let source_id = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: source_id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                ),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert!(bears_chars.colors.contains(&Color::Green));
        assert!(bears_chars.colors.contains(&Color::Red));

        let opp_chars = compute_characteristics(&game, opp_id).unwrap();
        assert!(opp_chars.colors.contains(&Color::White));
        assert!(!opp_chars.colors.contains(&Color::Red));
    }

    // COVERS-PARTIAL: ATOM-613.4d-004
    #[test]
    fn test_counters_applied_before_switch_pt() {

        // Regression test: counters are in 7c, switch is 7d.
        // A 1/4 creature with two +1/+1 counters and a switch effect:
        // 7c: 1+2=3 / 4+2=6, then 7d: swap → 6/3
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall")
            .card_type(CardType::Creature)
            .power_toughness(1, 4)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);
        game.add_counters(id, CounterType::PlusOnePlusOne, 2);

        // Register a switch P/T effect (layer 7d)
        let effect = registered(
            id,
            Layer::Layer7dSwitchPT,
            1,
            EffectModification::SwitchPowerToughness,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // 7c: 1+2=3, 4+2=6; 7d: swap → 6/3
        assert_eq!(chars.power, Some(6));
        assert_eq!(chars.toughness, Some(3));
    }

    // === Layer 4 type-changing tests ===

    // COVERS-PARTIAL: ATOM-205.1b-004
    #[test]
    fn test_add_type_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register "becomes also a creature" effect
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
    }

    #[test]
    fn test_remove_type() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mycosynth Lattice")
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Remove Creature type
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::RemoveType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(!chars.types.contains(&CardType::Creature));
    }

    // COVERS-PARTIAL: ATOM-205.1a-003
    #[test]
    fn test_set_subtypes_replaces_all() {
        use crate::types::card_types::{LandType, Subtype};
        use std::collections::HashSet;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Steam Vents")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Island))
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // SetSubtypes to just Forest
        let mut forest_set = HashSet::new();
        forest_set.insert(Subtype::Land(LandType::Forest));
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::SetSubtypes(forest_set),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Forest)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Island)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert_eq!(chars.subtypes.len(), 1);
    }

    #[test]
    fn test_add_subtype_preserves_existing() {
        use crate::types::card_types::{LandType, Subtype};

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Add Swamp subtype ("in addition to")
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSubtype(Subtype::Land(LandType::Swamp)),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Swamp)));
        assert_eq!(chars.subtypes.len(), 2);
    }

    #[test]
    fn test_add_supertype() {
        use crate::types::card_types::Supertype;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Add Legendary supertype
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSupertype(Supertype::Legendary),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.supertypes.contains(&Supertype::Legendary));
    }

    // COVERS-PARTIAL: ATOM-613.1d-001
    #[test]
    fn test_type_change_before_color_change() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Test layer ordering: L4 (type) applies before L5 (color)
        // A filter-based color effect that checks types should see the
        // type as it stands after L4.
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L4: Add Creature type
        let l4_effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(l4_effect);

        // L5: "Creatures are also red" (filter-based)
        let l5_source = crate::types::ids::new_object_id();
        let l5_effect = ContinuousEffect {
            id: 0,
            source: l5_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(l5_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Should be both Artifact and Creature (L4 applied)
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
        // Should be Red (L5 filter sees the L4-modified type = Creature)
        assert!(chars.colors.contains(&Color::Red));
    }
}
