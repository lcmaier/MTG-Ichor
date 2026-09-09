//! Cards for Phase RD-1 — the damage event's two subjects and its results
//! (CR 614.5, 615.10, 701.10g, 120.3).
//!
//! **Four printed cards on two axes, and a fixture for the third result.**
//! `replacement-architecture.md` §9's RD-1 section names them; what follows is
//! why each is here rather than a cheaper one.
//!
//! The two axes are the ones the phase builds. *Which subject* an effect is
//! about — an object, a player, or (Furnace of Rath) both — and *what it does
//! to the amount*: multiply, halve, or prevent half. Every card below is a
//! distinct pair of those, and none of them is a second copy of another's
//! engine path (`engineering-practices.md` §3.3, tier 2):
//!
//! | Card | Subject | Amount |
//! |---|---|---|
//! | [`furnace_of_rath`] | every object **and** every player | `Multiplier(2)` |
//! | [`ghosts_of_the_innocent`] | the same | `Halve(Down)` |
//! | [`gisela_blade_of_goldnight`] | opponents' / yours, two rows on one card | `Multiplier(2)` and `PreventHalf(Up)` |
//! | [`angel_of_suffering`] | **you** and no object at all | whole-event `Prevent`, with a rider |
//!
//! **Two Furnaces are what CR 614.5's own example needs**, and the card is not
//! legendary, so the registered pool can build the board — which
//! `ATOM-614.5-001` had never had (§3.3's tier-1 finding, applied here).
//! Ghosts beside Furnace is the first **non-commuting** CR 616.1 choice a fuzz
//! game can reach: 3 damage becomes 1 then 2, or 6 then 3, and the affected
//! player picks. Dictate of the Twin Gods was the first draft's second doubler
//! and is dropped — same shape as Furnace, and two Furnaces already give the
//! commuting pair.
//!
//! Gisela is the card that needs [`PlayerSet`] twice on one object, in both of
//! its non-trivial arms, and both halves of `Rounding` are on the same
//! battlefield as her doubler. Angel of Suffering is the only prevention
//! consumer here and the only rider that rides on a *player* subject
//! (`codebase-state.md` item 27) — its "twice that many cards" is the whole
//! reason `AmountExpr::ReplacedAmount` and `Multiply` exist.
//!
//! # The fixture, and why it is a genuine consumer
//!
//! [`loyalty_probe`] is not a printed card. CR 120.3c — damage to a
//! planeswalker removes that many loyalty counters — needs a planeswalker on
//! the battlefield, and registering a *printed* one fails
//! `register-a-card-only-once-the-engine-can-play-it`: loyalty abilities have
//! no `AbilityType` and no activation path, so a real Jace would be a card with
//! three dead abilities wearing a real name, which `engineering-practices.md`
//! §3 forbids. Leaving the arm out instead would leave `perform_action` marking
//! damage on a permanent the targeting code already validates as "any target",
//! which is the wrong answer waiting for a card.
//!
//! So the fixture is the middle, on the `graveyard_probe` convention §3.3
//! already admits — and it is a consumer rather than a test prop.
//! `SelectionFilter::Any` enumerates planeswalkers, so the random agent bolts
//! it; three damage is exactly its printed loyalty, so **CR 704.5i becomes
//! reachable from a game** for the first time (`phase_sba_cards` measured it
//! at 0 across 200 stress games). It is registered in the stress pool and
//! stays out of `PERFORMANCE_POOL`.
//!
//! # Colors, and what a random deck can draw
//!
//! `random_deck` filters nonlands by color, so a two-color card reaches
//! roughly one deck in sixteen. Furnace of Rath is mono-red at four mana and
//! is the pooled one for that reason; Ghosts is mono-white at seven, Angel of
//! Suffering mono-black at five, and Gisela is `{4}{R}{W}{W}` and legendary —
//! registered for the stress pool, where the point is breadth rather than
//! frequency.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::ids::new_ability_id;
use crate::types::effects::{
    AffectedSet, AmountExpr, Effect, EffectRecipient, ObjectFilter, PlayerRef, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding,
};

/// The static ability wrapper every card in this file uses.
///
/// Local rather than shared with `test_support::static_ability`: that one is a
/// test helper, and a card file must not depend on one.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "A permanent or player" — CR 614.1's two kinds of subject, as the pair of
/// sets that says "everything".
///
/// Furnace of Rath and Ghosts of the Innocent print the same scope word for
/// word, so they share this rather than each spelling it: a divergence between
/// two cards whose oracle text is identical would be an authoring bug nothing
/// could catch.
fn every_permanent_or_player() -> (AffectedSet, PlayerSet) {
    (AffectedSet::Filter { filter: ObjectFilter::All }, PlayerSet::Everyone)
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
///   CR 616.1 question with two rewrites this PR does build.
pub fn furnace_of_rath() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Furnace of Rath")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals double that \
             damage to that permanent or player instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage,
                affected,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(players),
        ))
        .build()
}

