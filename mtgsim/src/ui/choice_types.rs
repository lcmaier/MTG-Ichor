use std::fmt;

use crate::engine::combat::validation::CombatError;
use crate::engine::replacement::{CounterEffectKind, GameRuleReplacement};
use crate::oracle::x_values::LegalXValues;
use crate::state::battlefield::AttackTarget;
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::effects::{CounterType, EffectRecipient};
use crate::types::ids::{AbilityId, ObjectId, ObjectOrPlayerId, ObjectRef, PlayerId};
use crate::state::game_state::GameState;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::zones::Zone;

use super::decision::PriorityAction;
use super::display::{ABILITY_IT_NO_LONGER_HAS, additional_cost_label, alternative_cost_label, color_name, printed_name};

/// What kind of decision is being made. UIs use this to render appropriate
/// screens. AI agents can match on this for specialized heuristics.
/// Adding a new variant here is the ONLY change needed when a new decision
/// type is introduced — no trait methods or impl changes.
///
/// Exhaustive matching is intentional: single-crate project, compiler flags
/// every match site when a variant is added. **One of those sites is the
/// variant's own contract**: [`Self::subject`] matches without a wildcard, so
/// a new variant decides at birth which object it is about (`backlog.md`
/// §2.21). What it *shows* is `ui::display::question`'s, which both clients
/// read and which is exhaustive for the same reason; no text crosses the
/// boundary itself.
///
/// **A payload names things by id and in the CR's vocabulary, never by an
/// engine AST** — `codebase-state.md` item 141. What a client needs is what
/// the engine already computed: the options are the legality, the subject is
/// which object is asking, and the variant with its fields is the question.
/// `SelectRecipients`' `EffectRecipient` predates the rule and is its own
/// piece of work.
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
    AssignTrampleDamage { attacker_id: ObjectId, defending_target: ObjectOrPlayerId },

    // --- Casting Pipeline (601.2) ---
    /// CR 601.2b — the value of X for `spell_id`. `values` is what the
    /// engine offers and why (`oracle::x_values`), how many times the cost
    /// pays X among it:
    /// asked by `pick_number` over an interval, by `pick_n` over
    /// `ChoiceOption::Number` when a rule cuts a gap, and not asked at all
    /// when there is one value. Boxed, since every other kind is smaller.
    ChooseXValue { spell_id: ObjectId, values: Box<LegalXValues> },
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
    /// CR 601.2d — how `source`, the spell or ability on the stack, divides
    /// an effect among its targets. The buckets are the targets, each
    /// receiving at least one, and the total is what the text divides; asked
    /// after every target is chosen, and not at all when one split is left.
    DivideAmong { source: ObjectId },
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
    /// asked, as with [`Self::ChoosePlayerAsObjectEnters`].
    ChooseSacrificeForCost { spell_or_ability_id: ObjectId, count: u32 },

    // --- Replacement effects (CR 616.1) ---
    /// Two or more replacement or prevention effects want the same event and
    /// the affected object's controller (or the affected player) must choose
    /// one to apply.
    ///
    /// **Only asked with two or more candidates** (CR 616.1): there is no
    /// choice with one (`replacement-architecture.md` §4.1).
    ///
    /// `affected_object` is `None` when the event is about the choosing player
    /// rather than about an object.
    ChooseReplacementEffect { affected_object: Option<ObjectId> },

    // --- Triggered abilities (CR 603.3b) ---
    /// The order `player` puts their simultaneously triggered abilities of
    /// one tier onto the stack. The options are the entries' sources in
    /// trigger order, and the answer is a permutation; the first goes on
    /// lowest. **Asked only with two or more**, and not even then when no
    /// order can change the game — the elision `triggers-architecture.md`
    /// §5.2 gives the conditions of, in `engine::triggers::placement`.
    OrderTriggers { player: PlayerId, tier: crate::types::triggers::TriggerTier },

    /// A "you **may** ... instead" replacement effect is offering itself
    /// (CR 614.1a). Declining is CR 614.5's one opportunity taken.
    ApplyOptionalReplacement { affected_object: Option<ObjectId>, source: ObjectId },

    /// A resolving spell or ability's "[chooser] **may** [effect]" (CR 603.5):
    /// yes or no, as a `pick_n` of the one option or none. The ability went on
    /// the stack regardless; the choice is made now. `source` is the resolving
    /// object.
    ApplyOptionalEffect { source: ObjectId },

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
    /// and nothing is asked, as with
    /// [`Self::ChooseCopySource`].
    ///
    /// `source` is the object whose effect is choosing — [`Self::
    /// AllocateNextDamage`]'s field, for the same reason: "why am I being
    /// asked this" is answered by it and by nothing else on the prompt.
    ChooseDamageSource { source: ObjectId },

    /// CR 603.7b — a delayed triggered ability with no stated duration whose
    /// event happened more than once at the same time: its controller
    /// chooses which event causes it. The options are the events, each by
    /// its object, or by its record's number where objects do not tell them
    /// apart. Asked only between two or more events that give different
    /// games. `source` is the delayed trigger's (CR 603.7d–g).
    ChooseDelayedTriggerEvent { source: ObjectId },

    /// CR 608.2d — `source`'s resolving instruction names "an opponent" it
    /// did not target, and its controller chooses which as it applies the
    /// instruction (Akroan Horse's ruling: "in a multiplayer game, you choose
    /// the opponent as the ability resolves"). The options are the opponents
    /// still in the game; with one, nothing is asked.
    ChooseOpponent { source: ObjectId },

    /// CR 614.12a — an entry replacement asks for a player as `object`
    /// enters, before it does, and `role` says what the player will be: the
    /// opponent it enters under the control of (Xantcha, CR 616.1b), or the
    /// opponent who may put its counters on (tribute, CR 702.104a). The
    /// options are the opponents still in the game; with one, nothing is
    /// asked. `source` is the object whose effect asks, the entering one for
    /// its own ability.
    ChoosePlayerAsObjectEnters { object: ObjectId, source: ObjectId, role: EntryRole },

    /// CR 508.4 — `object` is put onto the battlefield attacking, and its
    /// controller chooses what it attacks as it enters, among two or more of
    /// what can be attacked. The options are players, and with `backlog.md`
    /// §2.23 the planeswalkers and battles among them. With one candidate
    /// nothing is asked.
    ChooseEnteringDefender { object: ObjectId },

    /// CR 614.12a — an entry replacement offers the asked player something
    /// they may do as `object` enters, yes or no. `offer` says what: a
    /// shockland's "you may pay 2 life", offered only when the payment is
    /// payable beside every cost the event's entries chose already (CR
    /// 614.12b). `source` is the object whose effect offers it, the entering
    /// one for its own ability.
    AcceptAsObjectEnters { object: ObjectId, source: ObjectId, offer: EntryOffer },

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
    /// A copy effect **chooses** the permanent whose copiable values (CR 707.2)
    /// it captures: Cytoshape's "Choose a nonlegendary creature on the
    /// battlefield" as it resolves (CR 608.2d), or Clone's "any creature on the
    /// battlefield" as it enters (CR 707.6). The options are permanents.
    ///
    /// **Not `SelectRecipients`.** There the chosen object is what the effect
    /// acts on; here it is the exact opposite — the donor is the one permanent
    /// a copy effect does not change — so a heuristic keyed on
    /// `SelectRecipients` would read the donor as the victim.
    ///
    /// `source` is the object whose copy effect is choosing: the resolving
    /// spell or ability, or the entering permanent whose own ability it is.
    /// A "you may" is asked here, as a pick of none; otherwise the choice is
    /// asked only with two or more candidates, since one is forced.
    ChooseCopySource { source: ObjectId },

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
    /// `GameState::rng`, nor when the count is the whole hand, which is
    /// answered unasked.
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
    /// cards in it — with one there is no order to choose, which is
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

