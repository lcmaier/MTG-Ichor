//! The CR 616.1 loop (`replacement-architecture.md` §4.1).

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::restriction::{is_prohibited, Query};
use crate::state::game_state::GameState;
use crate::types::effects::{AffectedSet, Effect, PermanentFilter, PlayerRef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{EnterMods, GameActionTemplate, Rewrite, Uses};
use crate::types::zones::Zone;
use crate::ui::ask::ask_apply_optional_replacement;
use crate::ui::ask::ask_choose_entering_controller;
use crate::ui::ask::ask_choose_replacement;

use super::gather::{applies_to, forced_bucket};
use super::{
    chooser_for, gather, subject_of, EntryFrame, EventSubject, ReplacementInstance,
    ReplacementInstanceId,
};

/// The "and also" half of an applied replacement, queued for after the event.
///
/// **Two rules, depending on which kind of effect queued it**, and only one of
/// them is CR 615.5:
///
/// > 615.5. Some *prevention* effects also include an additional effect ... The
/// > prevention takes place at the time the original event would have happened;
/// > the rest of the effect takes place immediately afterward.
///
/// A rider on a plain replacement has no such rule and needs none. CR 614.1a's
/// "instead" plus the effect's own text make the additional action part of what
/// happens instead — Kalitas exiles the creature *and* makes a Zombie — and CR
/// 614.6 says "a modified event occurs instead". Performing the substituted
/// event and then the rest is that modified event, in order.
///
/// Queued when its replacement is *applied* and resolved by the caller after
/// the surviving event is performed — never mid-loop. During the loop nothing
/// has happened yet: the loop is deciding what the event *is*, so a rider run
/// inside it would run before the event it rides on.
#[derive(Debug, Clone)]
pub(crate) struct Rider {
    /// The object whose effect this belongs to. Becomes the rider's
    /// `ResolutionContext::source`.
    pub source: ObjectId,
    pub controller: PlayerId,
    /// The event's subject, when it had one. Becomes the rider's single
    /// resolved target, so `EffectRecipient::Target` in a `then` names the
    /// permanent the replacement was about.
    pub subject: Option<ObjectId>,
    pub effect: Effect,
}

/// CR 614.7a / 120.8 / 119.10 — the proposals that describe an event which
/// never happens, so there is nothing for a replacement effect to replace.
///
/// > 120.8. If a source would deal 0 damage, it does not deal damage at all.
/// > ... replacement effects that would increase the damage dealt by that
/// > source, or would have that source deal that damage to a different object
/// > or player, have no event to replace, so they have no effect.
///
/// **The proposal side is where this rule lives, not the performer.**
/// `EventPattern::DealDamage` carries no amount constraint, so a 0-damage
/// proposal that reaches the loop is one a shield counter's prevention half
/// applies to — spending a counter (CR 615.5's rider) on an event CR 614.7a
/// says never happened. Combat cannot produce one, because CR 510.1a filters
/// 0-power attackers out of the assignment, but `Primitive::DealDamage` does no
/// such filtering: any X=0 or computed-0 damage effect proposes it.
///
/// Re-asked every iteration, like `is_blocked`: CR 616.1f rewrites the event
/// between iterations, and a prevention that reduces damage to 0 arrives here
/// by that road.
///
/// Life *gain* is here on CR 119.10's own words — "if a player gains 0 life, no
/// life gain event would occur, and these effects won't apply". Life *loss* and
/// counter changes of 0 have no such rule, so their no-op guards stay in
/// `perform_action`, where they are local conveniences rather than CR 614.7a.
fn never_happens(action: &GameAction) -> bool {
    match action {
        GameAction::DealDamage { amount, .. } => *amount == 0,
        GameAction::GainLife { amount, .. } => *amount == 0,
        _ => false,
    }
}

/// The CR 616.1 loop: decide what event actually happens.
///
/// Returns `None` when the event does not happen at all (CR 614.6). Queued
/// riders are pushed onto `riders` in application order and are the caller's to
/// resolve *after* performing the returned event — including when the return is
/// `None`, since CR 615.12 makes a rider unconditional once queued.
///
/// `inherited` is §3.2d's lineage rule: a **decomposed** event continues its
/// parent's applied set (`DrawCards{2}` → two `DrawCard`s), a **contained**
/// event of a different kind starts a fresh one (`CreateTokens` →
/// `EnterBattlefield`). Without inheritance on decomposition, Teferi's Ageless
/// Insight re-applies to its own output and the game hangs — so this parameter
/// is the termination argument, not a nicety.
pub(crate) fn apply_replacements(
    game: &mut GameState,
    action: GameAction,
    ctx: &ActionContext,
    inherited: &HashSet<ReplacementInstanceId>,
    riders: &mut Vec<Rider>,
) -> Result<Option<GameAction>, String> {
    let mut applied: HashSet<ReplacementInstanceId> = inherited.clone();
    // Declining is tracked **separately from CR 614.5's applied set**, and it
    // has to be.
    //
    // CR 903.9b is `exempt_from_614_5`, which means the applied set does not
    // filter it — that is the whole of the exception. It is also `optional`.
    // Put those together with §4.1's decline path, which marks the effect
    // applied and continues, and the loop re-offers the same declined choice
    // forever: the mark is there but the filter ignores it. A hang, not a wrong
    // answer, which is the worst shape of bug.
    //
    // The two sets are genuinely different questions. CR 614.5 is about
    // *applying* more than once, and 903.9b's exception is to that. Declining
    // is a final answer about this event, and no rule exempts anything from it.
    let mut declined: HashSet<ReplacementInstanceId> = HashSet::new();
    // Which exempt effect has applied, if any — see `check_exempt_terminates`,
    // which owns the whole termination argument for the effects CR 614.5 does
    // not govern.
    let mut exempt_applied: Option<ReplacementInstanceId> = None;
    let mut event = action;

    // Unbounded on purpose. **Every iteration consumes something finite**, and
    // the three things that guarantee it are each enforced in code rather than
    // asserted here: CR 614.5's `applied` set, the `declined` set, and
    // `check_exempt_terminates` for the one class CR 614.5 exempts. A candidate
    // pool cannot grow mid-loop either — `apply_rewrite` only rewrites the
    // proposal and riders are queued rather than run (§4.1a), so nothing
    // touches the board between iterations.
    loop {
        // CR 614.7a: an event that never happens has no replacement to make,
        // and any rider it queued would be spent on nothing. Ahead of even the
        // "can't" check, because there is no event here to forbid.
        if never_happens(&event) {
            return Ok(None);
        }

        // CR 614.12 / 614.17d — the frame both questions below read for an
        // entering permanent, built once per iteration and computed only if a
        // filter asks. Per iteration and not per event, because clause (1)
        // says the frame accounts for the replacements already applied.
        let frame = EntryFrame::new(game, &event);

        // CR 614.17: a "can't" is checked ahead of the pipeline and wins
        // (CR 101.2). Not a `ReplacementDef` and never one — modelling it as one
        // would have put it in the CR 616.1 choice list, where a player could
        // decline it.
        //
        // Re-asked on every iteration rather than once at the top, because
        // CR 614.17c lets a self-replacement change the event's *type*, and an
        // event of a different type is a different "can't" question.
        let blocked = is_prohibited(
            game,
            &Query::Event {
                action: &event,
                // CR 101.2 scoped by cause (§2.6). `ActionContext` already
                // threads the resolution that proposed this; a turn-based or
                // state-based action has none, and no `SourceFilter` matches it.
                cause: ctx.resolution.map(|r| r.controller),
                lookahead: Some(&frame),
            },
        );

        // CR 614.4 — gathered against live state at the moment of proposal.
        // There is no "go back in time" path because there is no other place
        // to ask.
        let applicable: Vec<ReplacementInstance> = gather(game, &event, ctx, blocked, &frame)
            .into_iter()
            // CR 614.5, with CR 903.9b as the rules' only stated exception.
            .filter(|c| c.def.exempt_from_614_5 || !applied.contains(&c.id))
            // No exception to this one — see `declined`.
            .filter(|c| !declined.contains(&c.id))
            .collect();

        if applicable.is_empty() {
            return Ok(if blocked { None } else { Some(event) });
        }

        // CR 616.1a–e.
        let bucket = forced_bucket(applicable);

        // CR 616.1 / 400.6 — the affected object's controller (or its owner if
        // it has no controller) or the affected player.
        let subject = subject_of(&event);
        let chooser = chooser_for(game, &event);

        // **Never prompt with fewer than two candidates.** CR-correct (there is
        // no choice to make with one), and it is what keeps every existing
        // `ScriptedDecisionProvider` test green rather than drowning it in
        // unexpected prompts now that every `execute_action` traverses this
        // loop. If a phase finds itself relaxing this to make something work,
        // it has found a design error, not a test problem
        // (`replacement-architecture.md` §11 item 7).
        //
        // **And never prompt for a choice with one outcome** — §11 item 19.
        // `order_invariant_entry_bucket` is the provable form of that rule,
        // and `unsuppressed` is what the debug build checks it against after
        // the rewrite below.
        let mut unsuppressed: Vec<ReplacementInstanceId> = Vec::new();
        let chosen = if bucket.len() == 1 {
            bucket.into_iter().next().expect("len checked")
        } else if order_invariant_entry_bucket(&bucket) {
            let mut members = bucket.into_iter();
            let first = members.next().expect("len checked");
            unsuppressed = members.map(|c| c.id).collect();
            first
        } else {
            let Some(chooser) = chooser else {
                // Nobody to ask. An object with neither controller nor owner is
                // not a board state the engine can produce, so this is a bug
                // rather than a rules corner — and taking the first candidate
                // silently would make it an unreproducible one.
                return Err(format!(
                    "CR 616.1 needs a chooser for {:?} and the affected object has \
                     neither a controller nor an owner",
                    subject
                ));
            };
            let index = ask_choose_replacement(
                ctx.dp,
                game,
                chooser,
                subject_object(subject),
                &bucket,
            );
            bucket.into_iter().nth(index).expect("index validated by ask_*")
        };

        // "You **may** ... instead". Declining marks it applied but does not
        // consume a use: being offered and refusing *is* CR 614.5's one
        // opportunity, and without the mark the loop re-gathers the same
        // candidate forever — a hang rather than a wrong answer. Not consuming
        // the use is what leaves a regeneration shield intact for the next
        // event.
        if chosen.def.optional {
            let chooser = chooser.ok_or_else(|| {
                format!("optional replacement on {:?} has no player to ask", subject)
            })?;
            if !ask_apply_optional_replacement(
                ctx.dp,
                game,
                chooser,
                subject_object(subject),
                &chosen,
            ) {
                // Marking it applied is CR 614.5's "one opportunity" — being
                // offered and refusing *is* the opportunity.
                applied.insert(chosen.id);
                declined.insert(chosen.id);
                continue;
            }
        }

        if !chosen.def.exempt_from_614_5 {
            applied.insert(chosen.id);
        }
        consume_use(game, &chosen);

        // Queued, not resolved (§4.1a). A later replacement in the same loop
        // further modifying or even dropping the event does not un-queue this.
        if let Some(then) = chosen.def.then.clone() {
            riders.push(Rider {
                source: chosen.source,
                controller: chosen.controller,
                subject: subject_object(subject),
                effect: then,
            });
        }

        match apply_rewrite(game, ctx, &chosen, event, subject)? {
            // CR 614.6 — the event does not happen. Queued riders still run.
            None => return Ok(None),
            // CR 616.1f — re-gather against the modified event, which is how
            // CR 616.2's "a replacement effect can become applicable as the
            // result of another" works without any special case.
            Some(next) => {
                check_exempt_terminates(game, &chosen, &next, &mut exempt_applied)?;
                check_order_invariance(game, ctx, &next, &unsuppressed);
                event = next;
            }
        }
    }
}

/// §11 item 19 — is this CR 616.1 bucket one whose ordering choice provably
/// cannot change the outcome, so the prompt is noise?
///
/// The theorem, and every clause of the predicate is a premise of it:
///
/// - **`EnterWith` only.** `EnterMods::merge` is `|=` and `+`, commutative and
///   associative, so the mods a member adds land the same whatever went
///   before. A `Prevent` or an `Instead` drops or replaces the event, and
///   whether the members after it ever apply is then a real question.
/// - **No rider.** Riders queue in choice order and run in queue order
///   (CR 615.5), so two members that both carry one make the order observable
///   in the event log even when the board is identical.
/// - **Mandatory, static, under CR 614.5, not counter-derived.** An optional
///   is a second prompt whose answer can differ per order; a `Uses::Once`
///   spends a registry row; an exempt effect may re-apply; a counter-derived
///   instance is re-synthesized per gather. Each is excluded so the argument
///   has nothing to say about it.
/// - **Applicability cannot depend on what the others add.** This is the
///   clause the CR 614.12 frame made necessary: `set_affects` now reads the
///   pending `EnterMods` through the look-ahead, so a `PowerLE` filter can
///   match before a `-1/-1` counter lands and stop matching after. Adaptive
///   Shimmerer under "creatures with power 1 or less enter tapped" enters
///   tapped or untapped depending on which applies first, and *that* prompt
///   is real. `affected_is_mods_invariant` admits only leaves no `EnterMods`
///   field can move.
///
/// With every member's applicability fixed and every application commuting,
/// each member applies exactly once in any order (CR 614.5) and the final
/// `EnterMods` is the merge of all of them. Root Maze beside Idyllic
/// Beachfront — the fuzz harness's every land drop under Root Maze — is the
/// case this exists for.
///
/// **A semantics-assuming shortcut, and it carries its expiry conditions**
/// (`layers-architecture.md` §12 item 3). It goes false the day
/// `EnterMods` gains a field that feeds a characteristic — face-down, which
/// is Layer 1 and changes everything — or `PermanentFilter` gains a leaf that
/// reads P/T, keywords or counters, or `EventPattern::EnterBattlefield` reads
/// `mods`. `check_order_invariance` is the debug-build check that computes it
/// the other way, and `codebase-state.md` records the three conditions.
fn order_invariant_entry_bucket(bucket: &[ReplacementInstance]) -> bool {
    bucket.iter().all(|c| {
        matches!(c.def.rewrite, Rewrite::EnterWith(_))
            && !c.def.optional
            && c.def.then.is_none()
            && matches!(c.def.uses, Uses::Static)
            && !c.def.exempt_from_614_5
            && !matches!(c.id, ReplacementInstanceId::Counter(..))
            && affected_is_mods_invariant(&c.def.affected)
    })
}

/// Can no `EnterMods` field change whether this set matches the entering
/// object? `SourceOnly` and `Fixed` match by id; a `Filter` is invariant iff
/// every leaf is.
fn affected_is_mods_invariant(affected: &AffectedSet) -> bool {
    match affected {
        AffectedSet::SourceOnly | AffectedSet::Fixed(_) => true,
        AffectedSet::Filter { filter } => filter_is_mods_invariant(filter),
    }
}

/// The leaf table for [`order_invariant_entry_bucket`]'s last premise. Types,
/// subtypes, supertypes, colors, controller, ownership and tokenness are fed
/// by no `EnterMods` field; power is fed by `+1/+1` and `-1/-1` counters
/// (CR 122.1a) and so `PowerLE` is not invariant. Matched exhaustively, so a
/// new leaf has to be classified rather than defaulting to "safe".
fn filter_is_mods_invariant(filter: &PermanentFilter) -> bool {
    match filter {
        PermanentFilter::All
        | PermanentFilter::ByType(_)
        | PermanentFilter::BySubtype(_)
        | PermanentFilter::BySupertype(_)
        | PermanentFilter::ByColor(_)
        | PermanentFilter::ByController(_)
        | PermanentFilter::Token
        | PermanentFilter::ByOwner(_) => true,
        PermanentFilter::PowerLE(_) => false,
        PermanentFilter::And(a, b) | PermanentFilter::Or(a, b) => {
            filter_is_mods_invariant(a) && filter_is_mods_invariant(b)
        }
        PermanentFilter::Not(inner) => filter_is_mods_invariant(inner),
    }
}

/// The debug-build check on [`order_invariant_entry_bucket`]: after the
/// suppressed choice applied, every member it was chosen over must still be
/// applicable to the rewritten event, or the predicate admitted a leaf that
/// reads `EnterMods` and the prompt it skipped was real.
///
/// Computes the theorem's premise the other way, as `layers-architecture.md`
/// §12 item 3 asks of any semantics-assuming shortcut. Debug builds only,
/// because it is a second gather per suppressed prompt and its
/// `record_replacement_gather` would move the fixtures table; the release
/// binary trusts the leaf table.
fn check_order_invariance(
    game: &GameState,
    ctx: &ActionContext,
    next: &GameAction,
    unsuppressed: &[ReplacementInstanceId],
) {
    if !cfg!(debug_assertions) || unsuppressed.is_empty() {
        return;
    }
    let frame = EntryFrame::new(game, next);
    let still: Vec<ReplacementInstanceId> = gather(game, next, ctx, false, &frame)
        .into_iter()
        .map(|c| c.id)
        .collect();
    for id in unsuppressed {
        debug_assert!(
            still.contains(id),
            "the CR 616.1 prompt suppressed as order-invariant was not: {:?} stopped \
             applying to {:?} after the chosen member applied. A `PermanentFilter` leaf \
             or an `EnterMods` field reads something `filter_is_mods_invariant` calls \
             invariant — see `order_invariant_entry_bucket`.",
            id, next
        );
    }
}

/// The termination argument for the one class of effect CR 614.5 does not
/// govern — the whole of it, in one place.
///
/// CR 614.5 is what makes the CR 616.1f loop finite: one opportunity per effect,
/// finitely many effects. CR 903.9b is the rules' only exemption from it, so an
/// exempt effect never enters the `applied` set and nothing there stops the loop
/// re-offering it forever.
///
/// **The rules are safe doing that because of two facts, and this checks both**
/// — they are properties of the `ReplacementDef`, and a `ReplacementDef` is data
/// a card file writes, so the engine cannot assume what the CR can.
///
/// 1. **An exemption's rewrite takes the event out of its own pattern.** 903.9b
///    watches a move to hand or library and produces a move to the command zone,
///    which is neither, so it cannot feed itself. It comes back only if some
///    *other* effect pushes the event toward hand or library again — and every
///    other effect is under CR 614.5, so each re-entry costs an applied-set
///    slot. That is what makes "may apply more than once to the same event"
///    bounded rather than infinite.
/// 2. **At most one exemption applies to one event.** Two exempt effects can
///    each satisfy (1) alone and still rewrite each other's events forever,
///    consuming nothing. 903.9b is the only exemption in the rules, so this
///    costs nothing today; a second one must arrive with a CR cite *and* a
///    fresh termination argument, which is what the error says.
///
/// Together with the `applied` and `declined` sets, this is why the loop needs
/// no iteration cap: every iteration spends a finite resource, and a failure is
/// reported at the application that causes it rather than N iterations later
/// with nothing to point at.
fn check_exempt_terminates(
    game: &GameState,
    chosen: &ReplacementInstance,
    next: &GameAction,
    exempt_applied: &mut Option<ReplacementInstanceId>,
) -> Result<(), String> {
    if !chosen.def.exempt_from_614_5 {
        return Ok(());
    }
    // A spent one-shot cannot be re-gathered whatever its rewrite does
    // (`consume_use` removed the row), so it bounds itself and takes part in
    // neither check — including not claiming the single-exemption slot, which
    // it could not use again anyway.
    if matches!(chosen.def.uses, Uses::Once) {
        return Ok(());
    }

    // Fact 2.
    match *exempt_applied {
        Some(first) if first != chosen.id => {
            return Err(format!(
                "two effects exempt from CR 614.5 applied to one event, {:?} \
                 and {:?}. Each may be individually well-behaved and the two \
                 can still rewrite each other's events forever, spending no \
                 applied-set slot — so the CR 616.1f loop has no termination \
                 argument left. CR 903.9b is meant to be the rules' only \
                 exemption; a second needs a CR cite and a fresh argument.",
                first, chosen.id
            ));
        }
        _ => *exempt_applied = Some(chosen.id),
    }

    // Fact 1.
    let frame = EntryFrame::new(game, next);
    if applies_to(game, chosen, next, subject_of(next), Some(&frame)) {
        return Err(format!(
            "replacement {:?} is exempt from CR 614.5 and still applies to its \
             own output {:?}, so the CR 616.1f loop cannot terminate. An \
             exemption is bounded only by the rewrite taking the event out of \
             the effect's own pattern; this one's `EventPattern` and `Rewrite` \
             describe the same event.",
            chosen.id, next
        ));
    }
    Ok(())
}

fn subject_object(subject: EventSubject) -> Option<ObjectId> {
    match subject {
        EventSubject::Object(id) => Some(id),
        EventSubject::Player(_) => None,
    }
}

/// Spend one application of the chosen effect.
///
/// `Uses::Once` (CR 701.19a's regeneration shield) removes the registry row, so
/// a shield spent on one member of a batch is correctly gone when the next
/// member asks — no batch special-casing needed either way, because this writes
/// game state.
///
/// A counter-derived effect consumes nothing here on purpose: CR 122.1c/d make
/// the counter removal the *substituted event* or the rider, which propose
/// through `execute_action` like every other mutation. See `Uses`' own docs.
fn consume_use(game: &mut GameState, chosen: &ReplacementInstance) {
    match chosen.def.uses {
        Uses::Static => {}
        Uses::Once => {
            if let ReplacementInstanceId::Registered(row) = chosen.id {
                game.replacement_effects.remove(row);
            } else {
                debug_assert!(
                    false,
                    "`Uses::Once` on a {:?}, which has no row to remove. A one-shot \
                     replacement has to live in the registry — a static ability and \
                     a counter are both re-derived on every gather, so 'spent' has \
                     nowhere to be recorded and the effect would apply forever.",
                    chosen.id
                );
            }
        }
    }
}

/// Apply the chosen effect's [`Rewrite`] to the event (`replacement-architecture.md`
/// §3.2b).
///
/// **Takes the event by value.** The CR 616.1f loop replaces its `event` with
/// whatever this returns and never reads the old one again, so borrowing it
/// would only force `Rewrite::EnterWith` to clone an `EnterMods` it is about to
/// own. Clone pressure matters here beyond tidiness: a tree search clones
/// `GameState`, and this loop runs inside every proposal.
///
/// `game` and `ctx` are for the two arms that ask a question as they apply:
/// `EnterWith` asks CR 614.17d whether its counters may be put on, and
/// `EnterUnderControlOf(Opponent)` may have to ask a player which opponent.
fn apply_rewrite(
    game: &GameState,
    ctx: &ActionContext,
    chosen: &ReplacementInstance,
    event: GameAction,
    subject: EventSubject,
) -> Result<Option<GameAction>, String> {
    match &chosen.def.rewrite {
        // CR 614.6 / 615.6.
        Rewrite::Prevent => Ok(None),

        // CR 614.1c/d — the event still happens; only *how* changes.
        //
        // Merged into the proposal rather than substituted for it, which is
        // what makes CR 616.1f's re-gather accumulate: a permanent facing an
        // "enters tapped" and an "enters with two charge counters" comes out
        // the far side with both, in either application order. CR 614.5 is
        // what stops the merge from repeating — the pattern still watches the
        // rewritten event, and the applied set is what makes that terminate
        // rather than loop.
        //
        // CR 101.2 at the door: a "can't have counters put on it" refuses the
        // counters this rewrite would add (CR 614.17d), and the entry goes on
        // without them.
        Rewrite::EnterWith(extra) => match event {
            GameAction::EnterBattlefield { object, controller, mut mods, cause } => {
                let extra = strip_prohibited_counters(
                    game, object, controller, &mods, extra, Some(chosen.controller),
                );
                mods.merge(&extra);
                Ok(Some(GameAction::EnterBattlefield { object, controller, mods, cause }))
            }
            other => Err(format!(
                "replacement {:?} modifies how a permanent enters but matched {:?}, \
                 which is not an entry. Its `EventPattern` and its `Rewrite` \
                 describe different events.",
                chosen.id, other
            )),
        },

        // CR 616.1b — the controller field of the proposal, and only that.
        Rewrite::EnterUnderControlOf(player_ref) => match event {
            GameAction::EnterBattlefield { object, mods, cause, .. } => {
                let controller = entering_controller(game, ctx, chosen, object, player_ref)?;
                Ok(Some(GameAction::EnterBattlefield { object, controller, mods, cause }))
            }
            other => Err(format!(
                "replacement {:?} modifies under whose control a permanent enters but \
                 matched {:?}, which is not an entry. Its `EventPattern` and its \
                 `Rewrite` describe different events.",
                chosen.id, other
            )),
        },

        Rewrite::Instead(template) => match (template, event) {
            (
                GameActionTemplate::ZoneChangeTo { to, cause },
                GameAction::ZoneChange { object, from, .. },
            ) => Ok(Some(GameAction::ZoneChange {
                object,
                from,
                to: *to,
                cause: *cause,
            })),

            // Containment Priest: "if a nontoken creature would enter … exile
            // it instead". The object is already in the battlefield zone with
            // no entity — RC-2's one-`emit`-wide window — so the substitute is
            // a move out of it, which `move_object` performs on an entity-less
            // object without complaint and which the outer `ZoneChange` arm
            // finds already performed. It never becomes a permanent:
            // `PermanentEnteredBattlefield` is the performer's to emit and the
            // performer never runs. (The `ZoneChange` *into* the zone is in the
            // log, though, which is why an ETB trigger must key on the
            // performer's event — `codebase-state.md`.)
            (
                GameActionTemplate::ZoneChangeTo { to, cause },
                GameAction::EnterBattlefield { object, .. },
            ) => Ok(Some(GameAction::ZoneChange {
                object,
                from: Zone::Battlefield,
                to: *to,
                cause: *cause,
            })),

            (GameActionTemplate::RemoveCountersFromAffected { counter, n }, _) => {
                match subject_object(subject) {
                    Some(object) => Ok(Some(GameAction::RemoveCounters {
                        object,
                        counter: *counter,
                        n: *n,
                    })),
                    None => Err(format!(
                        "a `RemoveCountersFromAffected` rewrite on {:?} has no affected \
                         object to take counters from",
                        chosen.id
                    )),
                }
            }

            // A template that cannot be built from this event is a card-
            // authoring error rather than a rules corner: the pattern is what
            // decides which events reach the rewrite, so a mismatch means the
            // two halves of one `ReplacementDef` disagree.
            (GameActionTemplate::ZoneChangeTo { .. }, other) => Err(format!(
                "replacement {:?} rewrites to a zone change but matched {:?}, which is \
                 neither one nor an entry. Its `EventPattern` and its `Rewrite` describe \
                 different events.",
                chosen.id, other
            )),
        },
    }
}

/// Who `Rewrite::EnterUnderControlOf(player_ref)` puts the permanent under,
/// relative to the effect's controller.
///
/// `Opponent` with several opponents is a choice the effect's controller makes
/// — Xantcha's "an opponent of your choice" — and CR 614.12a says it is made
/// before the permanent enters, which this is. Players who have lost are not
/// opponents to choose (CR 800.4a: a player who leaves the game leaves it), so
/// a three-player game down to two asks nothing.
fn entering_controller(
    game: &GameState,
    ctx: &ActionContext,
    chosen: &ReplacementInstance,
    object: ObjectId,
    player_ref: &PlayerRef,
) -> Result<PlayerId, String> {
    let you = chosen.controller;
    Ok(match player_ref {
        PlayerRef::You => you,
        PlayerRef::Player(pid) => *pid,
        PlayerRef::Owner => game
            .objects
            .get(&object)
            .map(|obj| obj.owner)
            .ok_or_else(|| format!("entering object {} is not in the object store", object))?,
        PlayerRef::Opponent => {
            let opponents: Vec<PlayerId> = (0..game.num_players())
                .filter(|&p| p != you && !game.player_lost[p])
                .collect();
            match opponents.as_slice() {
                [] => {
                    return Err(format!(
                        "replacement {:?} puts {} under an opponent's control, and player \
                         {} has no opponent left in the game",
                        chosen.id, object, you
                    ))
                }
                [only] => *only,
                many => ask_choose_entering_controller(ctx.dp, game, you, object, many),
            }
        }
    })
}

/// CR 614.17d meets CR 122.6a: the counters `extra` would give an entering
/// permanent, minus every kind a "can't have counters put on it" refuses.
///
/// Asked as the event it is — `AddCounters`, since CR 122.6 says counters an
/// object "is given as it enters" are *put on* it — against the CR 614.12
/// frame of the object with `so_far` already applied, so "creatures you
/// control" reads the controller it would enter under and the types it would
/// have. Nothing is blocked and nothing is dropped: CR 101.2 refuses the
/// counters, and the entry goes on without them. That is Melira's ruling — a
/// creature you control that would enter with -1/-1 counters enters with none
/// — and Solemnity's.
///
/// `cause` is who is putting them on: the replacement's controller, or `None`
/// for CR 306.5b's loyalty, which a rule gives rather than a player.
///
/// `tapped` passes through untouched. No printed "can't" refuses a status, and
/// ATOM-614.17d-001's "creatures can't enter the battlefield tapped" is a
/// representative the corpus invented; it is claimed for its counters half.
pub(crate) fn strip_prohibited_counters(
    game: &GameState,
    object: ObjectId,
    controller: PlayerId,
    so_far: &EnterMods,
    extra: &EnterMods,
    cause: Option<PlayerId>,
) -> EnterMods {
    if extra.counters.is_empty() {
        return extra.clone();
    }
    let frame = EntryFrame::for_entering(game, object, controller, so_far);
    let mut kept = EnterMods { tapped: extra.tapped, counters: Vec::with_capacity(extra.counters.len()) };
    for &(counter, n) in &extra.counters {
        let action = GameAction::AddCounters { object, counter, n };
        let refused = is_prohibited(
            game,
            &Query::Event { action: &action, cause, lookahead: Some(&frame) },
        );
        if !refused {
            kept.counters.push((counter, n));
        }
    }
    kept
}
