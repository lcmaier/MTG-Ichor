//! Cards for Phases CV-1 and CV-2 — the copy spine (CR 707, layer 1a), and a
//! permanent that enters as a copy (CR 707.5).
//!
//! **CV-1's three cards, and none of them is decoration.** `CopyRoles` has two arms
//! because Cytoshape and Mirrorweave bind the atom's target to opposite roles:
//! Cytoshape targets the permanent that *becomes* a copy and chooses its donor,
//! Mirrorweave targets the donor. Mirrorform is the third because it is the
//! card that made the donor exclusion a **field** rather than structure — it
//! prints Mirrorweave's shape without the word "other", and the first version of
//! this phase could not express it.
//!
//! None is a Clone: CR 707.5's "enters as a copy" is an entry replacement, and
//! [`clone`] is CV-2a's. [`spark_double`] is CV-2b's, the same entry copy
//! with CR 707.9's exceptions.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    CharacteristicEdit, CopyException, CopyRoles, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    ObjectSet, PlayerRef, Primitive, SelectionFilter, TargetCount, TypeChange,
};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    CopyDonor, EnterModsTemplate, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite,
};

/// "Nonlegendary creature" — the filter both cards scope their copy source
/// with, and the reason CR 707 cards say it at all: a copy of a legend meets
/// CR 704.5j the moment it exists, so the printed text keeps the effect from
/// being a sacrifice.
fn nonlegendary_creature() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::Not(Box::new(ObjectFilter::BySupertype(
            Supertype::Legendary,
        )))),
    )
}

