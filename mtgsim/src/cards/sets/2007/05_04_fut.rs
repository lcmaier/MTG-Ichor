//! Future Sight (`fut`), released 2007-05-04.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, create_tokens, creature, creatures_you_control, each, each_in, exile, functioning_in, has_base_power_and_toughness,
    loses_all_abilities, non, prevent_the_next_n_damage_this_turn, static_ability, this_object, token, triggered_ability, whenever, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, ObjectFilter, ObjectSet, PlayerRef, TokenDef};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneChangePattern, ZoneSet};

/// "Whenever [subject] is put into [whose] graveyard from the battlefield":
/// the graveyard named by its owner, whoever controlled the creature
/// (Bridge from Below's second ruling).
fn put_into_a_graveyard_from_the_battlefield(subject: ObjectFilter, whose: PlayerRef) -> TriggerEvent {
    TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(subject),
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: Some(whose),
        multiplicity: Multiplicity::PerOccurrence,
    }
}

/// Bridge from Below — {B}{B}{B}
/// Enchantment
///
/// > Whenever a nontoken creature is put into your graveyard from the
/// > battlefield, if this card is in your graveyard, create a 2/2 black Zombie
/// > creature token.
/// > When a creature is put into an opponent's graveyard from the
/// > battlefield, if this card is in your graveyard, exile this card.
///
/// Each intervening "if" states where its ability works (CR 113.6b): in a
/// graveyard and nowhere else, so on the battlefield the card does nothing.
/// Both abilities are leaves-the-battlefield abilities and look back (CR
/// 603.10a), so the card has to be in the graveyard before the creature dies.
///
/// It exercises an intervening "if" that states the zone its ability works in
/// (CR 113.6b), asked from a graveyard; a card that reaches the graveyard in
/// the event answers none of its look-back triggers (CR 603.10a).
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The phase's new dispatcher path: while the card sits in a graveyard,
/// every death is matched against a trigger there. Forced into every `stress`
/// deck, 200 games at seed 777 and two seats, it reached a graveyard 23 times
/// (18 discarded) and triggered 24 times, and the average game ran 27.1 turns
/// against 26.7. It does nothing on the battlefield, so it ends no game.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"While Bridge from Below is on the battlefield, it has no effect. It
///   does something only if it's in a graveyard. Yes, this is a bit weird."*
///   (#1) → `bridge_from_below_on_the_battlefield_does_nothing`.
/// - *"Neither ability cares who controlled the creature that died, only
///   which graveyard it was put into. ..."* (#2) →
///   `bridge_from_below_reads_whose_graveyard_not_who_controlled_the_creature`.
/// - *"If Bridge from Below isn't in your graveyard immediately before a
///   creature dies, its abilities won't trigger at all. If Bridge from Below
///   isn't still in your graveyard as its first ability resolves, you won't
///   create a token."* (#3) →
///   `bridge_from_below_put_into_the_graveyard_by_the_same_wipe_does_not_trigger`
///   and `bridge_from_below_gone_from_the_graveyard_makes_no_token`.
/// - *"If a nontoken creature you control and a creature an opponent controls
///   die at the same time, you choose the order in which Bridge from Below's
///   triggered abilities resolve. You can create a token before you exile
///   Bridge from Below."* (#4) →
///   `bridge_from_below_can_make_its_token_before_it_exiles_itself`.
pub fn bridge_from_below() -> Arc<CardData> {
    let zombie = "Whenever a nontoken creature is put into your graveyard from the battlefield, if this card is in your graveyard, create a 2/2 black Zombie creature token.";
    let exile_text = "When a creature is put into an opponent's graveyard from the battlefield, if this card is in your graveyard, exile this card.";
    let nontoken_creature =
        creature().and(non(token()));
    CardDataBuilder::new("Bridge from Below")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black, ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Whenever a nontoken creature is put into your graveyard from the battlefield, if this card is in your graveyard, create a 2/2 black Zombie creature token.\nWhen a creature is put into an opponent's graveyard from the battlefield, if this card is in your graveyard, exile this card.")
        // CR 113.6b's statement, "if this card is in your graveyard", printed
        // as each ability's intervening "if" (CR 603.4).
        .ability(functioning_in(
            ZoneSet::GRAVEYARD,
            triggered_ability(
                zombie,
                whenever(
                    put_into_a_graveyard_from_the_battlefield(nontoken_creature, PlayerRef::You),
                    create_tokens(you(), 1, TokenDef::creature(2, 2, &[Color::Black], &[CreatureType::Zombie])),
                ),
            ),
        ))
        .ability(functioning_in(
            ZoneSet::GRAVEYARD,
            triggered_ability(
                exile_text,
                whenever(
                    put_into_a_graveyard_from_the_battlefield(creature(), PlayerRef::Opponent),
                    exile(this_object()),
                ),
            ),
        ))
        .build()
}

