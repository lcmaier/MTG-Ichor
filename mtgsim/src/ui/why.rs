//! A why: the engine's answer to "why is this the way it is?", in the words
//! every client draws alike (`setup-architecture.md` §7c).
//!
//! The facts are the engine's: what the layers did to an object comes from
//! `engine::layers::explain`, the pass itself recorded, and why an option is
//! not offered comes from the checks the enumeration builds the options by.
//! What happened, and why two questions offer what they do, are read from a
//! replay's trace (`ui::what_happened`). This module words them, a line at a
//! time, each line with the rule it rests on and the objects it names. A
//! client lays the lines out and links each name to that object's own why,
//! and draws no case per mechanic.

use std::collections::HashSet;

use crate::engine::combat::validation::{can_block, CombatError};
use crate::engine::layers::types::{EffectOrigin, EffectiveCharacteristics, Layer};
use crate::engine::layers::{compute_characteristics, explain, AppliedBy, LayerStep, StepResult};
use crate::engine::replacement::{auxiliary_zone, not_an_opponent, not_auxiliary, NotAnOpponent, NotAuxiliary};
use crate::events::event::EventSeq;
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::oracle::characteristics::{controls, get_effective_abilities, is_creature};
use crate::oracle::legality::{can_attack, can_play_land};
use crate::oracle::mana_helpers::{can_activate, can_activate_its_abilities, can_cast};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::state::trace::{RecordKind, TraceRecord};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::Effect;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;
use crate::types::replacement::{ReplacementClass, Rewrite};
use crate::types::zones::Zone;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, Rejection};
use crate::ui::decision::PriorityAction;
use crate::ui::display::{
    cannot_activate, cannot_cast, cannot_pay, cannot_play_land, color_name, combat_refusal, format_phase, keyword_name,
    named, option_label, player_name, rejection_words, type_line, Declaring,
};
use crate::ui::what_happened::{cant_words, trigger_lines, what_happened, WHAT_HAPPENED};

/// What a why is about: an object, a player, or a performed event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhyAbout {
    Object(ObjectId),
    Player(PlayerId),
    /// An event, by its place in the performed stream: the trace's `index`.
    Event(EventSeq),
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
    pub(crate) fn new(text: impl Into<String>) -> WhyLine {
        WhyLine { text: text.into(), rule: None, names: Vec::new(), depth: 0 }
    }

    pub(crate) fn under(text: impl Into<String>) -> WhyLine {
        WhyLine { depth: 1, ..WhyLine::new(text) }
    }

    pub(crate) fn rule(self, rule: &'static str) -> WhyLine {
        WhyLine { rule: Some(rule), ..self }
    }

    pub(crate) fn naming(self, game: &GameState, ids: &[ObjectId]) -> WhyLine {
        WhyLine { names: ids.iter().map(|&id| (id, named(game, id))).collect(), ..self }
    }
}

/// Why `about` is the way it is now: whether the open question offers it,
/// and, for an object, what the layers did to it, from what it prints to what
/// it has. What an event did is read from a trace, [`why_from_trace`]'s.
pub fn why(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>) -> Why {
    answer(game, about, at, None)
}

/// [`why`] with the trace of a game replayed to the open question, or to its
/// end: what an event did, and what the trace says at the two questions whose
/// options it explains, the choice of a replacement effect (CR 616.1) and the
/// order of triggered abilities (CR 603.3b).
pub fn why_from_trace(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>, trace: &[TraceRecord]) -> Why {
    answer(game, about, at, Some(trace))
}

/// Whether a why at a question of this kind reads the trace, so that a client
/// asks it of a replay rather than of the seat.
pub fn read_from_the_trace(kind: &ChoiceKind) -> bool {
    matches!(kind, ChoiceKind::ChooseReplacementEffect { .. } | ChoiceKind::OrderTriggers { .. })
}

