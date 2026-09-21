//! Card-authoring vocabulary — the words a card file writes, above the
//! engine's type surface (the TR-1 review, theme B).
//!
//! `types/triggers.rs` is shaped for the matcher: every arm carries every
//! field it could be asked about, and `None` on one means "this arm does not
//! ask". That convention is right where it is and wrong in a card file,
//! where it leaves the reader to recognize `ZoneChange { from:
//! Some(Battlefield), to: Some(Graveyard), cause: None, owner: None,
//! multiplicity: PerOccurrence }` as the word *dies*. The constructors here
//! are that word. **The `Option`s stay at the type level**:
//! `types/replacement.rs`'s patterns share the convention
//! (`types/replacement.rs:237`), so changing it forks two surfaces or sweeps
//! both, and a GUI card builder presents "Any" itself whatever the Rust type
//! says.
//!
//! One file per subsystem whose vocabulary has landed — [`triggers`] today.
//! `codebase-state.md`'s "Before card breadth" item 10 is the rest of the
//! same debt: the static- and activated-ability shapes, written out 31 times
//! across `src/cards/`, belong in this directory and not in a 32nd private
//! copy.

pub mod triggers;

pub use triggers::{
    another, at_beginning_of, dies, enters, leaves_the_battlefield, triggered_ability, whenever,
    CountableEvent, Whose,
};