/// Dryad Arbor
/// Land Creature — Forest Dryad, 1/1 (green by color indicator, CR 204.1)
///
/// (This land isn't a spell, it's affected by summoning sickness, and it has
/// "{T}: Add {G}.")
///
/// (Oracle text verified on Scryfall, 2026-09-02. No mana cost, and no rules
/// text of its own: the parenthetical is reminder text, so `rules_text` is
/// left to `mana_ability_single`, which writes the intrinsic "{T}: Add {G}.")
///
/// # The land drop that "wasn't cast"
///
/// Registered for Containment Priest's reachability (see there), and it earns
/// the stress pool on its own: it is the only registered permanent with two
/// permanent types, so it is a creature to Doom Blade and Humility, a land to
/// Root Maze and Blood Moon, summoning-sick for its own mana ability (CR 302.6
/// — `Cost::TapSelf` asks, and the reminder text says so), and the land that
/// CR 305.7 turns into a Mountain *Dryad* rather than a Mountain — the
/// CR 205.1a case `land_types::apply_set_subtypes` got wrong until this card
/// made it reachable.
///
/// The mana ability is written out, as the basic and dual lands write
/// theirs: CR 305.6 makes it intrinsic to the Forest type, and the layer walk
/// synthesizes the intrinsic one only when a land type is *gained*.
pub fn dryad_arbor() -> Arc<CardData> {
    CardDataBuilder::new("Dryad Arbor")
        .card_type(CardType::Land)
        .card_type(CardType::Creature)
        .subtype(Subtype::Land(LandType::Forest))
        .subtype(Subtype::Creature(CreatureType::Dryad))
        .color(Color::Green)
        .color_indicator(vec![Color::Green])
        .power_toughness(1, 1)
        .mana_ability_single(ManaType::Green)
        .build()
}

/// Samite Censer-Bearer — {W}
///
/// > {W}, Sacrifice this creature: Prevent the next 1 damage that would be
/// > dealt to each creature you control this turn.
///
/// CR 615.11's consumer: "creates a prevention shield for each applicable
/// creature when the spell or ability … resolves". The recipient is
/// `FilteredPermanents`, so the resolution makes **one row per creature** it
/// finds, each with its own `NextDamage(1)` — the Censer-Bearer itself is
/// already in the graveyard, sacrificed as the cost.
///
/// The ruling (Scryfall, 2026-09-08): *this ability sets up a separate 1-point
/// damage prevention shield on each creature you control at the time the
/// ability resolves* → a creature entering afterwards has none
/// (`ATOM-615.11-001`), and two creatures each taking 2 each take 1, because
/// the counts are separate. Kitsune Palliator's "each creature and each
/// player" is this card plus an each-player recipient `EffectRecipient` lacks;
/// one customer, so it waits.
pub fn samite_censer_bearer() -> Arc<CardData> {
    CardDataBuilder::new("Samite Censer-Bearer")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Rebel))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text(
            "{W}, Sacrifice this creature: Prevent the next 1 damage that would be dealt to \
             each creature you control this turn.",
        )
        .ability(activated_ability("{W}, Sacrifice this creature: Prevent the next 1 damage that would be dealt to each creature you control this turn.",
            vec![Cost::Mana(ManaCost::build(&[ManaType::White], 0)), Cost::SacrificeSelf],
            prevent_the_next_n_damage_this_turn(each(creatures_you_control()), 1),
        ))
        .build()
}