fn answer(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>, trace: Option<&[TraceRecord]>) -> Why {
    match about {
        WhyAbout::Object(id) => {
            let mut answer = what_the_layers_did(game, id);
            if let Some(question) = at
                && game.objects.contains_key(&id)
            {
                answer.sections.insert(0, at_this_question(game, about, question, trace));
            }
            answer
        }
        WhyAbout::Player(player) => {
            let section = match at {
                Some(question) => at_this_question(game, about, question, trace),
                None => WhySection { heading: AT_THIS_QUESTION.to_string(), lines: vec![WhyLine::new("No question is open.")] },
            };
            Why { title: player_name(player), sections: vec![section] }
        }
        WhyAbout::Event(event) => {
            let Some(trace) = trace else {
                let lines = vec![WhyLine::new("What an event did is read from the trace of a replay.")];
                return Why { title: format!("Event {}", event.0), sections: vec![WhySection { heading: WHAT_HAPPENED.to_string(), lines }] };
            };
            let mut answer = what_happened(game, event, trace);
            let replayed = match at {
                Some(question) => format!(
                    "Read from a replay to {}'s question, {}: {}",
                    player_name(question.player),
                    format_phase(game),
                    question_words(game, question)
                ),
                None => "Read from a replay of the whole game.".to_string(),
            };
            if let Some(first) = answer.sections.first_mut() {
                first.lines.insert(0, WhyLine::new(replayed));
            }
            answer
        }
    }
}

const AT_THIS_QUESTION: &str = "At this question";

/// The question, as `ui::display` asks it.
fn question_words(game: &GameState, open: &OpenQuestion) -> String {
    crate::ui::display::question(game, &open.context.kind)
}

