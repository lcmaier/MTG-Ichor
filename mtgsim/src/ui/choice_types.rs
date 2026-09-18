use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, EffectRecipient, TargetCount};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::Zone;

use super::decision::PriorityAction;

/// What kind of decision is being made. UIs use this to render appropriate
/// screens. AI agents can match on this for specialized heuristics.
/// Adding a new variant here is the ONLY change needed when a new decision
/// type is introduced — no trait methods or impl changes.
///
/// Exhaustive matching is intentional: single-crate project, compiler flags
/// every match site when a variant is added. **Two of those sites are the
/// variant's own contract**: [`Self::subject`] and [`Self::describe`] match
/// without a wildcard, so a new variant decides at birth which object it is
/// about and what a client shows for it (`backlog.md` §2.21).
///
/// **A payload names things by id and in the CR's vocabulary, never by an
/// engine AST** — `codebase-state.md` item 141. What a client needs is what
/// the engine already computed: the options are the legality, the subject is
/// which object is asking, and `describe` says why. `SelectRecipients`'
/// `EffectRecipient` predates the rule and is its own piece of work.
///
/// Only variants that correspond to currently-implemented engine decisions
/// are included. New variants are added as the engine grows — the exhaustive
/// matching ensures every DP impl gets updated at compile time.
#[derive(Debug, Clone)]
pub enum ChoiceKind {
    // --- Priority & Turn Structure ---
    PriorityAction,

    // --- Combat ---
    DeclareAttackers,
    DeclareBlockers,
    AssignCombatDamage { attacker_id: ObjectId },
    AssignTrampleDamage { attacker_id: ObjectId, defending_target: DamageTarget },

    // --- Casting Pipeline (601.2) ---
    ChooseXValue { spell_id: ObjectId, x_count: u64 },
    /// CR 601.2b / 118.9 — whether to cast `spell_id` for its mana cost or
    /// for one of its alternative costs. The options are `NormalCost` first,
    /// then the alternatives in printed order.
    ChooseAlternativeCost { spell_id: ObjectId },
    /// CR 601.2b / 118.8 — which of `spell_id`'s *optional* additional costs
    /// (kicker) the player intends to pay, any number of them; a mandatory
    /// one is in the total and is not offered. Printed order.
    ChooseAdditionalCosts { spell_id: ObjectId },
    /// Select recipients for an effect (covers both MTG "target" and non-targeting
    /// "choose" — the `EffectRecipient` field distinguishes them).
    SelectRecipients { recipient: EffectRecipient, spell_id: ObjectId },
    /// CR 601.2h — how the generic part of `mana_cost` is split across the
    /// types in the payer's pool, each pip's own type reserved first. The
    /// buckets are the pool's types; asked only when the split is not forced
    /// (`ui::ask::forced_allocation`). `spell_or_ability_id` is what is being
    /// paid for.
    GenericManaAllocation { spell_or_ability_id: ObjectId, mana_cost: ManaCost },
    /// CR 601.2f — "if multiple cost reductions apply, the player may apply
    /// them in any order." Asked only with two or more; the options are the
    /// reductions' *sources*, in battlefield timestamp order, and the answer
    /// is a permutation of them. With the symbols the engine pays today the
    /// order never changes the total (`cost-architecture.md` §3.4), and it is
    /// asked anyway because the CR makes it the player's; a payer
    /// `DecisionProvider` may answer it without asking anyone.
    OrderCostReductions { spell_id: ObjectId },
    /// 601.2g / 602.1b — "mana ability window" inside spell cast or ability
    /// activation. The player may activate mana abilities (rule 605) to cover
    /// the spell's / ability's cost. Asked repeatedly in a loop: each prompt
    /// offers currently-available mana abilities; the DP picks one to activate
    /// or declines (empty pick = stop). The engine exits the loop when the
    /// pool covers the cost, the DP declines, or no abilities remain.
    ///
    /// This is the rules-correct mechanism for "tap lands to pay" — the
    /// decision of *which* ability to activate is a player decision (605.1a),
    /// preserving the engine invariant of not making strategic choices on
    /// behalf of players. Examples that require this granularity:
    /// - Cavern of Souls: choose between `{T}: add {C}` and `{T}: add any color`
    /// - Multiple equivalent `{T}: add {B}` sources (artifact vs land matters
    ///   for other spells like improvise or landfall triggers)
    /// - Generic vs colored ordering with mixed mana producers
    ManaAbilityWindow { spell_or_ability_id: ObjectId, remaining_cost: ManaCost },