/// Tarmogoyf — {1}{G}
/// Creature — Lhurgoyf
/// */1+*
/// Tarmogoyf's power is equal to the number of card types among cards in all
/// graveyards and its toughness is equal to that number plus 1.
///
/// The Layer 7a card (CR 613.4a). Three things about how it is authored:
///
/// **Printed P/T is `(0, 1)`, not `(0, 0)`.** The box reads `*/1+*`: the `*` is
/// what the CDA supplies and resolves to 0 when nothing supplies it (CR 208.2a),
/// but the `1+` is printed arithmetic and survives on its own. The Merfolk
/// Trickster ruling says so in as many words — "If the target creature has power
/// and toughness written as */* with an ability that defines its power and
/// toughness, it's 0/0 when it loses all abilities. If its power and toughness
/// are written as */*+1, it's 0/1, and so on."
///
/// This is load-bearing rather than cosmetic. Strip Tarmogoyf's abilities
/// without setting a P/T and it is a 0/1, which survives state-based actions;
/// authored as 0/0 it would die. (Under Humility the answer is 1/1 either way,
/// because Humility's Layer 7b half sets one.)
///
/// **Card types, not cards.** Ten artifact creatures in a graveyard are still
/// two card types — hence `CardTypesAmong` rather than `CountOf`.
///
/// **All graveyards, including its own.** `ObjectSet::filter_in(All, GRAVEYARD)`
/// spans every player, and a Tarmogoyf in a graveyard counts itself.
pub fn tarmogoyf() -> Arc<CardData> {
    let card_types_in_graveyards =
        || AmountExpr::CardTypesAmong(ObjectSet::filter_in(ObjectFilter::All, ZoneSet::GRAVEYARD));

    CardDataBuilder::new("Tarmogoyf")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Lhurgoyf))
        .power_toughness(0, 1)
        .rules_text(
            "Tarmogoyf's power is equal to the number of card types among cards in all \
             graveyards and its toughness is equal to that number plus 1.",
        )
        .ability(static_ability(
            "Tarmogoyf's power is equal to the number of card types among cards in all graveyards and its toughness is equal to that number plus 1.",
            has_base_power_and_toughness(this_object(), card_types_in_graveyards(), AmountExpr::Plus(Box::new(card_types_in_graveyards()), 1)),
        ).characteristic_defining())
        .build()
}

/// Thornweald Archer — {1}{G}
/// Creature — Elf Archer
/// 2/1 Reach, Deathtouch
pub fn thornweald_archer() -> Arc<CardData> {
    CardDataBuilder::new("Thornweald Archer")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Archer))
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 1)
        .keyword_flag(KeywordFlag::Reach)
        .keyword_flag(KeywordFlag::Deathtouch)
        .rules_text("Reach\nDeathtouch")
        .build()
}

/// Yixlid Jailer — {1}{B}
/// Creature — Zombie Wizard, 2/1
/// "Cards in graveyards lose all abilities."
///
/// **The first consumer of a filter that reaches another zone**, and the card
/// `codebase-state.md` item 9's correction 1 named for the job. Nothing about
/// it is new except the `ZoneSet`: `Primitive::LoseAllAbilities` already
/// existed and already writes `Channels::ABILITIES | Channels::KEYWORDS`, so
/// the row is the one Humility registers with a different affected set.
///
/// `ObjectFilter::All` and not a type leaf: the card says *cards*, with no
/// qualifier, and `ZoneSet::GRAVEYARD` alone is the whole restriction. Every
/// graveyard, not just its controller's — there is no `ByOwner` here because
/// the card names none.
///
/// # What the layer walk does with it
///
/// The row makes every graveyard card a member of the pass (`Board::seed`),
/// which is the change LJ is: before it, a graveyard card was a `NonMember`
/// and got printed characteristics plus its own CDAs. It costs nothing on a
/// board without this card, because `RegistryScopeSummary::reachable_zones`
/// is `BATTLEFIELD` there and the seed's loop does not run.
///
/// # The ruling this card is famous for, and why it is not LJ's
///
/// Yixlid Jailer's own rulings are about cards *changing zones* — a creature
/// card that leaves the graveyard has its abilities again, and a card put into
/// the graveyard already having lost its abilities does not keep them lost.
/// Both fall out of the row being re-read every pass rather than captured, so
/// neither needs code here. What the card cannot yet reach is anything that
/// *cares* about a graveyard card's abilities — flashback, retrace, Bridge
/// from Below's trigger — because CR 113.6 decides which of those function in
/// a graveyard at all, and that is A5.
pub fn yixlid_jailer() -> Arc<CardData> {
    CardDataBuilder::new("Yixlid Jailer")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Zombie))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 1)
        .rules_text("Cards in graveyards lose all abilities.")
        .ability(static_ability(
            "Cards in graveyards lose all abilities.",
            loses_all_abilities(each_in(ObjectFilter::All, ZoneSet::GRAVEYARD)),
        ))
        .build()
}