/// What [`ChoiceKind::AcceptAsObjectEnters`] offers, so a client can tell a cost
/// from a gift without reading the card.
#[derive(Debug, Clone, PartialEq)]
pub enum EntryOffer {
    /// CR 614.12's "you may pay [cost]".
    Pay(Cost),
    /// "That player may put N counters on it as it enters", the putter's own
    /// choice (tribute, CR 702.104a).
    PutCounters { counter: CounterType, n: u32 },
}

/// What the player [`ChoiceKind::ChoosePlayerAsObjectEnters`] asks for will be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryRole {
    /// The opponent the permanent enters under the control of (CR 616.1b).
    Controller,
    /// The opponent who may put its counters on (CR 702.104a, 122.6a).
    CounterPutter,
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
            ChoiceKind::DivideAmong { source } => Some(*source),
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
            ChoiceKind::ApplyOptionalEffect { source } => Some(*source),
            ChoiceKind::AllocateNextDamage { source, .. } => Some(*source),
            ChoiceKind::ChooseDamageSource { source } => Some(*source),
            ChoiceKind::ChooseDelayedTriggerEvent { source } => Some(*source),
            ChoiceKind::ChooseOpponent { source } => Some(*source),
            ChoiceKind::ChoosePlayerAsObjectEnters { source, .. } => Some(*source),
            ChoiceKind::ChooseEnteringDefender { object } => Some(*object),
            ChoiceKind::AcceptAsObjectEnters { source, .. } => Some(*source),
            ChoiceKind::ChooseAuxiliaryZoneChange { source, .. } => Some(*source),
            ChoiceKind::ChooseCopySource { source } => Some(*source),
            ChoiceKind::CommanderToCommandZoneSba { commander } => Some(*commander),
            ChoiceKind::Discard { source } => *source,
            ChoiceKind::Scry { source, .. } => *source,
            ChoiceKind::ScryOrder { source, .. } => *source,
            ChoiceKind::LegendRule { .. } => None,
            // Several objects at once, like `DeclareAttackers`: no one subject.
            ChoiceKind::OrderTriggers { .. } => None,
        }
    }

    /// The variant's name alone, `SelectRecipients` and not its fields: how
    /// the decision log, the trace and a setup line's refusal spell a kind,
    /// and what a replay compares (`setup-architecture.md` §7.2). Not `name`,
    /// which is a card's (CR 201).
    pub fn as_str(&self) -> &'static str {
        match self {
            ChoiceKind::PriorityAction => "PriorityAction",
            ChoiceKind::DeclareAttackers => "DeclareAttackers",
            ChoiceKind::DeclareBlockers => "DeclareBlockers",
            ChoiceKind::AssignCombatDamage { .. } => "AssignCombatDamage",
            ChoiceKind::AssignTrampleDamage { .. } => "AssignTrampleDamage",
            ChoiceKind::ChooseXValue { .. } => "ChooseXValue",
            ChoiceKind::ChooseAlternativeCost { .. } => "ChooseAlternativeCost",
            ChoiceKind::ChooseAdditionalCosts { .. } => "ChooseAdditionalCosts",
            ChoiceKind::SelectRecipients { .. } => "SelectRecipients",
            ChoiceKind::DivideAmong { .. } => "DivideAmong",
            ChoiceKind::GenericManaAllocation { .. } => "GenericManaAllocation",
            ChoiceKind::OrderCostReductions { .. } => "OrderCostReductions",
            ChoiceKind::ManaAbilityWindow { .. } => "ManaAbilityWindow",
            ChoiceKind::ChooseSacrificeForCost { .. } => "ChooseSacrificeForCost",
            ChoiceKind::ChooseReplacementEffect { .. } => "ChooseReplacementEffect",
            ChoiceKind::ApplyOptionalReplacement { .. } => "ApplyOptionalReplacement",
            ChoiceKind::ApplyOptionalEffect { .. } => "ApplyOptionalEffect",
            ChoiceKind::AllocateNextDamage { .. } => "AllocateNextDamage",
            ChoiceKind::ChooseDamageSource { .. } => "ChooseDamageSource",
            ChoiceKind::ChooseDelayedTriggerEvent { .. } => "ChooseDelayedTriggerEvent",
            ChoiceKind::ChooseOpponent { .. } => "ChooseOpponent",
            ChoiceKind::ChoosePlayerAsObjectEnters { .. } => "ChoosePlayerAsObjectEnters",
            ChoiceKind::ChooseEnteringDefender { .. } => "ChooseEnteringDefender",
            ChoiceKind::AcceptAsObjectEnters { .. } => "AcceptAsObjectEnters",
            ChoiceKind::ChooseAuxiliaryZoneChange { .. } => "ChooseAuxiliaryZoneChange",
            ChoiceKind::ChooseCopySource { .. } => "ChooseCopySource",
            ChoiceKind::CommanderToCommandZoneSba { .. } => "CommanderToCommandZoneSba",
            ChoiceKind::Discard { .. } => "Discard",
            ChoiceKind::Scry { .. } => "Scry",
            ChoiceKind::ScryOrder { .. } => "ScryOrder",
            ChoiceKind::LegendRule { .. } => "LegendRule",
            ChoiceKind::OrderTriggers { .. } => "OrderTriggers",
        }
    }
}

