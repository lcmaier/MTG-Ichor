//! CR 707.5 and 707.9 — the copy an entering permanent carries, with the
//! exceptions its copy effect makes (`copy-effects-architecture.md` §4.1a).
//!
//! 707.9a–d change the copiable values, and `layers::copy` makes those
//! changes. 707.9e and 707.9f are about the entry: an exception may add a
//! status or counters to it, and may apply only if the result is or has
//! something. Both are decided here, as the copy effect applies at CR
//! 616.1c's step.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::layers::copy::{copiable_values, CopiableValues};
use crate::state::game_state::GameState;
use crate::types::effects::{CopyException, ObjectFilter};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{CopyAdditions, EnterMods, EntryCopy};

use super::pipeline::{evaluate_enter_template, strip_prohibited_counters};
use super::{EntryFrame, ReplacementInstance};

/// `mods`, entering as the copy of `captured` that `chosen` makes, with
/// `except` made.
///
/// A copy applied after another one in the same entry replaces it and takes
/// back what the earlier one's 707.9e exceptions added. With no exceptions,
/// Clone's case, the capture goes onto the entry and nothing else happens.
pub(super) fn enter_as_copy(
    game: &GameState,
    chosen: &ReplacementInstance,
    object: ObjectId,
    controller: PlayerId,
    mut mods: EnterMods,
    captured: CopiableValues,
    except: &[CopyException],
) -> Result<EnterMods, String> {
    if let Some(earlier) = mods.copy.take() {
        mods.take_back(&earlier.added);
    }
    if except.is_empty() {
        mods.copy = Some(EntryCopy { values: Arc::new(captured), added: CopyAdditions::default() });
        return Ok(mods);
    }
    refuse_misplaced(chosen, except)?;
    let own = keeps_own_value(except).then(|| copiable_values(game, object)).flatten();
    let conditions = ExceptionConditions {
        game,
        chosen,
        object,
        controller,
        mods: &mods,
        captured: &captured,
        except,
        own: own.as_ref(),
        memo: RefCell::new(HashMap::new()),
    };
    let (values, additions) = conditions.outcome(conditions.holding(0)?)?;
    // CR 614.17d at the door, against the copy the entry now is: Melira's
    // "can't have -1/-1 counters" refuses Spark Double's counter as it
    // refuses any "enters with".
    let as_the_copy = EnterMods {
        copy: Some(EntryCopy { values: Arc::clone(&values), added: CopyAdditions::default() }),
        ..mods.clone()
    };
    let additions = strip_prohibited_counters(game, object, controller, &as_the_copy, &additions, chosen.controller);
    let replaced_status = additions.status.map(|_| mods.status);
    mods.merge(&additions);
    mods.copy = Some(EntryCopy { values, added: CopyAdditions { counters: additions.counters, replaced_status } });
    Ok(mods)
}

/// CR 707.9f's check of one copy effect's conditional exceptions: each
/// against the copy without it, with every other exception that applies
/// there (§4.1a, "The Kaito board"; the register row
/// `copy-exception-conditions`). Memoized on what is left out, so a subset
/// the recursion reaches twice is checked once.
struct ExceptionConditions<'a> {
    game: &'a GameState,
    chosen: &'a ReplacementInstance,
    object: ObjectId,
    controller: PlayerId,
    mods: &'a EnterMods,
    captured: &'a CopiableValues,
    except: &'a [CopyException],
    own: Option<&'a CopiableValues>,
    memo: RefCell<HashMap<u64, u64>>,
}

