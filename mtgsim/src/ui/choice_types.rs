use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, EffectRecipient};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};

use super::decision::PriorityAction;

/// What kind of decision is being made. UIs use this to render appropriate
/// screens. AI agents can match on this for specialized heuristics.
/// Adding a new variant here is the ONLY change needed when a new decision
/// type is introduced — no trait methods or impl changes.
///
/// Exhaustive matching is intentional: single-crate project, compiler flags
/// every match site when a variant is added.
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
    ChooseAlternativeCost,
    ChooseAdditionalCosts,
    /// Select recipients for an effect (covers both MTG "target" and non-targeting
    /// "choose" — the `EffectRecipient` field distinguishes them).
    SelectRecipients { recipient: EffectRecipient, spell_id: ObjectId },
    GenericManaAllocation { mana_cost: ManaCost },
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

    /// CR 616.1b / 614.12a — an entry replacement puts `object` under "an
    /// opponent of your choice" and there is more than one opponent to choose
    /// from. The options are players. With exactly one opponent nothing is
    /// asked (CR 102.2), and the choice is made before the permanent enters.
    ChooseEnteringController { object: ObjectId },

    // --- Copy effects (CR 707) ---
    /// CR 707.4 — a resolving copy effect must **choose** the permanent whose
    /// copiable values it captures. Cytoshape's "Choose a nonlegendary creature
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

    // --- State-Based & Cleanup ---
    DiscardToHandSize,
    LegendRule { legend_name: String },
}

/// Wrapper carrying the semantic kind. No display text — each DP impl formats
/// its own prompts by matching on `kind`. This keeps choice types pure (no
/// presentation leakage into the engine boundary).
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
