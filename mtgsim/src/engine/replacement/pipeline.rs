//! The CR 616.1 loop (`replacement-architecture.md` §4.1).

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::restriction::{is_prohibited, Query};
use crate::events::event::DamageTarget;
use crate::types::card_types::CardType;
use crate::types::restriction::ReplacementKindFilter;
use crate::state::game_state::GameState;
use crate::types::effects::{AffectedSet, AmountExpr, Effect, ObjectFilter, PlayerRef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{
    AmountRewrite, AuxiliaryMove, EnterMods, EnterModsTemplate, EventPattern, RetargetSpec,
    GameActionTemplate, Rewrite, Uses,
};
use crate::types::zones::Zone;
use crate::oracle::characteristics::{controller_or_owner, get_effective_power, get_effective_types};
use crate::ui::ask::ask_allocate_next_damage;
use crate::ui::ask::ask_apply_optional_replacement;
use crate::ui::ask::ask_choose_auxiliary_zone_change;
use crate::ui::ask::ask_choose_entering_controller;
use crate::ui::ask::ask_choose_replacement;

use super::gather::{applies_to, must_choose_among};
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
    /// The event's subject. Becomes the rider's single resolved target, so
    /// `EffectRecipient::Target` in a `then` names the object or the player the
    /// replacement was about.
    ///
    /// **An [`EventSubject`], not an `Option<ObjectId>`, from RD-1 on.** It was
    /// the latter while every rider rode on an object event, and flattening a
    /// player subject to `None` cost nothing then. Reverse Damage's "you gain
    /// life" and Angel of Suffering's mill are riders on a *player* subject, and
    /// they need to name that player (`replacement-architecture.md` §11
    /// item 16, `codebase-state.md` item 27).
    pub subject: EventSubject,
    /// The amount the replaced event carried when this rider was queued, read
    /// by `AmountExpr::ReplacedAmount` — CR 615.5's "that much"/"that many".
    ///
    /// Taken from the event as the loop sees it, so a rider queued after a
    /// doubler reads the doubled number; `None` for an event with no amount.
    /// Angel of Suffering's "mill twice that many cards" is the customer, and
    /// its ruling that unpreventable damage still mills is why the number is
    /// captured here rather than recomputed after the event.
    pub replaced_amount: Option<u64>,
    /// How much damage the application that queued this rider prevented, read
    /// by `AmountExpr::DamagePrevented` — CR 615.5's "the amount of damage
    /// that was prevented". 0 for a rider on anything but a prevention effect,
    /// and 0 for a prevention effect that prevented nothing, which is the
    /// answer CR 615.12 needs from it (RD-4).
    pub prevented: u64,
    pub effect: Effect,
}

/// What applying a rewrite to one event did — what a rider reads, and what a
/// use is spent by (`replacement-architecture.md` §9, RD decision 7).
#[derive(Debug, Clone, Copy, Default)]
struct Applied {
    /// Did the rewrite change the event? CR 609.7b's "prevents no damage or
    /// replaces no damage" from the other side: a `Uses::Once` effect is spent
    /// only when this is true.
    took_effect: bool,
    /// How much damage it prevented (CR 615.1a's arms only; 0 otherwise).
    ///
    /// **A number and not an `Option`, because 0 is a real answer here rather
    /// than a missing one.** A doubler prevents zero damage; that is what
    /// CR 615.1a's definition says about it, and `AmountRewrite::prevented`
    /// already reports it as 0 for the same reason. An `Option` would make
    /// every reader decide what `None` meant, and every one of them would
    /// answer "treat it as 0".
    ///
    /// The `Option` that *is* meaningful is one level out:
    /// `ResolutionContext::damage_prevented` is `None` outside a CR 615.5
    /// rider — where the question has no answer at all — and `Some(0)` inside
    /// one that prevented nothing, which is the number Reverse Damage's "life
    /// equal to the damage prevented this way" needs.
    prevented: u64,
}

/// One member of a subject group as the loop carries it: its index in the
/// batch, and its proposal as decided so far — `None` once dropped (CR 614.6,
/// 614.7a, 614.17).
struct Member {
    index: usize,
    event: Option<GameAction>,
}

/// One applicable effect in a group's iteration, with the members it applies
/// to (positions into the group's member list, in member order).
struct Candidate {
    instance: ReplacementInstance,
    members: Vec<usize>,
}

