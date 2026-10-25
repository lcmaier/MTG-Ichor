//! Every record the engine writes to the trace sink, one function per kind.
//!
//! The emit points call these and nothing else, so an emit point is one line —
//! `self.trace(|| trace_records::batch(self, &groups, inherited))` —
//! and the shape of every record is in this file rather than inline in the
//! function it observes. The sink, the handle and the JSON builder are
//! `state::trace`; the one record built elsewhere is the CR 616.1 iteration's,
//! which `replacement::pipeline` accumulates across its loop's three exits and
//! cannot hand over whole.
//!
//! Nothing here runs in an untraced game. Each function is the body of the
//! closure [`GameState::trace`] skips when no sink is attached, which is where
//! the sink's cost model lives: rendering an action, sorting a type set and
//! resolving a name all happen inside these functions and nowhere else.

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::replacement::{BatchMember, SubjectGroup};
use crate::engine::targeting::TargetInstance;
use crate::events::event::EventId;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::state::trace::{render_debug, render_subject, Record, RecordKind};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::{PendingTrigger, TriggerTier};
use crate::types::zones::Zone;
use crate::ui::choice_types::{ChoiceContext, ChoiceOption};
use crate::ui::decision::PriorityAction;

/// A record of `kind`, its `batch` field naming the batch open now.
pub(crate) fn in_the_open_batch(kind: RecordKind, game: &GameState) -> Record {
    let mut r = Record::new(kind);
    r.field_opt_u64("batch", game.events.current_batch().map(|b| b.0));
    r
}

/// `batch` — the proposals as they entered `execute_batch_inner`, and the
/// subject groups CR 616.1 decides them in, in APNAP order.
pub(crate) fn batch(game: &GameState, groups: &[SubjectGroup], inherited: usize) -> Record {
    let mut r = in_the_open_batch(RecordKind::Batch, game);
    r.field_u64("depth", game.nesting.batch_depth as u64);
    r.field_u64("inherited", inherited as u64);
    let mut proposals: Vec<&BatchMember> = groups.iter().flat_map(|g| &g.members).collect();
    proposals.sort_by_key(|m| m.place);
    r.key("members").begin_array();
    for m in proposals {
        r.begin_object();
        r.field_u64("i", m.place as u64);
        r.field_opt_str("action", m.event.as_ref().map(render_debug).as_deref());
        r.end();
    }
    r.end();
    r.key("groups").begin_array();
    for group in groups {
        r.begin_object();
        r.field_str("subject", &render_subject(group.subject));
        let places: Vec<usize> = group.members.iter().map(|m| m.place).collect();
        r.field_usizes("members", &places);
        r.end();
    }
    r.end();
    r
}

/// `batch_end` — what each proposal became once the survivors were performed:
/// its event as decided, or `null` for one CR 614.6 dropped. `decided` is
/// rendered ahead of the perform loop, which consumes the decisions.
pub(crate) fn batch_end(game: &GameState, decided: &[Option<String>], riders: usize) -> Record {
    let mut r = in_the_open_batch(RecordKind::BatchEnd, game);
    r.field_u64("depth", game.nesting.batch_depth as u64);
    r.key("members").begin_array();
    for (i, action) in decided.iter().enumerate() {
        r.begin_object();
        r.field_u64("i", i as u64);
        r.field_opt_str("performed", action.as_deref());
        r.end();
    }
    r.end();
    r.field_u64("riders", riders as u64);
    r
}

/// Which entry a `layer_walk` came through, and how it classified the object.
#[derive(Clone, Copy)]
pub(crate) enum WalkKind {
    /// A member of the working set, walked by the board pass.
    Member,
    /// In the battlefield zone with no entity, walked as a member of one pass.
    ZoneOnly,
    /// A card in a library or a hand the pass leaves out, walked alone with
    /// the pass's notes.
    LeftOut,
    /// An object no row reaches, walked alone.
    NonMember,
    /// CR 614.12's look-ahead: the object as it would exist on the battlefield.
    Entering,
    /// CR 603.10a's capture: the frame a leaving permanent's event will store.
    Lki,
}

impl WalkKind {
    fn name(self) -> &'static str {
        match self {
            WalkKind::Member => "member",
            WalkKind::ZoneOnly => "zone_only",
            WalkKind::LeftOut => "left_out",
            WalkKind::NonMember => "non_member",
            WalkKind::Entering => "entering",
            WalkKind::Lki => "lki",
        }
    }
}

/// `layer_walk` — one top-level walk: what was asked, how the entry
/// classified it, how many frames the answer cost (`frames_before` is the
/// diagnostics count read before the walk), and the answer's shape. The type
/// set is sorted because a `HashSet`'s order is the process's.
pub(crate) fn layer_walk(
    game: &GameState,
    id: ObjectId,
    kind: WalkKind,
    frames_before: u64,
    frame: &EffectiveCharacteristics,
) -> Record {
    let mut r = Record::new(RecordKind::LayerWalk);
    r.field_u64("object", id.raw());
    r.field_str("membership", kind.name());
    r.field_u64("epoch", game.layer_epoch());
    r.field_u64("frames", game.diagnostics.layer_frames() - frames_before);
    r.field_str("name", &frame.name);
    let mut types: Vec<String> = frame.types.iter().map(|t| format!("{:?}", t)).collect();
    types.sort();
    r.field_strs("types", &types);
    r.field_opt_i64("power", frame.power.map(i64::from));
    r.field_opt_i64("toughness", frame.toughness.map(i64::from));
    r.field_u64("controller", frame.controller as u64);
    r
}

