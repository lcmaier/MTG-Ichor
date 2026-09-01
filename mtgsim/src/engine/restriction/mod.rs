//! "Can't" effects — CR 101.2, 614.17 (`cant-effects-architecture.md` §3).
//!
//! > 101.2. When a rule or effect allows or directs something to happen, and
//! > another effect states that it can't happen, the "can't" effect takes
//! > precedence.
//!
//! **A "can't" is discovered exactly the way a replacement effect is, and
//! differs from one only in what it is asked at.** So [`predicate`] is
//! `engine::replacement::gather` with the `Rewrite` half deleted: the same
//! sources, the same battlefield sweep off *effective* ability lists, the same
//! `EventPattern`/`AffectedSet` matchers — reused rather than reimplemented.
//!
//! - [`Query`] names what is being asked.
//! - [`is_prohibited`] answers it, and is the **only** reader of every
//!   restriction. One predicate is load-bearing rather than tidy: CR 101.3's
//!   "if you can't" (§4.8) and §4.9's candidate filter have to get the same
//!   answer as the event chokepoint, and Plaguecrafter ("each player sacrifices
//!   a creature … each player who **can't** discards a card") is one card that
//!   asks both.
//!
//! # What ships here, and what does not
//!
//! RS-1 is Tier 2 (the event chokepoint) plus Tier 3 (withholding a replacement
//! effect). The axis-2 arms — casting, activating, targeting, attacking,
//! blocking, paying costs — belong to RS-2/RS-3/RS-4 and are not in
//! [`Restriction`](crate::types::restriction::Restriction) yet, on
//! `types::replacement`'s rule: an arm no enforcement point consults is a card
//! that silently does nothing.

mod predicate;

pub(crate) use predicate::{is_prohibited, Query};
