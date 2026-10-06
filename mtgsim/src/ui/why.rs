//! A why: the engine's answer to "why is this the way it is?", in the words
//! every client draws alike (`setup-architecture.md` §7c).
//!
//! The facts are the engine's: what the layers did to an object comes from
//! `engine::layers::explain`, the pass itself recorded, and why an option is
//! not offered comes from the checks the enumeration builds the options by.
//! This module words them, a line at a time, each line with the rule it rests
//! on and the objects it names. A client lays the lines out and links each
//! name to that object's own why, and draws no case per mechanic.

use std::collections::HashSet;

use crate::engine::combat::validation::{can_block, CombatError};
use crate::engine::layers::types::{EffectOrigin, EffectiveCharacteristics, Layer};
use crate::engine::layers::{compute_characteristics, explain, AppliedBy, LayerStep, StepResult};
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::oracle::characteristics::{controls, get_effective_abilities, is_creature};
use crate::oracle::legality::{can_attack, can_play_land};
use crate::oracle::mana_helpers::{can_activate, can_activate_its_abilities, can_cast};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;
use crate::types::zones::Zone;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, Rejection};
use crate::ui::decision::PriorityAction;
use crate::ui::display::{
    cannot_activate, cannot_cast, cannot_pay, cannot_play_land, color_name, combat_refusal, keyword_name, named,
    option_label, player_name, rejection_words, type_line, Declaring,
};

/// What a why is about: an object, or a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhyAbout {
    Object(ObjectId),
    Player(PlayerId),
}

/// The question open where a why is asked: whom it asks, what, and the
/// options it offers, as the seat was handed them.
#[derive(Clone, Copy, Debug)]
pub struct OpenQuestion<'a> {
    pub player: PlayerId,
    pub context: &'a ChoiceContext,
    pub options: &'a [ChoiceOption],
}