/// `decision` — a prompt at the decision boundary, as far as every primitive
/// shares it: who was asked, which primitive, which `ChoiceKind` (its variant
/// name and its subject — main item 141's payload rule, so a
/// `SelectRecipients` does not drag its filter tree in), the answer it last
/// rejected when it is asking again, and the options by id. The validator
/// that calls this adds the primitive's own bounds and the answer.
pub(crate) fn decision(
    prompt: &str,
    player: PlayerId,
    ctx: &ChoiceContext,
    options: &[ChoiceOption],
) -> Record {
    let mut r = Record::new(RecordKind::Decision);
    r.field_u64("player", player as u64);
    r.field_str("prompt", prompt);
    r.field_str("choice", ctx.kind.as_str());
    r.field_opt_u64("subject", ctx.kind.subject().map(|o| o.raw()));
    r.field_opt_str("rejected", ctx.rejected.as_ref().map(render_debug).as_deref());
    let rendered: Vec<String> = options.iter().map(render_option).collect();
    r.field_strs("options", &rendered);
    r
}

/// `trigger` — one matcher decision (`codebase-state.md` "Before Triggered
/// abilities" item 9): the record asked about, the candidate's identity and
/// zone, whether it matched, and which predicate refused it when it did not.
/// `mana` says the match resolved at dispatch and never reached the queue
/// (CR 605.4a).
pub(crate) fn trigger(
    game: &GameState,
    record: EventId,
    identity: &AbilityIdentity,
    zone: Zone,
    matched: bool,
    refused_by: Option<&str>,
    mana: bool,
) -> Record {
    let mut r = Record::new(RecordKind::Trigger);
    r.field_u64("record", record.0 as u64);
    r.field_u64("source", identity.source.id.raw());
    r.field_str("name", &crate::ui::display::printed_name(game, identity.source.id));
    r.field_str("ability", &identity.ability.to_string());
    r.field_str("zone", &format!("{:?}", zone));
    r.field_bool("matched", matched);
    r.field_opt_str("refused_by", refused_by);
    r.field_bool("mana", mana);
    r
}

/// `trigger` for a state trigger (CR 603.8): the state check's decision. It
/// names no record, since no event triggered it, and says `state`; `after` is
/// the newest record when the board was asked, which is all a reader can say
/// about when the state held.
pub(crate) fn state_trigger(game: &GameState, identity: &AbilityIdentity, zone: Zone, refused_by: Option<&str>) -> Record {
    let mut r = Record::new(RecordKind::Trigger);
    r.field_bool("state", true);
    r.field_opt_u64("after", game.events.next_seq().0.checked_sub(1).map(|newest| newest as u64));
    r.field_u64("source", identity.source.id.raw());
    r.field_str("name", &crate::ui::display::printed_name(game, identity.source.id));
    r.field_str("ability", &identity.ability.to_string());
    r.field_str("zone", &format!("{:?}", zone));
    r.field_bool("matched", refused_by.is_none());
    r.field_opt_str("refused_by", refused_by);
    r
}

/// `pending` — one entry leaving the queue at placement (CR 603.3): onto
/// the stack as `object`, with the targets it announced, or refused with
/// the reason — CR 800.4d's departed controller, or CR 603.3d's "no legal
/// choices".
pub(crate) fn pending(
    game: &GameState,
    entry: &PendingTrigger,
    object: Option<ObjectId>,
    refused_by: Option<&str>,
    targets: &[TargetInstance],
) -> Record {
    let identity = entry.origin.identity();
    let mut r = Record::new(RecordKind::Pending);
    r.field_u64("trigger", entry.id.0);
    r.field_u64("tier", match entry.tier() { TriggerTier::First => 1, TriggerTier::Second => 2 });
    r.field_u64("controller", entry.controller as u64);
    r.field_u64("source", identity.source.id.raw());
    r.field_str("name", &crate::ui::display::printed_name(game, identity.source.id));
    r.field_str("ability", &identity.ability.to_string());
    let records: Vec<u64> = entry.binding.records().map(|r| r.seq.0 as u64).collect();
    r.field_u64s("records", &records);
    r.field_opt_u64("object", object.map(|id| id.raw()));
    r.field_opt_str("refused_by", refused_by);
    let rendered: Vec<String> = targets
        .iter()
        .flat_map(|t| t.as_resolved_targets().map(|target| render_debug(&target)))
        .collect();
    r.field_strs("targets", &rendered);
    r
}

/// One option as the record spells it: an object by `#id`, a player by
/// `P<n>`, a priority action by its variant and id, the rest by `{:?}`.
fn render_option(option: &ChoiceOption) -> String {
    match option {
        ChoiceOption::Object(id) => id.to_string(),
        ChoiceOption::Player(p) => format!("P{}", p),
        ChoiceOption::Action(a) => render_debug(a),
        other => render_debug(other),
    }
}

/// `priority_rejected` — an action the prompt offered and the engine refused
/// (A4h, `codebase-state.md` "Before Triggered abilities" item 5): the
/// `decision` before this is the list that offered it, the one after is the
/// re-ask — unless the re-ask offers `Pass` alone, which the engine takes
/// without one (`engine::priority`) — and this is what happened in between,
/// which no event log can show because a rejected action performs nothing.
/// The error is the engine's own text, which the re-ask's
/// `ChoiceContext::rejected` names by rule instead. `retry` counts this
/// window's rejections.
pub(crate) fn priority_rejected(
    player: PlayerId,
    action: &PriorityAction,
    error: &str,
    retry: usize,
) -> Record {
    let mut r = Record::new(RecordKind::PriorityRejected);
    r.field_u64("player", player as u64);
    r.field_str("action", &render_debug(action));
    r.field_str("error", error);
    r.field_u64("retry", retry as u64);
    r
}