/// The amount a proposal carries, for a rider that refers to it (CR 615.5).
///
/// **Matched exhaustively, with no `_` arm**, for `filter_is_mods_invariant`'s
/// reason: a `GameAction` variant added later has to be classified rather than
/// defaulting to "no amount". The failure a fallthrough would cause is quiet at
/// the point it happens and loud in the wrong place — `AmountExpr::ReplacedAmount`
/// would report "no meaning outside a CR 615.5 rider" from inside a rider,
/// which is the one message guaranteed to send a reader looking somewhere else.
///
/// `AddCounters`/`RemoveCounters` carry a count of *counters*, not the amount
/// CR 615.5's "that much" is about, and no rider reads one; the first that does
/// changes these two arms and says why.
fn event_amount(action: &GameAction) -> Option<u64> {
    match action {
        GameAction::DealDamage { amount, .. }
        | GameAction::GainLife { amount, .. }
        | GameAction::LoseLife { amount, .. } => Some(*amount),
        GameAction::AddCounters { .. }
        | GameAction::RemoveCounters { .. }
        | GameAction::DrawCard { .. }
        | GameAction::ZoneChange { .. }
        | GameAction::Untap { .. }
        | GameAction::Tap { .. }
        | GameAction::Attach { .. }
        | GameAction::Destroy { .. }
        | GameAction::EnterBattlefield { .. } => None,
    }
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
/// Re-asked every iteration, like `is_prohibited`: CR 616.1f rewrites the event
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

/// The CR 616.1 loop: decide what event actually happens, for every member
/// of a batch that is about one subject.
///
/// **Decisions are per `(batch, subject)`; rewrites are per member**
/// (`replacement-architecture.md` §9, RD decision 3; §11 items 15 and 24).
/// `group` is every batch member sharing one [`EventSubject`] — the two
/// blockers' damage to one attacker, two attackers' damage to one player —
/// and it gets **one** applied set, one chooser and one loop: a chosen
/// instance is applied to each member it applies to, its rider queued once
/// with the members' amounts summed, its use spent once by what the whole
/// application did. Two printed rulings are what fixed the unit. Kalitas's
/// says N opposing creatures dying at once make N Zombies — N subjects, N
/// applications — and CR 122.1c's says two blockers hitting one creature with
/// shield counters remove **one** counter — one subject, one application. The
/// per-member shape got the second wrong; a per-batch shape would get the
/// first wrong; the subject is the key both agree on. And it is the *batch*
/// that scopes it, not the turn: CR 510.4's two combat damage steps are two
/// batches, so a first striker and a regular blocker spend two counters.
///
/// Returns each member's index with what happens to it — `None` when its
/// event does not happen at all (CR 614.6). Queued riders are pushed onto
/// `riders` in application order and are the caller's to resolve *after*
/// performing the surviving events, including for a member that ended as
/// `None`, since CR 615.12 makes a rider unconditional once queued.
///
/// `later` is the rest of the batch — the groups not yet decided, at their
/// proposed amounts — read for one thing only: CR 615.7's allocation is per
/// *instance*, over every member it applies to whatever their subjects, so a
/// count spanning "you and permanents you control" is asked once here and its
/// answer kept on `GameState` for the groups after (`PreventionAllocationScope`).
///
/// `inherited` is §3.2d's lineage rule: a **decomposed** event continues its
/// parent's applied set (`DrawCards{2}` → two `DrawCard`s), a **contained**
/// event of a different kind starts a fresh one (`CreateTokens` →
/// `EnterBattlefield`). Without inheritance on decomposition, Teferi's Ageless
/// Insight re-applies to its own output and the game hangs — so this parameter
/// is the termination argument, not a nicety.
pub(crate) fn apply_replacements(
    game: &mut GameState,
    group: Vec<(usize, GameAction)>,
    later: &[(usize, &GameAction)],
    ctx: &ActionContext,
    inherited: &HashSet<ReplacementInstanceId>,
    riders: &mut Vec<Rider>,
) -> Result<Vec<(usize, Option<GameAction>)>, String> {
    let subject = subject_of(&group[0].1);
    debug_assert!(
        group.iter().all(|(_, a)| subject_of(a) == subject),
        "a subject group is built by `execute_batch_inner` from one subject"
    );
    // An entry is its own subject, so the CR 614.12 frame below — built for
    // one entering permanent — is built for a group of one.
    debug_assert!(
        group.len() == 1
            || !group.iter().any(|(_, a)| matches!(a, GameAction::EnterBattlefield { .. })),
        "an entry shares its subject with nothing"
    );

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
    let mut members: Vec<Member> = group
        .into_iter()
        .map(|(index, event)| Member { index, event: Some(event) })
        .collect();
    let finish = |members: Vec<Member>| members.into_iter().map(|m| (m.index, m.event)).collect();

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
        // "can't" check, because there is no event here to forbid. Per member,
        // because a prevention can empty one member of a group and leave the
        // rest for the next iteration to see.
        for m in &mut members {
            if m.event.as_ref().is_some_and(never_happens) {
                m.event = None;
            }
        }
        // Owned, because the gather below walks `members` mutably; one small
        // clone per iteration of a loop that runs once for most proposals.
        let Some(first) = members.iter().find_map(|m| m.event.clone()) else {
            return Ok(finish(members));
        };

        // **The group's key is not the members' subject after RD-4**, and the
        // shadow is where that stops being a distinction without a difference.
        // `subject` above is the key `execute_batch_inner` grouped by — one
        // CR 616.1 loop, one applied set, one chooser — and it is fixed for the
        // life of the group. A `Rewrite::Retarget` moves an *event's* subject
        // (CR 614.9), so from the second iteration on the two can disagree, and
        // every question below is about the event rather than about the group:
        // who chooses (CR 616.1's affected object's controller), which object a
        // prompt names, and which object a `RemoveCountersFromAffected` takes a
        // counter from. Re-derived per iteration for the same reason `gather`
        // is — CR 616.1f re-gathers against the modified event.
        let subject = subject_of(&first);

        // CR 614.12 / 614.17d — the frame both questions below read for an
        // entering permanent, built once per iteration and computed only if a
        // filter asks. Per iteration and not per event, because clause (1)
        // says the frame accounts for the replacements already applied.
        let frame = EntryFrame::new(game, &first);

        // CR 614.4 — gathered against live state at the moment of proposal.
        // There is no "go back in time" path because there is no other place
        // to ask. Per member, and the union keyed by CR 614.5's identity: one
        // instance that applies to two members is one candidate with two
        // members, and it is offered to the chooser once.
        let mut candidates: Vec<Candidate> = Vec::new();
        for (pos, m) in members.iter_mut().enumerate() {
            let Some(event) = m.event.as_ref() else {
                continue;
            };
            // CR 614.17: a "can't" is checked ahead of the pipeline and wins
            // (CR 101.2). Not a `ReplacementDef` and never one — modelling it
            // as one would have put it in the CR 616.1 choice list, where a
            // player could decline it.
            //
            // Re-asked on every iteration rather than once at the top, because
            // CR 614.17c lets a self-replacement change the event's *type*,
            // and an event of a different type is a different "can't"
            // question.
            let blocked = is_prohibited(
                game,
                &Query::Event {
                    action: event,
                    // CR 101.2 scoped by cause (§2.6). `ActionContext` already
                    // threads the resolution that proposed this; a turn-based
                    // or state-based action has none, and no `SourceFilter`
                    // matches it.
                    cause: ctx.resolution.map(|r| r.controller),
                    lookahead: Some(&frame),
                },
            );
            let mut any = false;
            for c in gather(game, event, ctx, blocked, &frame)
                .into_iter()
                // CR 614.5, with CR 903.9b as the rules' only stated exception.
                .filter(|c| c.def.exempt_from_614_5 || !applied.contains(&c.id))
                // No exception to this one — see `declined`.
                .filter(|c| !declined.contains(&c.id))
            {
                any = true;
                match candidates.iter_mut().find(|k| k.instance.id == c.id) {
                    Some(k) => k.members.push(pos),
                    None => candidates.push(Candidate { instance: c, members: vec![pos] }),
                }
            }
            // CR 614.17c: a blocked event can only be replaced by a
            // self-replacement effect, and with none left to offer it does
            // not happen.
            if blocked && !any {
                m.event = None;
            }
        }
        if candidates.is_empty() {
            return Ok(finish(members));
        }

        // CR 616.1a–e's ladder: everything below the first non-empty step is
        // not a choice this pass has.
        let choosable = must_choose_among(candidates, |c| c.instance.def.class);

        // CR 616.1 / 400.6 — the affected object's controller (or its owner if
        // it has no controller) or the affected player. One subject, so one
        // chooser for the whole group.
        let chooser = chooser_for(game, &first);

        // **Never prompt with fewer than two candidates.** CR-correct (there is
        // no choice to make with one), and it is what keeps every existing
        // `ScriptedDecisionProvider` test green rather than drowning it in
        // unexpected prompts now that every `execute_action` traverses this
        // loop. If a phase finds itself relaxing this to make something work,
        // it has found a design error, not a test problem
        // (`replacement-architecture.md` §11 item 7).
        //
        // **And never prompt for a choice with one outcome** — §11 item 19.
        // `ordering_cannot_change_outcome` is the provable form of that
        // rule, and `unsuppressed` — the members it was chosen over, each with
        // the group members it applied to — is what the debug build checks it
        // against after the rewrite below.
        let mut unsuppressed: Vec<(ReplacementInstanceId, Vec<usize>)> = Vec::new();
        let chosen = if choosable.len() == 1 {
            choosable.into_iter().next().expect("len checked")
        } else if ordering_cannot_change_outcome(&choosable, subject_object(subject)) {
            let mut rest = choosable.into_iter();
            let first = rest.next().expect("len checked");
            unsuppressed = rest.map(|c| (c.instance.id, c.members)).collect();
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
            let sources: Vec<ObjectId> = choosable.iter().map(|c| c.instance.source).collect();
            let index = ask_choose_replacement(
                ctx.dp,
                game,
                chooser,
                subject_object(subject),
                &sources,
            );
            choosable.into_iter().nth(index).expect("index validated by ask_*")
        };
        let Candidate { instance: chosen, members: applicable } = chosen;

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

        // **The one rewrite that is not the same for every member**, and it is
        // one board: Mending Hands' count facing two attackers at once, which
        // CR 615.7 splits by the shielded player's own allocation. Every other
        // rewrite in the algebra applies identically to each member — a
        // doubler doubles both, a `Prevent` drops both.
        //
        // Not Harm's Way, which is the *other* non-uniformity and is unbuilt:
        // it splits one event into two with different targets, a phase-1
        // member insertion rather than a per-member rewrite (§11 item 23,
        // RD-5's gate).
        let shares = match (&chosen.def.rewrite, chosen.def.uses) {
            (Rewrite::Amount(AmountRewrite::PreventRemaining), Uses::NextDamage(remaining)) => {
                Some(next_damage_shares(game, ctx, &chosen, remaining, &members, &applicable, later)?)
            }
            _ => None,
        };

        // Apply to each member it applies to, in member order. The rider
        // reads the members' amounts summed — CR 615.7's last sentence, "such
        // effects count only the amount of damage" — and the use is spent by
        // what the whole application did (decision 7).
        let mut replaced_amount: Option<u64> = None;
        let mut rider_subject: Option<EventSubject> = None;
        let mut outcome = Applied::default();
        for (k, &pos) in applicable.iter().enumerate() {
            let event = members[pos].event.take().expect("gathered this iteration, so live");
            if let Some(amount) = event_amount(&event) {
                replaced_amount = Some(replaced_amount.unwrap_or(0) + amount);
            }
            // Per member, and it is the member's own event that answers: two
            // members of one group can have different subjects once a redirect
            // has moved one of them.
            let member_subject = subject_of(&event);
            rider_subject.get_or_insert(member_subject);
            let share = shares.as_ref().map(|s| s[k]);
            let (next, did) = apply_rewrite(game, ctx, &chosen, event, member_subject, share)?;
            outcome.took_effect |= did.took_effect;
            outcome.prevented += did.prevented;
            if let Some(next) = &next {
                // CR 616.1f — the modified event is what the next iteration
                // re-gathers against, which is how CR 616.2's "a replacement
                // effect can become applicable as the result of another"
                // works without any special case.
                check_exempt_terminates(game, &chosen, next, &mut exempt_applied)?;
                check_order_invariance(game, ctx, next, pos, &unsuppressed);
            }
            // CR 614.6 — `None` here means this member's event does not
            // happen. Queued riders still run.
            members[pos].event = next;
        }

        // Queued, not resolved (§4.1a). A later replacement in the same loop
        // further modifying or even dropping the event does not un-queue this.
        if let Some(then) = chosen.def.then.clone() {
            riders.push(Rider {
                source: chosen.source,
                controller: chosen.controller,
                // The subject of the first member this application touched,
                // read *before* its own rewrite — CR 615.5's "that much" is
                // about the event the effect replaced. Reverse Damage's rider
                // names the player the damage was headed for, and a redirect
                // applied later in the same loop does not rename him.
                subject: rider_subject.unwrap_or(subject),
                replaced_amount,
                prevented: outcome.prevented,
                effect: then,
            });
        }

        consume_use(game, &chosen, outcome);
    }
}

