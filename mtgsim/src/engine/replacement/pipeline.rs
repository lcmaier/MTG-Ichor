//! The CR 616.1 loop (`replacement-architecture.md` §4.1).

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::restriction::{is_prohibited, Query};
use crate::events::event::{CounterSubject, DamageTarget};
use crate::types::card_types::CardType;
use crate::types::restriction::ReplacementKindFilter;
use crate::state::game_state::GameState;
use crate::types::effects::{
    ObjectSet, AmountExpr, CounterType, Effect, ObjectFilter, PlayerRef, TokenDef,
};
use crate::types::replacement::{TokenKind, TokenSubstitution};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaAtom, ManaType};
use crate::types::replacement::{
    AmountRewrite, AuxiliaryMove, EnterMods, EnterModsTemplate, EntryCounters, EventPattern,
    ReplacementDef,
    RetargetSpec, GameActionTemplate, Rewrite, TemplateAmount, Uses,
};
use crate::types::zones::{DrawCause, LifeLossCause, Zone};
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
    /// CR 614.5's applied set for the event this rider is the rest of — the
    /// group's *final* set, filled in by `execute_batch_inner` once the loop
    /// has returned, and handed to every proposal the rider makes.
    ///
    /// A rider's proposals are "modified events that may replace that
    /// event", not new events: Alms Collector's *"you and that player each
    /// draw a card"* is one replaced event with two draws, and its own ruling
    /// is that an effect already applied "can't be applied again to the
    /// resulting events". Given a fresh set instead, two Reflections and two
    /// Collectors across two seats handed one draw back and forth until the
    /// stack overflowed (`replacement-architecture.md` §11 item 77).
    pub lineage: HashSet<ReplacementInstanceId>,
    pub controller: PlayerId,
    /// The event's subject. Becomes the rider's single resolved target, so
    /// `EffectRecipient::Target` in a `then` names the object or the player the
    /// replacement was about.
    ///
    /// An [`EventSubject`] rather than an object id: a rider on a *player* event
    /// (Reverse Damage's gain, Angel of Suffering's mill) has to name the player
    /// (`replacement-architecture.md` §11 item 16).
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
        // CR 121.2a's "the number of cards drawn" is exactly what CR 615.5's
        // "that many" would mean about a draw instruction, so the instruction
        // reports it and the individual draw does not — one card, and no rider
        // says "that many" about one.
        GameAction::DrawCards { n, .. } => Some(*n),
        // CR 614.16's "that many" is the count proposed, and it is what a
        // rider saying "that many" about a creation would mean.
        GameAction::CreateTokens { defs, .. } => Some(defs.len() as u64),
        // CR 701.22a's N. Eligeth's "draw **that many** cards instead" reads
        // it through `TemplateAmount::ReplacedAmount`; it is the instruction's
        // number, so a short library does not shrink it, which is CR 701.22d's
        // "even if some or all of those actions were impossible".
        GameAction::Scry { n, .. } => Some(*n),
        // CR 106.6a's "the amount of mana produced": every unit, plain and
        // restricted alike — what Deep Water's "instead of any other type"
        // keeps through `TemplateAmount::ReplacedAmount`, and what a rider's
        // "that much" would mean about a production.
        GameAction::ProduceMana { mana, special, .. } => {
            Some(mana.iter().map(|(_, n)| n).sum::<u64>() + special.len() as u64)
        }
        GameAction::AddCounters { .. }
        | GameAction::RemoveCounters { .. }
        | GameAction::DrawCard { .. }
        | GameAction::CreateTokenIn { .. }
        | GameAction::ZoneChange { .. }
        | GameAction::Untap { .. }
        | GameAction::Tap { .. }
        | GameAction::Attach { .. }
        | GameAction::Destroy { .. }
        | GameAction::EnterBattlefield { .. }
        // A shuffle has no amount: CR 701.24a randomizes a whole library.
        | GameAction::ShuffleLibrary { .. }
        // A turn's number is not an amount: no rider says "that many" about a
        // turn, a phase or a step, and 614.10b — the one rule that makes a
        // skip do something afterwards — has zero printed cards.
        | GameAction::BeginTurn { .. }
        | GameAction::BeginPhase { .. }
        | GameAction::BeginStep { .. }
        // A game's end has no amount; Stunning Reversal's "draw seven" is a
        // number printed on the card, not one read off the loss.
        | GameAction::PlayerLoses { .. }
        | GameAction::PlayerWins { .. } => None,
    }
}

