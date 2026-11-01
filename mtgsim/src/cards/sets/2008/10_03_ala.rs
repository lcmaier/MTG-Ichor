//! Shards of Alara (`ala`), released 2008-10-03.

use std::sync::Arc;

use crate::cards::authoring::{creature, destroy, spell_ability, static_replacement, target_permanent, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, Cost};
use crate::types::effects::{CounterType, ObjectSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::replacement::{AuxiliaryMove, EventPattern, PerChosen, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangeAct};

/// Bone Splinters — {B}
/// Sorcery
///
/// As an additional cost to cast this spell, sacrifice a creature.
/// Destroy target creature.
///
/// (Oracle text and rulings verified on Scryfall, 2026-09-08.)
///
/// **The pooled carrier of `Cost::Sacrifice`**, chosen over Altar's Reap for
/// what it does *not* do: Altar's Reap draws two, which makes every measured
/// game bigger and cost the A/B ~20% of its wall clock for engine coverage it
/// shares with this card exactly. Bone Splinters opens the same four paths —
/// a mandatory additional cost, a sacrifice paid through the chokepoint, a
/// castability answer that turns on something other than mana, and a payment
/// prompt that is not an allocation — and removes a creature instead of adding
/// two cards. Altar's Reap stays registered: it is CR 601.2h's own example and
/// its tests are the rule's.
///
/// Its two rulings are the two CM-3 already tests on Altar's Reap, which
/// is the same code path: "you must sacrifice exactly one creature … you can't
/// sacrifice additional creatures", and "once you begin to cast Bone Splinters,
/// no player may take actions until you're done."
pub fn bone_splinters() -> Arc<CardData> {
    CardDataBuilder::new("Bone Splinters")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text(
            "As an additional cost to cast this spell, sacrifice a creature.\nDestroy target creature.",
        )
        .additional_cost(AdditionalCost::Mandatory(vec![Cost::Sacrifice(
            creature(),
            1,
        )]))
        .ability(spell_ability(
            "Destroy target creature.",
            destroy(target_permanent(creature())),
        ))
        .build()
}

/// Rhox War Monk — {G}{W}{U}
/// Creature — Rhino Monk
/// 3/4 Lifelink
pub fn rhox_war_monk() -> Arc<CardData> {
    CardDataBuilder::new("Rhox War Monk")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .color(Color::White)
        .color(Color::Blue)
        .mana_cost(ManaCost { symbols: vec![
            ManaSymbol::Colored(ManaType::Green),
            ManaSymbol::Colored(ManaType::White),
            ManaSymbol::Colored(ManaType::Blue),
        ] })
        .power_toughness(3, 4)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Lifelink")
        .build()
}

/// Thunder-Thrash Elder — {2}{R}
/// Creature — Lizard Warrior, 1/1
///
/// Devour 3 (As this creature enters, you may sacrifice any number of
/// creatures. It enters with three times that many +1/+1 counters on it.)
///
/// (Oracle text verified on Scryfall, 2026-09-03.)
///
/// # The card CR 614.13 is written about
///
/// CR 614.13b's own example names it: "A player controls Runeclaw Bear and
/// casts Thunder-Thrash Elder, a red creature spell with devour 3 ... its
/// controller can choose to sacrifice Runeclaw Bear when applying the devour 3
/// effect or when applying the devour 5 effect, but not both." So the rule's
/// worked example and the phase's card are the same object, which is the
/// cheapest possible check that the arm models the rule and not a paraphrase
/// of it.
///
/// **Devour is a replacement effect, not a triggered ability**, and the whole
/// arm exists because of that one word. A trigger would put the sacrifices on
/// the stack after the Elder was already a 1/1 on the battlefield; CR 614.13
/// makes them part of applying the entry, so the counters are on before
/// anything — a state-based action, a `PermanentEnteredBattlefield` reader —
/// can see the permanent at all. A 1/1 that arrives as a 7/7 is what a game
/// shows of that.
///
/// # In `PERFORMANCE_POOL`, and why this one rather than Sutured Ghoul
///
/// RC-5 opens a new engine path — an application that prompts and mutates — and
/// `engineering-practices.md#cards-and-pools` says a phase that opens one adds
/// a card to the measured pool. `{2}{R}` is castable in a random game where
/// `{4}{B}{B}{B}` is not, its candidates are permanents the pool already
/// produces in numbers, and it is the only registered card that makes a player
/// *choose* how much of their own board to spend. It also puts a second
/// replacement effect on the same entry as Root Maze whenever both are out,
/// which is CR 616.1's multi-candidate branch with a member that is not an
/// `EnterWith` — the case `ordering_cannot_change_outcome` must refuse to
/// suppress.
///
/// **Sigarda, Host of Herons is in the same pool and does not stop it**, which
/// is worth stating because the opposite is the easy assumption. Her sentence
/// is "spells and abilities **your opponents control** can't cause you to
/// sacrifice permanents" — a `SourceFilter::ControlledBy(Opponent)` — and
/// devour is your own creature's ability, so your own Sigarda is a legal
/// candidate for it. What the pair does measure is that the `cause` the
/// candidate filter asks CR 101.2 with is the *effect's* controller: get that
/// wrong and Sigarda silently switches devour off for the player who controls
/// her. The filter itself is the resolution-time choice's axis-1 shape — a
/// prohibited move removes the candidate rather than being chosen and then
/// refused, which would leave the Elder counting creatures that never went
/// anywhere.
pub fn thunder_thrash_elder() -> Arc<CardData> {
    CardDataBuilder::new("Thunder-Thrash Elder")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Lizard))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(1, 1)
        .rules_text("Devour 3")
        .ability(static_replacement(
            "Devour 3",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                // CR 614.12's first sentence — it affects only this permanent,
                // so it is found on the entering object itself (gather source
                // 1a) and needs no board sweep.
                ObjectSet::SourceOnly,
                Rewrite::EnterAfterMoving(AuxiliaryMove {
                    from: Zone::Battlefield,
                    // "Creatures" is every creature *you control*: CR 701.21a
                    // says a sacrifice is its controller moving it, so nothing
                    // else could be chosen. On the battlefield that has to be a
                    // filter leaf, because control and ownership diverge there.
                    filter: creature().and(you_control()),
                    to: Zone::Graveyard,
                    act: ZoneChangeAct::Sacrificed,
                    // "Any number", so the bound is the candidate list and the
                    // floor is zero — which is where devour's "you **may**"
                    // lives. Marking the def `optional` instead would ask twice
                    // and let a decline spend CR 614.5's one opportunity.
                    up_to: None,
                    per_chosen: Some((CounterType::PlusOnePlusOne, PerChosen::Fixed(3))),
                }),
            ),
        ))
        .build()
}