/// CR 615.7's allocation: how much of a "prevent the next N damage" count each
/// of the group's applicable members is given.
///
/// > 615.7 … If damage would be dealt to the shielded permanent or player by
/// > two or more applicable sources at the same time, the player or the
/// > controller of the permanent chooses which damage the shield prevents.
///
/// **Per instance, over every batch member it applies to, asked once.** The
/// buckets are this group's applicable members plus every not-yet-decided
/// member of the batch the instance also applies to, at their current amounts
/// — Divine Deflection's "you and/or permanents you control" spans subjects,
/// and its ruling is "you don't decide until the point at which the damage
/// would be dealt", so a later group's doubling chosen ahead of the count moves
/// that member and not the allocation (§9's recorded corner). The answer is
/// kept on `GameState` (`PreventionAllocationScope`, `codebase-state.md` item
/// 40) and a later group's loop reads its own shares from it rather than
/// asking again. **Never with one source**: with one, every point is prevented
/// unasked, and nothing is recorded.
///
/// One chooser across the buckets, asserted rather than guessed: every printed
/// multi-subject count is scoped to one player and that player's permanents.
fn next_damage_shares(
    game: &mut GameState,
    ctx: &ActionContext,
    chosen: &ReplacementInstance,
    remaining: u64,
    members: &[Member],
    applicable: &[usize],
    later: &[(usize, &GameAction)],
) -> Result<Vec<u64>, String> {
    // This group's applicable members: batch index, damage source, amount.
    let mut here: Vec<(usize, ObjectId, u64)> = Vec::with_capacity(applicable.len());
    for &pos in applicable {
        let m = &members[pos];
        match m.event.as_ref() {
            Some(GameAction::DealDamage { source, amount, .. }) => here.push((m.index, *source, *amount)),
            other => {
                return Err(format!(
                    "replacement {:?} prevents \"the remaining\" damage but matched {:?}, \
                     which is not damage. Its `EventPattern` and its `Rewrite` describe \
                     different events.",
                    chosen.id, other
                ))
            }
        }
    }

    // Asked already, by an earlier group of this batch: read our shares.
    if let Some(stored) = game.prevention_allocations.shares_for(chosen.id) {
        return Ok(here
            .iter()
            .map(|(index, _, amount)| {
                stored
                    .iter()
                    .find(|(i, _)| i == index)
                    .map(|(_, share)| *share)
                    .unwrap_or(0)
                    .min(*amount)
                    .min(remaining)
            })
            .collect());
    }

    // The buckets: here, then the later members the instance also applies to,
    // each with the chooser CR 616.1 would give its own group.
    let mut buckets: Vec<(usize, ObjectId, u64, Option<PlayerId>)> = here
        .iter()
        .map(|&(index, source, amount)| {
            let action = members.iter().find(|m| m.index == index).and_then(|m| m.event.as_ref());
            (index, source, amount, action.and_then(|a| chooser_for(game, a)))
        })
        .collect();
    for (index, action) in later {
        if let GameAction::DealDamage { source, amount, .. } = action {
            if applies_to(game, chosen, action, subject_of(action), None) {
                buckets.push((*index, *source, *amount, chooser_for(game, action)));
            }
        }
    }

    if buckets.len() == 1 {
        return Ok(vec![remaining.min(buckets[0].2)]);
    }

    // One chooser across every bucket, or this is a def that names two sides —
    // matched in one expression, so there is no second, unreachable check for
    // a reader to explain (`codebase-state.md`, the RD-2 review, item 98).
    let chooser = match buckets[0].3 {
        Some(p) if buckets.iter().all(|b| b.3 == Some(p)) => p,
        _ => {
            return Err(format!(
                "replacement {:?}'s CR 615.7 count applies to simultaneous damage whose \
                 subjects have different choosers ({:?}); every printed \"next N damage\" \
                 effect is scoped to one player and that player's permanents, so this is a \
                 def that names two sides.",
                chosen.id,
                buckets.iter().map(|b| b.3).collect::<Vec<_>>()
            ))
        }
    };
    game.counters.record_prevention_allocation();
    let offer: Vec<(ObjectId, u64)> = buckets.iter().map(|b| (b.1, b.2)).collect();
    let shares = ask_allocate_next_damage(ctx.dp, game, chooser, chosen.source, remaining, &offer);

    // Recorded before anything is applied, so a fork at a later group's
    // prompt sees it (item 40), and so the later group reads rather than asks.
    game.prevention_allocations.allocations.push((
        chosen.id,
        buckets.iter().zip(&shares).map(|(b, s)| (b.0, *s)).collect(),
    ));
    Ok(shares[..here.len()].to_vec())
}