/// The question, and the seat's last answer to it when the engine rejected
/// that answer and is asking again. No display text: a client words the
/// question with `ui::display::question`, each option with
/// `ui::display::option_label` and a rejection with `ui::display::rejection`,
/// which keeps presentation off the boundary.
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub kind: ChoiceKind,
    /// `None` the first time a question is asked. Only the last answer
    /// rejected: a seat that wants every rejection of a priority window keeps
    /// them itself, as the random agent does (`codebase-state.md` item 193).
    pub rejected: Option<Rejection>,
}

impl ChoiceContext {
    /// A question asked for the first time.
    pub const fn new(kind: ChoiceKind) -> ChoiceContext {
        ChoiceContext { kind, rejected: None }
    }
}

/// An answer the engine would not take, and the rule it broke, by id: never
/// the engine's own error text, which stays in the trace's
/// `priority_rejected` record (item 141's payload rule).
#[derive(Debug, Clone, PartialEq)]
pub enum Rejection {
    /// CR 732.1: the action could not be completed, so it was reversed and its
    /// payments canceled. The player keeps priority and may take it again or
    /// another action (CR 732.2). Most often a cost the player stopped paying,
    /// or could not pay, in CR 601.2g's window.
    Reversed(PriorityAction),
    /// The attacks declared were illegal, so the active player declares
    /// again; `why` names the creature and the rule (CR 802.3).
    IllegalAttacks { attacks: Vec<(ObjectId, AttackTarget)>, why: CombatError },
    /// The blocks declared were illegal, so the defending player declares
    /// again; `why` names the creatures and the rule (CR 509.1a–c).
    IllegalBlocks { blocks: Vec<(ObjectId, ObjectId)>, why: CombatError },
}

/// A single selectable option presented to the DP.
#[derive(Debug, Clone, PartialEq)]
pub enum ChoiceOption {
    /// A game object (creature, card in hand, permanent, etc.)
    Object(ObjectId),
    /// An object by its last known information: the existence that left a
    /// zone (CR 400.7, 608.2h), which `Object`'s id would name the new
    /// object for. CR 609.7a's source of damage "no longer in the zone it
    /// used to be in".
    LastKnown(ObjectRef),
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
    /// A triggered ability waiting to be put on the stack (CR 603.3b), named
    /// by its source, its ability and what triggered it, so two triggers of
    /// one card are two options a log and a client tell apart
    /// (`setup-architecture.md#7e-pausing-at-a-decision`).
    Trigger(TriggerOption),
    /// A replacement or prevention effect CR 616.1 offers, named by where it
    /// comes from.
    Replacement(ReplacementOption),
}

/// One of the triggers CR 603.3b's question asks a player to order, named by
/// what it is in the game: its source, its ability and what triggered it.
/// Never by a number the engine keeps for itself, such as a registry id or a
/// place in a queue, so a replay on another build finds the same trigger.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerOption {
    /// Its source, and which existence of it (CR 400.7).
    pub source: ObjectRef,
    /// Which of the source's triggered abilities.
    pub ability: TriggeredAbility,
    /// The objects its triggering events are about (CR 603.2c), in event
    /// order. By id: a dies trigger's event is about the card's new object in
    /// the graveyard (CR 603.6c), which no script written before the event
    /// can name.
    pub triggered_by: Vec<ObjectId>,
}

