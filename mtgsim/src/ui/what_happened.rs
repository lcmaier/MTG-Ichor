//! The why of an event, read from a replay's trace (`setup-architecture.md`
//! §7c, SU-8). A question about the past is answered from what the sink's
//! emit points wrote as it happened, never by deciding it again against the
//! board now: the batch the event was performed in, as the engine decided it,
//! and every triggered ability asked about the event, matched or refused, and
//! by what.
//!
//! The records spell proposals in the engine's own words, as the trace viewer
//! shows them (`plans/traces/viewer.html`), and a line links each object it
//! writes as `#id`.

use crate::events::event::EventSeq;
use crate::oracle::characteristics::get_effective_abilities;
use crate::state::game_state::GameState;
use crate::state::trace::{FieldValue, RecordKind, TraceRecord};
use crate::types::ids::ObjectId;
use crate::ui::display::{named, player_name};
use crate::ui::why::{Why, WhyLine, WhySection};

/// The heading of an event's first section.
pub(crate) const WHAT_HAPPENED: &str = "What happened";

/// What `event` did: the batch it was performed in, a batch decided after it
/// that performed nothing, and the triggered abilities asked about it.
pub(crate) fn what_happened(game: &GameState, event: EventSeq, trace: &[TraceRecord]) -> Why {
    let index = event.0 as u64;
    let Some(at) = trace.iter().position(|r| r.kind == RecordKind::Event && r.u64("index") == Some(index)) else {
        let missing = WhyLine::new(format!("The trace holds no event {index}."));
        return Why { title: format!("Event {index}"), sections: vec![section(WHAT_HAPPENED, vec![missing])] };
    };
    let performed = &trace[at];
    let batch = performed.u64("batch");
    let mut lines = match batch {
        Some(batch) => {
            let records = trace.iter().filter(|r| r.u64("batch") == Some(batch));
            let head = WhyLine::new(format!("It was performed in batch {batch}."));
            std::iter::once(head).chain(records.flat_map(|r| record_lines(game, r, index))).collect()
        }
        None => vec![WhyLine::new("It was performed on its own, outside any batch of proposed events.")],
    };
    lines.extend(nothing_performed_after(game, &trace[at + 1..], batch));
    let asked: Vec<WhyLine> = trace
        .iter()
        .filter(|r| r.kind == RecordKind::Trigger && r.u64("record") == Some(index))
        .flat_map(|r| trigger_lines(game, trace, r))
        .collect();
    let asked = if asked.is_empty() { vec![WhyLine::new("No triggered ability was asked about it.")] } else { asked };
    Why {
        title: performed.str("text").unwrap_or_default().to_string(),
        sections: vec![section(WHAT_HAPPENED, lines), section("Triggered abilities asked about it", asked)],
    }
}

/// One record of a batch, as lines: a `batch`'s proposals, a `pipeline`'s
/// CR 616.1 iteration, an `event` other than event `index`, and a
/// `batch_end`'s proposals that were not performed.
fn record_lines(game: &GameState, record: &TraceRecord, index: u64) -> Vec<WhyLine> {
    match record.kind {
        RecordKind::Batch => {
            record.items("members").iter().map(|m| linked(game, 0, format!("Proposed: {}", text(m, "action")))).collect()
        }
        RecordKind::ReplacementPipeline => iteration_lines(game, record),
        RecordKind::Event if record.u64("index") != Some(index) => {
            vec![linked(game, 0, format!("Performed beside it: {}", record.str("text").unwrap_or_default()))]
        }
        RecordKind::BatchEnd => record
            .items("members")
            .iter()
            .filter(|m| m.get("performed") == Some(&FieldValue::Null))
            .map(|m| WhyLine::new(format!("Proposal {} was not performed.", number(m, "i"))))
            .collect(),
        _ => Vec::new(),
    }
}