/// §11 item 19 — do the effects CR 616.1 would have the player order here
/// provably reach one outcome whatever order they apply in, so the prompt is
/// noise?
///
/// Two shapes qualify, and a mix of them never does.
///
/// **Every member an `EnterWith`.** The theorem, and every clause of the
/// predicate is a premise of it:
///
/// - **`EnterWith` only.** `EnterMods::merge` is `|=` and `+`, commutative and
///   associative, so the mods a member adds land the same whatever went
///   before. A `Prevent` or an `Instead` drops or replaces the event, and
///   whether the members after it ever apply is then a real question.
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
/// **Every member an `Amount(Multiplier(n))`, `n ≥ 1`, on `DealDamage`**
/// (RD-2; `replacement-architecture.md` §11 item 29). Multiplication over
/// `u64` is commutative and associative — saturating included, since
/// saturation is monotone — so each member's final amount is the product of
/// the multipliers that apply to it whatever the order. No multiplier can
/// remove another's applicability: an amount above 0 stays above 0 under any
/// `n ≥ 1`, so `never_happens` cannot fire between members, and
/// `EventPattern::DealDamage`'s two fields are about the *source* and about
/// CR 510.2's combat flag — neither reads the amount, so no member can fall
/// out of applicability as another changes the number (re-derived at RD-3,
/// which added them; `codebase-state.md` item 47's condition (d)). Two
/// Furnaces of Rath are that shape, and the prompt was
/// noise a human would resent. **Not `Halve`, `Plus` or any prevention arm**:
/// `Halve` beside `Multiplier` is the phase's headline non-commuting board
/// (3 → 1 → 2 or 3 → 6 → 3), `Plus` beside `Multiplier` does not commute
/// either, and a prevention arm can empty the event.
///
/// **Shared by both shapes — mandatory, static, under CR 614.5, not
/// counter-derived, no rider.** An optional is a second prompt whose answer
/// can differ per order; a `Uses::Once` or `NextDamage` spends a registry row;
/// an exempt effect may re-apply; a counter-derived instance is re-synthesized
/// per gather; riders queue in choice order and run in queue order
/// (CR 615.5), so two members that both carry one make the order observable
/// in the event log even when the board is identical. Each is excluded so the
/// argument has nothing to say about it.
///
/// With every member's applicability fixed and every application commuting,
/// each member applies exactly once in any order (CR 614.5) and the result is
/// the same. Root Maze beside Idyllic Beachfront — the fuzz harness's every
/// land drop under Root Maze — and two Furnaces in one red deck are the cases
/// this exists for.
///
/// **A semantics-assuming shortcut, and it carries its expiry conditions**
/// (`layers-architecture.md` §12 item 3; `codebase-state.md` item 47). The
/// entry shape goes false the day `EnterMods` gains a field that feeds a
/// characteristic — face-down, which is Layer 1 and changes everything — or
/// `ObjectFilter` gains a leaf that reads P/T, keywords or counters, or
/// `EventPattern::EnterBattlefield` reads `mods`. The multiplier shape goes
/// false the day an `EventPattern::DealDamage` field reads the *amount*, or a
/// `Multiplier(0)` is printed (refused here by `n ≥ 1`). `check_order_invariance`
/// is the debug-build check that computes it the other way. The name is the
/// question's, not the implementation's (item 65): does CR 616.1's ordering
/// prompt here have more than one outcome.
fn ordering_cannot_change_outcome(choosable: &[Candidate], entering: Option<ObjectId>) -> bool {
    let all_entries = choosable
        .iter()
        .all(|c| matches!(c.instance.def.rewrite, Rewrite::EnterWith(_)));
    let all_multipliers = choosable.iter().all(|c| {
        matches!(c.instance.def.pattern, EventPattern::DealDamage { .. })
            && matches!(
                c.instance.def.rewrite,
                Rewrite::Amount(AmountRewrite::Multiplier(n)) if n >= 1
            )
    });
    if !(all_entries || all_multipliers) {
        return false;
    }
    choosable.iter().all(|c| {
        let def = &c.instance.def;
        (match &def.rewrite {
            // Reads the frame only when the source is the object being
            // computed, so anything else is a board read and commutes.
            Rewrite::EnterWith(t) => {
                (t.is_fixed() || Some(c.instance.source) != entering)
                    && affected_is_mods_invariant(&def.affected)
            }
            Rewrite::Amount(AmountRewrite::Multiplier(_)) => true,
            _ => false,
        }) && !def.optional
            && def.then.is_none()
            && matches!(def.uses, Uses::Static)
            && !def.exempt_from_614_5
            && !matches!(c.instance.id, ReplacementInstanceId::Counter(..))
    })
}