/// Which triggered ability a trigger is.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggeredAbility {
    /// An ability its source has: printed on it, or granted or copied onto it
    /// by an effect of `granted_by`'s. Named by its definition
    /// (`AbilityId::definition`, the same whichever grant made the instance).
    OfItsSource { ability: AbilityId, granted_by: Option<ObjectId>, read_as: AbilityAsRead },
    /// A delayed triggered ability (CR 603.7), which an effect created and
    /// which is no ability of its source, named by the objects it refers to
    /// (CR 603.7c): Flickerwisp's "return the exiled card".
    Delayed { refers_to: Vec<ObjectId> },
}

/// An ability as the engine read it, on the board it checked the ability
/// against: its place among its object's abilities, 1 for the first, as a
/// scenario's `ability N` names it, and its rules text; `None` for one the
/// object no longer has. An entering object has its abilities only on that
/// board (CR 614.12), so the option carries what the log and a client write
/// rather than either reading the live one. It is how the ability reads, not
/// which ability it is, so every reading compares equal: a script names an
/// ability by its definition and leaves this `None`.
#[derive(Debug, Clone, Copy)]
pub struct AbilityAsRead(pub Option<(usize, &'static str)>);

impl AbilityAsRead {
    /// The ability's rules text, which a client draws.
    pub fn text(self) -> Option<&'static str> {
        self.0.map(|(_, text)| text)
    }
}

impl PartialEq for AbilityAsRead {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// The log's words for the ability.
impl fmt::Display for AbilityAsRead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some((place, _)) => write!(f, "ability {place}"),
            None => f.write_str(ABILITY_IT_NO_LONGER_HAS),
        }
    }
}

/// One of the effects CR 616.1's question asks a player to choose among,
/// named by where it comes from in the game, never by a number the engine
/// keeps for itself. A replacement or prevention effect comes from one of
/// four places, one arm each.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplacementOption {
    /// One a resolving spell or ability created (CR 614.3, 615.7): Words of
    /// Worship's "the next time you would draw a card this turn, you gain 5
    /// life instead", a "prevent the next 3 damage" shield. Named by the card
    /// it came from and `resolution`, the stack object that was resolving
    /// (`ResolutionStamp::source`), which tells two activations apart.
    FromResolution { source: ObjectId, resolution: ObjectId },
    /// A static ability of a permanent that is itself a replacement effect
    /// (CR 614.1a): Rest in Peace's "exile it instead", devour as its creature
    /// enters. Named by the object, the ability by its definition (with
    /// `granted_by` when an effect gave it), and `member`, which of its
    /// effects when one ability makes several, counted from 0.
    Ability { object: ObjectRef, ability: AbilityId, granted_by: Option<ObjectId>, member: u8, read_as: AbilityAsRead },
    /// Game counters of one kind on a permanent, which make a replacement or
    /// prevention effect (CR 122.1c/d/h): a finality counter's "exile it
    /// instead", and a shield counter's two effects, which `effect` tells
    /// apart.
    Counter { object: ObjectRef, counter: CounterType, effect: CounterEffectKind },
    /// A rule that acts as a replacement effect (CR 903.9b): a commander put
    /// into the command zone instead.
    Rule { object: ObjectRef, rule: GameRuleReplacement },
}