/// A CR 616.1 iteration that met something: each "can't" (CR 614.17), each
/// effect that applied, which one was applied and how it was chosen, and what
/// the proposals became. An iteration that met nothing, nearly every one,
/// says nothing.
fn iteration_lines(game: &GameState, record: &TraceRecord) -> Vec<WhyLine> {
    let (candidates, stopped) = (record.items("candidates"), record.items("prohibited"));
    if candidates.is_empty() && stopped.is_empty() {
        return Vec::new();
    }
    let iteration = record.u64("iteration").unwrap_or_default();
    let mut lines = vec![WhyLine::new(format!("Replacement and prevention effects, iteration {iteration}:")).rule("616.1")];
    for stop in stopped {
        let words = cant_words(game, id(stop, "source"), text(stop, "by"), stop.get("words").and_then(FieldValue::as_str));
        let line = WhyLine::under(format!("A “can't” stops proposal {}: {words}", number(stop, "i")));
        lines.push(line.rule("614.17").naming(game, &[id(stop, "source")]));
    }
    for candidate in candidates {
        let source = id(candidate, "source");
        let applies = format!("{}'s effect applies ({})", named(game, source), text(candidate, "class"));
        let line = match candidate.get("bucket") {
            Some(FieldValue::Bool(false)) => WhyLine::under(format!("{applies}, and waits for one earlier in the order")),
            _ => WhyLine::under(applies),
        };
        lines.push(line.naming(game, &[source]));
    }
    let chosen = candidates.iter().find(|c| c.get("id").and_then(FieldValue::as_str) == record.str("choice"));
    if let Some(chosen) = chosen.map(|c| id(c, "source")) {
        let chooser = record.u64("chooser").map_or_else(|| "Nobody".to_string(), |p| player_name(p as usize));
        let effect = format!("{}'s effect", named(game, chosen));
        let (how, rule) = match (record.str("decided"), record.str("optional")) {
            (_, Some("declined")) => (format!("{chooser} declined {effect}: its one chance at this event"), "614.5"),
            (Some("asked"), _) => (format!("{chooser} chose {effect} to apply first"), "616.1"),
            (Some("order_invariant"), _) => (format!("{effect} applied first, unasked: no order could change the outcome"), "616.1"),
            _ => (format!("{effect} applied, the only one that could"), "616.1"),
        };
        lines.push(WhyLine::under(how).rule(rule).naming(game, &[chosen]));
    }
    for member in record.items("results") {
        lines.push(match member.get("event").and_then(FieldValue::as_str) {
            Some(event) => linked(game, 1, format!("Proposal {} is now: {event}", number(member, "i"))),
            None => WhyLine::under(format!("Proposal {} does not happen", number(member, "i"))),
        });
    }
    lines
}

/// A batch decided after the event and before the next event was performed,
/// in which nothing was performed: a destruction a "can't" stopped has no
/// event of its own, so its why is here.
fn nothing_performed_after(game: &GameState, after: &[TraceRecord], own: Option<u64>) -> Vec<WhyLine> {
    let window = &after[..after.iter().position(|r| r.kind == RecordKind::Event).unwrap_or(after.len())];
    let idle: Vec<u64> = window
        .iter()
        .filter(|r| r.kind == RecordKind::BatchEnd && r.u64("batch") != own)
        .filter(|r| r.items("members").iter().all(|m| m.get("performed") == Some(&FieldValue::Null)))
        .filter_map(|r| r.u64("batch"))
        .collect();
    let Some(&first) = idle.first() else { return Vec::new() };
    let mut lines = vec![WhyLine::new(format!("Then batch {first} was decided, and performed nothing:"))];
    let records = window.iter().filter(|r| r.u64("batch") == Some(first) && r.kind != RecordKind::BatchEnd);
    lines.extend(records.flat_map(|r| record_lines(game, r, u64::MAX)).map(|line| WhyLine { depth: line.depth + 1, ..line }));
    if idle.len() > 1 {
        lines.push(WhyLine::new(format!("{} more batches performed nothing before the next event.", idle.len() - 1)));
    }
    lines
}

/// One matcher decision (CR 603.2): the ability asked, whether it triggered,
/// and then where it went (CR 603.3) or what refused it.
pub(crate) fn trigger_lines(game: &GameState, trace: &[TraceRecord], record: &TraceRecord) -> Vec<WhyLine> {
    let source = ObjectId::from_trace(record.u64("source").unwrap_or_default());
    let matched = record.bool("matched") == Some(true);
    let verdict = if matched { "triggered" } else { "did not trigger" };
    let mut lines = vec![WhyLine::new(format!("{}: {verdict}", named(game, source))).rule("603.2").naming(game, &[source])];
    lines.extend(ability_words(game, source, record.str("ability")).map(|words| WhyLine::under(format!("“{words}”"))));
    lines.push(if !matched {
        refused(record.str("refused_by"))
    } else if record.bool("mana") == Some(true) {
        WhyLine::under("A mana ability: it resolved at once, without the stack").rule("605.4a")
    } else {
        placement(game, trace, record)
    });
    lines
}