    /// CR 601.2h / 701.21a — which permanents to sacrifice to pay a
    /// `Cost::Sacrifice`. Altar's Reap's "sacrifice a creature", Krark-Clan
    /// Ironworks' "Sacrifice an artifact".
    ///
    /// **Asked at payment, not at announcement.** CR 601.2b announces an
    /// *intention to pay* an additional cost and never which object pays it,
    /// which is the whole of CR 601.2h's own example: the total is locked at
    /// {B} before Thunderscape Familiar is picked as the thing sacrificed.
    ///
    /// The options are permanents the payer controls that match the cost's
    /// filter, in `battlefield_ids_ordered`, and the source of the spell or
    /// ability is among them when it matches — Ironworks sacrificing itself
    /// is the filter matching normally, not a special case.
    ///
    /// Asked only when there are more candidates than the cost needs. With
    /// exactly as many as it needs the payment is forced and nothing is
    /// asked, which is [`Self::ChooseEnteringController`]'s CR 102.2 shape.
    ChooseSacrificeForCost { spell_or_ability_id: ObjectId, count: u32 },

    // --- Replacement effects (CR 616.1) ---
    /// Two or more replacement or prevention effects want the same event and
    /// the affected object's controller (or the affected player) must choose
    /// one to apply.
    ///
    /// **Only asked with two or more candidates.** There is no choice to make
    /// with one, and that rule is what keeps every existing scripted test green
    /// now that every `execute_action` traverses the pipeline
    /// (`replacement-architecture.md` §4.1).
    ///
    /// `affected_object` is `None` when the event is about the choosing player
    /// rather than about an object.
    ChooseReplacementEffect { affected_object: Option<ObjectId> },

    /// A "you **may** ... instead" replacement effect is offering itself
    /// (CR 614.1a). Declining is CR 614.5's one opportunity taken.
    ApplyOptionalReplacement { affected_object: Option<ObjectId>, source: ObjectId },

    /// CR 615.7 — a "prevent the next N damage" effect applies to damage from
    /// two or more sources at once, and the affected player (or the affected
    /// permanent's controller) chooses which damage it prevents.
    ///
    /// The buckets are the **damage sources**, in batch order — combat's
    /// `battlefield_ordered`, so the order is process-independent — each
    /// capped at the damage that source would deal; the total is the smaller
    /// of the count left and the damage on offer, so an allocation always
    /// prevents as much as the effect can. Asked once per instance per batch,
    /// the first time the instance is chosen, over every member it applies to
    /// — for Mending Hands the one subject's sources, for a "you and/or
    /// permanents you control" effect the sources hitting each of them.
    ///
    /// **Never asked with one source.** 615.7's choice exists only among "two
    /// or more"; with one, every point is prevented unasked.
    ///
    /// `source` is the object whose effect is allocating, not any of the
    /// bucket sources; `remaining` is its count before this allocation.
    AllocateNextDamage { source: ObjectId, remaining: u64 },

    /// CR 609.7a — a resolving spell or ability creates a prevention or
    /// replacement effect that names "a source of your choice", and the
    /// choice is made now: "the source is chosen when the effect is created".
    ///
    /// The options are permanents in battlefield order followed by spells on
    /// the stack (`SelectionFilter::DamageSource`), and **no property is
    /// filtered out** — Circle of Protection: Red's "a *red* source of your
    /// choice" offers every source and its row simply never applies to a
    /// source that is not red when the damage comes, which is CR 609.7b's
    /// recheck rather than an enumeration rule.
    ///
    /// Asked only with two or more candidates; with one the choice is forced
    /// and nothing is asked, which is [`Self::ChooseCopySource`]'s CR 102.2
    /// shape.
    ///
    /// `source` is the object whose effect is choosing — [`Self::
    /// AllocateNextDamage`]'s field, for the same reason: "why am I being
    /// asked this" is answered by it and by nothing else on the prompt.
    ChooseDamageSource { source: ObjectId },

    /// CR 616.1b / 614.12a — an entry replacement puts `object` under "an
    /// opponent of your choice" and there is more than one opponent to choose
    /// from. The options are players. With exactly one opponent nothing is
    /// asked (CR 102.2), and the choice is made before the permanent enters.
    ChooseEnteringController { object: ObjectId },

