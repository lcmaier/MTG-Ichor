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
//! kinds of ability every card is written in; [`instructions`], the verbs an
//! ability's effect is made of; [`recipients`], who and what each verb acts
//! on; and [`triggers`], CR 603's words. A word sits in its file by the CR
//! rule it names, not by the card that brought it, so the file reads in the
//! CR's order whichever card came first.

pub mod abilities;
pub mod instructions;
pub mod recipients;
pub mod triggers;

pub use abilities::{
    activated_ability, as_long_as, equip, mana_ability, spell_ability, static_ability, static_replacement, static_restriction, StaticBody,
};
pub use instructions::{
    add_mana, add_phases, attach_to, becomes_a_blocked_creature, becomes_color, becomes_colorless, becomes_types, cant, copy, counter_spell,
    create_delayed_trigger, create_replacement, create_replacement_for, create_replacement_from_a_chosen_source, create_tokens, deal_damage,
    deal_damage_unpreventable, destroy, discard, draw_cards, exile, exile_until, gain_control_of, gain_life, gains_ability, gains_keyword, get_counters,
    gets, has_base_power_and_toughness, is_also_color, its_controller_adds, lose_life, lose_the_game, loses_all_abilities, mill, put_counters,
    remove_counters, return_to_hand, return_to_the_battlefield, sacrifice, scry, set_life_total, shuffle_library, switch_power_and_toughness,
    take_an_extra_turn_after_this_one, tap, untap, win_the_game, ContinuousChange, CreatedEffect,
};
pub use recipients::{
    any_target, attacked_by, chosen_by, creatures_you_control, each, each_in, each_opponent, each_player, host, referred, target, target_creature,
    target_permanent, target_player, target_spell, targets, that_target, this_object, triggering_object, you, you_and_that_player,
};
pub use triggers::{
    activates_an_ability_of, an_opponent_gains_control_of, another, at_beginning_of, attacks, attacks_alone, attacks_and_isnt_blocked, attacks_defender, becomes_attached,
    becomes_blocked, becomes_blocked_by, becomes_the_target_of, becomes_unattached, blocks, blocks_a_creature, counters_put_on, creates_a_token, counters_removed_from, damage_to_is_prevented, dies, draws_a_card, enters, functioning_in, is_attacked, is_sacrificed, leaves_the_battlefield, loses_the_game, persist,
    player_attacks, player_attacks_player, player_becomes_the_target_of, player_or_their_planeswalker, scries, shuffles_their_library, spell_is_countered, triggered_ability, undying, when_you_do,
    when_you_dont, whenever, you_lose_control_of, CountableEvent, CounterEvent, Whose,
};