/// Where a matched trigger went: the first `pending` record after the match
/// that names its source and this event.
fn placement(game: &GameState, trace: &[TraceRecord], matched: &TraceRecord) -> WhyLine {
    let event = matched.u64("record");
    let pending = trace.iter().filter(|r| r.kind == RecordKind::Pending && r.seq > matched.seq).find(|r| {
        r.u64("source") == matched.u64("source") && r.items("records").iter().any(|n| n.as_u64() == event)
    });
    match pending.map(|p| (p.u64("object"), p.str("refused_by"))) {
        None => WhyLine::under("It waits to be put on the stack the next time a player would receive priority").rule("603.3"),
        Some((Some(object), _)) => {
            let object = ObjectId::from_trace(object);
            WhyLine::under(format!("Put on the stack as {}", named(game, object))).rule("603.3").naming(game, &[object])
        }
        Some((None, Some("departed"))) => {
            WhyLine::under("Removed: the player who would control it has left the game").rule("800.4d")
        }
        Some((None, _)) => WhyLine::under("Removed: it has no legal choice to make as it is put on the stack").rule("603.3d"),
    }
}

/// What refused a trigger, as the matcher names it (`engine::triggers::dispatch`).
fn refused(by: Option<&str>) -> WhyLine {
    let (words, rule) = match by {
        Some("condition") => ("Its trigger condition does not match this event", Some("603.2")),
        Some("intervening_if") => ("Its intervening “if” was false when the event happened", Some("603.4")),
        Some("limit") => ("It had already used what it may trigger each turn", None),
        Some("visibility") => ("Its source was not visible to all players", Some("603.2f")),
        Some("state") => ("A state trigger, which is checked against the board, not against events", Some("603.8")),
        _ => ("Refused by a check this build does not name", None),
    };
    WhyLine { rule, ..WhyLine::under(words) }
}

/// The text of `source`'s ability whose id the trace wrote: as it has it now,
/// or else as printed, for one a later effect took away.
fn ability_words(game: &GameState, source: ObjectId, ability: Option<&str>) -> Option<&'static str> {
    let ability = ability?;
    let now = get_effective_abilities(game, source).iter().find(|a| a.id.to_string() == ability).map(|a| a.rules_text.words);
    // AS PRINTED: an ability's printed words, for a line no rule reads.
    now.or_else(|| {
        let card = &game.objects.get(&source)?.card_data;
        card.abilities.iter().find(|a| a.id.to_string() == ability).map(|a| a.rules_text.words)
    })
    .filter(|words| !words.is_empty())
}

/// A "can't", as the trace records one (`engine::restriction::ProhibitedBy`):
/// a keyword its source has, one of its source's static abilities, or an
/// effect a resolution registered.
pub(crate) fn cant_words(game: &GameState, source: ObjectId, by: &str, words: Option<&str>) -> String {
    match (by, words) {
        ("keyword", Some(keyword)) => format!("{} has {keyword}", named(game, source)),
        ("static_ability", Some(words)) => format!("{}'s “{words}”", named(game, source)),
        _ => format!("an effect of {}", named(game, source)),
    }
}

fn section(heading: &str, lines: Vec<WhyLine>) -> WhySection {
    WhySection { heading: heading.to_string(), lines }
}

fn text<'a>(value: &'a FieldValue, key: &str) -> &'a str {
    value.get(key).and_then(FieldValue::as_str).unwrap_or_default()
}

fn number(value: &FieldValue, key: &str) -> u64 {
    value.get(key).and_then(FieldValue::as_u64).unwrap_or_default()
}

fn id(value: &FieldValue, key: &str) -> ObjectId {
    ObjectId::from_trace(number(value, key))
}

/// A line at `depth` naming each object its text writes as `#id`.
fn linked(game: &GameState, depth: u8, text: String) -> WhyLine {
    let mut ids: Vec<ObjectId> = Vec::new();
    for (at, _) in text.match_indices('#') {
        let digits: String = text[at + 1..].chars().take_while(char::is_ascii_digit).collect();
        if let Ok(raw) = digits.parse()
            && !ids.contains(&ObjectId::from_trace(raw))
        {
            ids.push(ObjectId::from_trace(raw));
        }
    }
    WhyLine { depth, ..WhyLine::new(text) }.naming(game, &ids)
}