    /// CR 614.13/13a — an entry replacement is being applied and it moves other
    /// objects: devour's "you may sacrifice any number of creatures", Sutured
    /// Ghoul's "exile any number of creature cards from your graveyard". The
    /// options are the objects that may be chosen, already filtered by
    /// CR 614.13a/b and by CR 101.2.
    ///
    /// **The minimum is zero and that is the card's text, not a courtesy.**
    /// "Any number" includes none, so declining is a count rather than a
    /// separate optional-replacement prompt — which is why devour is not a
    /// `ReplacementDef::optional` and why a decline here does not spend
    /// CR 614.5's opportunity.
    ///
    /// `entering` is the permanent whose entry is being modified — it can never
    /// itself be an option (CR 614.13a), and it is not yet on the battlefield,
    /// so a UI reads it out of the object store rather than off the board.
    ///
    /// **`source` is not always `entering`, and a UI that assumed so would
    /// mislabel half the population.** Devour is the entering creature's own
    /// ability, so the two coincide; an effect that modifies *someone else's*
    /// entry — the plane in CR 614.13b's example, granting devour 5 — is a
    /// different object, and "why am I being asked this" is answered by the
    /// source and by nothing else on this prompt. It is the same field
    /// [`Self::ApplyOptionalReplacement`] carries, for the same reason.
    ///
    /// `to` is where the chosen objects go, which is what separates "sacrifice
    /// these" from "exile these" without the UI having to know the card.
    ChooseAuxiliaryZoneChange {
        entering: ObjectId,
        source: ObjectId,
        to: Zone,
    },

    // --- Copy effects (CR 707) ---
    /// CR 608.2d — a resolving copy effect must **choose** the permanent whose
    /// copiable values (CR 707.2) it captures. Cytoshape's "Choose a nonlegendary creature
    /// on the battlefield". The options are permanents.
    ///
    /// **Not `SelectRecipients`.** There the chosen object is what the effect
    /// acts on; here it is the exact opposite — the donor is the one permanent
    /// a copy effect does not change — so a heuristic keyed on
    /// `SelectRecipients` would read the donor as the victim.
    ///
    /// Asked only with two or more candidates. With one the choice is forced and
    /// nothing is asked, which is `ChooseEnteringController`'s CR 102.2 shape.
    ChooseCopySource { spell_id: ObjectId },

    // --- Commander (CR 903) ---
    /// CR 704.6d / 903.9a — a commander is in a graveyard or exile and its
    /// owner **may** put it into the command zone. A state-based action with a
    /// choice, not a replacement effect.
    CommanderToCommandZoneSba { commander: ObjectId },

    // --- CR 701 keyword actions ---
    /// CR 701.9b — which card or cards the affected player discards.
    ///
    /// **One kind for both producers.** CR 514.1's cleanup discard and a
    /// resolving effect's "discards two cards" are the same keyword action
    /// asked of the same player, and a UI that had to know which was which
    /// would be reading the engine's call sites rather than the rules.
    /// `source` is the difference and is the only one: `None` is the
    /// turn-based action, `Some` the spell or ability that caused it, which
    /// is the "why am I being asked this" field
    /// [`Self::ChooseAuxiliaryZoneChange`] and
    /// [`Self::ApplyOptionalReplacement`] already carry.
    ///
    /// Not asked at all for CR 701.9b's "at random" shape, which chooses from
    /// `GameState::rng`, nor when the count is the whole hand (CR 102.2 — a
    /// forced choice is not a choice).
    Discard { source: Option<ObjectId> },

    /// CR 701.22a — which of the cards looked at go on the **bottom** of the
    /// library. The options are those cards, top-most first; "any number of
    /// them" is the bounds, `(0, k)`.
    ///
    /// `n` is the instruction's number and `k` the cards actually there, and
    /// they differ when the library is short — CR 701.22d's "even if some or
    /// all of those actions were impossible". A UI wants the instruction's
    /// number on the prompt and the options are the truth.
    Scry { source: Option<ObjectId>, n: u64 },

    /// CR 701.22a's "in any order", asked once per group that has two or more
    /// cards in it — with one there is no order to choose (CR 102.2), which is
    /// every Scry 1 and so every prompt Opt makes.
    ///
    /// `bottom` says which group: the cards going to the bottom, or the ones
    /// staying on top. Two prompts and not two kinds, because it is one
    /// question about two piles and a UI renders it once.
    ScryOrder { source: Option<ObjectId>, bottom: bool },