/// Cytoshape — {1}{G}{U}
/// Instant
///
/// Choose a nonlegendary creature on the battlefield. Target creature becomes
/// a copy of that creature until end of turn.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # The card the phase is sized against
///
/// `copy-effects-architecture.md` §7 names it CV-1's consumer, and it is the
/// minimum board that exercises the whole spine: a resolution capture (CR
/// 707.2), an `ObjectSet::Fixed` locked as the effect begins (CR 611.2c), a
/// turn-bounded `Duration`, and a CR 707.4 *choice* that is not a target.
///
/// # Two selections, and only one of them is targeting
///
/// The **target** is what becomes a copy: hexproof and shroud apply to it, and
/// CR 608.2b makes the spell do nothing if it has left the battlefield. The
/// **choice** is the donor: CR 707.4 says "choose", so protection does not
/// apply and nothing fizzles. The engine keeps them apart by putting the target
/// in the atom's `EffectRecipient` and the donor's filter inside `CopyRoles`,
/// which is what the enum exists for.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// It is the only card in the crate that can put a row in layer 1, and a gated
/// subsystem no pooled card can open is the failure RS-1 taught
/// (`registry.rs`'s `PERFORMANCE_POOL` doc). Registering it alone would leave
/// the A/B measuring an engine path no measured game ever walks. Mirrorweave is
/// deliberately *not* in the pool: the two cards open the same path, and a
/// second copy of a path buys a slower fuzz run rather than a wider one.
pub fn cytoshape() -> Arc<CardData> {
    CardDataBuilder::new("Cytoshape")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Blue], 1))
        .color(Color::Green)
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Choose a nonlegendary creature on the battlefield. Target creature \
             becomes a copy of that creature until end of turn.",
        )
        .ability(AbilityDef {
            rules_text: "Choose a nonlegendary creature on the battlefield. Target creature becomes a copy of that creature until end of turn.".into(),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Copy {
                    roles: CopyRoles::RecipientsCopyChosen(SelectionFilter::Permanent(nonlegendary_creature())),
                    except: Vec::new(),
                    duration: Duration::UntilEndOfTurn,
                },
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Mirrorform — {4}{U}{U}
/// Instant
///
/// Each nonland permanent you control becomes a copy of target non-Aura
/// permanent.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why this card is here, and what it cost to find
///
/// **It is the counter-example that turned an exclusion from structure into
/// data.** CV-1 first shipped `CopyRoles::OthersCopyRecipient`, on the reading
/// that a class-scoped copy always means "each *other*" — so the exclusion of
/// the donor could be structural and no card would have to spell it. Mirrorform
/// says "each nonland permanent **you control**", which *includes* the target
/// whenever you control it, and the old arm could not express that at all. Found
/// in review, not by the census: `copy-census.py` partitions by mechanism and a
/// one-word difference inside one mechanism is exactly what it cannot see.
///
/// A permanent copying itself is very nearly a no-op — the capture is its own
/// post-layer-1 state. Not exactly one, and CR-correctly so: the row carries its
/// own `Duration`, so it holds those values past the expiry of an earlier copy
/// row that put them there.
///
/// # No duration, so until the end of the game
///
/// The card states none, and CR 611.2a makes such an effect last "until the
/// end of the game". It was spelled `UntilEndOfTurn` until CV-1b, because an
/// indefinite row is reached by neither CR 514.2's expiry nor
/// `remove_by_source`; CR 400.7 is what ends one now, when its subject moves.
///
/// # Registered, not pooled
///
/// It opens no engine path Mirrorweave does not, so it stays out of
/// `PERFORMANCE_POOL` for the reason given there. Its job is to keep
/// `exclude_donor: false` from being scaffolding with no consumer.
///
/// # The rulings, and where each is tested
///
/// All four are in `tests/phase_cv1b_integration_test.rs`: nothing enters,
/// so no "enters" ability applies (1); only the printed values are copied
/// (2); a copy of a copy copies what it copied (3); X is 0 (4).
pub fn mirrorform() -> Arc<CardData> {
    CardDataBuilder::new("Mirrorform")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Each nonland permanent you control becomes a copy of target \
             non-Aura permanent.",
        )
        .ability(AbilityDef {
            rules_text: "Each nonland permanent you control becomes a copy of target non-Aura permanent.".into(),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Copy {
                    roles: CopyRoles::FilteredCopyRecipient {
                        // "Each nonland permanent you control" — and no
                        // "other", which is the whole reason this card is
                        // registered.
                        filter: ObjectFilter::And(
                            Box::new(ObjectFilter::Not(Box::new(
                                ObjectFilter::ByType(CardType::Land),
                            ))),
                            Box::new(ObjectFilter::ByController(PlayerRef::You)),
                        ),
                        exclude_donor: false,
                    },
                    except: Vec::new(),
                    // No duration printed: CR 611.2a's "until the end of the game".
                    duration: Duration::Indefinite,
                },
                // "target non-Aura permanent" — the donor, and the only place
                // in CV-1 where a copy source is not required to be a creature.
                EffectRecipient::Target(
                    SelectionFilter::Permanent(ObjectFilter::Not(Box::new(
                        ObjectFilter::BySubtype(Subtype::Enchantment(EnchantmentType::Aura)),
                    ))),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

/// Mirrorweave — {2}{W/U}{W/U}
/// Instant
///
/// Each other creature becomes a copy of target nonlegendary creature until
/// end of turn.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why the second card, and what it measures
///
/// It is `copy-effects-architecture.md` §9 item 4's named customer — one
/// capture applied to a whole class — and CV-1's answer to that question comes
/// from what this card actually builds: **one row, not one per creature.** CR
/// 611.2c locks the affected set as the effect begins, so the row is a single
/// `ObjectSet::Fixed` carrying a single `Box<CopiableValues>`. `Box` and `Arc`
/// allocate identically here, and the phases that would tell them apart are the
/// ones that create a row *per object* (CV-2's entry replacements, the
/// class-scoped statics).
///
/// It also makes the second half of §4.7 leg 2 observable: a copied static
/// ability's row takes **the copying permanent's** controller, not the spell's,
/// and "each other creature" reaches creatures both players control. Copy a
/// Glorious Anthem with Cytoshape and one controller answers for both; copy it
/// with Mirrorweave and they diverge.
///
/// # The hybrid cost, and a precedent this card deliberately does not follow
///
/// `{2}{W/U}{W/U}` is two generic plus two hybrid pips. `ManaSymbol::Hybrid`
/// exists, but no payment path handles it — `mana_helpers`' auto-tap returns
/// `None` on any non-`Colored`/`Generic` symbol — so a verbatim cost would make
/// the card *silently uncastable*, which is this project's named worst outcome.
/// Registered as **`{2}{W}{U}`**: a strict **subset** of the real card's legal
/// payments (it forbids WW and UU), so no game reaches a state the printed card
/// forbids, and CR 202.2's colors come out W and U exactly as printed.
///
/// `codebase-state.md`'s 2026-08-24 entry set the opposite precedent —
/// `phase5_pre_cards::inside_out` stays unregistered because simplifying its
/// hybrid cost would "misrepresent the card". That was the right call there and
/// is the wrong one here, for a reason that is about the *pool* rather than
/// about the card: Inside Out had a registered substitute for the engine path it
/// covered, and Mirrorweave and Mirrorform are the only consumers of
/// `CopyRoles::FilteredCopyRecipient` in the crate. Leaving Mirrorweave out
/// would ship the arm's `exclude_donor: true` leg with no random-play coverage,
/// which is the failure `PERFORMANCE_POOL`'s own doc records RS-1 for. Recorded rather than quietly done: the first real hybrid
/// payment path should delete this paragraph and the approximation together.
pub fn mirrorweave() -> Arc<CardData> {
    CardDataBuilder::new("Mirrorweave")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Blue], 2))
        .color(Color::White)
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Each other creature becomes a copy of target nonlegendary creature \
             until end of turn.",
        )
        .ability(AbilityDef {
            rules_text: "Each other creature becomes a copy of target nonlegendary creature until end of turn.".into(),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                // "Each **other** creature" — `exclude_donor` is the word
                // "other", and it is a field rather than structure because
                // Mirrorform prints the same shape without it.
                Primitive::Copy {
                    roles: CopyRoles::FilteredCopyRecipient {
                        filter: ObjectFilter::ByType(CardType::Creature),
                        exclude_donor: true,
                    },
                    except: Vec::new(),
                    duration: Duration::UntilEndOfTurn,
                },
                // The target is the *donor* here, and it is what the printed
                // "nonlegendary" scopes.
                EffectRecipient::Target(
                    SelectionFilter::Permanent(nonlegendary_creature()),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

/// Clone — {3}{U}
/// Creature — Shapeshifter, 0/0
///
/// You may have this creature enter as a copy of any creature on the
/// battlefield.
///
/// (Oracle text verified on Scryfall, 2026-09-28.)
///
/// # CV-2a's card: a chosen donor and no "except"
///
/// 52 of the 62 printed cards that enter as a copy choose their donor on the
/// battlefield, as Clone does, and 42 of the 62 print "except". Clone is the
/// base both kinds share; Spark Double, CV-2b's card, is the "except" half
/// (`copy-effects-architecture.md` §4.1a).
///
/// A `SourceOnly` entry replacement whose rewrite is
/// `Rewrite::EnterAsCopy`, so CR 616.1c's bucket has a printed producer and
/// the copy is the permanent's state from the moment it arrives (CV-2a's
/// D1, in `plans/archive/copy-effects-architecture-landed.md`). The donor is
/// chosen, not targeted, by the entering object's controller (CR 707.6), and
/// the "you may" is that choice's empty pick (D4). An unchosen Clone is the
/// 0/0 its card says, and CR 704.5f takes it.
///
/// In `PERFORMANCE_POOL`: it is the only card that opens the entry-copy path,
/// and a path no pooled card opens is RS-1's failure. At `{3}{U}` a random
/// game casts it, and it copies in about half its entries, since the random
/// provider declines a `(0, 1)` pick half the time.
///
/// # The rulings, and where each is tested
///
/// All eight are linked from `tests/phase_cv2a_integration_test.rs`: not a
/// target (1), X as 0 (2), the copied "enters" abilities (3), copying nothing
/// (4), nothing entering beside it (5), a token's values without being a
/// token (6), a copy's copy (7), and what is not copied (8).
pub fn clone() -> Arc<CardData> {
    CardDataBuilder::new("Clone")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Shapeshifter))
        .power_toughness(0, 0)
        .rules_text("You may have this creature enter as a copy of any creature on the battlefield.")
        .ability(AbilityDef {
            rules_text: "You may have this creature enter as a copy of any creature on the battlefield.".into(),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef {
                optional: true,
                ..ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::SourceOnly,
                    Rewrite::EnterAsCopy(EntryCopyTemplate {
                        donor: CopyDonor::Chosen(SelectionFilter::Creature),
                        except: Vec::new(),
                    }),
                )
            })),
        })
        .build()
}