/// Can no `EnterMods` field change whether this set matches the entering
/// object? `SourceOnly`, `Fixed` and `Host` match by id; a
/// `Filter` is invariant iff every leaf is. The entry half of
/// [`ordering_cannot_change_outcome`]'s premise.
fn affected_is_mods_invariant(affected: &AffectedSet) -> bool {
    match affected {
        AffectedSet::SourceOnly | AffectedSet::Fixed(_) | AffectedSet::Host => true,
        AffectedSet::Filter { filter } => filter_is_mods_invariant(filter),
    }
}

/// The leaf table for [`ordering_cannot_change_outcome`]'s entry premise. Types,
/// subtypes, supertypes, colors, controller, ownership and tokenness are fed
/// by no `EnterMods` field; power is fed by `+1/+1` and `-1/-1` counters
/// (CR 122.1a) and so `PowerLE` is not invariant. Matched exhaustively, so a
/// new leaf has to be classified rather than defaulting to "safe".
fn filter_is_mods_invariant(filter: &ObjectFilter) -> bool {
    match filter {
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::ByController(_)
        | ObjectFilter::Token
        | ObjectFilter::ByOwner(_)
        | ObjectFilter::EachOther => true,
        ObjectFilter::PowerLE(_) => false,
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => {
            filter_is_mods_invariant(a) && filter_is_mods_invariant(b)
        }
        ObjectFilter::Not(inner) => filter_is_mods_invariant(inner),
    }
}