/// Whether the question offers `about`, and as what; if it offers it nothing,
/// why, on each tier: never offered, by the checks the options were built by,
/// or offered and then reversed (CR 732.1).
fn at_this_question(game: &GameState, about: WhyAbout, question: &OpenQuestion, trace: Option<&[TraceRecord]>) -> WhySection {
    let asked = player_name(question.player);
    let mut lines = vec![WhyLine::new(format!("{asked}: {}", question_words(game, question)))];
    let offered: Vec<&ChoiceOption> = question.options.iter().filter(|option| names(option, about)).collect();
    if !offered.is_empty() {
        lines.push(WhyLine::new(format!("Offered to {asked}:")));
        lines.extend(offered.iter().map(|option| WhyLine::under(option_label(game, option)).naming(game, &option_objects(option))));
    }
    let refused = refusals(game, about, question, trace);
    if !refused.is_empty() {
        lines.push(WhyLine::new(format!("Never offered to {asked}:")));
        lines.extend(refused);
    } else if offered.is_empty() {
        let (range, rule) = ranges_over(game, &question.context.kind, question.player);
        lines.push(WhyLine { rule, ..WhyLine::new(format!("Not among the options: the question ranges over {range}.")) });
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
        // No option is an event: a question chooses among what is now.
        WhyAbout::Event(_) => false,
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
/// check or the filter the options were built by. Every kind answers here,
/// with no wildcard, and a kind with no line for `about` leaves it to
/// [`ranges_over`]: what the question ranges over, which is then not `about`.
///
/// The priority question, the two declarations and CR 601.2g's window ask
/// their checks; three kinds ask their filters, the opponents an entering
/// permanent may go to, an entry's auxiliary move and a copy effect's donor;
/// and two read the trace, when a replay has one: the effects a CR 616.1
/// choice has applied already, and the triggered abilities asked about the
/// events an ordering's abilities triggered on. The other filtered kinds'
/// reasons are their owners': a target's RS-2's, X's and the generic split's
/// MA-2's, a sacrifice's CP-2's (`setup-architecture.md` §8).
fn refusals(game: &GameState, about: WhyAbout, question: &OpenQuestion, trace: Option<&[TraceRecord]>) -> Vec<WhyLine> {
    let offered = question.options.iter().any(|option| names(option, about));
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
        (ChoiceKind::ChooseEnteringController { .. }, WhyAbout::Player(player)) => {
            let reason = not_an_opponent(game, question.player, player).map(|reason| match reason {
                NotAnOpponent::Yourself => (format!("{} is choosing, and an opponent is another player", player_name(player)), Some("102.2")),
                NotAnOpponent::LeftTheGame => (format!("{} has left the game", player_name(player)), Some("800.4a")),
            });
            reason.map(|reason| refusal("To control it", reason)).into_iter().collect()
        }
        (ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, to }, WhyAbout::Object(id)) if !offered => {
            auxiliary_refusal(game, question.player, (*entering, *source, *to), id).into_iter().collect()
        }
        (ChoiceKind::ChooseCopySource { source }, WhyAbout::Object(id)) if !offered => vec![copy_refusal(game, *source, id)],
        (ChoiceKind::ChooseReplacementEffect { .. }, WhyAbout::Object(id)) if !offered => {
            trace.map(|trace| applied_already(game, trace, id)).unwrap_or_default()
        }
        (ChoiceKind::OrderTriggers { .. }, WhyAbout::Object(id)) if !offered => {
            trace.map(|trace| asked_about_these_events(game, trace, id)).unwrap_or_default()
        }
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

/// What a question ranges over, said to a why about something it does not
/// offer, and the rule; one arm per kind and no wildcard, so a new kind says
/// at birth what it asks about.
fn ranges_over(game: &GameState, kind: &ChoiceKind, asked: PlayerId) -> (String, Option<&'static str>) {
    let n = |id: &ObjectId| named(game, *id);
    let who = player_name(asked);
    match kind {
        ChoiceKind::PriorityAction => (
            format!("the lands {who} can play, the spells {who} can cast, the abilities {who} can activate, and passing"),
            Some("117.1"),
        ),
        ChoiceKind::DeclareAttackers => (format!("the creatures {who} controls that can attack, and what each can attack"), Some("508.1a")),
        ChoiceKind::DeclareBlockers => (format!("the creatures {who} controls that can block, and what each can block"), Some("509.1a")),
        ChoiceKind::AssignCombatDamage { attacker_id } => {
            (format!("how {}'s combat damage is divided among the creatures blocking it", n(attacker_id)), Some("510.1c"))
        }
        ChoiceKind::AssignTrampleDamage { attacker_id, .. } => (
            format!("how {}'s combat damage is divided among the creatures blocking it and what it attacks", n(attacker_id)),
            Some("702.19b"),
        ),
        ChoiceKind::ChooseXValue { spell_id, .. } => (format!("the value of X for {}", n(spell_id)), Some("601.2b")),
        ChoiceKind::ChooseAlternativeCost { spell_id } => {
            (format!("whether {} is cast for its mana cost or for an alternative cost", n(spell_id)), Some("118.9"))
        }
        ChoiceKind::ChooseAdditionalCosts { spell_id } => {
            (format!("which of {}'s optional additional costs are paid", n(spell_id)), Some("118.8"))
        }
        ChoiceKind::SelectRecipients { spell_id, .. } => {
            (format!("the objects and players {} may target or choose", n(spell_id)), Some("601.2c"))
        }
        ChoiceKind::GenericManaAllocation { spell_or_ability_id, .. } => {
            (format!("the mana in the pool that pays the generic part of {}'s cost", n(spell_or_ability_id)), Some("601.2h"))
        }
        ChoiceKind::OrderCostReductions { spell_id } => {
            (format!("the order the cost reductions apply to {} in", n(spell_id)), Some("601.2f"))
        }
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id, .. } => {
            (format!("the mana abilities {who} can activate to pay for {}", n(spell_or_ability_id)), Some("601.2g"))
        }
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id, .. } => {
            (format!("the permanents {who} controls that can be sacrificed to pay for {}", n(spell_or_ability_id)), Some("701.21a"))
        }
        ChoiceKind::ChooseReplacementEffect { .. } => (
            "the replacement and prevention effects that apply to the event, from the first step of CR 616.1's order that has any"
                .to_string(),
            Some("616.1"),
        ),
        ChoiceKind::OrderTriggers { .. } => {
            (format!("{who}'s triggered abilities that triggered together, in the order they go on the stack"), Some("603.3b"))
        }
        ChoiceKind::ApplyOptionalReplacement { source, .. } => (format!("whether {}'s effect applies", n(source)), Some("614.1a")),
        ChoiceKind::ApplyOptionalEffect { source } => (format!("whether {} does what it says {who} may", n(source)), Some("603.5")),
        ChoiceKind::AllocateNextDamage { source, remaining } => {
            (format!("the damage {}'s effect prevents, {remaining} at most", n(source)), Some("615.7"))
        }
        ChoiceKind::ChooseDamageSource { source } => {
            (format!("every permanent and every spell on the stack, as the source {} names", n(source)), Some("609.7a"))
        }
        ChoiceKind::ChooseEnteringController { object } => {
            (format!("the opponents still in the game, one of whom controls {} as it enters", n(object)), Some("614.12a"))
        }
        ChoiceKind::ChooseAuxiliaryZoneChange { entering, source, .. } => {
            (format!("the objects {}'s effect may move as {} enters", n(source), n(entering)), Some("614.13a"))
        }
        ChoiceKind::ChooseCopySource { source } => (format!("the permanents {}'s effect may copy", n(source)), None),
        ChoiceKind::CommanderToCommandZoneSba { commander } => {
            (format!("whether {} goes to the command zone", n(commander)), Some("903.9a"))
        }
        ChoiceKind::Discard { .. } => (format!("the cards in {who}'s hand"), Some("701.9b")),
        ChoiceKind::Scry { .. } => (format!("the cards {who} is looking at, any of which go to the bottom"), Some("701.22a")),
        ChoiceKind::ScryOrder { bottom, .. } => {
            let pile = if *bottom { "bottom" } else { "top" };
            (format!("the order of the cards going on the {pile}"), Some("701.22a"))
        }
        ChoiceKind::LegendRule { legend_name } => {
            (format!("the permanents named {legend_name} {who} controls, one of which stays"), Some("704.5j"))
        }
    }
}