/// Spark Double — {3}{U}
/// Creature — Illusion, 0/0
///
/// You may have this creature enter as a copy of a creature or planeswalker
/// you control, except it enters with an additional +1/+1 counter on it if
/// it's a creature, it enters with an additional loyalty counter on it if
/// it's a planeswalker, and it isn't legendary.
///
/// (Oracle text verified on Scryfall, 2026-09-29.)
///
/// # CV-2b's card: CR 707.9's exceptions on Clone's entry copy
///
/// Three exceptions, in printed order, and each is one arm of
/// `CopyException` (`copy-effects-architecture.md` §4.1a): two CR 707.9f
/// conditions, each over a CR 707.9e additional counter, and a CR 707.9b
/// edit that is part of the copiable values, so a copy of this is not
/// legendary either. The conditions read the CR 614.12 frame of the copy,
/// each checked without itself and with the other where the other applies,
/// which is what the words say and what a planeswalker whose type hangs on
/// its counters makes observable (§4.1a, "The Kaito board").
///
/// # The rulings, and where each is tested
///
/// All nine are linked from `tests/phase_cv2b_integration_test.rs`: what is
/// not copied (1), not legendary, and a copy of it neither (2), X as 0 (3), a
/// copy's copy (4), a token's values without being a token (5), the copied
/// "enters" abilities and triggers (6), printed loyalty plus one (7, a
/// `// RULING-DEVIATION:` on Kaito's board, by the register row
/// `lookahead-entry-counters`), the characteristics as it enters, not the
/// donor's (8), and nothing entering beside it (9).
pub fn spark_double() -> Arc<CardData> {
    let additionally = |counter| CopyException::Additionally(EnterModsTemplate::with_counters(counter, 1));
    let creature_or_planeswalker_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::Or(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByType(CardType::Planeswalker)),
        )),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    );
    CardDataBuilder::new("Spark Double")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Illusion))
        .power_toughness(0, 0)
        .rules_text(
            "You may have this creature enter as a copy of a creature or planeswalker you control, \
             except it enters with an additional +1/+1 counter on it if it's a creature, it enters \
             with an additional loyalty counter on it if it's a planeswalker, and it isn't legendary.",
        )
        .ability(AbilityDef {
            rules_text: "You may have this creature enter as a copy of a creature or planeswalker you control, except it enters with an additional +1/+1 counter on it if it's a creature, it enters with an additional loyalty counter on it if it's a planeswalker, and it isn't legendary.".into(),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef {
                optional: true,
                ..ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::SourceOnly,
                    Rewrite::EnterAsCopy(EntryCopyTemplate {
                        donor: CopyDonor::Chosen(SelectionFilter::Permanent(creature_or_planeswalker_you_control)),
                        except: vec![
                            CopyException::If(
                                ObjectFilter::ByType(CardType::Creature),
                                vec![additionally(CounterType::PlusOnePlusOne)],
                            ),
                            CopyException::If(
                                ObjectFilter::ByType(CardType::Planeswalker),
                                vec![additionally(CounterType::Loyalty)],
                            ),
                            CopyException::Modifies(CharacteristicEdit::Types(TypeChange {
                                remove_supertypes: vec![Supertype::Legendary],
                                ..TypeChange::NONE
                            })),
                        ],
                    }),
                )
            })),
        })
        .build()
}
