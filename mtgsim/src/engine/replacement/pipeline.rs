//! The CR 616.1 loop (`replacement-architecture.md` §4.1).

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::restriction::{is_prohibited, Query};
use crate::state::game_state::GameState;
use crate::types::effects::{AffectedSet, AmountExpr, Effect, PermanentFilter, PlayerRef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{
    AuxiliaryMove, EnterMods, EnterModsTemplate, GameActionTemplate, Rewrite, Uses,
};
use crate::types::zones::Zone;
use crate::oracle::characteristics::get_effective_power;
use crate::ui::ask::ask_apply_optional_replacement;
use crate::ui::ask::ask_choose_auxiliary_zone_change;
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
        } else if order_invariant_entry_bucket(&bucket, subject_object(subject)) {
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
/// - **Every amount is constant, or is read off a source that is not the
///   entering object** (RC-5). The commuting half of the theorem was free
///   while `EnterModsTemplate` held literals. It is not free now: a
///   `SourcePower` amount is evaluated through `EntryFrame::frame_of(source)`,
///   which answers the *hypothetical* permanent when the source is the
///   entering object — so "enters with a counter for each point of its own
///   power" gives a different number before and after another member's
///   `-1/-1`. Master Biomancer is the reason this is the exact premise and not
///   the conservative "all `Fixed`": its source is a real permanent, so its
///   amount is read off the board and commutes, and Root Maze beside a
///   Biomancer keeps its suppressed prompt.
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
/// **The name is the implementation's, not the question's** — reported in
/// review, recorded as `codebase-state.md` item 65. The question is "does
/// CR 616.1's ordering prompt here have more than one outcome"; "bucket" is
/// 616.1a–e's forced-choice class, which a reader has to already know.
fn order_invariant_entry_bucket(
    bucket: &[ReplacementInstance],
    entering: Option<ObjectId>,
) -> bool {
    bucket.iter().all(|c| {
        (match &c.def.rewrite {
            // Reads the frame only when the source is the object being
            // computed, so anything else is a board read and commutes.
            Rewrite::EnterWith(t) => t.is_fixed() || Some(c.source) != entering,
            _ => false,
        }) && !c.def.optional
            && c.def.then.is_none()
            && matches!(c.def.uses, Uses::Static)
            && !c.def.exempt_from_614_5
            && !matches!(c.id, ReplacementInstanceId::Counter(..))
            && affected_is_mods_invariant(&c.def.affected)
    })
}

/// Can no `EnterMods` field change whether this set matches the entering
/// object? `SourceOnly`, `Fixed` and `AttachedToSource` match by id; a
/// `Filter` is invariant iff every leaf is.
fn affected_is_mods_invariant(affected: &AffectedSet) -> bool {
    match affected {
        AffectedSet::SourceOnly | AffectedSet::Fixed(_) | AffectedSet::AttachedToSource => true,
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
/// `game` and `ctx` are for the arms that ask a question as they apply:
/// `EnterWith` asks CR 614.17d whether its counters may be put on and
/// evaluates its amounts, `EnterUnderControlOf(Opponent)` may have to ask a
/// player which opponent, and `EnterAfterMoving` prompts for a set of objects
/// and moves them.
///
/// **`&mut` because of that last one, and only that one.** CR 614.13 is the
/// rules' own statement that applying an entry replacement may change the
/// board, so a rewrite is no longer a pure function of the event. Every other
/// arm still is, and the mutation goes through `execute_actions_new_batch`
/// rather than a direct write — the chokepoint invariant does not have an
/// exception here.
fn apply_rewrite(
    game: &mut GameState,
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
        Rewrite::EnterWith(template) => match event {
            GameAction::EnterBattlefield { object, from, controller, mut mods, cause } => {
                let extra = evaluate_enter_template(
                    game, template, chosen.source, object, controller, &mods,
                )?;
                let extra = strip_prohibited_counters(
                    game, object, controller, &mods, &extra, Some(chosen.controller),
                );
                mods.merge(&extra);
                Ok(Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }))
            }
            other => Err(format!(
                "replacement {:?} modifies how a permanent enters but matched {:?}, \
                 which is not an entry. Its `EventPattern` and its `Rewrite` \
                 describe different events.",
                chosen.id, other
            )),
        },

        // CR 614.13 — choose, move, and count.
        Rewrite::EnterAfterMoving(aux) => match event {
            GameAction::EnterBattlefield { object, from, controller, mut mods, cause } => {
                let extra = apply_auxiliary_move(game, ctx, chosen, aux, object, controller)?;
                let extra = strip_prohibited_counters(
                    game, object, controller, &mods, &extra, Some(chosen.controller),
                );
                mods.merge(&extra);
                Ok(Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }))
            }
            other => Err(format!(
                "replacement {:?} moves other objects as a permanent enters (CR 614.13) \
                 but matched {:?}, which is not an entry. Its `EventPattern` and its \
                 `Rewrite` describe different events.",
                chosen.id, other
            )),
        },

        // CR 616.1b — the controller field of the proposal, and only that.
        Rewrite::EnterUnderControlOf(player_ref) => match event {
            GameAction::EnterBattlefield { object, from, mods, cause, .. } => {
                let controller = entering_controller(game, ctx, chosen, object, player_ref)?;
                Ok(Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }))
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
            // it instead". The entry is the zone change (CR 614.1c), so the
            // substitute is a zone change from where the card is — one move,
            // no hop through the battlefield — and the card never becomes a
            // permanent: `PermanentEnteredBattlefield` is the entry
            // performer's to emit, and it never runs.
            //
            // A token has no `from`. It is created in the battlefield zone and
            // sits there with no entity until its entry is decided
            // (`Primitive::CreateToken`), so the substitute moves it out of
            // that zone, and the log says `from: Battlefield` for a token
            // CR 111 says was created in exile. That is the cheap answer, on
            // record under Phase RE, whose `CreateTokens` proposal is where a
            // creation's destination belongs (`replacement-architecture.md`
            // §9, RC-4b).
            (
                GameActionTemplate::ZoneChangeTo { to, cause },
                GameAction::EnterBattlefield { object, from, .. },
            ) => Ok(Some(GameAction::ZoneChange {
                object,
                from: from.unwrap_or(Zone::Battlefield),
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

/// Turn an [`EnterModsTemplate`] into the [`EnterMods`] the event carries, by
/// evaluating each amount at the moment the effect is applied.
///
/// **Which board an amount reads is the whole of §5b, and it falls out of
/// `frame_of` rather than being decided here.** The frame is built for the
/// *entering* permanent with the mods applied so far (CR 614.12 clause 1), and
/// `frame_of(source)` answers only when the source *is* that permanent — so an
/// entering creature's own "with a counter for each ..." reads its hypothetical
/// self, and Master Biomancer, a permanent already on the battlefield, is read
/// off the real board. Elvish Archdruid entering under Biomancer therefore gets
/// **2** counters and not 3: Archdruid's own anthem is in Archdruid's frame and
/// in nothing else's.
///
/// A negative power is CR 122.6a's nothing rather than an error — no counters
/// are put on — matching `add_counters`' `u32`.
fn evaluate_enter_template(
    game: &GameState,
    template: &EnterModsTemplate,
    source: ObjectId,
    entering: ObjectId,
    controller: PlayerId,
    so_far: &EnterMods,
) -> Result<EnterMods, String> {
    let mut out = EnterMods { tapped: template.tapped, counters: Vec::new() };
    if template.counters.is_empty() {
        return Ok(out);
    }
    let frame = EntryFrame::for_entering(game, entering, controller, so_far);
    for (counter, amount) in &template.counters {
        let n = match amount {
            AmountExpr::Fixed(n) => *n as i64,
            AmountExpr::SourcePower => match frame.frame_of(source) {
                Some(chars) => chars.power.unwrap_or(0) as i64,
                None => get_effective_power(game, source).unwrap_or(0) as i64,
            },
            other => {
                return Err(format!(
                    "an entry replacement on {} gives counters with amount {:?}, which \
                     `evaluate_enter_template` cannot read. CR 614.12 asks this question \
                     of a permanent that does not exist yet, so an amount here has to \
                     name either a constant or the effect's own source.",
                    source, other
                ))
            }
        };
        if n > 0 {
            out.counters.push((*counter, n as u32));
        }
    }
    Ok(out)
}

/// CR 614.13's application: choose a number of objects, move them, and report
/// what the entering permanent gets for it.
///
/// The three things §9 said did not exist, in order.
///
/// 1. **The prompt.** Candidates are filtered by CR 614.13a/b
///    (`GameState::entry_selection`) and by CR 101.2 before they are offered,
///    which is `sacrifice_of_choice`'s axis-1 shape: a Sigarda that makes a
///    creature unsacrificeable removes it from the list rather than letting it
///    be chosen and then refusing the move — the count would be wrong either
///    way, and only one of them is a *choice* the player was allowed to make.
/// 2. **The moves.** Proposed, never written: `execute_actions_new_batch` puts
///    them through the whole CR 614 pipeline with fresh applied sets
///    (CR 614.5), so a finality counter still exiles a devoured creature and
///    Kalitas still gets its Zombie.
/// 3. **The count.** What was *performed*, not what was picked. A move a
///    replacement dropped entirely never happened (CR 614.6), so it is not a
///    creature that was sacrificed; a move some effect redirected still is one.
///
/// CR 614.13b is recorded **before** the moves, so a nested batch cannot lose
/// it and the state a fork would need at the next prompt is already in
/// `GameState` (`codebase-state.md` item 40).
fn apply_auxiliary_move(
    game: &mut GameState,
    ctx: &ActionContext,
    chosen: &ReplacementInstance,
    aux: &AuxiliaryMove,
    entering: ObjectId,
    entering_controller: PlayerId,
) -> Result<EnterMods, String> {
    // "You" for CR 614.13a's choice. For devour and Sutured Ghoul the effect is
    // the entering permanent's own, so this is the controller it will enter
    // under (CR 110.2b, off the proposal); for a filter-scoped effect it is the
    // other permanent's controller, which is what the card's "you" means.
    let you = chosen.controller;
    let _ = entering_controller;

    let candidates = auxiliary_candidates(game, aux, you)?;
    if candidates.is_empty() {
        return Ok(EnterMods::NONE);
    }
    let max = aux
        .up_to
        .map(|n| n as usize)
        .unwrap_or(candidates.len())
        .min(candidates.len());
    if max == 0 {
        return Ok(EnterMods::NONE);
    }

    let picked = ask_choose_auxiliary_zone_change(
        ctx.dp, game, you, entering, chosen.source, aux.to, &candidates, max,
    );
    if picked.is_empty() {
        return Ok(EnterMods::NONE);
    }
    game.entry_selection.chosen.extend(picked.iter().copied());

    // AUXILIARY-MOVE: CR 614.13's moves are not a result of the entry — they
    // are performed while it is still being decided — so they get their own
    // batch id rather than joining the one this pipeline is inside
    // (`replacement-architecture.md` §4.2).
    let batch: Vec<GameAction> = picked
        .iter()
        .map(|&id| GameAction::ZoneChange {
            object: id,
            from: aux.from,
            to: aux.to,
            cause: aux.cause,
        })
        .collect();
    let moved = game.execute_actions_new_batch(batch, ctx)?.len() as u32;

    Ok(match aux.per_chosen {
        Some((counter, per)) if moved > 0 => EnterMods::with_counters(counter, moved * per),
        _ => EnterMods::NONE,
    })
}

/// Every object CR 614.13a lets this effect choose.
///
/// **Candidate scoping differs by zone, and the asymmetry is the CR's.** A
/// player zone *is* the scope — "creature cards from **your** graveyard" is the
/// chooser's own graveyard, in its own order — while the battlefield is one
/// shared zone where control and ownership diverge, so "creatures you control"
/// has to be a `ByController(You)` leaf on the effect's own filter. Both walk
/// an ordered collection, because a candidate list reaches a `DecisionProvider`
/// by index.
///
/// A zone with no card asking for it is an error rather than an empty list: an
/// arm the pipeline cannot apply is worse than a missing one, and a silently
/// empty candidate set is a devour that never devours.
fn auxiliary_candidates(
    game: &GameState,
    aux: &AuxiliaryMove,
    you: PlayerId,
) -> Result<Vec<ObjectId>, String> {
    let ids: Vec<ObjectId> = match aux.from {
        Zone::Battlefield => game.battlefield_ids_ordered(),
        Zone::Graveyard => game.players[you].graveyard.clone(),
        other => {
            return Err(format!(
                "CR 614.13 auxiliary move reads {:?}, which has no candidate enumeration. \
                 The battlefield and a player's graveyard are the two the printed cards \
                 use; a third needs its ordering and its ownership scope stated.",
                other
            ))
        }
    };

    Ok(ids
        .into_iter()
        .filter(|&id| {
            // CR 614.13a/b, ahead of everything else: an excluded object is not
            // a candidate that fails a check, it is not a candidate.
            if !game.entry_selection.admits(id) {
                return false;
            }
            if !game.permanent_matches_filter(id, &aux.filter, you).unwrap_or(false) {
                return false;
            }
            // CR 101.2 on the move this choice would produce — the axis-1
            // question `sacrifice_of_choice` asks, for the same reason.
            //
            // `cause` is a `PlayerId` and not a richer context because that is
            // the only question `SourceFilter` asks: its one variant is
            // `ControlledBy(PlayerRef)`, and the printed population it serves —
            // Sigarda's "spells and abilities **your opponents** control",
            // Tamiyo's mirror of it — reads control and nothing else. A
            // restriction that scoped by *which* ability ("abilities of
            // creatures you control can't …") would need the source object
            // here, and the field widens with the `SourceFilter` variant that
            // needs it rather than ahead of it.
            //
            // The player is the effect's controller, which is why Sigarda does
            // **not** stop her own controller's devour: her filter is
            // `Opponent`, relative to her own controller.
            !is_prohibited(
                game,
                &Query::Event {
                    action: &GameAction::ZoneChange {
                        object: id,
                        from: aux.from,
                        to: aux.to,
                        cause: aux.cause,
                    },
                    cause: Some(you),
                    lookahead: None,
                },
            )
        })
        .collect())
}
