//! The question as owned data: which primitive asked, what it asks, and each
//! option as a label plus the board things it names. The names are what let the
//! window make exactly those things clickable with no case per `ChoiceKind`.

use mtgsim::engine::layers::compute::compute_characteristics;
use mtgsim::events::event::DamageTarget;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::GameState;
use mtgsim::types::costs::{AdditionalCost, AlternativeCost};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::mana::ManaSymbol;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::PriorityAction;

use crate::snapshot::{attack_target_name, named, player_name};

/// An answer, in the shape of the primitive that asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Picks(Vec<usize>),
    Number(u64),
    Allocation(Vec<u64>),
    Order(Vec<usize>),
}

/// Which of the four `DecisionProvider` methods asked, with its bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Primitive {
    PickN { min: usize, max: usize },
    Number { min: u64, max: u64 },
    Allocate { total: u64, mins: Vec<u64>, maxs: Option<Vec<u64>> },
    Order,
}

/// A thing on the board an option names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoardRef {
    Object(ObjectId),
    Player(PlayerId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionView {
    pub label: String,
    /// In order: none, one, or a pair's two — attacker then what it attacks,
    /// blocker then what it blocks.
    pub refs: Vec<BoardRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    /// The `ChoiceKind` variant's name, which the decision log records.
    pub kind: String,
    pub question: String,
    /// `ChoiceKind::subject()`.
    pub subject: Option<ObjectId>,
    pub primitive: Primitive,
    /// A pick's or an ordering's options, an allocation's buckets; none for a number.
    pub options: Vec<OptionView>,
}

impl Prompt {
    pub fn pick_n(game: &GameState, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Prompt {
        Prompt::new(game, context, Primitive::PickN { min: bounds.0, max: bounds.1 }, options)
    }

    pub fn number(game: &GameState, context: &ChoiceContext, min: u64, max: u64) -> Prompt {
        Prompt::new(game, context, Primitive::Number { min, max }, &[])
    }

    pub fn allocate(
        game: &GameState,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Prompt {
        let primitive = Primitive::Allocate { total, mins: mins.to_vec(), maxs: maxs.map(<[u64]>::to_vec) };
        Prompt::new(game, context, primitive, buckets)
    }

    pub fn order(game: &GameState, context: &ChoiceContext, items: &[ChoiceOption]) -> Prompt {
        Prompt::new(game, context, Primitive::Order, items)
    }

    fn new(game: &GameState, context: &ChoiceContext, primitive: Primitive, options: &[ChoiceOption]) -> Prompt {
        Prompt {
            kind: kind_name(&context.kind),
            question: question(game, &context.kind),
            subject: context.kind.subject(),
            primitive,
            options: options.iter().map(|option| option_view(game, option)).collect(),
        }
    }
}

/// `AssignCombatDamage { attacker_id: #7 }` → `AssignCombatDamage`.
fn kind_name(kind: &ChoiceKind) -> String {
    let debug = format!("{kind:?}");
    debug.split(|c: char| !c.is_alphanumeric()).next().unwrap_or_default().to_string()
}

/// The question in words, one arm per kind — the only per-kind code in the
/// window, and exhaustive, so a new kind is asked a question at birth.
fn question(game: &GameState, kind: &ChoiceKind) -> String {
    let n = |id: &ObjectId| named(game, *id);
    match kind {
        ChoiceKind::PriorityAction => "You have priority".to_string(),
        ChoiceKind::DeclareAttackers => "Declare attackers".to_string(),
        ChoiceKind::DeclareBlockers => "Declare blockers".to_string(),
        ChoiceKind::AssignCombatDamage { attacker_id } => format!("Assign {}'s combat damage", n(attacker_id)),
        ChoiceKind::AssignTrampleDamage { attacker_id, defending_target } => format!(
            "Assign {}'s trample damage; what is left goes to {}",
            n(attacker_id),
            damage_target_name(game, defending_target)
        ),
        ChoiceKind::ChooseXValue { spell_id, .. } => format!("Choose X for {}", n(spell_id)),
        ChoiceKind::ChooseAlternativeCost { spell_id } => format!("Choose how to pay for {}", n(spell_id)),
        ChoiceKind::ChooseAdditionalCosts { spell_id } => format!("Choose additional costs for {}", n(spell_id)),
        ChoiceKind::SelectRecipients { spell_id, .. } => format!("Choose targets for {}", n(spell_id)),
        ChoiceKind::GenericManaAllocation { spell_or_ability_id, mana_cost } => {
            format!("Split the generic part of {mana_cost} for {}", n(spell_or_ability_id))
        }
        ChoiceKind::OrderCostReductions { spell_id } => {
            format!("Order the cost reductions for {}; the first applies first", n(spell_id))
        }
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id, remaining_cost } => {
            format!("Pay {remaining_cost} more for {}: activate a mana ability, or stop", n(spell_or_ability_id))
        }
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, count } => {
            format!("Sacrifice {count} for {}", n(spell_or_ability_id))
        }
        ChoiceKind::ChooseReplacementEffect { affected_object } => match affected_object {
            Some(id) => format!("Choose the replacement effect that applies to {}", n(id)),
            None => "Choose the replacement effect that applies to you".to_string(),
        },
        ChoiceKind::OrderTriggers { .. } => {
            "Order your triggered abilities; the first goes on the stack first and resolves last".to_string()
        }
        ChoiceKind::ApplyOptionalReplacement { source, .. } => format!("Apply {}'s replacement effect?", n(source)),
        ChoiceKind::ApplyOptionalEffect { source } => format!("{}: you may", n(source)),
        ChoiceKind::AllocateNextDamage { source, remaining } => {
            format!("Choose the damage {} prevents ({remaining} left)", n(source))
        }
        ChoiceKind::ChooseDamageSource { source } => format!("Choose a source of damage for {}", n(source)),
        ChoiceKind::ChooseEnteringController { object } => {
            format!("Choose the opponent who controls {} as it enters", n(object))
        }
        ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, to } => {
            format!("Choose what goes to the {to:?} as {} changes how {} enters", n(source), n(entering))
        }
        ChoiceKind::ChooseCopySource { source } => format!("Choose what {} copies", n(source)),
        ChoiceKind::CommanderToCommandZoneSba { commander } => format!("Put {} into the command zone?", n(commander)),
        ChoiceKind::Discard { source } => match source {
            Some(id) => format!("Discard for {}", n(id)),
            None => "Discard down to your maximum hand size".to_string(),
        },
        ChoiceKind::Scry { n: count, .. } => format!("Scry {count}: choose the cards that go on the bottom"),
        ChoiceKind::ScryOrder { bottom, .. } => {
            format!("Order the cards going on the {}, top-most first", if *bottom { "bottom" } else { "top" })
        }
        ChoiceKind::LegendRule { legend_name } => format!("Legend rule: choose the {legend_name} to keep"),
    }
}