    // --- State-Based & Cleanup ---
    /// CR 704.5j — which of two or more legendary permanents named
    /// `legend_name` under one controller stays; the rest go to their owners'
    /// graveyards. The options are those permanents, in battlefield order,
    /// **and they are the whole subject**: the rule singles out none of them,
    /// so [`Self::subject`] is `None` rather than one member the rule does not
    /// name. The name is the group's key, which is why it is here.
    LegendRule { legend_name: String },
}

/// What [`ChoiceKind::describe`] renders: the rule the question is asked
/// under, as the stable handle a client keys on, and one line of text.
///
/// Text and ids, never engine structure (`codebase-state.md` item 141). A
/// client that wants more than the line is asking an oracle question, not
/// reading a prompt field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptText {
    /// The CR rule — `"601.2c"`, `"704.5j"` — as the baseline numbers it,
    /// without a "CR" prefix. `tests/prompt_subject_test.rs` checks every one
    /// against `MTG-Rules/versions/tmnt.txt`.
    pub rule: &'static str,
    /// One line in the CR's words, naming objects by id.
    pub text: String,
}

impl ChoiceKind {
    /// The object this question is about — the spell or ability asking, or
    /// the permanent it concerns — when there is exactly one.
    ///
    /// **This method is the contract, not a field name.** The variants carry
    /// it as `spell_id`, `spell_or_ability_id`, `source`, `object`,
    /// `commander`, `attacker_id` or `affected_object`, and a client that
    /// looked for one of those names would miss the rest.
    ///
    /// `None` is a decision, never a default, and each says why:
    /// - `PriorityAction`, `DeclareAttackers`, `DeclareBlockers` — the turn
    ///   asks (CR 117.1, 508.1a, 509.1a); no object does.
    /// - `LegendRule` — CR 704.5j names no member of the group; the options
    ///   are the subject.
    /// - `Discard { source: None }` — CR 514.1's turn-based discard; one a
    ///   spell or ability causes carries it.
    /// - `ChooseReplacementEffect { affected_object: None }` — the event is
    ///   about the choosing player, not an object.
    /// - `Scry`/`ScryOrder { source: None }` — only a test builds one; every
    ///   scry a game asks comes from a resolving spell or ability.
    ///
    /// Where a prompt carries both an effect's source and the object it
    /// affects, the source is the subject: "why am I being asked" is answered
    /// by the effect offering itself (`ApplyOptionalReplacement`,
    /// `ChooseAuxiliaryZoneChange`).
    pub fn subject(&self) -> Option<ObjectId> {
        match self {
            ChoiceKind::PriorityAction => None,
            ChoiceKind::DeclareAttackers => None,
            ChoiceKind::DeclareBlockers => None,
            ChoiceKind::AssignCombatDamage { attacker_id } => Some(*attacker_id),
            ChoiceKind::AssignTrampleDamage { attacker_id, .. } => Some(*attacker_id),
            ChoiceKind::ChooseXValue { spell_id, .. } => Some(*spell_id),
            ChoiceKind::ChooseAlternativeCost { spell_id } => Some(*spell_id),
            ChoiceKind::ChooseAdditionalCosts { spell_id } => Some(*spell_id),
            ChoiceKind::SelectRecipients { spell_id, .. } => Some(*spell_id),
            ChoiceKind::GenericManaAllocation { spell_or_ability_id, .. } => {
                Some(*spell_or_ability_id)
            }
            ChoiceKind::OrderCostReductions { spell_id } => Some(*spell_id),
            ChoiceKind::ManaAbilityWindow { spell_or_ability_id, .. } => Some(*spell_or_ability_id),
            ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, .. } => {
                Some(*spell_or_ability_id)
            }
            ChoiceKind::ChooseReplacementEffect { affected_object } => *affected_object,
            ChoiceKind::ApplyOptionalReplacement { source, .. } => Some(*source),
            ChoiceKind::AllocateNextDamage { source, .. } => Some(*source),
            ChoiceKind::ChooseDamageSource { source } => Some(*source),
            ChoiceKind::ChooseEnteringController { object } => Some(*object),
            ChoiceKind::ChooseAuxiliaryZoneChange { source, .. } => Some(*source),
            ChoiceKind::ChooseCopySource { spell_id } => Some(*spell_id),
            ChoiceKind::CommanderToCommandZoneSba { commander } => Some(*commander),
            ChoiceKind::Discard { source } => *source,
            ChoiceKind::Scry { source, .. } => *source,
            ChoiceKind::ScryOrder { source, .. } => *source,
            ChoiceKind::LegendRule { .. } => None,
        }
    }

    /// One line a client can show, under the rule that asks it.
    ///
    /// Rendered by the engine so that every client shows the same question
    /// and none has to know the card: objects by id, the rest in the CR's
    /// words. The options are not repeated — they are the `ChoiceOption`s
    /// beside the prompt — and the bounds are the call's.
    pub fn describe(&self) -> PromptText {
        let (rule, text) = match self {
            ChoiceKind::PriorityAction => {
                ("117.1", "Take an action, or pass priority".to_string())
            }
            ChoiceKind::DeclareAttackers => (
                "508.1a",
                "Declare attackers, each with the player or permanent it attacks".to_string(),
            ),
            ChoiceKind::DeclareBlockers => (
                "509.1a",
                "Declare blockers, each with the attacking creature it blocks".to_string(),
            ),
            ChoiceKind::AssignCombatDamage { attacker_id } => (
                "510.1c",
                format!("Assign {attacker_id}'s combat damage among the creatures blocking it"),
            ),
            ChoiceKind::AssignTrampleDamage { attacker_id, defending_target } => (
                "702.19b",
                format!(
                    "Assign {attacker_id}'s combat damage among the creatures blocking it, the rest to {}",
                    damage_target(defending_target)
                ),
            ),
            ChoiceKind::ChooseXValue { spell_id, x_count } => (
                "601.2b",
                if *x_count > 1 {
                    format!("Choose the value of X for {spell_id}, which its cost names {x_count} times")
                } else {
                    format!("Choose the value of X for {spell_id}")
                },
            ),
            ChoiceKind::ChooseAlternativeCost { spell_id } => (
                "601.2b",
                format!("Choose whether to cast {spell_id} for its mana cost or for an alternative cost"),
            ),
            ChoiceKind::ChooseAdditionalCosts { spell_id } => (
                "601.2b",
                format!("Choose which optional additional costs of {spell_id} to pay"),
            ),
            ChoiceKind::SelectRecipients { recipient, spell_id } => match recipient {
                EffectRecipient::Target(_, count) => (
                    "601.2c",
                    format!("Choose {} for {spell_id}", how_many(count, "a target", "targets")),
                ),
                EffectRecipient::Choose(_, count) => (
                    "608.2d",
                    format!(
                        "Choose {} for {spell_id}; not a target",
                        how_many(count, "an object or player", "objects or players")
                    ),
                ),
                EffectRecipient::Implicit
                | EffectRecipient::Controller
                | EffectRecipient::SameInstanceAs(_)
                | EffectRecipient::FilteredPermanents(_)
                | EffectRecipient::FilteredObjectsIn(_, _)
                | EffectRecipient::Host => {
                    ("608.2d", format!("Choose for {spell_id}"))
                }
            },
            ChoiceKind::GenericManaAllocation { spell_or_ability_id, mana_cost } => (
                "601.2h",
                format!(
                    "Choose which mana in your pool pays the generic part of {mana_cost} for {spell_or_ability_id}"
                ),
            ),
            ChoiceKind::OrderCostReductions { spell_id } => (
                "601.2f",
                format!("Order the cost reductions that apply to {spell_id}; the first applies first"),
            ),
            ChoiceKind::ManaAbilityWindow { spell_or_ability_id, remaining_cost } => (
                "601.2g",
                format!(
                    "Activate a mana ability toward the {remaining_cost} still owed for {spell_or_ability_id}, or stop"
                ),
            ),
            ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, count } => (
                "601.2h",
                format!("Choose {count} permanent(s) to sacrifice to pay for {spell_or_ability_id}"),
            ),
            ChoiceKind::ChooseReplacementEffect { affected_object } => (
                "616.1",
                match affected_object {
                    Some(id) => format!(
                        "Choose which replacement or prevention effect applies to the event affecting {id}"
                    ),
                    None => "Choose which replacement or prevention effect applies to the event affecting you"
                        .to_string(),
                },
            ),
            ChoiceKind::ApplyOptionalReplacement { affected_object, source } => (
                "614.1a",
                match affected_object {
                    Some(id) => format!(
                        "Choose whether {source}'s replacement effect applies to the event affecting {id}"
                    ),
                    None => format!(
                        "Choose whether {source}'s replacement effect applies to the event affecting you"
                    ),
                },
            ),
            ChoiceKind::AllocateNextDamage { source, remaining } => (
                "615.7",
                format!(
                    "Choose which damage {source}'s effect prevents, of the {remaining} it can still prevent"
                ),
            ),
            ChoiceKind::ChooseDamageSource { source } => {
                ("609.7a", format!("Choose a source of damage for {source}'s effect"))
            }
            ChoiceKind::ChooseEnteringController { object } => (
                "614.12a",
                format!("Choose the opponent under whose control {object} enters the battlefield"),
            ),
            ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, to } => (
                "614.13a",
                format!(
                    "Choose any number of objects to put into {} as {source}'s effect modifies how {entering} enters the battlefield",
                    zone_name(to)
                ),
            ),
            ChoiceKind::ChooseCopySource { spell_id } => {
                ("608.2d", format!("Choose the permanent for {spell_id}'s effect to copy"))
            }
            ChoiceKind::CommanderToCommandZoneSba { commander } => {
                ("704.6d", format!("Choose whether {commander} goes to the command zone"))
            }
            ChoiceKind::Discard { source } => (
                "701.9b",
                match source {
                    Some(id) => format!("Choose the cards to discard for {id}"),
                    None => "Choose the cards to discard down to your maximum hand size".to_string(),
                },
            ),
            ChoiceKind::Scry { source, n } => (
                "701.22a",
                match source {
                    Some(id) => format!(
                        "Scry {n} for {id}: choose which of the cards looked at go on the bottom of your library"
                    ),
                    None => format!(
                        "Scry {n}: choose which of the cards looked at go on the bottom of your library"
                    ),
                },
            ),
            ChoiceKind::ScryOrder { source, bottom } => {
                let pile = if *bottom { "going to the bottom" } else { "staying on top" };
                (
                    "701.22a",
                    match source {
                        Some(id) => format!(
                            "Scry for {id}: order the cards {pile} of your library, top-most first"
                        ),
                        None => format!("Scry: order the cards {pile} of your library, top-most first"),
                    },
                )
            }
            ChoiceKind::LegendRule { legend_name } => (
                "704.5j",
                format!(
                    "Choose which permanent named {legend_name} to keep; the rest go to their owners' graveyards"
                ),
            ),
        };
        PromptText { rule, text }
    }
}