/// What *caused* a proposed event: the controller of the resolving spell or
/// ability that proposed it (CR 608.2's resolution; CR 109.5 makes that
/// controller the effect's "you").
///
/// `None` for a turn-based action, a state-based action, cost payment or
/// combat damage — none of which belongs to a resolution, and none of which
/// any [`SourceFilter`](crate::types::restriction::SourceFilter) matches.
///
/// **No rule defines this predicate; it is a shape printed on cards** — "a
/// spell or ability an opponent controls causes you to …", which seventeen
/// discard cards and Sigarda's family say. So it is one expression with three
/// readers — `Query::Event::cause`, `gather`'s `ReplacementDef::by`, and the
/// exemption check's re-ask — named here rather than spelled out at each,
/// because a "can't" and a replacement effect printing that clause must answer
/// it the same way or Tamiyo, Collector of Tales and Nephalia Academy stop
/// agreeing about one discard.
fn cause_of(ctx: &ActionContext) -> Option<PlayerId> {
    ctx.resolution.map(|r| r.controller)
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
///
/// **A scry 0 is here and a `DrawCards { n: 0 }` still is not**, and the
/// difference is that the rules say so in one case and not the other.
/// CR 701.22b: "If a player is instructed to scry 0, **no scry event occurs**.
/// Abilities that trigger whenever a player scries won't trigger." That is
/// CR 120.8's and CR 119.10's sentence about a third event, so scry joins them.
///
/// **Nothing here says a card is drawn when N is 0, and nothing ever draws
/// one.** The event a `DrawCards { n: 0 }` describes is CR 121.2a's
/// *instruction* — the thing a replacement effect "that refers to the number
/// of cards drawn" modifies — and performing it runs a loop zero times, so no
/// `DrawCard` is proposed and no `CardDrawn` is emitted. What is being said is
/// narrower than it looks: no rule states that an instruction of zero is not an
/// event, where CR 120.8, 119.10 and now 701.22b each state exactly that about
/// their own. RE-2 decision 3 made the call and it stands.
///
/// Scry 0 matters twice: an Eligeth would otherwise turn it into a draw of
/// zero, and the performer would write a `Scried` line CR 701.22b says must
/// fire nothing.
fn never_happens(action: &GameAction) -> bool {
    match action {
        GameAction::DealDamage { amount, .. } => *amount == 0,
        GameAction::GainLife { amount, .. } => *amount == 0,
        GameAction::Scry { n, .. } => *n == 0,
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
/// event does not happen at all (CR 614.6) — **and the group's applied set**,
/// which is what a performer that decomposes hands to the events it decomposes
/// into (§3.2d; the `inherited` paragraph below is the same rule from the
/// other end). Queued riders are pushed onto `riders` in application order and
/// are the caller's to resolve *after* performing the surviving events,
/// including for a member that ended as `None`, since CR 615.12 makes a rider
/// unconditional once queued.
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
) -> Result<(Vec<(usize, Option<GameAction>)>, HashSet<ReplacementInstanceId>), String> {
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
    // Declining is tracked separately from CR 614.5's applied set: CR 903.9b is
    // both `exempt_from_614_5` and optional, so a decline recorded only in the
    // applied set is re-offered forever — a hang (`CLAUDE.md`; §11 item 14).
    let mut declined: HashSet<ReplacementInstanceId> = HashSet::new();
    // Which exempt effect has applied, if any — see `check_exempt_terminates`,
    // which owns the whole termination argument for the effects CR 614.5 does
    // not govern.
    let mut exempt_applied: Option<ReplacementInstanceId> = None;
    let mut members: Vec<Member> = group
        .into_iter()
        .map(|(index, event)| Member { index, event: Some(event) })
        .collect();
    let finish = |members: Vec<Member>| -> Vec<(usize, Option<GameAction>)> {
        members.into_iter().map(|m| (m.index, m.event)).collect()
    };

    // Unbounded on purpose: every iteration consumes something finite — CR 614.5's
    // `applied`, `declined`, or `check_exempt_terminates`'s slot — and nothing
    // touches the board between iterations (riders queue, §4.1a).
    loop {
        // CR 614.7a — a non-event has nothing to replace; ahead of the "can't" check,
        // and per member, since a prevention can empty one member of a group.
        for m in &mut members {
            if m.event.as_ref().is_some_and(never_happens) {
                m.event = None;
            }
        }
        // Owned, because the gather below walks `members` mutably; one small
        // clone per iteration of a loop that runs once for most proposals.
        let Some(first) = members.iter().find_map(|m| m.event.clone()) else {
            return Ok((finish(members), applied));
        };

        // The group's key, not the member's subject: a `Rewrite::Retarget` (CR 614.9)
        // moves an event's subject, so from the second iteration the two can differ,
        // and every question below — chooser, prompt, counter subject — is the event's.
        // Re-derived per iteration for the same reason `gather` is (CR 616.1f).
        let subject = subject_of(&first);

        // CR 614.12 / 614.17d — the frame both questions below read for an
        // entering permanent, built once per iteration and computed only if a
        // filter asks. Per iteration and not per event, because clause (1)
        // says the frame accounts for the replacements already applied.
        let frame = EntryFrame::new(game, &first);

        // CR 614.4 — gathered against live state at proposal, per member, keyed by
        // CR 614.5's identity: one instance over two members is one candidate.
        let mut candidates: Vec<Candidate> = Vec::new();
        for (pos, m) in members.iter_mut().enumerate() {
            let Some(event) = m.event.as_ref() else {
                continue;
            };
            // CR 614.17: a "can't" is checked ahead of the pipeline and wins (CR 101.2);
            // never a `ReplacementDef`, or a player could decline it. Re-asked every
            // iteration because CR 614.17c lets a self-replacement change the event's type.
            let blocked = is_prohibited(
                game,
                &Query::Event {
                    action: event,
                    // The source behind this event, for a "can't" that names one (Sigarda's
                    // family): the resolution `ActionContext` threads, or none for a turn-based
                    // or state-based action.
                    cause: cause_of(ctx),
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
            return Ok((finish(members), applied));
        }

        // CR 616.1a–e's ladder: everything below the first non-empty step is
        // not a choice this pass has.
        let choosable = must_choose_among(candidates, |c| c.instance.def.class);

        // CR 616.1 / 400.6 — the affected object's controller (or its owner if
        // it has no controller) or the affected player. One subject, so one
        // chooser for the whole group.
        let chooser = chooser_for(game, &first);

        // **Never prompt with fewer than two candidates** (`CLAUDE.md`; §11 item 7),
        // and never for a choice with one outcome (§11 item 19):
        // `ordering_cannot_change_outcome` is the provable form, and `unsuppressed`
        // is what the debug build checks it against after the rewrite.
        let mut unsuppressed: Vec<(ReplacementInstance, Vec<usize>)> = Vec::new();
        let chosen = if choosable.len() == 1 {
            choosable.into_iter().next().expect("len checked")
        } else if ordering_cannot_change_outcome(&choosable, subject_object(subject), &first) {
            let mut rest = choosable.into_iter();
            let first = rest.next().expect("len checked");
            // The instance rather than its id, and it costs nothing: `rest`
            // owns these and was dropping them. The fourth shape's check needs
            // the *rewrite* to compute its premise the other way.
            unsuppressed = rest.map(|c| (c.instance, c.members)).collect();
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

        // "You **may** … instead": declining marks it applied — the offer is CR 614.5's
        // one opportunity, and without the mark the loop re-gathers it forever — but
        // spends no use, so a declined regeneration shield stays for the next event.
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

        // The one rewrite that differs per member: Mending Hands' count facing two
        // attackers at once, split by CR 615.7's allocation. Every other rewrite
        // applies identically to each member. Harm's Way, which splits one event into
        // two, is a member insertion and unbuilt (§11 item 23).
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
            // Kept for `check_order_invariance`'s fourth shape, which asks what
            // a *different* member would have substituted for this same event.
            // Debug-only work, so the clone is behind the same gate.
            let before = cfg!(debug_assertions).then(|| event.clone());
            let (next, did) = apply_rewrite(game, ctx, &chosen, event, member_subject, share)?;
            outcome.took_effect |= did.took_effect;
            outcome.prevented += did.prevented;
            if let Some(next) = &next {
                // CR 616.1f — the modified event is what the next iteration
                // re-gathers against, which is how CR 616.2's "a replacement
                // effect can become applicable as the result of another"
                // works without any special case.
                check_exempt_terminates(game, &chosen, next, cause_of(ctx), &mut exempt_applied)?;
                if let Some(before) = &before {
                    check_order_invariance(
                        game,
                        ctx,
                        &chosen,
                        before,
                        next,
                        member_subject,
                        pos,
                        &unsuppressed,
                    );
                }
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
                // The group's final applied set, once the loop has it.
                lineage: HashSet::new(),
                controller: chosen.controller,
                // The first member's subject, read before its rewrite: CR 615.5's "that much"
                // is about the event the effect replaced, so a later redirect in the same loop
                // does not rename Reverse Damage's player.
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
        if let GameAction::DealDamage { source, amount, .. } = action
            && applies_to(game, chosen, action, subject_of(action), cause_of(ctx), None) {
            buckets.push((*index, *source, *amount, chooser_for(game, action)));
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

/// Does CR 616.1's ordering prompt here have more than one outcome? `false`
/// means it does, and the affected player is asked. The name is the
/// question's, not the implementation's (`codebase-state.md` item 65).
///
/// **A commutation table** (`backlog.md` §2.29 has the shape list it replaced
/// and why each shape was wrong). Each candidate is classified by what its
/// application does to the event another candidate could read —
/// [`Commuting`] — and the prompt is suppressed when every pair of members
/// commutes on every counter kind both touch; a pair with no commuting cell
/// is asked.
///
/// **What it checks** (§4.1's first half). Per member, def data: the rewrite's
/// arm, the pattern's kind and [`EventPattern::reads_the_amount`], a
/// template's kinds and whether its amounts read the frame
/// (`EnterModsTemplate::is_fixed`), the affected set's leaves
/// ([`object_set_is_mods_invariant`]). Plus one board read, for an entry only:
/// which kinds its mods hold now ([`kinds_present`]). Per pair, [`commutes`],
/// a pure function of two classes and that kind set. The shared clauses —
/// mandatory, static, under CR 614.5, not counter-derived, no rider — are
/// what the whole argument assumes: an optional is a second prompt whose
/// answer can differ per order; a `Uses::Once` or `NextDamage` spends a
/// registry row; an exempt effect may re-apply; a counter-derived instance is
/// re-synthesized per gather; riders queue in choice order and run in queue
/// order (CR 615.5), so two members that both carry one make the order
/// observable in the log even when the board is identical.
///
/// **What happens if it runs twice** (§4.1's second half), per class — the
/// property each commuting cell rests on, and what [`check_order_invariance`]
/// computes the other way in debug builds:
/// - *Multiplier by n ≥ 1*: multiplication commutes and leaves every kind on
///   "one or more"'s side, so no member falls out of applicability
///   (re-gather). Over an entry its filter must be mods-invariant, since
///   +1/+1 counters feed power — item 47's condition (c), fired at RE-5.
/// - *Additive*: addition commutes with addition on any kind; with a
///   multiplier only on disjoint kinds (3 → 6 → 8 or 3 → 5 → 10, Torbran's
///   ruling); with an `EnterWith` only on kinds the mods already hold, since
///   CR 614.5 gives the plus one opportunity and a kind written afterwards is
///   not raised (re-gather).
/// - *Mods-adding*: an `EnterWith` writes kinds and reads none — fixed
///   amounts, or a source that is not the entering object (the frame it would
///   read, §5b), and a filter no mods field feeds. Two merge in either order
///   (re-gather); beside a multiplier it commutes only on disjoint kinds,
///   since doubling before or after the write differs.
/// - *Draw doubler*: [`draw_doubler_commutes`] — the product, on inners that
///   carry one cause (re-gather against the first inner).
/// - *Substitute*: one instance-invariant, idempotent `Instead` shared by its
///   peers, so every trace ends at `T(e)` whatever `k` (each suppressed
///   member substituted, and the chosen one re-applied to its own output).
///   Beside a multiplier only the kind-changing creation with `count:
///   ReplacedAmount` and `mode: Replace` commutes — repeating defs and
///   replacing each matched def reach the same count either way, Divine
///   Visitation beside Parallel Lives, the pair `backlog.md` §2.29 named as
///   the sixth shape (re-gather).
/// - *Exit*: one `Instead(ZoneChangeTo)` on an entry absorbs everything a
///   mods-shaped member or an arithmetic one writes — applied last it
///   discards the mods, applied first nothing entry-shaped matches what is
///   left (substituted against the entry with its mods disturbed). Two exits
///   are a real choice.
///
/// **Expiry conditions**, each a compile error somewhere (`codebase-state.md`
/// item 47): a new `EnterModsTemplate` field breaks `is_fixed`; a new
/// `ObjectFilter` leaf breaks [`filter_is_mods_invariant`]; a new
/// `EventPattern::EnterBattlefield` field breaks `pattern_watches`' entry arm;
/// a new pattern arm breaks [`EventPattern::reads_the_amount`]; a new template
/// arm breaks [`template_is_instance_invariant`] and [`template_is_idempotent`];
/// a new `Rewrite` or `AmountRewrite` arm breaks [`classify`]. Whoever fixes
/// the error re-reads the cell it lands in. The exit cell goes false the day
/// [`substitute`]'s `ZoneChangeTo` leg reads the entry's mods, which the debug
/// check asks of every exit suppression.
fn ordering_cannot_change_outcome(
    choosable: &[Candidate],
    entering: Option<ObjectId>,
    event: &GameAction,
) -> bool {
    if !choosable.iter().all(|c| shared_clauses_hold(&c.instance)) {
        return false;
    }
    let Some(classes) = choosable
        .iter()
        .map(|c| classify(&c.instance, entering, event))
        .collect::<Option<Vec<Commuting>>>()
    else {
        return false;
    };
    let present = kinds_present(event);
    (0..classes.len())
        .all(|i| (i + 1..classes.len()).all(|j| commutes(&classes[i], &classes[j], &present)))
}

/// The clauses every commuting cell assumes of a member — see
/// [`ordering_cannot_change_outcome`].
fn shared_clauses_hold(instance: &ReplacementInstance) -> bool {
    let def = &instance.def;
    !def.optional
        && def.then.is_none()
        && matches!(def.uses, Uses::Static)
        && !def.exempt_from_614_5
        && !matches!(instance.id, ReplacementInstanceId::Counter(..))
}

/// What a candidate's application does to the event that another candidate
/// could read — [`ordering_cannot_change_outcome`]'s classes. A candidate with
/// no class keeps CR 616.1's question.
#[derive(Debug, Clone, PartialEq)]
enum Commuting<'a> {
    /// `Amount(Multiplier(n ≥ 1))` on a pattern that reads no amount, over the
    /// kinds it touches.
    Multiplier(Kinds),
    /// `Amount(Plus(k))`, over the kinds it touches.
    Additive(Kinds),
    /// An `EnterWith` that writes these kinds and reads nothing an
    /// application changes.
    ModsAdding(Kinds),
    /// [`draw_doubler_commutes`]'s member.
    DrawDoubler,
    /// An instance-invariant, idempotent `Instead`, compared by rewrite
    /// equality with its peers.
    Substitute(&'a Rewrite),
    /// `Instead(ZoneChangeTo)` on an entry.
    Exit,
}

/// The counter kinds a member touches: a counter pattern's one kind; every
/// kind for a pattern with no kind axis, since damage, life and a creation
/// have one amount; the kinds an `EnterWith` writes.
#[derive(Debug, Clone, PartialEq)]
enum Kinds {
    All,
    These(Vec<CounterType>),
}

impl Kinds {
    fn disjoint(&self, other: &Kinds) -> bool {
        match (self, other) {
            (Kinds::These(a), Kinds::These(b)) => a.iter().all(|k| !b.contains(k)),
            (Kinds::All, Kinds::These(b)) | (Kinds::These(b), Kinds::All) => b.is_empty(),
            (Kinds::All, Kinds::All) => false,
        }
    }

    /// These kinds minus `present` — what an `EnterWith` would write *new*.
    fn without(&self, present: &[CounterType]) -> Kinds {
        match self {
            Kinds::All => Kinds::All,
            Kinds::These(v) => {
                Kinds::These(v.iter().copied().filter(|k| !present.contains(k)).collect())
            }
        }
    }
}

/// The kinds a pattern is about — a counter pattern's kind, or every kind.
fn kinds_of(pattern: &EventPattern) -> Kinds {
    match pattern {
        EventPattern::AddCounters { counter: Some(k), .. } => Kinds::These(vec![*k]),
        _ => Kinds::All,
    }
}

/// The kinds an entry's mods hold now, with one or more of each; empty for
/// any other event.
fn kinds_present(event: &GameAction) -> Vec<CounterType> {
    match event {
        GameAction::EnterBattlefield { mods, .. } => {
            mods.counters.iter().filter(|r| r.n >= 1).map(|r| r.counter).collect()
        }
        _ => Vec::new(),
    }
}

/// A member's class on this event, or `None` for one the table has no word
/// for. **Matched exhaustively over `Rewrite` and `AmountRewrite`**, so a new
/// arm has to classify itself rather than defaulting to "safe" — the one
/// place the expiry conditions are a compile error for the arithmetic.
fn classify<'a>(
    instance: &'a ReplacementInstance,
    entering: Option<ObjectId>,
    event: &GameAction,
) -> Option<Commuting<'a>> {
    let def = &instance.def;
    let on_entry = matches!(event, GameAction::EnterBattlefield { .. });
    // A filter over an entering permanent reads the CR 614.12 frame, which
    // +1/+1 counters feed; over a finished permanent it reads the board,
    // which no count in the proposal touches.
    let arithmetic_ok = !def.pattern.reads_the_amount()
        && (!on_entry || object_set_is_mods_invariant(&def.affected_objects));
    match &def.rewrite {
        Rewrite::Amount(AmountRewrite::Multiplier(n)) => {
            (*n >= 1 && arithmetic_ok).then(|| Commuting::Multiplier(kinds_of(&def.pattern)))
        }
        Rewrite::Amount(AmountRewrite::Plus(_)) => {
            arithmetic_ok.then(|| Commuting::Additive(kinds_of(&def.pattern)))
        }
        // A halving can carry a kind from one to zero, out of "one or more";
        // the prevention arms spend a count or allocate; the floor reads a
        // life total. Each is a real order beside anything.
        Rewrite::Amount(
            AmountRewrite::Halve(_)
            | AmountRewrite::PreventHalf(_)
            | AmountRewrite::PreventUpTo(_)
            | AmountRewrite::PreventRemaining
            | AmountRewrite::LifeFloor(_),
        ) => None,
        // Reads the frame only when the source is the object being computed,
        // so anything else is a board read and commutes.
        Rewrite::EnterWith(t) => ((t.is_fixed() || Some(instance.source) != entering)
            && object_set_is_mods_invariant(&def.affected_objects))
        .then(|| Commuting::ModsAdding(Kinds::These(t.counters.iter().map(|c| c.counter).collect()))),
        // Devour prompts and moves the board; a control change is CR 616.1b's
        // own forced step; a prevention and a redirection change what the
        // others read.
        Rewrite::EnterAfterMoving(_)
        | Rewrite::EnterUnderControlOf(_)
        | Rewrite::Prevent
        | Rewrite::Retarget(_) => None,
        Rewrite::Instead(template) => {
            if on_entry && is_exit(&def.rewrite) {
                return Some(Commuting::Exit);
            }
            if draw_doubler_commutes(def, event) {
                return Some(Commuting::DrawDoubler);
            }
            (template_is_instance_invariant(template) && template_is_idempotent(template))
                .then_some(Commuting::Substitute(&def.rewrite))
        }
    }
}

/// The table: do two members' applications reach one outcome in either
/// order? `present` is the kinds an entry's mods hold before either applies.
/// The cells that commute are listed; every other pair is a real choice.
fn commutes(a: &Commuting, b: &Commuting, present: &[CounterType]) -> bool {
    use Commuting::*;
    match (a, b) {
        (Multiplier(_), Multiplier(_)) | (Additive(_), Additive(_)) => true,
        (Multiplier(m), Additive(p)) | (Additive(p), Multiplier(m)) => m.disjoint(p),
        (Multiplier(m), ModsAdding(w)) | (ModsAdding(w), Multiplier(m)) => m.disjoint(w),
        (Additive(p), ModsAdding(w)) | (ModsAdding(w), Additive(p)) => {
            p.disjoint(&w.without(present))
        }
        (ModsAdding(_), ModsAdding(_)) => true,
        (DrawDoubler, DrawDoubler) => true,
        (Substitute(x), Substitute(y)) => x == y,
        (Multiplier(_), Substitute(r)) | (Substitute(r), Multiplier(_)) => {
            replaces_that_many(r)
        }
        (Exit, ModsAdding(_) | Multiplier(_) | Additive(_))
        | (ModsAdding(_) | Multiplier(_) | Additive(_), Exit) => true,
        _ => false,
    }
}

/// The substitutes a multiplier commutes with: a creation replaced
/// kind-for-kind by "that many" — repeating each def and replacing each
/// matched def reach the same count either way — and a mana production
/// retyped at its own amount, Deep Water beside Mana Reflection, where
/// retype-then-double and double-then-retype are one event. An `Append` does
/// not (the appended count is read once), a `Fixed` retype does not
/// (`Fixed(1)` then ×2 is 2, ×2 then `Fixed(1)` is 1 — Contamination beside
/// Mana Reflection is CR 616.1's real question), and every other template is
/// about a different event.
fn replaces_that_many(rewrite: &Rewrite) -> bool {
    matches!(
        rewrite,
        Rewrite::Instead(GameActionTemplate::CreateTokens {
            count: TemplateAmount::ReplacedAmount,
            mode: TokenSubstitution::Replace,
            ..
        }) | Rewrite::Instead(GameActionTemplate::ProduceMana {
            amount: TemplateAmount::ReplacedAmount,
            ..
        })
    )
}

/// The exit an entry can take instead of entering — [`Commuting::Exit`].
fn is_exit(rewrite: &Rewrite) -> bool {
    matches!(rewrite, Rewrite::Instead(GameActionTemplate::ZoneChangeTo { .. }))
}

/// [`Commuting::DrawDoubler`]'s clause: a doubler that
/// leaves the draw where it is and admits every cause the decomposition can
/// stamp on an inner.
///
/// `player: None` keeps the subject, so the members differ only in `n` and the
/// total is their product. The `cause` test is the premise's second half: the
/// inners of a substituted instruction carry the parent's cause once and
/// [`DrawCause::Effect`] thereafter, so a member that admits one and not the
/// other applies to some inners and not others, and the orders diverge.
fn draw_doubler_commutes(def: &ReplacementDef, event: &GameAction) -> bool {
    let EventPattern::DrawCard { cause } = def.pattern else {
        return false;
    };
    let Rewrite::Instead(GameActionTemplate::DrawCards {
        n: TemplateAmount::Fixed(n),
        player: None,
    }) = &def.rewrite
    else {
        // A `ReplacedAmount` doubler would be the identity and would commute
        // with anything, but nothing prints one — and a member with no class
        // keeps CR 616.1's question, which is the safe direction for a
        // premise.
        return false;
    };
    let GameAction::DrawCard { cause: actual, .. } = event else {
        return false;
    };
    *n >= 1
        && cause.map(|c| c == *actual).unwrap_or(true)
        && cause.map(|c| c == DrawCause::Effect).unwrap_or(true)
}


/// Does this template produce the same `GameAction` whichever instance applies
/// it?
///
/// The leaf table [`Commuting::Substitute`] rests on. Matched
/// exhaustively, so a new template arm has to classify itself rather than
/// defaulting to "safe" — and the two `false`s are the reason the table is not
/// a constant.
fn template_is_instance_invariant(template: &GameActionTemplate) -> bool {
    match template {
        // Built from the event's own object and `from`.
        GameActionTemplate::ZoneChangeTo { .. } => true,
        // Built from the event's subject.
        GameActionTemplate::RemoveCountersFromAffected { .. } => true,
        // `None` keeps the event's own player; `PlayerRef::You` is
        // `chosen.controller` and `Owner`/`Opponent` are refused by
        // `draw_recipient`, so only the explicit `You` varies per instance.
        GameActionTemplate::DrawCards { player, .. } => {
            !matches!(player, Some(PlayerRef::You))
        }
        // CR 609.6's source is the *applying* effect's own, which is exactly
        // what varies between two otherwise-identical statics — and it is
        // visible on `GameEvent::LifeChanged` and to item 6's "whenever a
        // source causes you to gain life".
        GameActionTemplate::GainLife { .. } => false,
        // A substituted loss carries `LifeLossCause::Effect` and no source.
        GameActionTemplate::LoseLife { .. } => true,
        // Built from the event's subject: "you win" on Laboratory Maniac is
        // the drawing player, who is also its controller by the def's
        // `PlayerSet::You`, so no instance's own field reaches the event.
        GameActionTemplate::PlayerWins => true,
        // Built from the template's def and the event's defs; the applying
        // instance contributes its pattern's kind, which is def data.
        GameActionTemplate::CreateTokens { .. } => true,
        // The type and the amount are def data, identical across two Deep
        // Waters; the units come from the event.
        GameActionTemplate::ProduceMana { .. } => true,
    }
}

/// Does applying this template to its own output produce that output again?
///
/// [`Commuting::Substitute`]'s second clause, and the one that
/// is actually load-bearing: order can change how many of the shared members
/// apply, so the shape needs `T^k(e) = T(e)`.
///
/// **Every arm is idempotent today for one structural reason** — an `Instead`
/// *overwrites* the event rather than accumulating into it, and the only field
/// any template reads off the event is one the previous application already set
/// to the value it will read. `TemplateAmount::ReplacedAmount` is the case to
/// look at: applied to a gain of 3 it produces a loss of 3, and applied to
/// *that* it reads 3 again.
///
/// Matched exhaustively, so a new arm classifies itself. The arm that would
/// answer `false` is one whose output depends on the event in a way that
/// compounds — "loses twice that much life instead" as a template rather than
/// as an `AmountRewrite`, which is why doubling lives on that type.
///
/// **Why `k` is not pinned.** `applies_to` resolves the affected sets against
/// *each instance's own* controller and source, so two defs that are `==` as
/// data can differ on one event (`PlayerSet::Opponents` around two different
/// permanents is the printed case): order can change **which** members apply
/// and **how many**, and idempotence is what makes that not matter. `Rewrite`
/// derives `PartialEq`, so "the same rewrite" is def data; nothing here
/// compares game state. **The residual, named:** a candidate that becomes
/// applicable only after the rewrite (CR 616.2) joins a later `choosable`
/// beside whichever members have not applied, and those differ by order.
/// Their *outcomes* are equal by the argument above; what differs is the
/// source list a prompt would name, no printed card reaches it, and it is a
/// different question from the one the predicate answers.
fn template_is_idempotent(template: &GameActionTemplate) -> bool {
    match template {
        GameActionTemplate::ZoneChangeTo { .. } => true,
        GameActionTemplate::RemoveCountersFromAffected { .. } => true,
        GameActionTemplate::DrawCards { .. } => true,
        GameActionTemplate::GainLife { .. } => true,
        GameActionTemplate::LoseLife { .. } => true,
        // Derived rather than read off: `T(e)` preserves the subject, so T(T(e)) = T(e)
        // — and a win is terminal (CR 104.1), so no k > 1 trace is ever performed.
        // Two Laboratory Maniacs on one draw are one outcome, no prompt.
        GameActionTemplate::PlayerWins => true,
        // Retyping twice is retyping once, and `Fixed(n)` twice is `Fixed(n)`.
        GameActionTemplate::ProduceMana { .. } => true,
        // Replacing is idempotent — the Angels a second Visitation would
        // replace are Angels already. Appending is not: a second Chatterfang
        // joins Squirrels to the Squirrels, and how many is the order's, so
        // two of them keep CR 616.1's question.
        GameActionTemplate::CreateTokens { mode, .. } => {
            matches!(mode, TokenSubstitution::Replace)
        }
    }
}

/// The kind a creation pattern names, or `None` for any pattern that is not
/// one — a rewrite over a creation reads it off the applying instance.
fn pattern_kind(pattern: &EventPattern) -> Option<&TokenKind> {
    match pattern {
        EventPattern::CreateTokens { kind } => kind.as_ref(),
        _ => None,
    }
}

/// Does `def` fall under `kind`? No kind is every kind.
fn kind_matches(kind: Option<&TokenKind>, def: &TokenDef) -> bool {
    kind.is_none_or(|k| k.matches(def))
}

/// Can no `EnterMods` field change whether this set matches the entering
/// object? `SourceOnly`, `Fixed` and `Host` match by id; a
/// `Filter` is invariant iff every leaf is. The entry half of
/// [`ordering_cannot_change_outcome`]'s premise.
///
/// A `Filter`'s `zones` needs no arm: no `EnterMods` field moves an object
/// between zones, so the zone half is invariant whatever it holds.
fn object_set_is_mods_invariant(affected: &ObjectSet) -> bool {
    match affected {
        ObjectSet::SourceOnly | ObjectSet::Fixed(_) | ObjectSet::Host => true,
        ObjectSet::Filter { filter, .. } => filter_is_mods_invariant(filter),
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
///
/// **The draw shape asks about a different event, and that is the point.** A
/// suppressed draw doubler does not apply to the `DrawCards` the chosen one
/// produced — no `EventPattern::DrawCard` watches an instruction — it applies to
/// the inners that instruction decomposes into. So for that shape the gather is
/// taken against the **first inner**, which is the one carrying the parent's
/// cause; the premise's `Effect` clause is what makes checking one inner enough.
///
/// **And the substitute cell asks a different *question*, not a different event.**
/// Its members share one `Instead`, so a suppressed one does not have to keep
/// applying — Tainted Remedy's own ruling is that it stops. What that shape
/// claims instead is that every member would have produced the *same* event, so
/// that is what is asserted, by running each suppressed member's own
/// [`substitute`] against the event the chosen one replaced. A shape that cannot
/// be checked the same way needs its own check rather than an exemption
/// (`replacement-architecture.md` §11 item 55).
fn check_order_invariance(
    game: &GameState,
    ctx: &ActionContext,
    chosen: &ReplacementInstance,
    before: &GameAction,
    next: &GameAction,
    subject: EventSubject,
    member: usize,
    unsuppressed: &[(ReplacementInstance, Vec<usize>)],
) {
    if !cfg!(debug_assertions) || unsuppressed.is_empty() {
        return;
    }
    let mine: Vec<&(ReplacementInstance, Vec<usize>)> =
        unsuppressed.iter().filter(|(_, m)| m.contains(&member)).collect();
    if mine.is_empty() {
        return;
    }

    // The substitute cell: one shared `Instead`, so the claim is sameness of
    // output rather than continued applicability.
    if let Rewrite::Instead(_) = &chosen.def.rewrite
        && mine.iter().all(|(i, _)| i.def.rewrite == chosen.def.rewrite) {
        for (instance, _) in &mine {
            let Rewrite::Instead(template) = &instance.def.rewrite else {
                unreachable!("equal to the chosen rewrite, which is an `Instead`");
            };
            let theirs = substitute(instance, template, before.clone(), subject);
            debug_assert!(
                theirs.as_ref().ok() == Some(next),
                "CR 616.1 prompt suppressed as order-invariant was not: {:?} would \
                 have produced {:?} where {:?} produced {:?}. \
                 `template_is_instance_invariant` admitted a substitute that reads \
                 the applying effect.",
                instance.id,
                theirs,
                chosen.id,
                next
            );
        }
        // The second clause, checked where it is cheap: applying the same
        // substitute to its own output must not move it, or `k` — how many
        // of the shared members ended up applying, which order *can* change
        // — becomes observable.
        let Rewrite::Instead(template) = &chosen.def.rewrite else {
            unreachable!("matched one line above");
        };
        if let Ok(again) = substitute(chosen, template, next.clone(), subject) {
            debug_assert!(
                &again == next,
                "CR 616.1 prompt suppressed as order-invariant was not: {:?} is not                      idempotent — it took {:?} to {:?}. Order decides how many of the                      shared members apply, so a substitute that compounds makes that                      observable.",
                chosen.id,
                next,
                again
            );
        }
        return;
    }

    // The exit cell: the exit is applied first, so continued applicability is not
    // the claim — the claim is that its substitute ignores what the mods-shaped
    // and arithmetic members would have written, checked against the same entry
    // with its mods disturbed.
    if is_exit(&chosen.def.rewrite) {
        if let (
            Rewrite::Instead(template),
            GameAction::EnterBattlefield { object, from, controller, mods, cause },
        ) = (&chosen.def.rewrite, before)
        {
            let mut disturbed = mods.clone();
            disturbed.tapped = !disturbed.tapped;
            disturbed.counters.push(EntryCounters {
                counter: CounterType::PlusOnePlusOne,
                n: 1,
                by: None,
            });
            let with_mods = GameAction::EnterBattlefield {
                object: *object,
                from: *from,
                controller: *controller,
                mods: disturbed,
                cause: *cause,
            };
            let theirs = substitute(chosen, template, with_mods, subject);
            debug_assert!(
                theirs.as_ref().ok() == Some(next),
                "CR 616.1 prompt suppressed as order-invariant was not: the exit {:?} \
                 reads the entry's mods — {:?} against the disturbed entry, {:?} \
                 against the proposed one.",
                chosen.id,
                theirs,
                next
            );
        }
        return;
    }

    let probe = match next {
        GameAction::DrawCards { player, cause, .. } => {
            GameAction::DrawCard { player: *player, cause: *cause }
        }
        other => other.clone(),
    };
    let frame = EntryFrame::new(game, &probe);
    let still: Vec<ReplacementInstanceId> = gather(game, &probe, ctx, false, &frame)
        .into_iter()
        .map(|c| c.id)
        .collect();
    for (instance, _) in &mine {
        debug_assert!(
            still.contains(&instance.id),
            "CR 616.1 prompt suppressed as order-invariant was not: {:?} stopped applying to {:?}",
            instance.id,
            probe
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
    cause: Option<PlayerId>,
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
    if applies_to(game, chosen, next, subject_of(next), cause, Some(&frame)) {
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
/// always does, since a `Prevent` on a `Destroy` cannot do nothing — and
/// `Uses::NextDamage` is reduced by exactly the damage prevented (CR 615.7),
/// which for an unpreventable event (CR 615.12) or a `Once` half that rounded
/// to nothing is 0. Either way a shield spent on one group of a batch is gone
/// or reduced when the next group asks, because this writes game state.
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
    // Every arm but `Amount` changes the event whenever it is chosen, and
    // prevents nothing (CR 615.1a).
    let changed = Applied { took_effect: true, prevented: 0 };
    match &chosen.def.rewrite {
        // CR 614.6 / 615.6 — the event does not happen. On damage the whole amount is
        // what CR 615.5's rider reads (Reverse Damage's gain), reported here because
        // only this arm knows the event was damage: a `Prevent` on a destruction is
        // regeneration and prevents no damage (CR 615.1). CR 615.12's first site: an
        // unpreventable event survives untouched and nothing is spent; `prevented_or_0`
        // is the shared consult, and the `Amount` arm below is the second site.
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

        // CR 614.1c/d — the event still happens; only *how* changes. Merged into the
        // proposal rather than substituted, so CR 616.1f's re-gather accumulates
        // ("enters tapped" plus "enters with two charge counters", in either order)
        // and CR 614.5's applied set is what makes the merge terminate. CR 101.2 at
        // the door: a "can't have counters put on it" refuses the counters (CR 614.17d).
        Rewrite::EnterWith(template) => match event {
            GameAction::EnterBattlefield { object, from, controller, mut mods, cause } => {
                let extra = evaluate_enter_template(
                    game, template, chosen, object, controller, &mods,
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

        // CR 614.5's doublers and CR 615.10's partial prevention: the arm that reads
        // the amount the last application left, which is what makes CR 616.1's order
        // observable. CR 615.12's second site — both arms ask `is_unpreventable` the
        // same way (`replacement-architecture.md` §9, RD-4's "As landed").
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
                    // Refused here so `AmountRewrite::apply` never answers for it: the clamp is
                    // about a life total, and damage has none — Ali from Cairo watches the loss
                    // CR 120.3a contains inside the damage, the leg below.
                    (AmountRewrite::LifeFloor(_), _) => {
                        return Err(format!(
                            "replacement {:?} floors a life total but matched damage. \
                             CR 120.3a's contained loss is the event a floor is about, \
                             and the printed card says so: the effect \"does not prevent \
                             damage, it prevents the damage from turning into loss of \
                             life\".",
                            chosen.id
                        ))
                    }
                    (other, _) => *other,
                };
                // The consult is on the prevention arms only: Ghosts of the Innocent halves
                // Excruciator's unpreventable 7 to 3 and Gisela prevents none of it — why
                // `Halve` and `PreventHalf` are two arms (CR 615.12, §11 item 28).
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

            // CR 119.3's gain and CR 120.3a's loss on the same arithmetic minus CR 615
            // (Rhox Faithmender, Bloodletter of Aclazotz, Ali from Cairo's clamp). A
            // prevention arm here is an authoring error: CR 615.1 is about damage.
            GameAction::GainLife { player, amount, source } => {
                let after = life_arithmetic(chosen, *amount_rewrite, amount, None)?;
                Ok((
                    Some(GameAction::GainLife { player, amount: after, source }),
                    Applied { took_effect: after != amount, prevented: 0 },
                ))
            }
            GameAction::LoseLife { player, amount, cause } => {
                // The clamp's board read: CR 614.1a lets Ali from Cairo modify the loss by
                // how much of it the total can take, a fact about the player now
                // (`codebase-state.md` item 53). A read, not a write.
                let life = game.get_player(player)?.life_total;
                let after = life_arithmetic(chosen, *amount_rewrite, amount, Some(life))?;
                Ok((
                    Some(GameAction::LoseLife { player, amount: after, cause }),
                    Applied { took_effect: after != amount, prevented: 0 },
                ))
            }

            // CR 614.16's token half (Parallel Lives, Anointed Procession, Doubling
            // Season). A multiplier repeats each def the kind matched **in place**
            // (`[A, B]` → `[A, A, B, B]`: "twice as many of each kind", Anointed
            // Procession's ruling), which keeps the creation's batch order and so its
            // CR 613.7 timestamps; an unmatched def is left alone. Every other arm is an
            // authoring error — a printed "plus" is `CreateTokens { mode: Append }`.
            GameAction::CreateTokens { defs, controller } => match amount_rewrite {
                AmountRewrite::Multiplier(n) => {
                    let n = usize::try_from(*n).map_err(|_| {
                        format!(
                            "replacement {:?} multiplies a token creation by {}, which no \
                             board can hold",
                            chosen.id, n
                        )
                    })?;
                    let kind = pattern_kind(&chosen.def.pattern);
                    let before = defs.len();
                    let defs: Vec<TokenDef> = defs
                        .into_iter()
                        .flat_map(|d| {
                            let times = if kind_matches(kind, &d) { n } else { 1 };
                            std::iter::repeat_n(d, times)
                        })
                        .collect();
                    let took_effect = defs.len() != before;
                    Ok((
                        Some(GameAction::CreateTokens { defs, controller }),
                        Applied { took_effect, prevented: 0 },
                    ))
                }
                other => Err(format!(
                    "replacement {:?} applies {:?} to a token creation; CR 614.16's token \
                     half is a multiplier, and the printed \"plus\" adds a named token, \
                     which is `GameActionTemplate::CreateTokens` and not arithmetic",
                    chosen.id, other
                )),
            },

            // CR 614.16's counter half — Doubling Season's second ability,
            // Hardened Scales' plus, Vorinclex's halving — over a count of
            // counters being put on a permanent or a player.
            GameAction::AddCounters { subject, counter, n, by } => {
                let after = counter_arithmetic(chosen, *amount_rewrite, n)?;
                Ok((
                    Some(GameAction::AddCounters { subject, counter, n: after, by }),
                    Applied { took_effect: after != n, prevented: 0 },
                ))
            }

            // CR 122.6's second door: the counters a permanent enters with are *put on*
            // it, so the arithmetic applies per matched kind in the entry's mods, with
            // the entry's controller as putter (CR 122.6a). A kind a halving takes to
            // zero leaves the mods — "enters with counters" is not "with zero counters".
            GameAction::EnterBattlefield { object, from, controller, mut mods, cause } => {
                let EventPattern::AddCounters { counter: kind, by } = &chosen.def.pattern
                else {
                    return Err(format!(
                        "replacement {:?} changes an amount on an entry but its pattern is \
                         {:?}; only a `AddCounters` watches an entry's counters (CR 122.6)",
                        chosen.id, chosen.def.pattern
                    ));
                };
                let mut took_effect = false;
                let mut kept = Vec::with_capacity(mods.counters.len());
                for row in mods.counters.drain(..) {
                    let matched = kind.is_none_or(|c| c == row.counter)
                        && by
                            .as_ref()
                            .is_none_or(|set| set.contains(chosen.controller, row.putter(controller)));
                    let after =
                        if matched { counter_arithmetic(chosen, *amount_rewrite, row.n)? } else { row.n };
                    took_effect |= after != row.n;
                    if after > 0 {
                        kept.push(EntryCounters { n: after, ..row });
                    }
                }
                mods.counters = kept;
                Ok((
                    Some(GameAction::EnterBattlefield { object, from, controller, mods, cause }),
                    Applied { took_effect, prevented: 0 },
                ))
            }

            // CR 106.6a — "replacement effects [that] increase the amount of mana
            // produced" (Mana Reflection, Nyxbloom Ancient). Every plain entry is scaled
            // and every restricted atom repeated `n` times in place: the rule's next
            // sentence applies restrictions "to all mana produced", and an atom is one
            // unit carrying its restrictions (`ATOM-106.6a-001`). Every other arm is a
            // pairing error — nothing prints "one more mana" as a replacement, and
            // nothing halves mana. `took_effect` is any unit changing.
            GameAction::ProduceMana { player, source, mana, special, tapped_for_mana } => {
                match amount_rewrite {
                    AmountRewrite::Multiplier(n) => {
                        let times = usize::try_from(*n).map_err(|_| {
                            format!(
                                "replacement {:?} multiplies a mana production by {}, which no \
                                 pool can hold",
                                chosen.id, n
                            )
                        })?;
                        let scaled: Vec<(ManaType, u64)> =
                            mana.iter().map(|(t, a)| (*t, a.saturating_mul(*n))).collect();
                        let repeated: Vec<ManaAtom> = special
                            .iter()
                            .flat_map(|atom| std::iter::repeat_n(atom.clone(), times))
                            .collect();
                        let took_effect = scaled != mana || repeated.len() != special.len();
                        Ok((
                            Some(GameAction::ProduceMana {
                                player,
                                source,
                                mana: scaled,
                                special: repeated,
                                tapped_for_mana,
                            }),
                            Applied { took_effect, prevented: 0 },
                        ))
                    }
                    other => Err(format!(
                        "replacement {:?} applies {:?} to a mana production; CR 106.6a's \
                         increase is a multiplier, and the printed \"add an additional\" is \
                         CR 605.1b's triggered mana ability rather than a replacement",
                        chosen.id, other
                    )),
                }
            }

            // CR 701.22's count — Kenessos, Priest of Thassa's "scry that many cards plus
            // one instead", the one arithmetic "would scry" clause. The same
            // `counter_arithmetic` as the counter legs: a plain count, so the prevention
            // arms and the floor are pairing errors here too. A scry taken to 0 is not
            // dropped here: `never_happens` re-asks at the top of the next iteration.
            GameAction::Scry { player, n } => {
                let after = plain_arithmetic(chosen, *amount_rewrite, n)?;
                Ok((
                    Some(GameAction::Scry { player, n: after }),
                    Applied { took_effect: after != n, prevented: 0 },
                ))
            }

            // Pattern and rewrite describe different events — the same authoring error
            // every arm reports. `DrawCards` carries CR 121.2a's count for a rider to
            // read; `Rewrite::Amount`'s arithmetic is CR 615's and only about damage.
            other => Err(format!(
                "replacement {:?} changes an amount but matched {:?}, which has no `Rewrite::Amount` arm",
                chosen.id, other
            )),
        },

        // CR 614.9 — the same damage, somewhere else: `target` changes and nothing
        // else (Pariah's and Kor Chant's rulings). The re-check is here rather than
        // in `applies_to` because the difference is observable: a redirect whose
        // destination is gone is still gathered, offered and chosen, then does
        // nothing and is not spent (`ATOM-614.9-001`).
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

        // Every leg is a pure function of the event and the applying instance
        // — see [`substitute`], which is where they live so that the fourth
        // suppression shape can call them without a board.
        Rewrite::Instead(template) => {
            substitute(chosen, template, event, subject).map(|a| (Some(a), changed))
        }
    }
}

/// A [`Rewrite::Amount`] over a count of counters — CR 614.16's counter half:
/// Doubling Season's multiplier, Hardened Scales' plus, Vorinclex's halving.
/// The prevention arms are CR 615's and about damage, and the clamp is about
/// a life total; both are the pairing error every other leg reports. A count
/// no `u32` can hold is refused rather than wrapped, `CreateTokens`' reason.
fn counter_arithmetic(
    chosen: &ReplacementInstance,
    arm: AmountRewrite,
    n: u32,
) -> Result<u32, String> {
    let after = plain_arithmetic(chosen, arm, n as u64)?;
    u32::try_from(after).map_err(|_| {
        format!(
            "replacement {:?} takes a counter count to {}, which no permanent or player can hold",
            chosen.id, after
        )
    })
}

/// [`Rewrite::Amount`]'s arithmetic over a **plain count** — a number with no
/// life total under it and nothing to prevent.
///
/// **Why this is a second function and not `counter_arithmetic` widened to
/// `u64`** (RE-8's review): the ceiling is the two `u32` call sites —
/// `GameAction::AddCounters::n` and `EntryCounters::n`, because the counter
/// maps are `u32` — so a `u64` helper would hand each a number it still had
/// to narrow, with the same `try_from` and the same "which no permanent or
/// player can hold" message twice. A scry's N is CR 701.22a's `u64` and
/// narrows to nothing; routing it through the counter helper was a silent
/// wrap on a number a card could author, with a counter's error message.
///
/// The names stay two words apart on purpose: `count_arithmetic` beside
/// `counter_arithmetic` would be the coin flip at every call site that
/// `ReplacementDef::affecting_players` already refused once.
///
/// **The refused arms are refused here, once.** A prevention arm (CR 615) is
/// about damage and a [`AmountRewrite::LifeFloor`] is about a life total; over
/// a plain count each is a def whose pattern and rewrite describe different
/// events, which is the same card-authoring error every other arm reports.
fn plain_arithmetic(
    chosen: &ReplacementInstance,
    arm: AmountRewrite,
    n: u64,
) -> Result<u64, String> {
    match arm {
        AmountRewrite::Multiplier(_) | AmountRewrite::Halve(_) | AmountRewrite::Plus(_) => {
            Ok(arm.apply(n))
        }
        other => Err(format!(
            "replacement {:?} applies {:?} to a plain count: counters being put on              (CR 614.16), or cards scried (CR 701.22). That arithmetic is over a number,              and a prevention (CR 615) or a life floor is about damage or a life total",
            chosen.id, other
        )),
    }
}

/// A [`Rewrite::Amount`] over a life event — CR 614.1a's arithmetic, with
/// CR 615's removed.
///
/// `life` is the affected player's life total, and `None` says the event is a
/// *gain*: the clamp has no meaning over one, because a gain cannot carry a
/// total downward past a floor.
///
/// **Two arms are refused rather than computed.** A prevention arm over a life
/// event is a def whose pattern and rewrite describe different events —
/// CR 615.1 is about damage and nothing else — and [`AmountRewrite::LifeFloor`]
/// over a gain is the same error from the other side. Both are card-authoring
/// mistakes, reported the way every other half-disagreeing `ReplacementDef` is.
fn life_arithmetic(
    chosen: &ReplacementInstance,
    arm: AmountRewrite,
    amount: u64,
    life: Option<i64>,
) -> Result<u64, String> {
    match (arm, life) {
        // "Damage that would reduce your life total to less than N reduces it
        // to N instead." The clamp is on the *loss*: how much of it the total
        // can take before reaching the floor, which is 0 from a total already
        // there. A floor never hands life back — see [`AmountRewrite::LifeFloor`].
        (AmountRewrite::LifeFloor(floor), Some(life)) => {
            Ok(amount.min(life.saturating_sub(floor).max(0) as u64))
        }
        (AmountRewrite::LifeFloor(_), None) => Err(format!(
            "replacement {:?} floors a life total but matched a life gain, which carries \
             no total downward. Its `EventPattern` and its `Rewrite` describe different \
             events.",
            chosen.id
        )),
        (other, _) if other.prevents_damage() => Err(format!(
            "replacement {:?} prevents damage but matched a life gain or loss. CR 615.1 is \
             about damage, and the loss CR 120.3a contains inside it is a different event: \
             an effect that modifies one is not a prevention effect.",
            chosen.id
        )),
        (other, _) => Ok(other.apply(amount)),
    }
}

/// The number a [`TemplateAmount`] stands for, against the event being replaced.
///
/// [`TemplateAmount::ReplacedAmount`] is CR 615.5's "that much" read one step
/// earlier than a rider reads it — off the event as it stands when the
/// substitution is applied, which is what Tainted Remedy's "loses that much
/// life" means after Alhammarret's Archive has doubled it.
fn template_amount(
    chosen: &ReplacementInstance,
    amount: TemplateAmount,
    event: &GameAction,
) -> Result<u64, String> {
    match amount {
        TemplateAmount::Fixed(n) => Ok(n),
        TemplateAmount::ReplacedAmount => event_amount(event).ok_or_else(|| {
            format!(
                "replacement {:?} substitutes an event sized by the replaced event's own \
                 amount (CR 615.5's \"that much\"), but {:?} carries none to read.",
                chosen.id, event
            )
        }),
    }
}

/// The substitute event a [`Rewrite::Instead`] produces — CR 614.1a's "instead".
///
/// **A pure function of the event, the applying instance and the subject, and
/// the signature is what says so.** `apply_rewrite` takes `&mut GameState`
/// because CR 614.13's entry arm needs it; this one is extracted because
/// [`ordering_cannot_change_outcome`]'s fourth shape rests on the purity, and
/// its debug check calls this against an event it must not mutate.
///
/// # How big this gets, counted rather than guessed
///
/// **One arm per [`GameActionTemplate`] variant, and a nested match only
/// where a template reads the replaced event's *fields* or its subject.**
/// Counted 2026-09-15: eight templates and 310 lines, 110 of them comment —
/// about 25 lines of code each. `ZoneChangeTo`, `DrawCards`, `CreateTokens`
/// and `ProduceMana` read fields off the event; `RemoveCountersFromAffected`,
/// `GainLife`, `LoseLife` and `PlayerWins` read only [`subject_of`], which is
/// why the last three cost a dozen lines each as the `GameAction` vocabulary
/// grows.
///
/// So this does **not** grow as templates × actions. It grows with templates,
/// which grew 3 → 8 across RB, RC, RD and RE — roughly one a phase — against a
/// census of 574 printed "would … instead" clauses (§3.2c) that needed zero new
/// `Rewrite` arms. A thousand lines would take about forty templates.
///
/// **The split, when it is wanted, is mechanical**: one `fn substitute_<name>`
/// per template, or a method on the template itself. Nothing here reads
/// anything but `chosen`, the event and the subject, so the functions do not
/// share state — which is exactly why it is not being done speculatively now.
/// **Do it when a third template needs a nested match**, because that is the
/// point at which the arms stop being readable side by side.
///
/// The two things it reads off `chosen` are exactly the two
/// [`template_is_instance_invariant`] is about: CR 609.6's source for a
/// substituted life gain, and the controller for a draw handed to
/// [`PlayerRef::You`].
fn substitute(
    chosen: &ReplacementInstance,
    template: &GameActionTemplate,
    event: GameAction,
    subject: EventSubject,
) -> Result<GameAction, String> {
    match (template, event) {
        (
            GameActionTemplate::ZoneChangeTo { to, cause },
            GameAction::ZoneChange { object, from, .. },
        ) => Ok(GameAction::ZoneChange { object, from, to: *to, cause: *cause }),

        // Containment Priest, Hallowed Moonlight: the entry is the zone change
        // (CR 614.1c), so a card's substitute is one move from where it is and it
        // never becomes a permanent. A token has no `from` — created in the zone,
        // entity pending (`create_tokens`) — so its substitute is the creation
        // itself, elsewhere: `CreateTokenIn`, the Moonlight ruling's appearance.
        (
            GameActionTemplate::ZoneChangeTo { to, cause },
            GameAction::EnterBattlefield { object, from, .. },
        ) => Ok(match from {
            Some(from) => GameAction::ZoneChange { object, from, to: *to, cause: *cause },
            None => GameAction::CreateTokenIn { object, zone: *to },
        }),

        (GameActionTemplate::RemoveCountersFromAffected { counter, n }, _) => {
            match subject_object(subject) {
                Some(object) => Ok(GameAction::RemoveCounters {
                    subject: CounterSubject::Object(object),
                    counter: *counter,
                    n: *n,
                }),
                None => Err(format!(
                    "a `RemoveCountersFromAffected` rewrite on {:?} has no affected \
                     object to take counters from",
                    chosen.id
                )),
            }
        }

        // CR 614.1a's "instead … draw": the substitute is always the **instruction**
        // (CR 121.2a), so Alms Collector still sees the event it watches, and the
        // cause travels with it (CR 614.6), which is how Teferi's Ageless Insight's
        // exception survives doubling. The three printed legs — a draw replaced, an
        // instruction shrunk (Alms Collector: a rewrite, so the applied set carries),
        // a scry retyped (Eligeth) — are RE-2's decisions (`replacement-architecture.md`
        // §9). The amount is read before the `match` above moves the event.
        (GameActionTemplate::DrawCards { n, player }, event) => {
            let n = template_amount(chosen, *n, &event)?;
            match event {
                GameAction::DrawCard { player: affected, cause } => Ok(GameAction::DrawCards {
                    player: draw_recipient(chosen, player.as_ref(), affected)?,
                    n,
                    cause,
                }),
                GameAction::DrawCards { player: affected, cause, .. } => {
                    Ok(GameAction::DrawCards {
                        player: draw_recipient(chosen, player.as_ref(), affected)?,
                        n,
                        cause,
                    })
                }
                // A scry has no `DrawCause`; the substitute's is `Effect`, since these draws
                // are a replacement effect's and CR 121.1's turn-based draw is the step's own
                // card — so Teferi's Ageless Insight doubles Eligeth's draws, correctly.
                GameAction::Scry { player: affected, .. } => Ok(GameAction::DrawCards {
                    player: draw_recipient(chosen, player.as_ref(), affected)?,
                    n,
                    cause: DrawCause::Effect,
                }),
                // A template that cannot be built from this event is a card-
                // authoring error rather than a rules corner: the pattern is
                // what decides which events reach the rewrite, so a mismatch
                // means the two halves of one `ReplacementDef` disagree.
                other => Err(format!(
                    "replacement {:?} rewrites to a draw but matched {:?}, which is neither \
                     a card draw nor a scry. Its `EventPattern` and its `Rewrite` describe \
                     different events.",
                    chosen.id, other
                )),
            }
        }

        // CR 614.16's kind-changing substitution: the defs the kind matched are
        // replaced (Divine Visitation) or joined (Chatterfang, Xorn) by `count` of
        // the template's; unmatched defs keep their order and the new ones come
        // last, so a creation's timestamps stay its own.
        (
            GameActionTemplate::CreateTokens { def, count, mode },
            GameAction::CreateTokens { defs, controller },
        ) => {
            let kind = pattern_kind(&chosen.def.pattern);
            let matched = defs.iter().filter(|d| kind_matches(kind, d)).count();
            let n = match count {
                TemplateAmount::Fixed(n) => *n,
                TemplateAmount::ReplacedAmount => matched as u64,
            };
            let n = usize::try_from(n).map_err(|_| {
                format!("replacement {:?} substitutes {} tokens, which no board can hold", chosen.id, n)
            })?;
            let out: Vec<TokenDef> = match (mode, count) {
                // Each matched def becomes one of the template's, in place, keeping how the
                // creating effect said it enters — Divine Visitation's ruling ("anything
                // else specified … such as tapped … still applies").
                (TokenSubstitution::Replace, TemplateAmount::ReplacedAmount) => defs
                    .into_iter()
                    .map(|d| {
                        if kind_matches(kind, &d) {
                            TokenDef { enters_tapped: d.enters_tapped, ..def.clone() }
                        } else {
                            d
                        }
                    })
                    .collect(),
                (TokenSubstitution::Replace, TemplateAmount::Fixed(_)) => defs
                    .into_iter()
                    .filter(|d| !kind_matches(kind, d))
                    .chain(std::iter::repeat_n(def.clone(), n))
                    .collect(),
                (TokenSubstitution::Append, _) => {
                    defs.into_iter().chain(std::iter::repeat_n(def.clone(), n)).collect()
                }
            };
            Ok(GameAction::CreateTokens { defs: out, controller })
        }

        // CR 614.1a's kind changes: a draw becomes a gain (Words of Worship), a gain
        // a loss (Tainted Remedy). The subject does not move, so neither carries a
        // `player`; an `EventSubject::Object` is the authoring error. A substituted
        // loss is `LifeLossCause::Effect`, never `Damage` — calling it damage would
        // hand it to Ali from Cairo's clamp, which watches CR 120.3a's loss only.
        (GameActionTemplate::GainLife { amount }, event) => match subject_of(&event) {
            EventSubject::Player(player) => Ok(GameAction::GainLife {
                    player,
                    amount: template_amount(chosen, *amount, &event)?,
                    // CR 609.6: the replacement effect's own source is the
                    // source of what it substitutes.
                    source: chosen.source,
                }),
            EventSubject::Object(_) => Err(format!(
                "replacement {:?} rewrites to a life gain but matched {:?}, whose subject \
                 is an object, not a player. Its `EventPattern` and its `Rewrite` \
                 describe different events.",
                chosen.id, event
            )),
        },
        (GameActionTemplate::LoseLife { amount }, event) => match subject_of(&event) {
            EventSubject::Player(player) => Ok(GameAction::LoseLife {
                    player,
                    amount: template_amount(chosen, *amount, &event)?,
                    cause: LifeLossCause::Effect,
                }),
            EventSubject::Object(_) => Err(format!(
                "replacement {:?} rewrites to a life loss but matched {:?}, whose subject \
                 is an object, not a player. Its `EventPattern` and its `Rewrite` \
                 describe different events.",
                chosen.id, event
            )),
        },

        // CR 106.12b's "of a specific type" (Deep Water, Infernal Darkness,
        // Contamination). `ReplacedAmount` retypes every unit in place, restrictions
        // kept (CR 106.6: a restriction "doesn't affect the mana's type", and the
        // converse) — Deep Water's ruling. `Fixed(n)` makes `n` units of the type
        // carrying what the old units carried, which has one answer only when they
        // agree — all free, or all under one restriction, as every printed ability
        // is. A production that disagrees with itself has no printed instance and
        // no CR sentence, so it is refused rather than guessed; the fixture
        // `Half-Bound Grove` is what reaches it.
        (
            GameActionTemplate::ProduceMana { mana_type, amount },
            GameAction::ProduceMana { player, source, mana, special, tapped_for_mana },
        ) => {
            let plain: u64 = mana.iter().map(|(_, n)| n).sum();
            let (mana, special) = match amount {
                TemplateAmount::ReplacedAmount => (
                    if plain > 0 { vec![(*mana_type, plain)] } else { Vec::new() },
                    special
                        .into_iter()
                        .map(|atom| ManaAtom { mana_type: *mana_type, ..atom })
                        .collect(),
                ),
                TemplateAmount::Fixed(n) => {
                    let count = usize::try_from(*n).map_err(|_| {
                        format!(
                            "replacement {:?} sets a mana production to {} units, which no \
                             pool can hold",
                            chosen.id, n
                        )
                    })?;
                    let all_free = special.is_empty();
                    let all_restricted_alike = plain == 0
                        && special.windows(2).all(|pair| pair[0] == pair[1]);
                    if all_free {
                        (vec![(*mana_type, *n)], Vec::new())
                    } else if all_restricted_alike {
                        let unit = ManaAtom { mana_type: *mana_type, ..special[0].clone() };
                        (Vec::new(), std::iter::repeat_n(unit, count).collect())
                    } else {
                        return Err(format!(
                            "replacement {:?} sets a mana production to {} {:?}, but the \
                             production's units disagree about their restrictions (free \
                             beside restricted, or two restrictions), and no rule says \
                             which the new mana carries (CR 106.6a is about an ability's \
                             restrictions applying to all of its mana). No printed mana \
                             ability produces such a mix; the first that does brings the \
                             ruling that decides this.",
                            chosen.id, n, mana_type
                        ));
                    }
                }
            };
            Ok(GameAction::ProduceMana { player, source, mana, special, tapped_for_mana })
        }

        // CR 614.1a from a draw to the game's end (Laboratory Maniac). The affected
        // player wins: nothing printed hands a substituted win elsewhere, so there
        // is no `player` field until a card prints one.
        (GameActionTemplate::PlayerWins, event) => match subject_of(&event) {
            EventSubject::Player(player) => Ok(GameAction::PlayerWins { player }),
            EventSubject::Object(_) => Err(format!(
                "replacement {:?} rewrites to a win but matched {:?}, whose subject is \
                 an object, not a player. Its `EventPattern` and its `Rewrite` describe \
                 different events.",
                chosen.id, event
            )),
        },

        // Its `EventPattern` and its `Rewrite` describe different events —
        // the same card-authoring error every other arm reports.
        (GameActionTemplate::CreateTokens { .. }, other) => Err(format!(
            "replacement {:?} substitutes a token creation but matched {:?}, which is not one",
            chosen.id, other
        )),
        (GameActionTemplate::ZoneChangeTo { .. }, other) => Err(format!(
            "replacement {:?} rewrites to a zone change but matched {:?}, which is \
             neither one nor an entry. Its `EventPattern` and its `Rewrite` describe \
             different events.",
            chosen.id, other
        )),
        (GameActionTemplate::ProduceMana { .. }, other) => Err(format!(
            "replacement {:?} retypes a mana production but matched {:?}, which is not \
             one. Its `EventPattern` and its `Rewrite` describe different events.",
            chosen.id, other
        )),
    }
}

/// Who a [`GameActionTemplate::DrawCards`] substitution hands the draw to.
///
/// `None` is the affected player — Thought Reflection and Teferi's Ageless
/// Insight both leave the draw where it was — and [`PlayerRef::You`] is the
/// effect's controller, which is Notion Thief's whole point.
///
/// **`Owner` and `Opponent` are refused rather than guessed.** A draw's subject
/// is a player, so there is no object for `Owner` to be the owner of; and
/// `Opponent` would need a prompt to pick one of several, which CR 616.1 has
/// nowhere to hang — the choice that rule defines is *which effect applies*,
/// not where an applied effect's draw lands. Nothing prints either one, each is
/// one printed card away from an arm, and an arm the pipeline cannot apply is
/// worse than a missing one.
fn draw_recipient(
    chosen: &ReplacementInstance,
    player: Option<&PlayerRef>,
    affected: PlayerId,
) -> Result<PlayerId, String> {
    match player {
        None => Ok(affected),
        Some(PlayerRef::You) => Ok(chosen.controller),
        Some(PlayerRef::Player(pid)) => Ok(*pid),
        Some(other) => Err(format!(
            "replacement {:?} substitutes a draw for {:?}, which names no player a draw \
             event has: a draw's subject is a player, not an object.",
            chosen.id, other
        )),
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
        // read `ObjectSet::Host` makes, because an Aura registered its effect
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
        game.player_lost.get(pid).map(|lost| !lost).unwrap_or(false)
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
            let opponents = opponents_of(game, you);
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
/// `cause` is CR 101.2's — who controls the effect that proposed them: the
/// replacement's controller, or `None` for CR 306.5b's loyalty, which a rule
/// gives rather than a player. The synthetic event's putter is CR 122.6a's
/// default, the controller the permanent enters under, which is what the
/// entry door reads too.
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
    for row in &extra.counters {
        let action = GameAction::AddCounters {
            subject: CounterSubject::Object(object),
            counter: row.counter,
            n: row.n,
            by: row.putter(controller),
        };
        let refused = is_prohibited(
            game,
            &Query::Event { action: &action, cause, lookahead: Some(&frame) },
        );
        if !refused {
            kept.counters.push(*row);
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
    chosen: &ReplacementInstance,
    entering: ObjectId,
    controller: PlayerId,
    so_far: &EnterMods,
) -> Result<EnterMods, String> {
    let source = chosen.source;
    let mut out = EnterMods { tapped: template.tapped, counters: Vec::new() };
    if template.counters.is_empty() {
        return Ok(out);
    }
    let frame = EntryFrame::for_entering(game, entering, controller, so_far);
    for row in &template.counters {
        let n = match &row.amount {
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
        // CR 122.6a — the effect "may specify which player puts those
        // counters on it"; `None` stays the rule's default and is resolved
        // against the entry's controller where it is read, never here.
        let by = match &row.by {
            None => None,
            Some(player_ref) => Some(putter_of(game, chosen, entering, player_ref)?),
        };
        if n > 0 {
            out.counters.push(EntryCounters { counter: row.counter, n: n as u32, by });
        }
    }
    Ok(out)
}

/// CR 122.6a's named putter, resolved against the effect applying it.
///
/// `entering_controller`'s answers with one difference: an `Opponent` among
/// several is not asked for. CR 616.1b's control choice is the entering
/// permanent's controller's to make and the rule says so; "an opponent puts
/// those counters on it" with three opponents is an authoring error on a
/// static effect, and a resolution that means a particular one names it as
/// `PlayerRef::Player`, the way `PatternFill` fills a chosen source.
fn putter_of(
    game: &GameState,
    chosen: &ReplacementInstance,
    entering: ObjectId,
    player_ref: &PlayerRef,
) -> Result<PlayerId, String> {
    let you = chosen.controller;
    Ok(match player_ref {
        PlayerRef::You => you,
        PlayerRef::Player(pid) => *pid,
        PlayerRef::Owner => game
            .objects
            .get(&entering)
            .map(|obj| obj.owner)
            .ok_or_else(|| format!("entering object {} is not in the object store", entering))?,
        PlayerRef::Opponent => match opponents_of(game, you).as_slice() {
            [only] => *only,
            others => {
                return Err(format!(
                    "replacement {:?} names \"an opponent\" as the player putting counters on \
                     {}, and player {} has {} opponents in the game; name one as \
                     `PlayerRef::Player`",
                    chosen.id, entering, you, others.len()
                ))
            }
        },
    })
}

/// Every player still in the game who is not `you` — CR 102.1's "opponent",
/// with CR 102.3's teams not modeled.
fn opponents_of(game: &GameState, you: PlayerId) -> Vec<PlayerId> {
    (0..game.num_players()).filter(|&p| p != you && !game.player_lost[p]).collect()
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
            // CR 101.2 on the move this choice would produce — `sacrifice_of_choice`'s
            // axis-1 question. `cause` is a `PlayerId` because `SourceFilter`'s one
            // variant, `ControlledBy(PlayerRef)`, reads control and nothing else; it
            // widens with the variant that needs more. The player is the effect's
            // controller, which is why Sigarda does not stop her own devour.
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
