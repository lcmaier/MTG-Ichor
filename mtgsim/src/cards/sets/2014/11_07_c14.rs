//! Commander 2014 (`c14`), released 2014-11-07.

use std::sync::Arc;

use crate::cards::authoring::{creature, non, static_replacement, token};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::ObjectSet;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::Zone;

/// Containment Priest — {1}{W}
/// Creature — Human Cleric, 2/2
///
/// Flash
/// If a nontoken creature would enter and it wasn't cast, exile it instead.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # The card before the suppression — `replacement-architecture.md` §11 item 19
///
/// RC-4 stops asking CR 616.1's question when every member of the bucket is an
/// `EnterWith` (`pipeline::ordering_cannot_change_outcome`): Root Maze beside
/// Idyllic Beachfront is a choice with one outcome, and the fuzz harness paid a
/// decision round-trip for it on every land drop. Suppressing that prompt with
/// nothing else registered would have returned the multi-candidate branch to
/// dead code in every fuzz game — the Kalitas gap a third time, caused by a
/// fix. This card is the member that is *not* an `EnterWith`: an `Instead`
/// beside one is a bucket the rule keeps asking about, because which CR 614.5
/// slot is spent first is a different event log even when the board is not.
///
/// # What it reads, and where
///
/// - **"wasn't cast"** is `EventPattern::EnterBattlefield { cast: Some(false) }`
///   — CR 601's fact projected off the entry's `ZoneChangeAct`. A creature
///   spell resolving arrives with `Resolved` and is not matched.
/// - **"nontoken creature"** is an `ObjectSet::Filter`, matched against
///   CR 614.12's frame: the object *as it would exist on the battlefield*. A
///   noncreature artifact returned under March of the Machines is a creature to
///   the Priest and is exiled — the clause (3) case RC-3's gate opened and this
///   card is the first replacement to read.
/// - **"exile it instead"** is `Instead(ZoneChangeTo { Exile })` on an entry.
///   The entry is the zone change (RC-4b), so the substitute is one move from
///   where the card is — its graveyard, or the hand for a Dryad Arbor played
///   as a land — and the card never becomes a permanent. The log holds that
///   one zone change and no entry.
///
/// # Reachability — why Dryad Arbor is registered beside it
///
/// Flickerwisp, Banishing Light, undying and persist put creature cards onto
/// the battlefield without casting them, and the card excludes tokens. A land
/// drop is the other road: a Land Creature played as a land "wasn't cast", and
/// Containment Priest's ruling on Dryad Arbor is exile. Both are in the
/// stress pool and neither is in `PERFORMANCE_POOL` — the path they open is
/// the *branch*, not a cost, and Keldon Warlord is RC-4's deliberate addition
/// there.
pub fn containment_priest() -> Arc<CardData> {
    CardDataBuilder::new("Containment Priest")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Flash)
        .rules_text(
            "Flash\nIf a nontoken creature would enter and it wasn't cast, exile it instead.",
        )
        .ability(static_replacement(
            "If a nontoken creature would enter and it wasn't cast, exile it instead.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: Some(false) },
                ObjectSet::battlefield_filter(creature().and(non(token()))),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
            ),
        ))
        .build()
}