impl ChoiceOption {
    /// The option as a decision log writes it: what it is, never where it sat
    /// in the list, so that a build listing the options in another order still
    /// finds it (`setup-architecture.md` §7.2, decision 6). An object is its
    /// printed name and id, read off the card and never through the layers,
    /// since the log is an observer and a layer read counts a walk; a cost is
    /// its keyword and mana as CR 702.33a prints one. A change to these words
    /// is a change to the log's format.
    pub fn as_logged(&self, game: &GameState) -> String {
        let object = |id: &ObjectId| format!("{} ({id})", printed_name(game, *id));
        let objects = |ids: &[ObjectId]| ids.iter().map(object).collect::<Vec<_>>().join(" and ");
        let existence = |source: &ObjectRef| match game.object_ref(source.id) == Some(*source) {
            true => object(&source.id),
            false => format!("{} as it last existed", object(&source.id)),
        };
        let granted = |granted_by: &Option<ObjectId>| granted_by.map_or(String::new(), |by| format!(", granted by {}", object(&by)));
        match self {
            ChoiceOption::Object(id) => object(id),
            ChoiceOption::LastKnown(existence) => format!("{} as it last existed", object(&existence.id)),
            ChoiceOption::Player(player) => format!("player {player}"),
            ChoiceOption::Action(PriorityAction::Pass) => "pass".to_string(),
            ChoiceOption::Action(PriorityAction::CastSpell(id)) => format!("cast {}", object(id)),
            ChoiceOption::Action(PriorityAction::PlayLand(id)) => format!("play {}", object(id)),
            // Offered at priority, so the board it is logged on is the one it
            // was offered on; a trigger's and a replacement's options carry
            // the reading their builder made.
            ChoiceOption::Action(PriorityAction::ActivateAbility(id, ability)) => {
                format!("activate {} {}", object(id), ability_as_read(game, *id, *ability))
            }
            ChoiceOption::AttackerTarget(attacker, target) => {
                let attacked = match target {
                    AttackTarget::Player(player) => format!("player {player}"),
                    AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => object(id),
                };
                format!("{} attacks {attacked}", object(attacker))
            }
            ChoiceOption::BlockerAttacker(blocker, attacker) => format!("{} blocks {}", object(blocker), object(attacker)),
            ChoiceOption::NormalCost => "mana cost".to_string(),
            ChoiceOption::AlternativeCost(cost) => format!("alternative {}", alternative_cost_label(cost)),
            ChoiceOption::AdditionalCost(cost) => format!("additional {}", additional_cost_label(cost)),
            ChoiceOption::Number(number) => format!("number {number}"),
            ChoiceOption::Color(color) => format!("color {}", color_name(*color).to_lowercase()),
            ChoiceOption::CounterType(counter) => format!("counter {}", counter.name()),
            ChoiceOption::ManaType(mana) => format!("mana {}", ManaSymbol::Colored(*mana)),
            ChoiceOption::Trigger(TriggerOption { source, ability, triggered_by }) => {
                let which = match ability {
                    TriggeredAbility::OfItsSource { granted_by, read_as, .. } => format!("{read_as}{}", granted(granted_by)),
                    TriggeredAbility::Delayed { refers_to } if refers_to.is_empty() => "its delayed ability".to_string(),
                    TriggeredAbility::Delayed { refers_to } => format!("its delayed ability about {}", objects(refers_to)),
                };
                let about = match triggered_by.as_slice() {
                    [] => String::new(),
                    some => format!(", for {}", objects(some)),
                };
                format!("{}, {which}{about}", existence(source))
            }
            ChoiceOption::Replacement(ReplacementOption::FromResolution { source, resolution }) => {
                format!("{}, made as {resolution} resolved", object(source))
            }
            ChoiceOption::Replacement(ReplacementOption::Ability { object: of, granted_by, member, read_as, .. }) => {
                let member = if *member == 0 { String::new() } else { format!(", its effect {}", member + 1) };
                format!("{}, {read_as}{}{member}", existence(of), granted(granted_by))
            }
            ChoiceOption::Replacement(ReplacementOption::Counter { object: on, counter, effect }) => {
                let which = match effect {
                    CounterEffectKind::Replacement => "",
                    CounterEffectKind::Prevention => ", its prevention",
                };
                format!("{}, {} counter{which}", existence(on), counter.name())
            }
            ChoiceOption::Replacement(ReplacementOption::Rule { object: about, rule: GameRuleReplacement::CommanderZone }) => {
                format!("{}, the commander rule", existence(about))
            }
        }
    }
}

impl GameState {
    /// A pending trigger as CR 603.3b's question names it: its source as the
    /// existence that triggered, its ability, and what the events that
    /// triggered it are about.
    pub(crate) fn trigger_option(&self, trigger: &crate::types::triggers::PendingTrigger) -> TriggerOption {
        use crate::types::triggers::TriggerOrigin;
        let identity = trigger.origin.identity();
        let ability = match trigger.origin {
            TriggerOrigin::Object(_) => TriggeredAbility::OfItsSource {
                ability: identity.ability.definition(),
                granted_by: self.granted_by(identity.ability),
                read_as: ability_as_read(self, identity.source.id, identity.ability),
            },
            TriggerOrigin::Delayed(_) => TriggeredAbility::Delayed {
                refers_to: trigger.binding.referred.objects.iter().map(|remembered| remembered.object.id).collect(),
            },
            TriggerOrigin::Rule(inherent) => match inherent {},
        };
        let mut triggered_by: Vec<ObjectId> = Vec::new();
        for (_, object) in trigger.binding.triggering_events.iter().flat_map(|event| event.objects.objects()) {
            if !triggered_by.contains(&object.id) {
                triggered_by.push(object.id);
            }
        }
        TriggerOption { source: identity.source, ability, triggered_by }
    }

    /// A replacement effect CR 616.1 offers, as its question names it: by
    /// where it comes from, a resolution's by its source and the resolution
    /// that made it. `frame` is the board the effect was checked against,
    /// which an entering object's abilities are read off (CR 614.12).
    pub(crate) fn replacement_option(
        &self,
        instance: &crate::engine::replacement::ReplacementInstance,
        frame: &crate::engine::replacement::EntryFrame<'_>,
    ) -> ReplacementOption {
        use crate::engine::replacement::ReplacementInstanceId;
        match instance.id {
            // An instance is gathered off a live row, so the row is there.
            ReplacementInstanceId::Registered(row) => ReplacementOption::FromResolution {
                source: instance.source,
                resolution: self
                    .replacement_effects
                    .iter()
                    .find(|registered| registered.id == row)
                    .map_or(instance.source, |registered| registered.resolution.source),
            },
            ReplacementInstanceId::StaticAbility { object, ability, member } => ReplacementOption::Ability {
                object,
                ability: ability.definition(),
                granted_by: self.granted_by(ability),
                member,
                read_as: match frame.frame_of(object.id) {
                    Some(entering) => read_in(&entering.abilities, ability),
                    None => ability_as_read(self, object.id, ability),
                },
            },
            ReplacementInstanceId::Counter(object, counter, effect) => ReplacementOption::Counter { object, counter, effect },
            ReplacementInstanceId::GameRule(object, rule) => ReplacementOption::Rule { object, rule },
        }
    }