/// Ghosts of the Innocent — {5}{W}{W}
///
/// > If a source would deal damage to a permanent or player, it deals half
/// > that damage, rounded down, to that permanent or player instead.
///
/// The printed **inverse** of a doubler, and the reason `Rounding` exists:
/// CR 107.1a puts the direction on the card, and this one says "rounded down"
/// where Gisela says "rounded up".
///
/// # The rulings, and where each is tested
///
/// - *Half of 1 rounded down is 0. A source that would deal 1 damage won't
///   deal damage at all* → the rewrite leaves a 0-damage proposal and
///   `never_happens` drops it on the next iteration (CR 614.7a).
/// - *Multiple effects are cumulative … with three on the battlefield, 14
///   damage becomes 7, then 3, then finally 1* → three instances, each applied
///   once (CR 614.5).
/// - *This isn't a damage prevention effect. If Excruciator … would deal 7
///   damage, it deals 3 instead* → the reason [`AmountRewrite::Halve`] is a
///   separate arm from `PreventHalf`, which CR 615.12 would stop. Excruciator's
///   half is RD-4's, when a damage event can be unpreventable.
/// - *If damage is redirected, it's only halved once* → **RD-4's**: CR 614.5's
///   applied set survives a `Retarget`, and there is no `Retarget` yet.
/// - *If both Ghosts of the Innocent and Furnace of Rath are on the
///   battlefield, the controller of the permanent being dealt damage or the
///   player being dealt damage can apply the effects in either order. This can
///   matter if the original amount of damage is odd* → the phase's sharpest
///   test, and the first non-commuting CR 616.1 choice reachable from two
///   printed statics.
/// - *If a damage prevention effect and this effect would apply to the same
///   damage, the player … may apply the effects in either order* → the same
///   question against a prevention, which Gisela's other half supplies.
pub fn ghosts_of_the_innocent() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Ghosts of the Innocent")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 5))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(4, 5)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals half that \
             damage, rounded down, to that permanent or player instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage,
                affected,
                Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
            )
            .affecting_players(players),
        ))
        .build()
}

