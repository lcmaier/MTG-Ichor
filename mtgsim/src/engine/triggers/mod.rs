//! Triggered abilities — CR 603's two instants (`triggers-architecture.md` §2).
//!
//! **Detection** ([`dispatch`]) runs at the close of the outermost batch and
//! at an unbatched emission, never per record inside a batch: CR 603.6a's
//! "all permanents on the battlefield (including the newcomers) are checked"
//! makes the batch the event, so two permanents entering together each see
//! the other's static abilities before either is checked (§4.1). Nothing
//! happens when an ability triggers (CR 603.2, 117.2a): the dispatcher writes
//! a `PendingTrigger` onto `GameState` and one record, and touches the stack
//! never — with the one exception the CR makes, the triggered mana ability
//! (CR 605.4a), which resolves at once.
//!
//! **Placement** ([`placement`]) is where the choices are (CR 603.3b–d): the
//! refusal, the order, the targets. It runs inside the CR 117.5 / 704.3 loop
//! in `engine::priority`, over the seat list in APNAP order, and the drain
//! removes each entry as it places it so a clone taken at the ordering
//! prompt resumes by running the drain again (item 40). The order is not
//! asked when it cannot matter: one def, and agreement on every fact it
//! reads ([`bound_reads`]).
//!
//! [`audit`] answers every dispatch again the slow way in a game that turns
//! it on, and panics when the two answers differ (§4.10).
//!
//! [`binding`] is what the resolution reads back: the bound facts as
//! indices into the event log, resolved through the matched arm's
//! projections, with CR 603.6's "unable to be found" and CR 400.7 as one
//! epoch comparison.

pub mod audit;
pub mod binding;
pub mod bound_reads;
pub mod dispatch;
pub mod history;
pub mod placement;

pub use audit::DispatchAudit;
pub use dispatch::{is_mana_ability, visible_to_all, DepartureFrame, LookBackSnapshot, DISPATCH_NESTING_LIMIT};