/// At CR 614.13a's choice: why `id` may not be moved as `entering` enters, by
/// the check the candidates were built by, read off `source`'s effect.
fn auxiliary_refusal(game: &GameState, you: PlayerId, (entering, source, to): (ObjectId, ObjectId, Zone), id: ObjectId) -> Option<WhyLine> {
    let moved = get_effective_abilities(game, source).iter().find_map(|ability| match &ability.effect {
        Effect::Replacement(def) => match &def.rewrite {
            Rewrite::EnterAfterMoving(aux) if aux.to == to => Some(aux.clone()),
            _ => None,
        },
        _ => None,
    })?;
    let reason = if id == entering {
        Some(NotAuxiliary::Entering)
    } else if !auxiliary_zone(game, &moved, you).is_ok_and(|zone| zone.contains(&id)) {
        let zone = match moved.from {
            Zone::Battlefield => "on the battlefield".to_string(),
            from => format!("in {}'s {from:?}", player_name(you)),
        };
        return Some(refusal("To be moved", (format!("it is not {zone}, where the effect chooses from"), None)));
    } else {
        not_auxiliary(game, &moved, you, id)
    };
    let words = match reason? {
        NotAuxiliary::Entering => ("it is entering the battlefield in this event".to_string(), Some("614.13a")),
        NotAuxiliary::AlreadyChosen => ("an entry earlier in this event chose it already".to_string(), Some("614.13b")),
        NotAuxiliary::NotMatched => (format!("it is not what {}'s effect chooses", named(game, source)), None),
        NotAuxiliary::Prohibited(cant) => {
            let (by, words) = cant.by.as_recorded();
            (format!("a “can't” forbids it: {}", cant_words(game, cant.source, by, words)), Some("101.2"))
        }
    };
    Some(refusal("To be moved", words))
}

