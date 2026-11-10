//! CR 700.2's modal abilities, as the words a card file writes: "Choose two
//! —" and its bulleted list is `choose_two([…])`, each mode an effect in
//! printed order. The node is all of its ability's effect
//! (`Effect::modal`), so a card passes one of these straight to
//! `spell_ability`, `activated_ability` or a trigger's def. Each mode's
//! instances of "target" are filled when the card is built.

use crate::types::costs::Cost;
use crate::types::effects::{ChosenBefore, Condition, Effect, ModalCount, ModalNode, Mode, ModeChooser, ModeRules};

/// "Choose one —".
pub fn choose_one(modes: impl IntoIterator<Item = Effect>) -> Effect {
    choose(ModalCount::Exactly(1), modes)
}

/// "Choose two —".
pub fn choose_two(modes: impl IntoIterator<Item = Effect>) -> Effect {
    choose(ModalCount::Exactly(2), modes)
}

/// "Choose one or more —": one mode to every mode printed.
pub fn choose_one_or_more(modes: impl IntoIterator<Item = Effect>) -> Effect {
    choose(ModalCount::OneOrMore, modes)
}

/// "Choose up to [most] —", none among them.
pub fn choose_up_to(most: u32, modes: impl IntoIterator<Item = Effect>) -> Effect {
    choose(ModalCount::Range { fewest: 0, most }, modes)
}

/// "Choose one. If [condition] as you cast this spell, you may choose both
/// instead.": one mode, or either or both while `condition` holds as the
/// modes are chosen.
pub fn choose_one_or_both_if(condition: Condition, modes: impl IntoIterator<Item = Effect>) -> Effect {
    let count = ModalCount::RaisedIf {
        usual: Box::new(ModalCount::Exactly(1)),
        raised: Box::new(ModalCount::Range { fewest: 1, most: 2 }),
        condition: Box::new(condition),
    };
    choose(count, modes)
}

/// "Choose [count] —", the ability's controller choosing (CR 700.2a, 700.2b).
pub fn choose(count: ModalCount, modes: impl IntoIterator<Item = Effect>) -> Effect {
    modal(count, ModeChooser::Controller, ModeRules::DEFAULT, modes.into_iter().map(Mode::new))
}

/// "Choose [count]. You may choose the same mode more than once." (CR
/// 700.2d): Moment of Reckoning's "up to four" of two modes.
pub fn choose_repeatable(count: ModalCount, modes: impl IntoIterator<Item = Effect>) -> Effect {
    let rules = ModeRules { may_repeat: true, ..ModeRules::DEFAULT };
    modal(count, ModeChooser::Controller, rules, modes.into_iter().map(Mode::new))
}

/// "Choose [count]. Each mode must target a different player.": Vindictive
/// Lich's "one or more" (`ModalCount::OneOrMore`) and Shadrix Silverquill's
/// "you may choose two" (`ModalCount::NoneOr(2)`, zero modes or two, never
/// one).
pub fn choose_each_targeting_a_different_player(count: ModalCount, modes: impl IntoIterator<Item = Effect>) -> Effect {
    let rules = ModeRules { each_mode_targets_a_different_player: true, ..ModeRules::DEFAULT };
    modal(count, ModeChooser::Controller, rules, modes.into_iter().map(Mode::new))
}

/// "An opponent chooses one —" (CR 700.2e).
pub fn an_opponent_chooses_one(modes: impl IntoIterator<Item = Effect>) -> Effect {
    modal(ModalCount::Exactly(1), ModeChooser::AnOpponent, ModeRules::DEFAULT, modes.into_iter().map(Mode::new))
}

/// "Choose one that hasn't been chosen —" (Demonic Pact), or "… that
/// hasn't been chosen this turn —", by `scope`.
pub fn choose_one_not_chosen_before(scope: ChosenBefore, modes: impl IntoIterator<Item = Effect>) -> Effect {
    let rules = ModeRules { not_chosen_before: Some(scope), ..ModeRules::DEFAULT };
    modal(ModalCount::Exactly(1), ModeChooser::Controller, rules, modes.into_iter().map(Mode::new))
}

/// "Choose up to [budget] {P} worth of modes. You may choose the same mode
/// more than once." (CR 700.2i), each mode with its pawprints.
pub fn choose_pawprints_worth(budget: u32, modes: impl IntoIterator<Item = (u32, Effect)>) -> Effect {
    let rules = ModeRules { may_repeat: true, ..ModeRules::DEFAULT };
    let modes = modes.into_iter().map(|(pawprints, effect)| Mode::with_pawprints(pawprints, effect));
    modal(ModalCount::Pawprints(budget), ModeChooser::Controller, rules, modes)
}

/// Spree (CR 702.172a): "Choose one or more additional costs.", each mode
/// behind its "+ [cost] —", paid as an additional cost when chosen (700.2h).
pub fn spree(modes: impl IntoIterator<Item = (Vec<Cost>, Effect)>) -> Effect {
    let modes = modes.into_iter().map(|(cost, effect)| Mode::with_cost(cost, effect));
    modal(ModalCount::OneOrMore, ModeChooser::Controller, ModeRules::DEFAULT, modes)
}

/// The node the words above write.
fn modal(count: ModalCount, chooser: ModeChooser, rules: ModeRules, modes: impl IntoIterator<Item = Mode>) -> Effect {
    Effect::Modal(Box::new(ModalNode { count, chooser, rules, modes: modes.into_iter().collect() }))
}

