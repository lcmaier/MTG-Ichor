//! The window's part played by rule instead of by hand, through the same
//! `Input`s a click makes: the last option (so lands are played, spells cast
//! and abilities used), one pair per attacker or blocker, buckets filled in
//! order, things ordered as offered.

use std::sync::Arc;

use devgui::bridge::{Outcome, Play, ToWindow, spawn_game};
use devgui::prompt::{BoardRef, Primitive};
use devgui::view_model::{Input, WindowState};

use crate::games::next;

/// A game this long has stopped being a game; fail rather than hang.
const PROMPT_CAP: usize = 20_000;

pub fn inputs_by_rule(state: &WindowState) -> Vec<Input> {
    let Some(prompt) = &state.prompt else {
        return Vec::new();
    };
    let last = prompt.options.len().checked_sub(1);
    match &prompt.primitive {
        Primitive::PickN { max: 1, .. } => vec![last.map_or(Input::Done, Input::OptionButton)],
        Primitive::PickN { min, max } => {
            let mut actors: Vec<BoardRef> = Vec::new();
            let mut picks: Vec<usize> = Vec::new();
            for (i, option) in prompt.options.iter().enumerate() {
                let actor = option.refs.first().copied();
                if picks.len() < *max && !actor.is_some_and(|a| actors.contains(&a)) {
                    actors.extend(actor);
                    picks.push(i);
                }
            }
            let short = (0..prompt.options.len()).filter(|i| !picks.contains(i)).take(min.saturating_sub(picks.len()));
            picks.extend(short.collect::<Vec<_>>());
            picks.into_iter().map(Input::OptionButton).chain([Input::Done]).collect()
        }
        Primitive::Number { .. } => vec![Input::Done],
        Primitive::Allocate { total, .. } => (0..prompt.options.len())
            .flat_map(|bucket| std::iter::repeat_n(Input::OneMore(bucket), *total as usize))
            .chain([Input::Done])
            .collect(),
        Primitive::Order => (0..prompt.options.len()).map(Input::OptionButton).chain([Input::Done]).collect(),
    }
}

/// Play a whole game by rule. `watch` sees the window after each message,
/// before it answers. The outcome, and how many prompts the window answered.
pub fn play_by_rule(game: Play, mut watch: impl FnMut(&WindowState)) -> (Outcome, usize) {
    let engine = spawn_game(game, Arc::new(|| {}));
    let mut state = WindowState::default();
    for answered in 0..PROMPT_CAP {
        let message = next(&engine);
        let finished = match &message {
            ToWindow::Finished { outcome, .. } => Some(outcome.clone()),
            ToWindow::Panicked { message } => panic!("the engine panicked: {message}"),
            ToWindow::Refused { message } => panic!("the scenario did not load: {message}"),
            ToWindow::Prompt { .. } => None,
        };
        state.receive(message);
        watch(&state);
        if let Some(outcome) = finished {
            return (outcome, answered);
        }
        let answer = inputs_by_rule(&state)
            .into_iter()
            .find_map(|input| state.input(input))
            .unwrap_or_else(|| panic!("the rule built no answer for {:?}", state.prompt));
        engine.answers.send(answer).expect("the engine hung up with a prompt open");
    }
    panic!("no result after {PROMPT_CAP} prompts");
}