/// A count of targets in words — "a target", "two targets", "up to two
/// targets" — with `one` for exactly one and `many` as the plural noun.
fn how_many(count: &TargetCount, one: &str, many: &str) -> String {
    match count {
        TargetCount::Exactly(1) => one.to_string(),
        TargetCount::Exactly(n) => format!("{n} {many}"),
        TargetCount::UpTo(n) => format!("up to {n} {many}"),
    }
}

fn damage_target(target: &DamageTarget) -> String {
    match target {
        DamageTarget::Player(player) => format!("player {player}"),
        DamageTarget::Object(id) => id.to_string(),
    }
}

/// CR 400.1's zones, as a sentence says them.
fn zone_name(zone: &Zone) -> &'static str {
    match zone {
        Zone::Library => "the library",
        Zone::Hand => "the hand",
        Zone::Battlefield => "the battlefield",
        Zone::Graveyard => "the graveyard",
        Zone::Stack => "the stack",
        Zone::Exile => "exile",
        Zone::Command => "the command zone",
    }
}

/// Wrapper carrying the semantic kind. The display text is
/// [`ChoiceKind::describe`]'s, so that every client shows the same question;
/// a provider that wants its own wording matches on `kind`.
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub kind: ChoiceKind,
}

/// A single selectable option presented to the DP.
#[derive(Debug, Clone)]
pub enum ChoiceOption {
    /// A game object (creature, card in hand, permanent, etc.)
    Object(ObjectId),
    /// A player
    Player(PlayerId),
    /// A game action (for priority)
    Action(PriorityAction),
    /// An attacker-target pair (for declare attackers)
    AttackerTarget(ObjectId, AttackTarget),
    /// A blocker-attacker pair (for declare blockers)
    BlockerAttacker(ObjectId, ObjectId),
    /// Pay the normal mana cost (used in alternative cost selection)
    NormalCost,
    /// An alternative cost option
    AlternativeCost(AlternativeCost),
    /// An additional cost option
    AdditionalCost(AdditionalCost),
    /// A number (for X value ranges presented as discrete options)
    Number(u64),
    /// A color
    Color(Color),
    /// A counter type
    CounterType(CounterType),
    /// A mana type (for generic allocation)
    ManaType(ManaType),
}
