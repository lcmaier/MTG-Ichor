//! CR 700.2's modal abilities, as the words a card file writes: "Choose two
//! —" and its bulleted list is `choose_two([…])`, each mode an effect in
//! printed order. The node is all of its ability's effect
//! (`Effect::modal`), so a card passes one of these straight to
//! `spell_ability`, `activated_ability` or a trigger's def. Each mode's
//! instances of "target" are filled when the card is built.

use crate::types::effects::{Condition, Effect, ModalCount, ModalNode, Mode, ModeChooser};

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
    let modes: Vec<Effect> = modes.into_iter().collect();
    let most = u32::try_from(modes.len()).unwrap_or(u32::MAX);
    choose(ModalCount::Range { fewest: 1, most }, modes)
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
    Effect::Modal(Box::new(ModalNode {
        count,
        chooser: ModeChooser::Controller,
        modes: modes.into_iter().map(Mode::new).collect(),
    }))
}