/// The debug-build check on [`ordering_cannot_change_outcome`]: after
/// the suppressed choice applied to a member, every candidate it was chosen
/// over that applied to that member must still apply to the rewritten event,
/// or the predicate admitted something that reads what an application
/// changes and the prompt it skipped was real.
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
    member: usize,
    unsuppressed: &[(ReplacementInstanceId, Vec<usize>)],
) {
    if !cfg!(debug_assertions) || unsuppressed.is_empty() {
        return;
    }
    let frame = EntryFrame::new(game, next);
    let still: Vec<ReplacementInstanceId> = gather(game, next, ctx, false, &frame)
        .into_iter()
        .map(|c| c.id)
        .collect();
    for (id, _) in unsuppressed.iter().filter(|(_, m)| m.contains(&member)) {
        debug_assert!(
            still.contains(id),
            "CR 616.1 prompt suppressed as order-invariant was not: {:?} stopped applying to {:?}",
            id,
            next
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

/// Spend the chosen effect by what its application did — after
/// `apply_rewrite`, never before (`replacement-architecture.md` §9, RD
/// decision 7; §11 item 22).
///
/// > 609.7b … If for any reason the shield prevents no damage or replaces no
/// > damage, the shield isn't used up.
///
/// So `Uses::Once` (CR 701.19a's regeneration shield, CR 615.8's "next time")
/// removes the registry row only when the rewrite took effect — regeneration
/// always does, since a `Prevent` on a `Destroy` cannot do nothing, which is
/// why the order never mattered before RD-2 — and `Uses::NextDamage` is
/// reduced by exactly the damage prevented (CR 615.7), which for an
/// unpreventable event (RD-4) or a `Once` half that rounded to nothing is 0.
/// Either way a shield spent on one group of a batch is correctly gone or
/// reduced when the next group asks, because this writes game state.
///
/// A counter-derived effect consumes nothing here on purpose: CR 122.1c/d make
/// the counter removal the *substituted event* or the rider, which propose
/// through `execute_action` like every other mutation. See `Uses`' own docs.
fn consume_use(game: &mut GameState, chosen: &ReplacementInstance, outcome: Applied) {
    match chosen.def.uses {
        Uses::Static => {}
        Uses::Once => {
            if !outcome.took_effect {
                return;
            }
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
        Uses::NextDamage(_) => {
            if let ReplacementInstanceId::Registered(row) = chosen.id {
                game.replacement_effects.spend_next_damage(row, outcome.prevented);
            } else {
                debug_assert!(
                    false,
                    "`Uses::NextDamage` on a {:?}, which has no row to count down on. A \
                     CR 615.7 count has to live in the registry, for `Uses::Once`'s reason.",
                    chosen.id
                );
            }
        }
    }
}

/// CR 615.12 — may a prevention effect prevent anything here?
///
/// > 615.12. Some effects state that damage "can't be prevented." If
/// > unpreventable damage would be dealt, any applicable prevention effects are
/// > still applied to it. Those effects won't prevent any damage, but any
/// > additional effects they have will take place. Existing damage prevention
/// > shields won't be reduced by damage that can't be prevented.
///
/// **The rule's three printed shapes meet here and nowhere else** (§9's RD
/// decision 6). `unpreventable` is the per-event one — Pinpoint Avalanche's
/// "the damage can't be prevented" — and `Restriction::ApplyReplacement`
/// unions the other two: a resolution's "damage can't be prevented this turn"
/// as a registry row, and a static ability's as a sweep off the source's
/// effective ability list.
///
/// **Asked at application, not at `gather`'s door**, and the two rules say so
/// in different words: CR 701.19c makes a regeneration shield "not applied"
/// (so `push_if_applicable` withholds it), while this one applies the effect
/// and lets it prevent nothing. Merging the two sites would make one of the
/// two rules wrong.
///
/// The caller then reports `prevented: 0` and `took_effect: false`, which is
/// how the last sentence is enforced: `consume_use` spends a `NextDamage`
/// count by the damage prevented and a `Uses::Once` row only when the rewrite
/// took effect (decision 7). The rider is queued regardless, by the caller —
/// that is the middle sentence, and `AmountExpr::DamagePrevented` reading 0 is
/// what makes "if damage is prevented this way" need no `Effect::Conditional`.
fn is_unpreventable(game: &GameState, unpreventable: bool, subject: EventSubject) -> bool {
    unpreventable
        || is_prohibited(
            game,
            &Query::ApplyReplacement { kind: ReplacementKindFilter::Prevention, subject },
        )
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
/// and moves them. `share` is CR 615.7's answer for this member — how much of
/// a `PreventRemaining` count it is given — decided by the caller across the
/// members at once, and `None` for every other arm.
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
    share: Option<u64>,
) -> Result<(Option<GameAction>, Applied), String> {
    // Every arm but `Amount` changes the event whenever it is chosen — which
    // is why the order of `consume_use` never mattered before RD-2 — and
    // prevents nothing (CR 615.1a).
    let changed = Applied { took_effect: true, prevented: 0 };
    match &chosen.def.rewrite {
        // CR 614.6 / 615.6 — the event does not happen.
        //
        // **On damage it prevents all of it, and the number is what a rider
        // reads.** CR 615.6's "prevent that damage" is the whole amount, and
        // CR 615.5's "the damage prevented this way" is Reverse Damage's life
        // gain. Reported here rather than derived by the caller from a `None`
        // event, because only this arm knows the event was damage: a `Prevent`
        // on a destruction is regeneration and prevents no damage at all
        // (CR 615.1 is about damage), which is the same line
        // `ReplacementDef::is_prevention` draws.
        //
        // **CR 615.12's first application site.** On damage the whole event is
        // what this arm prevents, so an unpreventable event is exactly the case
        // the rule describes: the effect "is still applied", prevents nothing —
        // the event survives untouched — and nothing is spent. `prevented_or_0`
        // is the shared consult; the second site is the `Amount` arm below.
        Rewrite::Prevent => match &event {
            GameAction::DealDamage { amount, unpreventable, .. } => {
                if is_unpreventable(game, *unpreventable, subject) {
                    // Applied, prevented nothing, event untouched, nothing
                    // spent — and the caller queues the rider anyway.
                    Ok((Some(event), Applied::default()))
                } else {
                    Ok((None, Applied { took_effect: true, prevented: *amount }))
                }
            }
            // Regeneration and CR 122.1c's shield counter: a `Prevent` on
            // anything but damage prevents no *damage* (CR 615.1 is about
            // damage), and CR 615.12 has nothing to say to it.
            _ => Ok((None, Applied { took_effect: true, prevented: 0 })),
        },

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
                Ok((Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }), changed))
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
                Ok((Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }), changed))
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
                Ok((Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }), changed))
            }
            other => Err(format!(
                "replacement {:?} modifies under whose control a permanent enters but \
                 matched {:?}, which is not an entry. Its `EventPattern` and its \
                 `Rewrite` describe different events.",
                chosen.id, other
            )),
        },

        // CR 614.5's doublers and CR 615.10's partial prevention. The one arm
        // whose whole job is to read the amount the last application left,
        // which is what makes CR 616.1's ordering choice observable.
        //
        // **CR 615.12's second application site**, and there are two of them
        // because RD-3 gave `Rewrite::Prevent` a prevented amount of its own:
        // both arms have to answer the rule the same way, so both ask
        // `is_unpreventable` (`replacement-architecture.md` §9, RD-4's
        // "As landed").
        Rewrite::Amount(amount_rewrite) => match event {
            GameAction::DealDamage { source, target, amount, is_combat, unpreventable } => {
                // CR 615.7's cap is the instance's count. `PreventRemaining`
                // on anything else, or a count on anything else, is a def
                // whose halves disagree — the same authoring error every
                // other arm reports.
                let arm = match (amount_rewrite, chosen.def.uses) {
                    (AmountRewrite::PreventRemaining, Uses::NextDamage(remaining)) => {
                        amount_rewrite.capped(share.unwrap_or(remaining))
                    }
                    (AmountRewrite::PreventRemaining, uses) => {
                        return Err(format!(
                            "replacement {:?} prevents \"the remaining\" damage but has \
                             {:?} rather than a `Uses::NextDamage` count to read the \
                             remaining from.",
                            chosen.id, uses
                        ))
                    }
                    (other, Uses::NextDamage(_)) => {
                        return Err(format!(
                            "replacement {:?} carries a `Uses::NextDamage` count but its \
                             rewrite is {:?}, which prevents no amount for the count to \
                             count down (CR 615.7 is about prevention shields).",
                            chosen.id, other
                        ))
                    }
                    (other, _) => *other,
                };
                // The consult is on the *prevention* arms only. Ghosts of the
                // Innocent halves Excruciator's unpreventable 7 to 3 and
                // Gisela prevents none of it, which is the whole reason
                // `Halve` and `PreventHalf` are two arms (CR 615.12, §11
                // item 28) — and here it is one `prevents_damage()`.
                let (after, prevented) = if arm.prevents_damage() {
                    let prevented = if is_unpreventable(game, unpreventable, subject) {
                        0
                    } else {
                        arm.prevented(amount)
                    };
                    (amount - prevented, prevented)
                } else {
                    (arm.apply(amount), 0)
                };
                Ok((
                    Some(GameAction::DealDamage {
                        source,
                        target,
                        amount: after,
                        is_combat,
                        unpreventable,
                    }),
                    Applied { took_effect: after != amount, prevented },
                ))
            }
            // Its `EventPattern` and its `Rewrite` describe different events —
            // the same card-authoring error every other arm reports.
            other => Err(format!(
                "replacement {:?} changes an amount but matched {:?}, which has none",
                chosen.id, other
            )),
        },

        // CR 614.9 — the same damage, somewhere else.
        //
        // Rewrites `target` and nothing else: `source`, `amount`, `is_combat`
        // and `unpreventable` travel with the damage, which is Pariah's ruling
        // ("the damage dealt to the enchanted creature instead is still combat
        // damage") and Kor Chant's (it is still dealt by the original source).
        //
        // The re-check is here rather than in `applies_to`, and the difference
        // is observable: a redirect whose destination is gone is still
        // gathered, still offered to CR 616.1's chooser and still *chosen* —
        // it then does nothing and is not spent (`ATOM-614.9-001`). Filtering
        // it out at the door would make it vanish from a list the rule says it
        // belongs on.
        Rewrite::Retarget(spec) => match event {
            GameAction::DealDamage { source, target, amount, is_combat, unpreventable } => {
                // The two outcomes differ in one field and in what they claim:
                // a legal destination replaces `target` and took effect; an
                // illegal one is CR 614.9's "the effect does nothing", which
                // returns the event as proposed and spends nothing.
                let (target, outcome) = match retarget_destination(game, chosen, *spec, source)
                    .filter(|&to| redirection_is_legal(game, to, target))
                {
                    Some(to) => (to, Applied { took_effect: true, prevented: 0 }),
                    None => (target, Applied::default()),
                };
                Ok((
                    Some(GameAction::DealDamage {
                        source,
                        target,
                        amount,
                        is_combat,
                        unpreventable,
                    }),
                    outcome,
                ))
            }
            // Its `EventPattern` and its `Rewrite` describe different events —
            // the same card-authoring error every other arm reports.
            other => Err(format!(
                "replacement {:?} redirects damage but matched {:?}, which is not damage",
                chosen.id, other
            )),
        },

        Rewrite::Instead(template) => match (template, event) {
            (
                GameActionTemplate::ZoneChangeTo { to, cause },
                GameAction::ZoneChange { object, from, .. },
            ) => Ok((
                Some(GameAction::ZoneChange { object, from, to: *to, cause: *cause }),
                changed,
            )),

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
            ) => Ok((
                Some(GameAction::ZoneChange {
                    object,
                    from: from.unwrap_or(Zone::Battlefield),
                    to: *to,
                    cause: *cause,
                }),
                changed,
            )),

            (GameActionTemplate::RemoveCountersFromAffected { counter, n }, _) => {
                match subject_object(subject) {
                    Some(object) => Ok((
                        Some(GameAction::RemoveCounters { object, counter: *counter, n: *n }),
                        changed,
                    )),
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

/// Where a [`RetargetSpec`] sends the damage, before CR 614.9 is asked whether
/// that is still a legal place for it.
///
/// `None` is "there is nothing there to name" — an Aura attached to nothing,
/// or a damage source with neither controller nor owner — and the caller
/// treats it exactly as an illegal destination does, because CR 614.9's answer
/// to both is "the effect does nothing".
fn retarget_destination(
    game: &GameState,
    chosen: &ReplacementInstance,
    spec: RetargetSpec,
    damage_source: ObjectId,
) -> Option<DamageTarget> {
    match spec {
        RetargetSpec::ToEffectSource => Some(DamageTarget::Object(chosen.source)),
        // CR 303.4m, read now rather than captured — the same `attached_to`
        // read `AffectedSet::Host` makes, because an Aura registered its effect
        // before it was attached to anything.
        RetargetSpec::ToHost => game
            .battlefield
            .get(&chosen.source)
            .and_then(|e| e.attached_to)
            .map(DamageTarget::Object),
        // CR 109.5 asked of the *damage's* source, wherever it is: a spell on
        // the stack has no battlefield entry, so this falls through to its
        // owner, which is its controller for every card cast from its owner's
        // own hand.
        RetargetSpec::ToDamageSourceController => {
            controller_or_owner(game, damage_source).map(DamageTarget::Player)
        }
    }
}

/// CR 614.9's re-check, asked at the moment the rewrite is applied.
///
/// > If one of those permanents is no longer on the battlefield when the damage
/// > would be redirected, or is no longer a battle, creature, or planeswalker
/// > when the damage would be redirected, the effect does nothing. If damage
/// > would be redirected to or from a player who has left the game, the effect
/// > does nothing.
///
/// Both ends, because the rule names both: `to` is where the rewrite would send
/// the damage and `from` is where the proposal has it now.
///
/// **An existence-and-type check, not `validate_selection`.** Divine
/// Deflection's ruling draws the same line from the rider's side — *"whether
/// the targeted permanent or player is still a legal target is not checked"* —
/// and a legality re-check here would get shroud and protection wrong: a
/// redirect is not targeting (CR 115.1), so a hexproof creature is a perfectly
/// good destination.
fn redirection_is_legal(game: &GameState, to: DamageTarget, from: DamageTarget) -> bool {
    let player_in_game = |pid: PlayerId| {
        game.player_lost.get(pid as usize).map(|lost| !lost).unwrap_or(false)
    };
    let destination_ok = match to {
        DamageTarget::Player(pid) => player_in_game(pid),
        DamageTarget::Object(id) => {
            game.battlefield.contains_key(&id) && {
                let types = get_effective_types(game, id);
                types.contains(&CardType::Creature)
                    || types.contains(&CardType::Planeswalker)
                    || types.contains(&CardType::Battle)
            }
        }
    };
    let origin_ok = match from {
        DamageTarget::Player(pid) => player_in_game(pid),
        DamageTarget::Object(_) => true,
    };
    destination_ok && origin_ok
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
            if !game.object_matches_filter(id, &aux.filter, you).unwrap_or(false) {
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
