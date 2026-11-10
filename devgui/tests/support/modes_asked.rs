//! A choice of modes as the window is sent it, kept off a game a test builds:
//! the engine's own offer, for boards no review scenario can make, such as a
//! fixture's, which no scenario can name.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use devgui::bridge::ToWindow;
use devgui::prompt::Prompt;
use devgui::snapshot::Snapshot;
use devgui::view_model::WindowState;
use mtgsim::cards::fixtures;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, setup_game};
use mtgsim::types::ids::PlayerId;
use mtgsim::types::mana::ManaType;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// What the bridge's seat would send at a choice of modes.
pub struct Asked {
    pub snapshot: Snapshot,
    pub prompt: Prompt,
}

impl Asked {
    /// The window at this question, as it is received.
    pub fn window(self) -> WindowState {
        let mut state = WindowState::default();
        state.receive(ToWindow::Prompt { snapshot: Box::new(self.snapshot), prompt: self.prompt, yielding: None, why: None });
        state
    }
}

/// A seat that keeps what the window would be sent at the first choice of
/// modes it is asked, and gives the first legal answer to everything.
pub struct Keeping {
    kept: Rc<RefCell<Option<Asked>>>,
}

impl DecisionProvider for Keeping {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::ChooseModes { .. }) && self.kept.borrow().is_none() {
            let prompt = Prompt::pick_n(game, player, context, options, bounds);
            *self.kept.borrow_mut() = Some(Asked { snapshot: Snapshot::build(game, player), prompt });
        }
        (0..bounds.0).collect()
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }

    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        panic!("nothing on these boards divides")
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        (0..items.len()).collect()
    }
}

/// A keeping seat, and where it keeps what it was asked.
pub fn keeping() -> (Keeping, Rc<RefCell<Option<Asked>>>) {
    let kept = Rc::new(RefCell::new(None));
    (Keeping { kept: Rc::clone(&kept) }, kept)
}

/// The Pawprint Parade fixture's choice of modes, cast at two seats: three
/// pawprints over modes of one, two and one, so a mode both repeats and fits
/// once.
pub fn pawprint_parade() -> Asked {
    cast(&mut setup_game(2), fixtures::pawprint_parade(), &[(ManaType::Green, 2)])
}

/// `card`'s choice of modes, cast by player 0 from `mana` floating.
pub fn cast(game: &mut GameState, card: Arc<CardData>, mana: &[(ManaType, u64)]) -> Asked {
    let id = put_in_hand(game, card, 0);
    for &(mana_type, n) in mana {
        game.players[0].mana_pool.add(mana_type, n);
    }
    let (seat, kept) = keeping();
    game.cast_spell(0, id, &ManaWindowStop::new(seat)).expect("castable from the mana floating");
    kept.take().expect("its modes were asked")
}