/// One answer, about one object.
#[derive(Clone, Debug, PartialEq)]
pub struct Why {
    /// What it is about, as the board names it: `Serra Angel (#24)`.
    pub title: String,
    pub sections: Vec<WhySection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhySection {
    pub heading: String,
    pub lines: Vec<WhyLine>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhyLine {
    pub text: String,
    /// The CR rule the line rests on, as the CR numbers it: `613.1f`.
    pub rule: Option<&'static str>,
    /// The objects the line names, each with the label the board gives it.
    pub names: Vec<(ObjectId, String)>,
    /// 0 for a line that stands alone, 1 for one that says more about the
    /// line above it.
    pub depth: u8,
}

impl WhyLine {
    fn new(text: impl Into<String>) -> WhyLine {
        WhyLine { text: text.into(), rule: None, names: Vec::new(), depth: 0 }
    }

    fn under(text: impl Into<String>) -> WhyLine {
        WhyLine { depth: 1, ..WhyLine::new(text) }
    }

    fn rule(self, rule: &'static str) -> WhyLine {
        WhyLine { rule: Some(rule), ..self }
    }

    fn naming(self, game: &GameState, ids: &[ObjectId]) -> WhyLine {
        WhyLine { names: ids.iter().map(|&id| (id, named(game, id))).collect(), ..self }
    }
}

/// Why `about` is the way it is now: whether the open question offers it,
/// and, for an object, what the layers did to it, from what it prints to what
/// it has.
pub fn why(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>) -> Why {
    match about {
        WhyAbout::Object(id) => {
            let mut answer = what_the_layers_did(game, id);
            if let Some(question) = at
                && game.objects.contains_key(&id)
            {
                answer.sections.insert(0, at_this_question(game, about, question));
            }
            answer
        }
        WhyAbout::Player(player) => {
            let section = match at {
                Some(question) => at_this_question(game, about, question),
                None => WhySection { heading: AT_THIS_QUESTION.to_string(), lines: vec![WhyLine::new("No question is open.")] },
            };
            Why { title: player_name(player), sections: vec![section] }
        }
    }
}

const AT_THIS_QUESTION: &str = "At this question";

/// Whether the question offers `about`, and as what; if it offers it nothing,
/// why, on each tier: never offered, by the checks the options were built by,
/// or offered and then reversed (CR 732.1).
fn at_this_question(game: &GameState, about: WhyAbout, question: &OpenQuestion) -> WhySection {
    let asked = player_name(question.player);
    let mut lines = vec![WhyLine::new(format!("{asked}: {}", crate::ui::display::question(game, &question.context.kind)))];
    let offered: Vec<&ChoiceOption> = question.options.iter().filter(|option| names(option, about)).collect();
    if !offered.is_empty() {
        lines.push(WhyLine::new(format!("Offered to {asked}:")));
        lines.extend(offered.iter().map(|option| WhyLine::under(option_label(game, option)).naming(game, &option_objects(option))));
    }
    let refused = refusals(game, about, question);
    if !refused.is_empty() {
        lines.push(WhyLine::new(format!("Never offered to {asked}:")));
        lines.extend(refused);
    } else if offered.is_empty() {
        lines.push(WhyLine::new(format!("Not among the options {asked} is offered here.")));
    }
    if let Some(rejected) = &question.context.rejected
        && rejection_names(rejected, about)
    {
        lines.push(WhyLine::new(format!("Offered to {asked}, then reversed:")));
        let (words, rule) = rejection_words(game, rejected);
        lines.push(WhyLine { rule, ..WhyLine::under(words) });
    }
    WhySection { heading: AT_THIS_QUESTION.to_string(), lines }
}

/// Does `option` name `about`?
fn names(option: &ChoiceOption, about: WhyAbout) -> bool {
    match about {
        WhyAbout::Object(id) => option_objects(option).contains(&id),
        WhyAbout::Player(player) => matches!(
            option,
            ChoiceOption::Player(named) | ChoiceOption::AttackerTarget(_, AttackTarget::Player(named)) if *named == player
        ),
    }
}

/// The objects an option names: an action's object, or both sides of an
/// attack or a block. One arm per kind of option and no wildcard.
fn option_objects(option: &ChoiceOption) -> Vec<ObjectId> {
    match option {
        ChoiceOption::Object(id)
        | ChoiceOption::Action(
            PriorityAction::CastSpell(id) | PriorityAction::PlayLand(id) | PriorityAction::ActivateAbility(id, _),
        ) => vec![*id],
        ChoiceOption::AttackerTarget(attacker, AttackTarget::Player(_)) => vec![*attacker],
        ChoiceOption::AttackerTarget(attacker, AttackTarget::Planeswalker(id) | AttackTarget::Battle(id)) => {
            vec![*attacker, *id]
        }
        ChoiceOption::BlockerAttacker(blocker, attacker) => vec![*blocker, *attacker],
        ChoiceOption::Action(PriorityAction::Pass)
        | ChoiceOption::Player(_)
        | ChoiceOption::NormalCost
        | ChoiceOption::AlternativeCost(_)
        | ChoiceOption::AdditionalCost(_)
        | ChoiceOption::Number(_)
        | ChoiceOption::Color(_)
        | ChoiceOption::CounterType(_)
        | ChoiceOption::ManaType(_) => Vec::new(),
    }
}

/// Does the answer the question rejected name `about`?
fn rejection_names(rejected: &Rejection, about: WhyAbout) -> bool {
    match rejected {
        Rejection::Reversed(action) => names(&ChoiceOption::Action(action.clone()), about),
        Rejection::IllegalBlocks { blocks, .. } => {
            blocks.iter().any(|&(blocker, attacker)| names(&ChoiceOption::BlockerAttacker(blocker, attacker), about))
        }
    }
}

/// Why each thing `about` could be at this question is not offered, from the
/// check the options were built by. The priority question, the two
/// declarations and CR 601.2g's window have such checks; at every other
/// question the options come from a filter of the question's own, with no
/// typed reason yet (`codebase-state.md` item 212), and a target's are RS-2's.
fn refusals(game: &GameState, about: WhyAbout, question: &OpenQuestion) -> Vec<WhyLine> {
    match (&question.context.kind, about) {
        (ChoiceKind::PriorityAction, WhyAbout::Object(id)) => priority_refusals(game, question, id),
        (ChoiceKind::DeclareAttackers, WhyAbout::Object(id)) => {
            let attacking = question.options.iter().any(|o| matches!(o, ChoiceOption::AttackerTarget(a, _) if *a == id));
            match can_attack(game, question.player, id) {
                Err(error) if !attacking => vec![refusal("To attack", combat_refusal(game, &error, Declaring::Attackers))],
                _ => Vec::new(),
            }
        }
        (ChoiceKind::DeclareAttackers, WhyAbout::Player(player)) if !question.options.iter().any(|o| names(o, about)) => {
            if player == question.player {
                vec![refusal("To be attacked", ("a creature attacks one of its controller's opponents".to_string(), Some("506.2")))]
            } else if !game.in_game(player) {
                vec![refusal("To be attacked", (format!("{} has left the game", player_name(player)), Some("800.4a")))]
            } else {
                Vec::new()
            }
        }
        (ChoiceKind::DeclareBlockers, WhyAbout::Object(id)) => block_refusals(game, question, id),
        (ChoiceKind::ManaAbilityWindow { .. }, WhyAbout::Object(id)) => window_refusals(game, question, id),
        (
            ChoiceKind::PriorityAction
            | ChoiceKind::DeclareAttackers
            | ChoiceKind::DeclareBlockers
            | ChoiceKind::AssignCombatDamage { .. }
            | ChoiceKind::AssignTrampleDamage { .. }
            | ChoiceKind::ChooseXValue { .. }
            | ChoiceKind::ChooseAlternativeCost { .. }
            | ChoiceKind::ChooseAdditionalCosts { .. }
            | ChoiceKind::SelectRecipients { .. }
            | ChoiceKind::GenericManaAllocation { .. }
            | ChoiceKind::OrderCostReductions { .. }
            | ChoiceKind::ManaAbilityWindow { .. }
            | ChoiceKind::ChooseSacrificeForCost { .. }
            | ChoiceKind::ChooseReplacementEffect { .. }
            | ChoiceKind::OrderTriggers { .. }
            | ChoiceKind::ApplyOptionalReplacement { .. }
            | ChoiceKind::ApplyOptionalEffect { .. }
            | ChoiceKind::AllocateNextDamage { .. }
            | ChoiceKind::ChooseDamageSource { .. }
            | ChoiceKind::ChooseEnteringController { .. }
            | ChoiceKind::ChooseAuxiliaryZoneChange { .. }
            | ChoiceKind::ChooseCopySource { .. }
            | ChoiceKind::CommanderToCommandZoneSba { .. }
            | ChoiceKind::Discard { .. }
            | ChoiceKind::Scry { .. }
            | ChoiceKind::ScryOrder { .. }
            | ChoiceKind::LegendRule { .. },
            _,
        ) => Vec::new(),
    }
}

/// At a priority question: a card is played if it is a land and cast if it
/// is not (CR 305.1, 305.9), and a permanent's abilities are activated.
fn priority_refusals(game: &GameState, question: &OpenQuestion, id: ObjectId) -> Vec<WhyLine> {
    let player = question.player;
    let offered = |ability: AbilityId| {
        question.options.iter().any(|option| {
            matches!(option, ChoiceOption::Action(PriorityAction::ActivateAbility(source, offered))
                if *source == id && *offered == ability)
        })
    };
    let Some(obj) = game.objects.get(&id) else {
        return Vec::new();
    };
    match obj.zone {
        Zone::Battlefield => {
            let abilities = get_effective_abilities(game, id);
            let activated: Vec<&AbilityDef> = abilities
                .iter()
                .filter(|ability| matches!(ability.ability_type, AbilityType::Activated | AbilityType::Mana))
                .collect();
            let Some(first) = activated.first() else {
                return vec![WhyLine::under("It has no ability to activate.")];
            };
            if let Err(reason) = can_activate_its_abilities(game, player, id) {
                return vec![refusal("To activate its abilities", cannot_activate(game, player, id, first, &reason))];
            }
            activated
                .iter()
                .filter(|ability| !offered(ability.id))
                .filter_map(|ability| {
                    let reason = can_activate(game, player, id, ability).err()?;
                    let action = format!("To activate “{}”", ability.rules_text.words);
                    Some(refusal(action, cannot_activate(game, player, id, ability, &reason)))
                })
                .collect()
        }
        Zone::Stack => Vec::new(),
        Zone::Hand | Zone::Library | Zone::Graveyard | Zone::Exile | Zone::Command => {
            // PRE-LAYER ZONE: a card is played or cast by what it is in its
            // zone, before it is a permanent or a spell.
            if obj.card_data.types.contains(&CardType::Land) {
                can_play_land(game, player, id)
                    .err()
                    .map(|reason| refusal("To play it", cannot_play_land(&reason, player)))
                    .into_iter()
                    .collect()
            } else {
                can_cast(game, player, id)
                    .err()
                    .map(|reason| refusal("To cast it", cannot_cast(game, player, id, &reason)))
                    .into_iter()
                    .collect()
            }
        }
    }
}

/// In CR 601.2g's window: each of the permanent's mana abilities the window
/// does not offer, with the cost that cannot be paid (`mana-architecture.md`
/// §3.13). Two grants of one ability are one option, so an ability is
/// offered when any instance of its definition is.
fn window_refusals(game: &GameState, question: &OpenQuestion, id: ObjectId) -> Vec<WhyLine> {
    let player = question.player;
    if !game.battlefield.contains_key(&id) {
        return Vec::new();
    }
    let abilities = get_effective_abilities(game, id);
    let mana: Vec<&AbilityDef> = abilities.iter().filter(|ability| ability.ability_type == AbilityType::Mana).collect();
    let Some(first) = mana.first() else {
        return vec![WhyLine::under("It has no mana ability.")];
    };
    if let Err(reason) = can_activate_its_abilities(game, player, id) {
        return vec![refusal("To activate its mana abilities", cannot_activate(game, player, id, first, &reason))];
    }
    let offered = |ability: AbilityId| {
        question.options.iter().any(|option| {
            matches!(option, ChoiceOption::Action(PriorityAction::ActivateAbility(source, offered))
                if *source == id && offered.definition() == ability.definition())
        })
    };
    mana.iter()
        .filter(|ability| !offered(ability.id))
        .filter_map(|ability| {
            let reason = game.can_pay_costs(&ability.costs, player, id).err()?;
            let action = format!("To activate “{}”", ability.rules_text.words);
            Some(refusal(action, cannot_pay(game, player, id, &reason)))
        })
        .collect()
}

/// At a declare-blockers question: for an attacker, which of the defending
/// player's creatures can't block it; for anything else, which attackers it
/// can't block. A reason about the blocker alone is the same for every
/// attacker, so it is said once.
fn block_refusals(game: &GameState, question: &OpenQuestion, id: ObjectId) -> Vec<WhyLine> {
    let defender = question.player;
    let offered = |blocker: ObjectId, attacker: ObjectId| {
        question.options.iter().any(|option| {
            matches!(option, ChoiceOption::BlockerAttacker(b, a) if *b == blocker && *a == attacker)
        })
    };
    let attackers: Vec<ObjectId> = game
        .battlefield_ids_ordered()
        .into_iter()
        .filter(|a| game.battlefield.get(a).is_some_and(|entry| entry.attacking.is_some()))
        .collect();
    if attackers.contains(&id) {
        return game
            .battlefield_ids_ordered()
            .into_iter()
            .filter(|&blocker| controls(game, blocker, defender) && is_creature(game, blocker) && !offered(blocker, id))
            .filter_map(|blocker| {
                let error = can_block(game, defender, blocker, id).err()?;
                let line = refusal(format!("To be blocked by {}", named(game, blocker)), combat_refusal(game, &error, Declaring::Blockers));
                Some(line.naming(game, &[blocker]))
            })
            .collect();
    }
    let errors: Vec<(ObjectId, CombatError)> = attackers
        .into_iter()
        .filter(|&attacker| !offered(id, attacker))
        .filter_map(|attacker| can_block(game, defender, id, attacker).err().map(|error| (attacker, error)))
        .collect();
    if let Some((_, error)) = errors.first()
        && about_the_blocker_alone(error, id)
    {
        return vec![refusal("To block", combat_refusal(game, error, Declaring::Blockers))];
    }
    errors
        .iter()
        .map(|(attacker, error)| {
            refusal(format!("To block {}", named(game, *attacker)), combat_refusal(game, error, Declaring::Blockers))
                .naming(game, &[*attacker])
        })
        .collect()
}

/// Is `error` about `blocker` alone, so that it refuses every block the
/// creature could make?
fn about_the_blocker_alone(error: &CombatError, blocker: ObjectId) -> bool {
    match error {
        CombatError::NotOnBattlefield(id)
        | CombatError::NotACreature(id)
        | CombatError::NotControlledByPlayer(id, _)
        | CombatError::CreatureIsTapped(id) => *id == blocker,
        CombatError::CreatureHasSummoningSickness(_)
        | CombatError::InvalidAttackTarget(_)
        | CombatError::AttackerNotAttackingThisPlayer(..)
        | CombatError::TooManyBlocks(..)
        | CombatError::HasDefender(_)
        | CombatError::CantBlockFlyer(..)
        | CombatError::ConstraintViolation(_) => false,
    }
}

/// A line under "Never offered": what was not offered, and why.
fn refusal(action: impl Into<String>, (words, rule): (String, Option<&'static str>)) -> WhyLine {
    WhyLine { rule, ..WhyLine::under(format!("{}: {words}.", action.into())) }
}

/// What the layers did to `about`, from what it prints to what it has.
fn what_the_layers_did(game: &GameState, about: ObjectId) -> Why {
    let title = named(game, about);
    let Some(explanation) = explain(game, about) else {
        let gone = WhySection { heading: "Gone".to_string(), lines: vec![WhyLine::new("No object has this id now.")] };
        return Why { title, sections: vec![gone] };
    };
    let shows_controller = game.battlefield.contains_key(&about);
    let mut applied = vec![WhyLine::new(format!("Printed: {}", summary(&explanation.seed, shows_controller)))];
    applied.extend(abilities_lines(&explanation.seed));
    let mut missed = Vec::new();
    for step in &explanation.steps {
        let (label, rule) = layer_words(step.layer);
        let mut head = WhyLine::new(format!("{label} · {}", applied_by_words(game, step))).rule(rule);
        if let AppliedBy::Effect { source, .. } = step.by
            && source != about
        {
            head = head.naming(game, &[source]);
        }
        let quoted = step_words(game, step).map(|words| WhyLine::under(format!("“{words}”")));
        let waited = (!step.waited_for.is_empty()).then(|| {
            WhyLine::under("It applied after these, out of timestamp order, since it depends on them")
                .rule("613.8")
                .naming(game, &step.waited_for)
        });
        match &step.result {
            StepResult::Applied { before, after } => {
                applied.push(head);
                applied.extend(quoted);
                let changed = changes(before, after);
                if changed.is_empty() {
                    applied.push(WhyLine::under("No change: it already was so."));
                }
                applied.extend(changed.into_iter().map(WhyLine::under));
                applied.extend(waited);
            }
            StepResult::NotMatched | StepResult::Gone | StepResult::LockedOut => {
                missed.push(head);
                missed.extend(quoted);
                missed.extend(waited);
                missed.push(match step.result {
                    StepResult::Gone => {
                        WhyLine::under("Its source no longer has the ability, so the effect does not exist.").rule("604.2")
                    }
                    StepResult::LockedOut => WhyLine::under(
                        "The effect began in an earlier layer, and the objects it began with do not include this one.",
                    )
                    .rule("613.6"),
                    _ => WhyLine::under("What it applies to does not include this object."),
                });
                if !step.affected.is_empty() {
                    missed.push(WhyLine::under("It applied to").naming(game, &step.affected));
                }
            }
        }
    }
    applied.push(WhyLine::new(format!("Now: {}", summary(&explanation.result, shows_controller))));
    applied.extend(abilities_lines(&explanation.result));
    if explanation.steps.is_empty() {
        applied.insert(1, WhyLine::new("No continuous effect reaches it."));
    }
    let mut sections = vec![WhySection { heading: "What the layers did".to_string(), lines: applied }];
    if !missed.is_empty() {
        sections.push(WhySection { heading: "Reached its zone, and did not apply to it".to_string(), lines: missed });
    }
    Why { title, sections }
}

/// A layer as a person reads it, and the rule that defines it.
fn layer_words(layer: Layer) -> (&'static str, &'static str) {
    match layer {
        Layer::Layer1Copy => ("Layer 1a, copy", "613.2a"),
        Layer::Layer2Control => ("Layer 2, control", "613.1b"),
        Layer::Layer3Text => ("Layer 3, text", "613.1c"),
        Layer::Layer4Type => ("Layer 4, type", "613.1d"),
        Layer::Layer5Color => ("Layer 5, color", "613.1e"),
        Layer::Layer6Ability => ("Layer 6, abilities", "613.1f"),
        Layer::Layer7aCdaPT => ("Layer 7a, power and toughness defined", "613.4a"),
        Layer::Layer7bSetPT => ("Layer 7b, power and toughness set", "613.4b"),
        Layer::Layer7cModifyPT => ("Layer 7c, power and toughness modified", "613.4c"),
        Layer::Layer7dSwitchPT => ("Layer 7d, power and toughness switched", "613.4d"),
    }
}

/// What applied, in words, with its timestamp.
fn applied_by_words(game: &GameState, step: &LayerStep) -> String {
    let by = match step.by {
        AppliedBy::Effect { source, origin: EffectOrigin::StaticAbility { .. }, .. } => {
            format!("{}'s static ability", named(game, source))
        }
        AppliedBy::Effect { source, origin: EffectOrigin::Resolution, .. } => {
            format!("{}, as it resolved", named(game, source))
        }
        AppliedBy::Cda { .. } => "its characteristic-defining ability (CR 604.3)".to_string(),
        AppliedBy::KeywordCounter { keyword } => format!("a {} counter (CR 122.1b)", keyword_name(keyword)),
        AppliedBy::PtCounters { kind, count: 1 } => format!("a {} counter (CR 122.1a)", kind.name()),
        AppliedBy::PtCounters { kind, count } => format!("{count} {} counters (CR 122.1a)", kind.name()),
        AppliedBy::EnteredAsCopy => "the copy it entered as (CR 614.1c, 707.5)".to_string(),
        AppliedBy::EnteredWith => "what it entered with (CR 614.1c)".to_string(),
        AppliedBy::IntrinsicLoyalty => "a planeswalker's intrinsic ability (CR 306.5b)".to_string(),
    };
    match step.timestamp {
        Some(timestamp) => format!("{by} · timestamp {timestamp}"),
        None => by,
    }
}

/// The words of what applied, where it has some: an ability's own text, or
/// the spell that resolved.
fn step_words(game: &GameState, step: &LayerStep) -> Option<&'static str> {
    let words = match step.by {
        AppliedBy::Effect { source, origin: EffectOrigin::StaticAbility { ability }, .. } => {
            static_ability_words(game, source, ability)
        }
        AppliedBy::Effect { source, origin: EffectOrigin::Resolution, .. } => {
            // AS PRINTED: the resolved spell's words, for a line no rule reads.
            let card = &game.objects.get(&source)?.card_data;
            card.abilities.iter().find(|ability| ability.ability_type == AbilityType::Spell).map(|ability| ability.rules_text.words)
        }
        AppliedBy::Cda { ability } => match &step.result {
            StepResult::Applied { before, .. } => before.abilities.iter().find(|a| a.id == ability).map(|a| a.rules_text.words),
            StepResult::NotMatched | StepResult::Gone | StepResult::LockedOut => None,
        },
        AppliedBy::KeywordCounter { .. }
        | AppliedBy::PtCounters { .. }
        | AppliedBy::EnteredAsCopy
        | AppliedBy::EnteredWith
        | AppliedBy::IntrinsicLoyalty => None,
    };
    words.filter(|words| !words.is_empty())
}

/// A static ability's words: as its source has it now, which finds one it
/// was granted or copied, or else as printed, since an ability a later layer
/// took away still made the effect it had begun (CR 613.6), as Humility's own
/// does by layer 7b.
fn static_ability_words(game: &GameState, source: ObjectId, ability: AbilityId) -> Option<&'static str> {
    let now = compute_characteristics(game, source)
        .and_then(|frame| frame.abilities.iter().find(|a| a.id == ability).map(|a| a.rules_text.words));
    // AS PRINTED: the ability's printed words, for a line no rule reads.
    now.or_else(|| {
        let card = &game.objects.get(&source)?.card_data;
        card.abilities.iter().find(|a| a.id.definition() == ability.definition()).map(|a| a.rules_text.words)
    })
}

/// A frame on one line: its name and mana cost, colors, type line, power and
/// toughness or loyalty, and, for a permanent, who controls it.
fn summary(frame: &EffectiveCharacteristics, shows_controller: bool) -> String {
    let mut parts = vec![match &frame.mana_cost {
        Some(cost) => format!("{} {cost}", frame.name),
        None => frame.name.clone(),
    }];
    parts.push(colors_words(&frame.colors));
    parts.push(type_line(&frame.supertypes, &frame.types, &frame.subtypes));
    if let (Some(power), Some(toughness)) = (frame.power, frame.toughness) {
        parts.push(format!("{power}/{toughness}"));
    }
    if let Some(loyalty) = frame.loyalty {
        parts.push(format!("loyalty {loyalty}"));
    }
    if shows_controller {
        parts.push(format!("{} controls it", player_name(frame.controller)));
    }
    parts.join(" · ")
}

/// A frame's keywords and abilities, one line under its summary: its keywords
/// in `KeywordFlag`'s order, then each ability's words, an ability built as
/// several parts once.
fn abilities_lines(frame: &EffectiveCharacteristics) -> Vec<WhyLine> {
    let keywords = keywords_words(&frame.keyword_flags);
    let abilities = ability_words(&frame.abilities);
    if keywords.is_empty() && abilities.is_empty() {
        return vec![WhyLine::under("No abilities.")];
    }
    let mut lines = Vec::new();
    if !keywords.is_empty() {
        lines.push(WhyLine::under(keywords.join(", ")));
    }
    lines.extend(abilities.into_iter().map(|words| WhyLine::under(format!("“{words}”"))));
    lines
}

/// What changed from `before` to `after`, a phrase each. Every field is named
/// with no `..`, so a field the frame gains does not compile until it says
/// how it changes.
fn changes(before: &EffectiveCharacteristics, after: &EffectiveCharacteristics) -> Vec<String> {
    let EffectiveCharacteristics {
        name,
        mana_cost,
        colors,
        types,
        subtypes,
        supertypes,
        keyword_flags,
        abilities,
        power,
        toughness,
        loyalty,
        controller,
        control_since_turn,
    } = after;
    let mut out = Vec::new();
    if *name != before.name {
        out.push(format!("Name from {} to {name}", before.name));
    }
    if *mana_cost != before.mana_cost {
        out.push(format!("Mana cost from {} to {}", cost_words(&before.mana_cost), cost_words(mana_cost)));
    }
    if *colors != before.colors {
        out.push(format!("Colors from {} to {}", colors_words(&before.colors), colors_words(colors)));
    }
    if (types, subtypes, supertypes) != (&before.types, &before.subtypes, &before.supertypes) {
        let was = type_line(&before.supertypes, &before.types, &before.subtypes);
        out.push(format!("Type line from {was} to {}", type_line(supertypes, types, subtypes)));
    }
    let gained: HashSet<KeywordFlag> = keyword_flags.difference(&before.keyword_flags).copied().collect();
    let lost: HashSet<KeywordFlag> = before.keyword_flags.difference(keyword_flags).copied().collect();
    if !gained.is_empty() {
        out.push(format!("Gains {}", keywords_words(&gained).join(", ")));
    }
    if !lost.is_empty() {
        out.push(format!("Loses {}", keywords_words(&lost).join(", ")));
    }
    let ids = |list: &[AbilityDef]| list.iter().map(|a| a.id).collect::<HashSet<AbilityId>>();
    let (had, has) = (ids(&before.abilities), ids(abilities));
    let gained: Vec<AbilityDef> = abilities.iter().filter(|a| !had.contains(&a.id)).cloned().collect();
    let lost: Vec<AbilityDef> = before.abilities.iter().filter(|a| !has.contains(&a.id)).cloned().collect();
    for words in ability_words(&gained) {
        out.push(format!("Gains “{words}”"));
    }
    for words in ability_words(&lost) {
        out.push(format!("Loses “{words}”"));
    }
    if (*power, *toughness) != (before.power, before.toughness) {
        let was = pt_words(before.power, before.toughness);
        out.push(format!("Power and toughness from {was} to {}", pt_words(*power, *toughness)));
    }
    if *loyalty != before.loyalty {
        out.push(format!("Loyalty from {} to {}", number_words(before.loyalty), number_words(*loyalty)));
    }
    if *controller != before.controller || *control_since_turn != before.control_since_turn {
        out.push(format!(
            "Controller from {} to {}, since turn {control_since_turn} (CR 302.6)",
            player_name(before.controller),
            player_name(*controller)
        ));
    }
    out
}

/// Each ability's words, an ability the engine builds as several parts once,
/// as `ui::display` prints a permanent's: its parts share a paragraph and,
/// when granted, the grant.
fn ability_words(abilities: &[AbilityDef]) -> Vec<&'static str> {
    let mut shown: Vec<(&'static str, Option<u16>, Option<u64>)> = Vec::new();
    for ability in abilities {
        let part = (ability.rules_text.words, ability.rules_text.paragraph(), ability.id.granting_row());
        if part.1.is_some() && shown.contains(&part) {
            continue;
        }
        shown.push(part);
    }
    shown.into_iter().map(|(words, _, _)| if words.is_empty() { "an ability with no text" } else { words }).collect()
}

fn keywords_words(flags: &HashSet<KeywordFlag>) -> Vec<&'static str> {
    let mut flags: Vec<KeywordFlag> = flags.iter().copied().collect();
    flags.sort();
    flags.into_iter().map(keyword_name).collect()
}

/// Colors in WUBRG order, or colorless.
fn colors_words(colors: &HashSet<Color>) -> String {
    let order = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let named: Vec<&str> = order.into_iter().filter(|c| colors.contains(c)).map(color_name).collect();
    if named.is_empty() { "colorless".to_string() } else { named.join(" and ") }
}

fn cost_words(cost: &Option<ManaCost>) -> String {
    cost.as_ref().map_or_else(|| "none".to_string(), ToString::to_string)
}

fn pt_words(power: Option<i32>, toughness: Option<i32>) -> String {
    match (power, toughness) {
        (None, None) => "none".to_string(),
        _ => format!("{}/{}", number_words(power), number_words(toughness)),
    }
}

fn number_words(n: Option<i32>) -> String {
    n.map_or_else(|| "none".to_string(), |n| n.to_string())
}