fn option_view(game: &GameState, option: &ChoiceOption) -> OptionView {
    let n = |id: &ObjectId| named(game, *id);
    let (label, refs) = match option {
        ChoiceOption::Object(id) => (n(id), vec![BoardRef::Object(*id)]),
        ChoiceOption::Player(player) => (player_name(*player), vec![BoardRef::Player(*player)]),
        ChoiceOption::Action(PriorityAction::Pass) => ("Pass".to_string(), vec![]),
        ChoiceOption::Action(PriorityAction::CastSpell(id)) => (format!("Cast {}", n(id)), vec![BoardRef::Object(*id)]),
        ChoiceOption::Action(PriorityAction::PlayLand(id)) => (format!("Play {}", n(id)), vec![BoardRef::Object(*id)]),
        ChoiceOption::Action(PriorityAction::ActivateAbility(id, ability)) => (
            format!("Activate {}: ability {}", n(id), ability_index(game, *id, *ability)),
            vec![BoardRef::Object(*id)],
        ),
        ChoiceOption::AttackerTarget(attacker, target) => (
            format!("{} attacks {}", n(attacker), attack_target_name(game, target)),
            vec![BoardRef::Object(*attacker), attack_target_ref(target)],
        ),
        ChoiceOption::BlockerAttacker(blocker, attacker) => (
            format!("{} blocks {}", n(blocker), n(attacker)),
            vec![BoardRef::Object(*blocker), BoardRef::Object(*attacker)],
        ),
        ChoiceOption::NormalCost => ("Its mana cost".to_string(), vec![]),
        ChoiceOption::AlternativeCost(cost) => (alternative_cost_name(cost), vec![]),
        ChoiceOption::AdditionalCost(cost) => (additional_cost_name(cost), vec![]),
        ChoiceOption::Number(number) => (number.to_string(), vec![]),
        ChoiceOption::Color(color) => (format!("{color:?}"), vec![]),
        ChoiceOption::CounterType(counter) => (counter.name().to_string(), vec![]),
        ChoiceOption::ManaType(mana) => (ManaSymbol::Colored(*mana).to_string(), vec![]),
    };
    OptionView { label, refs }
}

/// Its place in the effective ability list, the way `display.rs` numbers
/// abilities; `?` if the list no longer has it.
fn ability_index(game: &GameState, id: ObjectId, ability: AbilityId) -> String {
    compute_characteristics(game, id)
        .and_then(|chars| chars.abilities.iter().position(|a| a.id == ability))
        .map_or_else(|| "?".to_string(), |i| i.to_string())
}

fn attack_target_ref(target: &AttackTarget) -> BoardRef {
    match target {
        AttackTarget::Player(player) => BoardRef::Player(*player),
        AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => BoardRef::Object(*id),
    }
}

fn damage_target_name(game: &GameState, target: &DamageTarget) -> String {
    match target {
        DamageTarget::Player(player) => player_name(*player),
        DamageTarget::Object(id) => named(game, *id),
    }
}

/// The keyword's name. The cost it names is a `Cost` tree, and a payload that
/// carries an engine AST is `codebase-state.md` main item 141's open case.
fn alternative_cost_name(cost: &AlternativeCost) -> String {
    match cost {
        AlternativeCost::Custom(name, _) => name.clone(),
        other => variant_name(&format!("{other:?}")),
    }
}

fn additional_cost_name(cost: &AdditionalCost) -> String {
    match cost {
        AdditionalCost::Custom(name, _) => name.clone(),
        other => variant_name(&format!("{other:?}")),
    }
}

fn variant_name(debug: &str) -> String {
    debug.split(|c: char| !c.is_alphanumeric()).next().unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_is_named_by_its_variant() {
        assert_eq!(kind_name(&ChoiceKind::DeclareBlockers), "DeclareBlockers");
        assert_eq!(kind_name(&ChoiceKind::Discard { source: None }), "Discard");
    }
}