/// Gisela, Blade of Goldnight — {4}{R}{W}{W}
///
/// > Flying, first strike
/// >
/// > If a source would deal damage to an opponent or a permanent an opponent
/// > controls, that source deals double that damage to that player or permanent
/// > instead.
/// >
/// > If a source would deal damage to you or a permanent you control, prevent
/// > half that damage, rounded up.
///
/// **Two statics on one card, and each is a `Filter` beside a [`PlayerSet`].**
/// That is decision 0's shape used in both of its non-trivial arms — CR 109.5's
/// "you" and CR 102.1's "an opponent", resolved against the ability's current
/// controller on every gather — and both halves of `Rounding` on one
/// battlefield.
///
/// # The rulings, and where each is tested
///
/// - *Gisela doubles damage dealt to opponents and permanents your opponents
///   control from any source, **including sources controlled by those
///   opponents*** → the pattern carries no source-side constraint, so this is
///   true by construction; asserted anyway, because a `SourcePattern` arrives
///   in RD-3 and this is the assertion that would fail if one were added here
///   by mistake.
/// - *If multiple replacement effects would modify how damage would be dealt,
///   the player being dealt damage (or the controller of the permanent) chooses
///   the order* → the Ghosts/Furnace test generalised to a prevention:
///   Gisela's half beside an opponent's Furnace on 5 damage is prevent-3-then-
///   double-2 or double-to-10-then-prevent-5.
/// - *If damage … is being divided or assigned among multiple permanents an
///   opponent controls … divide the original amount and double the results* →
///   the War Mammoth test, from the other side of the table.
pub fn gisela_blade_of_goldnight() -> Arc<CardData> {
    CardDataBuilder::new("Gisela, Blade of Goldnight")
        .mana_cost(ManaCost::build(
            &[ManaType::Red, ManaType::White, ManaType::White],
            4,
        ))
        .color(Color::Red)
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text(
            "Flying, first strike\nIf a source would deal damage to an opponent or a \
             permanent an opponent controls, that source deals double that damage to that \
             player or permanent instead.\nIf a source would deal damage to you or a \
             permanent you control, prevent half that damage, rounded up.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage,
                AffectedSet::Filter {
                    filter: ObjectFilter::ByController(PlayerRef::Opponent),
                },
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage,
                AffectedSet::Filter {
                    filter: ObjectFilter::ByController(PlayerRef::You),
                },
                Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Up)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Angel of Suffering — {3}{B}{B}
///
/// > Flying
/// >
/// > If damage would be dealt to you, prevent that damage and mill twice that
/// > many cards.
///
/// The phase's only prevention consumer, and the only effect in it that is
/// about **no object at all**: `AffectedSet::Fixed(vec![])` beside
/// `PlayerSet::You`. Its rider is what makes `Rider.subject` an `EventSubject`
/// rather than an `Option<ObjectId>` — "mill" needs to name the player the
/// damage was aimed at (`codebase-state.md` item 27) — and "twice that many"
/// is `AmountExpr::Multiply(ReplacedAmount, 2)`, read off the event as the
/// CR 616.1 loop saw it.
///
/// # The rulings, and where each is tested
///
/// - *If you would mill more cards than are in your library, you mill all
///   cards in your library* → CR 701.2's "as much as it can", in
///   `Primitive::Mill`.
/// - *Damage that would be dealt to you will be prevented even if you can't
///   mill twice that many* → the prevention is not conditional on the rider,
///   which falls out of §4.1a's queue-then-perform order.
/// - *If the damage can't be prevented for some reason, you'll still mill
///   twice that many* → **RD-4's** dovetail test, beside Reverse Damage: the
///   rider is unconditional once queued (CR 615.12), and RD-1 has no way to
///   make damage unpreventable.
pub fn angel_of_suffering() -> Arc<CardData> {
    CardDataBuilder::new("Angel of Suffering")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Nightmare))
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 3)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf damage would be dealt to you, prevent that damage and mill twice \
             that many cards.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage,
                // No object is affected. The empty `Fixed` is the honest way to
                // say so: a `Filter` would have to describe an empty set, and
                // `SourceOnly` would make the Angel shield *itself*.
                AffectedSet::Fixed(Vec::new()),
                Rewrite::Prevent,
            )
            .affecting_players(PlayerSet::You)
            .with_then(Effect::Atom(
                Primitive::Mill(AmountExpr::Multiply(Box::new(AmountExpr::ReplacedAmount), 2)),
                // The rider's single resolved target is the event's subject —
                // the player the damage was aimed at, which for this card is
                // always the Angel's controller and for the next one on this
                // path may not be.
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            )),
        ))
        .build()
}