    /// The object whose effect granted or copied `ability` onto the object
    /// that has it: the source of the row its instance names, while the row
    /// lasts.
    pub(crate) fn granted_by(&self, ability: AbilityId) -> Option<ObjectId> {
        let row = ability.granting_row()?;
        self.continuous_effects.iter().find(|effect| effect.id == row).map(|effect| effect.source)
    }
}

/// `ability` among `id`'s abilities as the layers give them. Read as an
/// observer, so a game logged and one not count alike.
pub(crate) fn ability_as_read(game: &GameState, id: ObjectId, ability: AbilityId) -> AbilityAsRead {
    game.unobserved(|| read_in(&crate::oracle::characteristics::get_effective_abilities(game, id), ability))
}

/// `ability` as `abilities` has it.
fn read_in(abilities: &[crate::objects::card_data::AbilityDef], ability: AbilityId) -> AbilityAsRead {
    let at = crate::objects::card_data::printed_place(abilities, ability);
    AbilityAsRead(at.map(|index| (index + 1, abilities[index].rules_text.words)))
}

/// Where the option a replayed answer writes as `chosen` sits in `options`:
/// the first not in `taken` whose words are those. Every option writes what
/// it is, so two whose words are the same are one choice twice, and an
/// answer naming those words twice takes both.
pub fn position_of(options: &[ChoiceOption], game: &GameState, chosen: &str, taken: &[usize]) -> Option<usize> {
    (0..options.len()).find(|&i| !taken.contains(&i) && options[i].as_logged(game) == chosen)
}

/// Where in `options` the option a script names sits, past the positions
/// `taken`, wherever the engine lists it; `None` when it is not offered. Two
/// equal options offered are one choice twice, taken in turn. A name that
/// fits two different options is refused, since it would mean whichever the
/// engine lists first: a source with two effects offered needs the effect
/// named.
pub fn position_named(options: &[ChoiceOption], game: &GameState, named: &ChoiceOption, taken: &[usize]) -> Option<usize> {
    let fits = named_by_script(options, named);
    if fits.iter().any(|&i| options[i] != options[fits[0]]) {
        panic!(
            "{} names {} different options offered, so name the one meant: {:?}",
            named.as_logged(game),
            fits.len(),
            options.iter().map(|o| o.as_logged(game)).collect::<Vec<_>>(),
        );
    }
    fits.into_iter().find(|i| !taken.contains(i))
}

/// The places in `options`, in the list's order, of what a script's or a
/// setup line's `named` names: the options equal to it, or, for an object
/// no option equals, each trigger and replacement of that object offered.
fn named_by_script(options: &[ChoiceOption], named: &ChoiceOption) -> Vec<usize> {
    let equal: Vec<usize> = (0..options.len()).filter(|&i| options[i] == *named).collect();
    match named {
        ChoiceOption::Object(id) if equal.is_empty() => {
            (0..options.len()).filter(|&i| options[i].effect_source() == Some(*id)).collect()
        }
        _ => equal,
    }
}