/// At a copy effect's choice of donor: a permanent its filter does not admit,
/// in the effect's own words, or what is not a permanent at all.
fn copy_refusal(game: &GameState, source: ObjectId, id: ObjectId) -> WhyLine {
    if !game.battlefield.contains_key(&id) {
        return refusal("To be copied", ("it is not a permanent on the battlefield".to_string(), None));
    }
    let entering = get_effective_abilities(game, source).iter().find_map(|ability| match &ability.effect {
        Effect::Replacement(def) if def.class == ReplacementClass::CopyAsEnters => Some(ability.rules_text.words),
        _ => None,
    });
    // AS PRINTED: a resolving spell's words, for a line no rule reads.
    let resolving = || {
        let card = &game.objects.get(&source)?.card_data;
        card.abilities.iter().find(|ability| ability.ability_type == AbilityType::Spell).map(|ability| ability.rules_text.words)
    };
    let words = match entering.or_else(resolving) {
        Some(words) if !words.is_empty() => format!("it is not what {}'s effect may copy: “{words}”", named(game, source)),
        _ => format!("it is not what {}'s effect may copy", named(game, source)),
    };
    refusal("To be copied", (words, None))
}

/// At a CR 616.1 choice, from the trace: what this batch's earlier iterations
/// did with `id`'s effects, each of which gets one chance at an event.
fn applied_already(game: &GameState, trace: &[TraceRecord], id: ObjectId) -> Vec<WhyLine> {
    let Some(batch) = game.events.current_stamp().batch else { return Vec::new() };
    let iterations = trace.iter().filter(|r| r.kind == RecordKind::ReplacementPipeline && r.u64("batch") == Some(batch.0));
    iterations
        .filter_map(|record| {
            let mine = record.items("candidates").iter().find(|c| c.get("source").and_then(|s| s.as_u64()) == Some(id.raw()))?;
            let chosen = mine.get("id").and_then(|i| i.as_str()) == record.str("choice");
            let iteration = record.u64("iteration").unwrap_or_default();
            let what = match (chosen, record.str("optional")) {
                (true, Some("declined")) => format!("its effect was declined at iteration {iteration}"),
                (true, _) => format!("its effect applied at iteration {iteration}"),
                (false, _) => return None,
            };
            Some(refusal("To apply again", (format!("{what}, and an effect gets one chance at an event"), Some("614.5"))))
        })
        .collect()
}

/// At CR 603.3b's ordering, from the trace: what `id`'s triggered abilities
/// answered when asked about the events the queued abilities triggered on.
fn asked_about_these_events(game: &GameState, trace: &[TraceRecord], id: ObjectId) -> Vec<WhyLine> {
    let bound: Vec<u64> =
        game.pending_triggers.iter().flat_map(|t| t.binding.records.iter().map(|r| r.seq.0 as u64)).collect();
    trace
        .iter()
        .filter(|r| r.kind == RecordKind::Trigger && r.u64("source") == Some(id.raw()))
        .filter(|r| r.u64("record").is_some_and(|record| bound.contains(&record)))
        .flat_map(|r| {
            let event = trace.iter().find(|e| e.kind == RecordKind::Event && e.u64("index") == r.u64("record"));
            let asked = WhyLine::under(format!("Asked about: {}", event.and_then(|e| e.str("text")).unwrap_or("an event")));
            std::iter::once(asked).chain(trigger_lines(game, trace, r).into_iter().map(|line| WhyLine { depth: line.depth + 1, ..line }))
        })
        .collect()
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