/// Loyalty Probe — a fixture planeswalker, and CR 120.3c's consumer.
///
/// **Not a printed card**, and the module docs say why a printed one could not
/// take its place. Printed loyalty 3, no abilities at all, `{2}` so any deck
/// can cast it.
///
/// Three is chosen rather than five: `SelectionFilter::Any` enumerates
/// planeswalkers, so a random agent's Lightning Bolt finds this one, and three
/// damage takes it to exactly zero — which is what makes **CR 704.5i**
/// reachable from a fuzz game for the first time (`phase_sba_cards` measured
/// that state-based action at 0 across 200 stress games, blocked on "loyalty
/// abilities + CR 120.3c"; RD-1 supplies the second half and the first is
/// combat's and Phase 8's).
///
/// It has no subtype. CR 205.3j gives every printed planeswalker one, and every
/// name in `PlaneswalkerType` belongs to a real character — so borrowing one
/// would put an invented card under a real planeswalker's name, which is
/// exactly the thing `engineering-practices.md` §3 keeps out of the registry.
///
/// Attacking one stays out of reach: `combat::validation` refuses planeswalker
/// attack targets, and that is combat's phase rather than this one's.
pub fn loyalty_probe() -> Arc<CardData> {
    CardDataBuilder::new("Loyalty Probe")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Planeswalker)
        .loyalty(3)
        .rules_text("")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::actions::{ActionContext, GameAction};
    use crate::events::event::DamageTarget;
    use crate::state::game_state::GameState;
    use crate::test_support::{
        place_vanilla_creature, put_on_battlefield, setup_two_player_game, test_ctx,
        RecordingDecisionProvider,
    };
    use crate::types::effects::CounterType;

    /// A 2/2 for P0 and a board with no replacement source but the one under
    /// test. Damage to a *creature* throughout: RD-1's tests of damage to a
    /// **player** need CR 120.3a's decomposition, which is two commits away,
    /// and they live in `tests/phase_rd_integration_test.rs`.
    fn board() -> (GameState, ObjectIdPair) {
        let mut game = setup_two_player_game();
        let bolt_source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let victim = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
        (game, ObjectIdPair { source: bolt_source, victim })
    }

    struct ObjectIdPair {
        source: crate::types::ids::ObjectId,
        victim: crate::types::ids::ObjectId,
    }

    fn deal(game: &mut GameState, ids: &ObjectIdPair, amount: u64) {
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: DamageTarget::Object(ids.victim),
                amount,
                is_combat: false,
            },
            &test_ctx(),
        )
        .unwrap();
    }

    fn marked(game: &GameState, ids: &ObjectIdPair) -> u32 {
        game.battlefield[&ids.victim].damage_marked
    }

    // CR 701.10g on a real card: 3 becomes 6.
    #[test]
    fn furnace_of_rath_doubles_damage_to_a_permanent() {
        let (mut game, ids) = board();
        put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        assert_eq!(marked(&game, &ids), 6);
    }

    // "The multiplied damage counts in all ways as if it came from the
    // original source. Furnace of Rath is not the source."
    #[test]
    fn furnace_of_rath_is_not_the_source_of_the_doubled_damage() {
        use crate::events::event::GameEvent;
        let (mut game, ids) = board();
        let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        let dealt: Vec<_> = game
            .events
            .events()
            .filter_map(|e| match e {
                GameEvent::DamageDealt { source_id, amount, .. } => Some((*source_id, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(dealt, vec![(ids.source, 6)]);
        assert_ne!(dealt[0].0, furnace);
    }

    // "Half of 1 rounded down is 0. A source that would deal 1 damage won't
    // deal damage at all" — CR 614.7a drops the emptied event, so nothing is
    // marked and no `DamageDealt` is emitted.
    #[test]
    fn ghosts_of_the_innocent_halves_one_damage_to_nothing() {
        use crate::events::event::GameEvent;
        let (mut game, ids) = board();
        put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        deal(&mut game, &ids, 1);
        assert_eq!(marked(&game, &ids), 0);
        assert!(!game
            .events
            .events()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })));
    }

    // "Multiple Ghosts of the Innocent effects are cumulative … with three on
    // the battlefield, 14 damage becomes 7, then 3, then finally 1."
    //
    // Three instances is three CR 616.1 candidates, so the affected side is
    // asked twice — and the answer cannot matter, because halving commutes with
    // itself. `picking_all` is the provider that would notice a candidate the
    // list should not have offered.
    #[test]
    fn three_ghosts_take_fourteen_damage_to_one() {
        let (mut game, ids) = board();
        for _ in 0..3 {
            put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        }
        let dp = RecordingDecisionProvider::picking_all();
        let ctx = ActionContext::new(&dp);
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: DamageTarget::Object(ids.victim),
                amount: 14,
                is_combat: false,
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(marked(&game, &ids), 1);
        assert_eq!(dp.prompts(), 2, "three candidates, then two, then one");
    }

    // Gisela's doubler is scoped by CR 102.1's "an opponent", resolved against
    // her controller: P0's Gisela doubles damage to P1's creature and leaves
    // damage to P0's own alone.
    #[test]
    fn gisela_doubles_only_damage_to_an_opponents_permanent() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);

        for victim in [theirs, mine] {
            game.execute_action(
                GameAction::DealDamage {
                    source,
                    target: DamageTarget::Object(victim),
                    amount: 3,
                    is_combat: false,
                },
                &test_ctx(),
            )
            .unwrap();
        }
        assert_eq!(game.battlefield[&theirs].damage_marked, 6);
        // Her *other* half applies here: prevent half of 3, rounded up, is 2.
        assert_eq!(game.battlefield[&mine].damage_marked, 1);
    }

    // "Gisela doubles damage … from any source, including sources controlled
    // by those opponents." The pattern carries no source-side constraint, and
    // RD-3 is where one could be added by mistake.
    #[test]
    fn gisela_doubles_an_opponents_own_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let their_source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let their_victim = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source: their_source,
                target: DamageTarget::Object(their_victim),
                amount: 2,
                is_combat: false,
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&their_victim].damage_marked, 4);
    }

    // CR 107.1a, on the board rather than on the arithmetic: Gisela rounds up,
    // so 5 damage to something she protects leaves 2.
    #[test]
    fn gisela_prevents_half_rounded_up() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Object(mine),
                amount: 5,
                is_combat: false,
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&mine].damage_marked, 2);
    }

    // The fixture is a planeswalker that enters with its printed loyalty
    // (CR 306.5b), which is what CR 120.3c will take counters off.
    #[test]
    fn loyalty_probe_enters_with_three_loyalty() {
        let mut game = setup_two_player_game();
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        assert_eq!(
            game.battlefield[&probe].counter_count(CounterType::Loyalty),
            3
        );
    }
}