impl ExceptionConditions<'_> {
    /// The conditional exceptions that hold, as a mask over `except`, for the
    /// copy effect with those in `left_out` removed.
    ///
    /// 707.9f checks exception E against "the copy effect applied without
    /// that exception, taking into account any other exceptions that effect
    /// includes". So for each conditional E not left out: first find which of
    /// the others hold in the effect without E (the recursive call, with E's
    /// bit added to `left_out`), then build that copy and match E's filter
    /// against its CR 614.12 frame. The unconditional exceptions are always
    /// made. Every call adds one bit to `left_out`, so the depth is at most
    /// the number of conditional exceptions (two on Spark Double), and the
    /// memo makes each subset a single check. No order is chosen and nothing
    /// loops: unlike CR 613.8's dependency walk, this is one evaluation of a
    /// finite tree.
    fn holding(&self, left_out: u64) -> Result<u64, String> {
        if let Some(&known) = self.memo.borrow().get(&left_out) {
            return Ok(known);
        }
        let mut holding = 0;
        for (i, exception) in self.except.iter().enumerate() {
            let CopyException::If(filter, _) = exception else { continue };
            let bit = 1u64 << i;
            if left_out & bit != 0 {
                continue;
            }
            // `copy-exception-conditions`' switch: the other conditional
            // exceptions that hold without this one are in the frame it is
            // checked against. Reading `0` here would check it against the
            // unconditional ones alone.
            if self.matches(filter, self.holding(left_out | bit)?)? {
                holding |= bit;
            }
        }
        self.memo.borrow_mut().insert(left_out, holding);
        Ok(holding)
    }

    /// Does `filter` match the entering permanent as the copy would make it
    /// with the conditional exceptions in `holding`? Read off the CR 614.12
    /// frame, as Spark Double's eighth ruling says: "use the characteristics
    /// of Spark Double as it enters".
    fn matches(&self, filter: &ObjectFilter, holding: u64) -> Result<bool, String> {
        let (values, additions) = self.outcome(holding)?;
        let mut would_be =
            EnterMods { copy: Some(EntryCopy { values, added: CopyAdditions::default() }), ..self.mods.clone() };
        would_be.merge(&additions);
        let frame = EntryFrame::for_entering(self.game, self.object, self.controller, &would_be);
        let Some(chars) = frame.frame_of(self.object) else {
            return Ok(false);
        };
        self.game.object_matches_filter_of_source(
            self.object,
            filter,
            self.chosen.controller,
            self.chosen.source,
            Some(chars),
        )
    }

    /// The copy's values, and what its 707.9e exceptions add to the entry,
    /// with every unconditional exception and the conditional ones in
    /// `holding`.
    fn outcome(&self, holding: u64) -> Result<(Arc<CopiableValues>, EnterMods), String> {
        let made: Vec<&CopyException> = self
            .except
            .iter()
            .enumerate()
            .flat_map(|(i, exception)| match exception {
                CopyException::If(_, inner) if holding & (1 << i) != 0 => inner.iter().collect(),
                CopyException::If(..) => Vec::new(),
                unconditional => vec![unconditional],
            })
            .collect();
        let mut values = self.captured.clone();
        values.except(&made, self.own)?;
        let values = Arc::new(values);
        // An amount reads the entry as the copy makes it, before what the
        // additions themselves add.
        let as_the_copy = EnterMods {
            copy: Some(EntryCopy { values: Arc::clone(&values), added: CopyAdditions::default() }),
            ..self.mods.clone()
        };
        let mut additions = EnterMods::NONE;
        for exception in &made {
            if let CopyException::Additionally(template) = exception {
                let extra = evaluate_enter_template(
                    self.game,
                    template,
                    self.chosen,
                    self.object,
                    self.controller,
                    &as_the_copy,
                )?;
                additions.merge(&extra);
            }
        }
        Ok((values, additions))
    }
}

/// Does an exception keep the copying object's own value (CR 707.9c)? Only
/// then is that object's copiable value captured.
fn keeps_own_value(except: &[CopyException]) -> bool {
    except.iter().any(|exception| match exception {
        CopyException::DoesNotCopy(_) => true,
        CopyException::If(_, inner) => keeps_own_value(inner),
        CopyException::Modifies(_) | CopyException::Additionally(_) => false,
    })
}

/// An exception in an arm CR 707.9 does not give it. An edit as an
/// additional effect: 707.9e's exception is "an additional effect rather than
/// a modification of the affected object's characteristics", so the edit is
/// `Modifies`. And a condition inside a condition, which `ObjectFilter::And`
/// already says; nesting would make 707.9f's "that exception" ambiguous.
fn refuse_misplaced(chosen: &ReplacementInstance, except: &[CopyException]) -> Result<(), String> {
    if except.len() > 64 {
        return Err(format!(
            "the copy effect {:?} carries {} exceptions, and CR 707.9f's check masks them in 64 bits",
            chosen.id,
            except.len()
        ));
    }
    let misplaced = |exception: &CopyException, nested: bool| match exception {
        CopyException::Additionally(template) if !template.edits.is_empty() => Some(format!(
            "the copy effect {:?} gives an edit as a CR 707.9e additional effect, which is not a \
             modification of characteristics by definition; the edit is `CopyException::Modifies`",
            chosen.id
        )),
        CopyException::If(..) if nested => Some(format!(
            "the copy effect {:?} nests a CR 707.9f condition inside another; write the two as one \
             `ObjectFilter::And`",
            chosen.id
        )),
        _ => None,
    };
    for exception in except {
        if let Some(refusal) = misplaced(exception, false) {
            return Err(refusal);
        }
        if let CopyException::If(_, inner) = exception
            && let Some(refusal) = inner.iter().find_map(|e| misplaced(e, true))
        {
            return Err(refusal);
        }
    }
    Ok(())
}
