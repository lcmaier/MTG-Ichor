//! Tempest (`tmp`), released 1997-10-14.

use std::sync::Arc;

use crate::cards::authoring::{
    artifact, at_beginning_of, chosen_by, create_replacement, create_tokens, creature, draw_cards, each, every_permanent_or_player, land,
    loses_all_abilities, sacrifice, skip, spell_ability, static_ability, static_replacement, target_player, triggered_ability, whenever, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{Choice, ChoiceScope, ChoiceSide, Effect, ObjectSet, Pick, PlayerSet, TokenDef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

/// Diabolic Edict — {1}{B}
/// Instant
///
/// Target player sacrifices a creature of their choice.
///
/// (Oracle text verified on Scryfall, 2026-08-31.)
///
/// # "Of their choice" is the load-bearing phrase
///
/// The *target* chooses, not the caster — which is what makes this the
/// resolution-time selection path §4.9 puts its candidate filter in, and what
/// makes it the honest test of Sigarda. An edict where the caster picked would
/// exercise a different rule and would not distinguish "no prompt" from "a
/// prompt asked of the wrong player".
///
/// It also carries CR 608.2d for free: a targeted player who controls no
/// creatures is asked nothing, and no error is raised. That is the same empty
/// candidate list Sigarda produces, which is `cant-effects-architecture.md`
/// §4.9's whole claim — a restriction is a *second reason a candidate is
/// unavailable*, not a second mechanism.
///
/// # In the stress pool, not the frozen one
///
/// Registered, so `fuzz_games --pool stress` plays both cards; absent from
/// `PERFORMANCE_POOL`, so the recorded A/B baseline still measures the same
/// board it always did.
pub fn diabolic_edict() -> Arc<CardData> {
    CardDataBuilder::new("Diabolic Edict")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text("Target player sacrifices a creature of their choice.")
        .ability(spell_ability(
            "Target player sacrifices a creature of their choice.",
            // What is sacrificed is chosen as it resolves, by the target (CR
            // 115.1) and from their own creatures (CR 701.21a).
            sacrifice(chosen_by(Choice {
                chooser: target_player(),
                among: ChoiceScope::ChoosersPermanents,
                picks: vec![Pick::exactly(1, creature())],
                acts_on: ChoiceSide::Chosen,
            })),
        ))
        .build()
}

/// Furnace of Rath — {1}{R}{R}{R}
///
/// > If a source would deal damage to a permanent or player, it deals double
/// > that damage to that permanent or player instead.
///
/// CR 701.10g's doubling, and the first `Rewrite::Amount` in the crate.
///
/// **The pooled card of the phase.** It is the first static `DealDamage`
/// source in `PERFORMANCE_POOL`, so it opens the gather sweep on every damage
/// event while it is on the battlefield — a new engine path, and the one this
/// PR measures. Mono-red at four mana, and *not legendary*, which is what lets
/// a random deck put two of them down: CR 614.5's own example ("if you have
/// two of these on the battlefield, the damage is multiplied by 4") needs two
/// instances, and `ATOM-614.5-001` had never had a registered board that could
/// build one.
///
/// # The rulings, and where each is tested
///
/// - *Two of these multiply by 4* → `tests/phase_rd_integration_test.rs`, and
///   it is CR 616.1's multi-candidate prompt from two **printed** cards.
/// - *The multiplied damage counts in all ways as if it came from the original
///   source; Furnace of Rath is not the source* → asserted on
///   `GameEvent::DamageDealt`'s `source_id`, which the rewrite does not touch.
/// - *If a spell or ability damages multiple things, divide up the damage
///   before applying this effect* and *the trample rules cause damage to be
///   divided before it is doubled* → structurally true rather than coded:
///   `assign_combat_damage` divides before anything is proposed, and the
///   pipeline never sees the undivided number. Asserted with War Mammoth,
///   which is already in the pool.
/// - *Prevent 4 then double the remaining 1, or double to 10 then prevent 4* →
///   **RD-2's**, when Mending Hands exists to be the prevention half. Its
///   ordering half is covered here by Ghosts of the Innocent, which is the same
///   CR 616.1 question with two rewrites RD-1 does build.
pub fn furnace_of_rath() -> Arc<CardData> {
    CardDataBuilder::new("Furnace of Rath")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals double that \
             damage to that permanent or player instead.",
        )
        .ability(static_replacement("If a source would deal damage to a permanent or player, it deals double that damage to that permanent or player instead.",
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                every_permanent_or_player(),
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .build()
}

/// Humility — {2}{W}{W}
/// Enchantment
/// All creatures lose all abilities and have base power and toughness 1/1.
///
/// The Layer 6 card, and the one the corpus names in ATOM-613.1f-001 and
/// COMP-613-TARMOGOYF-HUMILITY-001. Real text, verbatim; nothing here is
/// simplified.
///
/// **One ability, two atoms, and that matters.** The card is a single static
/// `AbilityDef` whose effect is a `Sequence`: `LoseAllAbilities` in Layer 6 and
/// `SetPowerToughness(1, 1)` in Layer 7b, both over "each creature". They
/// therefore share an `EffectGroup` and a timestamp, which is what CR 613.6
/// wants — "if an effect starts to apply in one layer, it will continue to be
/// applied to the same set of objects in each other applicable layer". Split
/// into two `AbilityDef`s they would be two groups, and the 7b half would
/// re-filter against a board the 6 half had already changed.
///
/// **It does not strip itself.** Humility is an Enchantment and its filter is
/// creatures, so CR 604.2's existence check finds its ability intact at every
/// layer. That is why this card, unlike `fixtures::self_stripping_land`, needs
/// nothing from the frame-cache termination argument.
///
/// **Humility + Opalescence** is the famous case, and since LI-2 it is in
/// reach: `printings::opalescence` is registered, and its rulings
/// (2009-10-01, 2006-02-01) are asserted in `tests/phase_li2_integration_test.rs`
/// — the two 7b parts apply in timestamp order, each to the set it locked in
/// layer 4 (CR 613.6). Humility against a single opponent's creatures —
/// including a Tarmogoyf, whose CDA dies in Layer 6 before Layer 7a can read
/// it — is what the tests here exercise.
pub fn humility() -> Arc<CardData> {
    let creatures = each(creature());

    CardDataBuilder::new("Humility")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("All creatures lose all abilities and have base power and toughness 1/1.")
        .ability(static_ability(
            "All creatures lose all abilities and have base power and toughness 1/1.",
            loses_all_abilities(creatures).and_has_base_power_and_toughness(1, 1),
        ))
        .build()
}

/// Meditate — {2}{U}
/// Instant
///
/// > Draw four cards. You skip your next turn.
///
/// The **consumable** skip, and the first row in the crate whose duration is
/// `Indefinite` and whose end is a use: "your next turn" is not a length of
/// time, so nothing but `Uses::Once` can retire it. CR 614.10a's second
/// sentence is the board two of these build — *"if two effects each cause a
/// player to skip their next occurrence, that player must skip the next
/// two"* — and it is the test that could not be written against a per-player
/// counter, which has no way to be offered to CR 616.1 twice.
///
/// Ruling (2026-09-11): *"You skip one turn as part of the effect."* → one row,
/// one turn.
/// # The rulings, and where each is tested
///
/// - *"You skip one turn as part of the effect."* → one row, `Uses::Once`, so
///   one turn; and two Meditates skip two, which is CR 614.10a's own sentence
///   and the board that needed a real queue.
pub fn meditate() -> Arc<CardData> {
    CardDataBuilder::new("Meditate")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Draw four cards. You skip your next turn.")
        .ability(spell_ability("Draw four cards. You skip your next turn.",
            Effect::Sequence(vec![
                draw_cards(you(), 4),
                // `PlayerSet::You` and a `you()` recipient: the row is the def
                // as authored, and CR 109.5 resolves "you" against the row's
                // controller at each event rather than at resolution.
                create_replacement(skip(EventPattern::BeginTurn, PlayerSet::You).once()).until_end_of_game(),
            ]),
        ))
        .build()
}

/// Root Maze — {G}
/// Enchantment
///
/// Artifacts and lands enter tapped.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why RC-3 ships a card at all
///
/// `engineering-practices.md#how-many-cards-a-mechanic-owes`'s axis, not a
/// quota: **RC-3's own claim needs no new card, and that is argued rather than
/// assumed.** The gate change widens a path the pool already walks — Blood Moon
/// and Humility are both in `PERFORMANCE_POOL`, both are filter-scoped layer
/// effects, and RC-2 put two permanents that enter modified in beside them. A
/// tapland under Blood Moon and a Scarecrow under Humility are reachable in a
/// fuzz game today and answered differently the moment the gate moves.
///
/// This card is for a **different** gap, and it is one RC-2 left by mistake.
/// RC-2 recorded that CR 616.1's multi-candidate branch was unreachable on an
/// entry until RC-3 opened the gate, on the grounds that no `ObjectSet::Filter`
/// effect could match an entering permanent. That was never true of the
/// *replacement* pipeline: `gather::set_affects` matches a `ReplacementDef`'s
/// filter through `GameState::object_matches_filter_of_source`, which has no
/// battlefield gate and never had one. The branch has been reachable since RB
/// merged, one registered card away, and RB left the identical gap with Kalitas
/// — a Legendary whose two copies each apply only to the other player's
/// creatures, so the ordering choice, the applied set across instances and CR
/// 101.4's APNAP ordering among simultaneous choosers were all dead code that
/// tested green.
///
/// **Two applicable effects on one entry, from a registered pool, is what this
/// card buys.** Root Maze plus [`idyllic_beachfront`] is exactly that: two
/// `EnterWith(tapped)` rewrites on one land, one from a filter on another
/// permanent and one `SourceOnly` from the land itself, so the pipeline asks
/// CR 616.1's question of a real player in a real game.
///
/// # Why this card out of the four
///
/// Kismet, Loxodon Gatekeeper and Frozen Aether all scope to "your opponents",
/// which is `ObjectFilter::ByController(PlayerRef::Opponent)` — and that
/// leaf reads `chars.controller`, which for an entering permanent comes from
/// `compute::base_controller`'s owner fallback. Right for a land drop, wrong
/// for a permanent spell cast by a non-owner, and unowned until RC-3 recorded it
/// (see `codebase-state.md`). Root Maze scopes by *type* and asks nothing about
/// controllers, so it exercises the multi-candidate branch without standing on
/// a read RC-3 is still fixing.
///
/// Its reachability is the other half. `{G}` at one mana means it lands on turn
/// one and is out before most permanents arrive. (This sentence used to open
/// with "`random_deck` picks one or two colors out of five, so a mono-green
/// card is in about a third of decks" — true when RC-3 shipped, false since
/// the `Everywhere` land removed the color filter on 2026-09-03. The cheapness
/// is the whole of the argument now.)
/// It also matches **artifacts**, which is what puts it and [`chainbreaker`] on
/// the same board — a permanent whose entry both a filter and CR 122.6a modify.
///
/// # What it is not
///
/// Not a look-ahead test. Root Maze is on the battlefield before the entry it
/// modifies, so it is one of CR 614.12 clause (3)'s effects that "already
/// exist" and needs no frame. The self-exclusion half of CR 614.12 — Orb of
/// Dreams entering untapped under its own "Permanents enter tapped" — is
/// covered by a fixture in `tests/phase_rc_integration_test.rs`, because Root
/// Maze is an Enchantment and its own filter excludes it by type. A registered
/// card that matches *itself* would be the sharper board, and Orb of Dreams is
/// the card for it whenever the pool wants a second one.
pub fn root_maze() -> Arc<CardData> {
    CardDataBuilder::new("Root Maze")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("Artifacts and lands enter tapped.")
        .ability(static_replacement(
            "Artifacts and lands enter tapped.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::battlefield_filter(artifact().or(land())),
                Rewrite::EnterWith(EnterModsTemplate::tapped()),
            ),
        ))
        .build()
}

/// Verdant Force — {5}{G}{G}{G}
/// Creature — Elemental 7/7
///
/// > At the beginning of each upkeep, create a 1/1 green Saproling creature token.
///
/// "Each upkeep" is `Whose::Each`: the step's record carries whose it is
/// (item 10) and this ability does not ask.
///
/// It exercises "each upkeep" — a step beginning, a token from a trigger.
///
/// # The rulings, and where each is tested
///
/// - *"Verdant Force's ability triggers at the beginning of each upkeep, not
///   just each of your upkeeps."*
///   → `phase_tr1_integration_test::verdant_force_triggers_at_an_opponents_upkeep_too`.
/// - The Two-Headed Giant ruling is a format-variant disposition in the
///   ledger: CR 810 is not built.
pub fn verdant_force() -> Arc<CardData> {
    CardDataBuilder::new("Verdant Force")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 5))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(7, 7)
        .rules_text("At the beginning of each upkeep, create a 1/1 green Saproling creature token.")
        .ability(triggered_ability("At the beginning of each upkeep, create a 1/1 green Saproling creature token.", whenever(
            at_beginning_of(StepType::Upkeep, Whose::Each),
            create_tokens(you(), 1, TokenDef::creature(1, 1, &[Color::Green], &[CreatureType::Saproling])),
        )))
        .build()
}
