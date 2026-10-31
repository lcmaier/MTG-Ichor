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
//! One file per subsystem whose vocabulary has landed: [`abilities`], the
//! kinds of ability every card is written in, and [`triggers`], CR 603's
//! words. A word sits in its file by the CR rule it names, not by the card
//! that brought it (`plans/handoffs/card-layer-cleanup.md`, decision 1).

pub mod abilities;
pub mod triggers;

pub use abilities::{activated_ability, equip, mana_ability, spell_ability, static_ability};
pub use triggers::{
    activates_an_ability_of, an_opponent_gains_control_of, another, at_beginning_of, attacks, attacks_alone, attacks_and_isnt_blocked, attacks_defender, becomes_attached,
    becomes_blocked, becomes_blocked_by, becomes_the_target_of, becomes_unattached, blocks, blocks_a_creature, counters_put_on, creates_a_token, counters_removed_from, damage_to_is_prevented, dies, draws_a_card, enters, functioning_in, is_attacked, is_sacrificed, leaves_the_battlefield, loses_the_game, persist,
    player_attacks, player_attacks_player, player_becomes_the_target_of, player_or_their_planeswalker, scries, shuffles_their_library, spell_is_countered, triggered_ability, undying, when_you_do,
    when_you_dont, whenever, you_lose_control_of, CountableEvent, CounterEvent, Whose,
};