impl ChoiceOption {
    /// The object an effect's option comes from: a trigger's source, a
    /// replacement ability's or counter's object, a row's source.
    pub fn effect_source(&self) -> Option<ObjectId> {
        match self {
            ChoiceOption::Trigger(trigger) => Some(trigger.source.id),
            ChoiceOption::Replacement(ReplacementOption::FromResolution { source, .. }) => Some(*source),
            ChoiceOption::Replacement(
                ReplacementOption::Ability { object, .. }
                | ReplacementOption::Counter { object, .. }
                | ReplacementOption::Rule { object, .. },
            ) => Some(object.id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::registry::CardRegistry;
    use crate::engine::priority::PriorityResult;
    use crate::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game};
    use crate::types::costs::Cost;
    use crate::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

    fn card(name: &str) -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardRegistry::default_registry().create(name).unwrap()
    }

    /// The log's words for each shape of option, pinned: a change here is a
    /// change to the log's format.
    #[test]
    fn an_option_is_logged_by_what_it_is() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, card("Grizzly Bears"), 0);
        let wall = put_on_battlefield(&mut game, card("Wall of Stone"), 1);
        let land = put_on_battlefield(&mut game, card("Forest"), 0);
        let bolt = put_in_hand(&mut game, card("Lightning Bolt"), 0);
        let forest = put_in_hand(&mut game, card("Forest"), 0);
        let warden = put_on_battlefield(&mut game, card("Soul Warden"), 0);
        let mana = game.objects[&land].card_data.abilities[0].id;
        let gains = game.objects[&warden].card_data.abilities[0].id;
        let red = vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 1))];
        let existence = |id: ObjectId| game.object_ref(id).unwrap();
        let logged = |option: ChoiceOption| option.as_logged(&game);

        assert_eq!(logged(ChoiceOption::Object(bears)), format!("Grizzly Bears ({bears})"));
        assert_eq!(logged(ChoiceOption::Player(1)), "player 1");
        assert_eq!(logged(ChoiceOption::Action(PriorityAction::Pass)), "pass");
        assert_eq!(logged(ChoiceOption::Action(PriorityAction::CastSpell(bolt))), format!("cast Lightning Bolt ({bolt})"));
        assert_eq!(logged(ChoiceOption::Action(PriorityAction::PlayLand(forest))), format!("play Forest ({forest})"));
        assert_eq!(
            logged(ChoiceOption::Action(PriorityAction::ActivateAbility(land, mana))),
            format!("activate Forest ({land}) ability 1")
        );
        assert_eq!(
            logged(ChoiceOption::AttackerTarget(bears, AttackTarget::Player(1))),
            format!("Grizzly Bears ({bears}) attacks player 1")
        );
        assert_eq!(
            logged(ChoiceOption::BlockerAttacker(wall, bears)),
            format!("Wall of Stone ({wall}) blocks Grizzly Bears ({bears})")
        );
        assert_eq!(logged(ChoiceOption::NormalCost), "mana cost");
        assert_eq!(logged(ChoiceOption::AlternativeCost(AlternativeCost::Flashback(red.clone()))), "alternative Flashback {1}{R}");
        assert_eq!(logged(ChoiceOption::AdditionalCost(AdditionalCost::Kicker(red))), "additional Kicker {1}{R}");
        assert_eq!(logged(ChoiceOption::Number(3)), "number 3");
        assert_eq!(logged(ChoiceOption::Color(Color::Red)), "color red");
        assert_eq!(logged(ChoiceOption::CounterType(CounterType::PlusOnePlusOne)), "counter +1/+1");
        assert_eq!(logged(ChoiceOption::ManaType(ManaType::Green)), "mana {G}");
        let triggered = |ability| TriggerOption { source: existence(warden), ability, triggered_by: vec![bears] };
        assert_eq!(
            logged(ChoiceOption::Trigger(triggered(TriggeredAbility::OfItsSource { ability: gains, granted_by: None, read_as: AbilityAsRead(Some((1, ""))) }))),
            format!("Soul Warden ({warden}), ability 1, for Grizzly Bears ({bears})")
        );
        assert_eq!(
            logged(ChoiceOption::Trigger(triggered(TriggeredAbility::OfItsSource { ability: gains, granted_by: Some(wall), read_as: AbilityAsRead(Some((1, ""))) }))),
            format!("Soul Warden ({warden}), ability 1, granted by Wall of Stone ({wall}), for Grizzly Bears ({bears})")
        );
        assert_eq!(
            logged(ChoiceOption::Trigger(TriggerOption {
                source: existence(warden),
                ability: TriggeredAbility::Delayed { refers_to: vec![bears] },
                triggered_by: Vec::new(),
            })),
            format!("Soul Warden ({warden}), its delayed ability about Grizzly Bears ({bears})")
        );
        assert_eq!(
            logged(ChoiceOption::Replacement(ReplacementOption::FromResolution { source: wall, resolution: bolt })),
            format!("Wall of Stone ({wall}), made as {bolt} resolved")
        );
        assert_eq!(
            logged(ChoiceOption::Replacement(ReplacementOption::Ability {
                object: existence(warden),
                ability: gains,
                granted_by: None,
                member: 1,
                read_as: AbilityAsRead(Some((1, ""))),
            })),
            format!("Soul Warden ({warden}), ability 1, its effect 2")
        );
        assert_eq!(
            logged(ChoiceOption::Replacement(ReplacementOption::Counter {
                object: existence(bears),
                counter: CounterType::Finality,
                effect: CounterEffectKind::Replacement,
            })),
            format!("Grizzly Bears ({bears}), finality counter")
        );
        assert_eq!(
            logged(ChoiceOption::Replacement(ReplacementOption::Counter {
                object: existence(bears),
                counter: CounterType::Shield,
                effect: CounterEffectKind::Prevention,
            })),
            format!("Grizzly Bears ({bears}), shield counter, its prevention")
        );
    }

    /// An answer names what it chose, so a list in another order, or a
    /// longer one, still finds it; and of two options with the same words, it
    /// says which.
    #[test]
    fn an_answer_is_found_wherever_another_build_lists_it() {
        let mut game = setup_two_player_game();
        let bolt = put_in_hand(&mut game, card("Lightning Bolt"), 0);
        let forest = put_in_hand(&mut game, card("Forest"), 0);
        let (pass, cast, play) = (
            ChoiceOption::Action(PriorityAction::Pass),
            ChoiceOption::Action(PriorityAction::CastSpell(bolt)),
            ChoiceOption::Action(PriorityAction::PlayLand(forest)),
        );
        let chosen = cast.as_logged(&game);
        let reordered = [play, cast, pass];
        assert_eq!(position_of(&reordered, &game, &chosen, &[]), Some(1));
        assert_eq!(position_of(&reordered, &game, &chosen, &[1]), None, "a taken option is passed over");
        assert_eq!(position_of(&reordered[..1], &game, &chosen, &[]), None, "not offered");

        // Two options whose words are the same are one choice twice: an
        // answer naming them twice takes both, whichever first.
        let twins = [ChoiceOption::Number(2), ChoiceOption::Number(2)];
        assert_eq!(position_of(&twins, &game, "number 2", &[]), Some(0));
        assert_eq!(position_of(&twins, &game, "number 2", &[0]), Some(1));
    }

    /// A test's script can name the option it picks instead of its position.
    #[test]
    fn a_script_picks_an_option_by_what_it_is() {
        let mut game = setup_two_player_game();
        let forest = put_in_hand(&mut game, card("Forest"), 0);
        let seats = ScriptedDecisionProvider::new();
        seats.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::PlayLand(forest))]);
        assert_eq!(game.run_priority_round(&seats), Ok(PriorityResult::ActionTaken));
        assert!(game.battlefield.contains_key(&forest), "the Forest was played");
    }

    /// An allocation names each bucket with its amount, and an order names
    /// its items, so either is found wherever the engine lists them.
    #[test]
    fn a_script_allocates_and_orders_by_what_each_option_is() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, card("Grizzly Bears"), 1);
        let wall = put_on_battlefield(&mut game, card("Wall of Stone"), 1);
        let listed = [ChoiceOption::Object(wall), ChoiceOption::Object(bears)];
        let seats = ScriptedDecisionProvider::new();
        let damage = ChoiceKind::AssignCombatDamage { attacker_id: bears };
        seats.expect_allocation(damage.clone(), vec![(ChoiceOption::Object(bears), 2), (ChoiceOption::Object(wall), 1)]);
        let order = ChoiceKind::ScryOrder { source: None, bottom: false };
        seats.expect_order(order.clone(), vec![ChoiceOption::Object(bears), ChoiceOption::Object(wall)]);

        assert_eq!(seats.allocate(&game, 0, &ChoiceContext::new(damage), 3, &listed, &[0, 0], None), vec![1, 2]);
        assert_eq!(seats.choose_ordering(&game, 0, &ChoiceContext::new(order), &listed), vec![1, 0]);
    }

    /// An allocation that leaves a bucket out is refused, as a replayed one
    /// is: the script would otherwise give it nothing without saying so.
    #[test]
    #[should_panic(expected = "the allocation names 1 buckets")]
    fn an_allocation_names_every_bucket() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, card("Grizzly Bears"), 1);
        let wall = put_on_battlefield(&mut game, card("Wall of Stone"), 1);
        let seats = ScriptedDecisionProvider::new();
        let damage = ChoiceKind::AssignCombatDamage { attacker_id: bears };
        seats.expect_allocation(damage.clone(), vec![(ChoiceOption::Object(bears), 3)]);
        let listed = [ChoiceOption::Object(wall), ChoiceOption::Object(bears)];
        seats.allocate(&game, 0, &ChoiceContext::new(damage), 3, &listed, &[0, 0], None);
    }

    /// Two equal options are one choice twice, as two alike costs are: a
    /// script naming the option twice takes both, in a pick and in an order.
    #[test]
    fn a_script_takes_two_equal_options_in_turn() {
        let game = setup_two_player_game();
        let twins = [ChoiceOption::Number(2), ChoiceOption::Number(2)];
        let seats = ScriptedDecisionProvider::new();
        let which = ChoiceKind::ChooseReplacementEffect { affected_object: None };
        seats.expect_choice(which.clone(), vec![ChoiceOption::Number(2), ChoiceOption::Number(2)]);
        let order = ChoiceKind::ScryOrder { source: None, bottom: false };
        seats.expect_order(order.clone(), vec![ChoiceOption::Number(2), ChoiceOption::Number(2)]);

        assert_eq!(seats.pick_n(&game, 0, &ChoiceContext::new(which.clone()), &twins, (2, 2)), vec![0, 1]);
        assert_eq!(seats.choose_ordering(&game, 0, &ChoiceContext::new(order.clone()), &twins), vec![0, 1]);

        // A test's recording provider finds what it names the same way.
        let twice = || [ChoiceOption::Number(2), ChoiceOption::Number(2)];
        let recording = crate::test_support::RecordingDecisionProvider::choosing(twice()).ordering(twice());
        assert_eq!(recording.pick_n(&game, 0, &ChoiceContext::new(which), &twins, (2, 2)), vec![0, 1]);
        assert_eq!(recording.choose_ordering(&game, 0, &ChoiceContext::new(order), &twins), vec![0, 1]);
    }

    /// A source names its one effect offered; naming a source that has two
    /// would mean whichever the engine lists first, so it is refused.
    #[test]
    #[should_panic(expected = "different options offered")]
    fn a_source_with_two_effects_offered_needs_the_effect_named() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, card("Grizzly Bears"), 0);
        let wall = put_on_battlefield(&mut game, card("Wall of Stone"), 0);
        let source = game.object_ref(bears).unwrap();
        let ability = TriggeredAbility::OfItsSource { ability: AbilityId::UNASSIGNED, granted_by: None, read_as: AbilityAsRead(Some((1, ""))) };
        let triggered = |by| ChoiceOption::Trigger(TriggerOption { source, ability: ability.clone(), triggered_by: vec![by] });
        let seats = ScriptedDecisionProvider::new();
        let order = ChoiceKind::OrderTriggers { player: 0, tier: crate::types::triggers::TriggerTier::First };
        seats.expect_order(order.clone(), vec![ChoiceOption::Object(bears), ChoiceOption::Object(bears)]);
        seats.choose_ordering(&game, 0, &ChoiceContext::new(order), &[triggered(bears), triggered(wall)]);
    }

    /// A take answers a take-or-leave question's one option, and nothing
    /// that offers two.
    #[test]
    #[should_panic(expected = "takes the one option offered")]
    fn a_take_is_refused_where_two_options_are_offered() {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, card("Grizzly Bears"), 0);
        let wall = put_on_battlefield(&mut game, card("Wall of Stone"), 0);
        let seats = ScriptedDecisionProvider::new();
        let may = ChoiceKind::ApplyOptionalEffect { source: bears };
        seats.expect_take(may.clone());
        let two = [ChoiceOption::Object(bears), ChoiceOption::Object(wall)];
        seats.pick_n(&game, 0, &ChoiceContext::new(may), &two, (0, 1));
    }
}
