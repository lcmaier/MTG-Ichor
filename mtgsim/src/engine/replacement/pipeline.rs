//! The CR 616.1 loop (`replacement-architecture.md` §4.1).

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::oracle::characteristics::has_keyword;
use crate::state::game_state::GameState;
use crate::types::effects::Effect;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::{GameActionTemplate, Rewrite, Uses};
use crate::ui::ask::ask_apply_optional_replacement;
use crate::ui::ask::ask_choose_replacement;

use super::gather::forced_bucket;
use super::{subject_of, chooser_for, gather, EventSubject, ReplacementInstance, ReplacementInstanceId};

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

/// CR 614.17 — "some effects state that something can't happen. These effects
/// aren't replacement effects, but follow similar rules."
///
/// Checked *before* the pipeline and winning over it (CR 101.2). Today there is
/// exactly one: CR 702.12b's indestructible, which moved here from
/// `Primitive::Destroy` when `GameAction::Destroy` landed — a "can't" is not a
/// `ReplacementDef` and modelling it as one would have put it in the CR 616.1
/// choice list, where a player could decline it.
///
/// Re-asked on every iteration of the loop rather than once at the top, because
/// CR 614.17c lets a self-replacement change the event's *type*, and an event
/// of a different type is a different "can't" question.
pub(crate) fn is_blocked(game: &GameState, action: &GameAction) -> bool {
    match action {
        GameAction::Destroy { object, .. } => {
            has_keyword(game, *object, KeywordFlag::Indestructible)
        }
        _ => false,
    }
}

/// The maximum number of CR 616.1f iterations one event may go through.
///
/// Not a rules concept and not a loop guard for CR 731 (which is out of scope):
/// it is a **test harness**, because the two acid tests that pin §3.2d's
/// lineage rule fail by *hanging* rather than by producing a wrong answer.
/// CR 614.5's applied set is what actually guarantees termination — every
/// iteration either applies an effect, which permanently adds to that set, or
/// declines an optional one, which also adds to it. So the true bound is the
/// number of applicable effects, and reaching this cap means the applied set is
/// not doing its job.
const MAX_616_1F_ITERATIONS: usize = 256;

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
    let mut event = action;

    for _ in 0..MAX_616_1F_ITERATIONS {
        // CR 614.17: a "can't" is checked ahead of the pipeline and wins.
        // CR 614.17c narrows what may still apply rather than ending the loop,
        // because a self-replacement that changes the event's type would lift
        // the block.
        let blocked = is_blocked(game, &event);

        // CR 614.4 — gathered against live state at the moment of proposal.
        // There is no "go back in time" path because there is no other place
        // to ask.
        let applicable: Vec<ReplacementInstance> = gather(game, &event, ctx, blocked)
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
        let chooser = chooser_for(game, subject);

        // **Never prompt with fewer than two candidates.** CR-correct (there is
        // no choice to make with one), and it is what keeps every existing
        // `ScriptedDecisionProvider` test green rather than drowning it in
        // unexpected prompts now that every `execute_action` traverses this
        // loop. If a phase finds itself relaxing this to make something work,
        // it has found a design error, not a test problem
        // (`replacement-architecture.md` §11 item 7).
        let chosen = if bucket.len() == 1 {
            bucket.into_iter().next().expect("len checked")
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

        match apply_rewrite(&chosen, &event, subject)? {
            // CR 614.6 — the event does not happen. Queued riders still run.
            None => return Ok(None),
            // CR 616.1f — re-gather against the modified event, which is how
            // CR 616.2's "a replacement effect can become applicable as the
            // result of another" works without any special case.
            Some(next) => event = next,
        }
    }

    Err(format!(
        "CR 616.1f did not converge in {} iterations on {:?}. The CR 614.5 applied \
         set is what terminates this loop — every iteration either applies an \
         effect or declines an optional one, and both insert into it — so \
         reaching this cap means an effect is being re-offered after it has \
         already had its one opportunity.",
        MAX_616_1F_ITERATIONS, event
    ))
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
fn apply_rewrite(
    chosen: &ReplacementInstance,
    event: &GameAction,
    subject: EventSubject,
) -> Result<Option<GameAction>, String> {
    match &chosen.def.rewrite {
        // CR 614.6 / 615.6.
        Rewrite::Prevent => Ok(None),

        Rewrite::Instead(template) => match (template, event) {
            (
                GameActionTemplate::ZoneChangeTo { to, cause },
                GameAction::ZoneChange { object, from, .. },
            ) => Ok(Some(GameAction::ZoneChange {
                object: *object,
                from: *from,
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
                 not one. Its `EventPattern` and its `Rewrite` describe different events.",
                chosen.id, other
            )),
        },
    }
}
