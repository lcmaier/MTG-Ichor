//! The why panel with no window (`setup-architecture.md` §7c, SU-6): a
//! right-click asks the open question's seat, the answer shows beside the
//! board, and the panel follows its object to each question after, through
//! Undo answer too. Each test plays a sample board through the session, as
//! clicks do.

#[path = "support/games.rs"]
#[allow(dead_code, reason = "these tests drive a session; the bridge-level helpers are the other files'")]
mod games;
#[path = "support/sessions.rs"]
#[allow(dead_code, reason = "these tests start from the sample boards, not the support's own")]
mod sessions;
#[path = "support/window_by_rule.rs"]
#[allow(dead_code, reason = "these tests answer by the rule one question at a time, through a session")]
mod window_by_rule;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use devgui::session::Session;
use devgui::view_model::{Input, WhyLineView, WhyView};
use mtgsim::types::ids::ObjectId;
use sessions::{next_prompt, scenario_game, session_with};
use window_by_rule::inputs_by_rule;

/// Humility arrived after Opalescence: every creature is a 1/1 with no
/// abilities, the Angel among them.
fn humility_board() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios/humility-opalescence.scenario");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The object called `name` on the battlefield the window shows.
fn on_board(session: &Session, name: &str) -> ObjectId {
    let board = session.state.board.as_ref().expect("a board at the open question");
    let permanents = board.players.iter().flat_map(|player| &player.battlefield);
    permanents.map(|p| &p.card).find(|card| card.name == name).map(|card| card.id).unwrap_or_else(|| panic!("no {name}"))
}

/// The panel once the seat's answer about `about` has arrived.
fn answered(session: &mut Session, about: &str) -> WhyView {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        session.receive();
        if let Some(view) = session.state.why_view()
            && view.title.starts_with(about)
        {
            return view;
        }
        assert!(Instant::now() < deadline, "no why about {about} in a minute");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn line<'a>(view: &'a WhyView, text: &str) -> &'a WhyLineView {
    view.sections.iter().flat_map(|section| &section.lines).find(|line| line.text == text).unwrap_or_else(|| {
        let texts: Vec<&str> = view.sections.iter().flat_map(|s| &s.lines).map(|l| l.text.as_str()).collect();
        panic!("no line {text:?} among {texts:#?}")
    })
}

/// The open question answered by `window_by_rule`'s rule, and the next one
/// waited for.
fn answer_by_rule(session: &mut Session) {
    for input in inputs_by_rule(&session.state) {
        session.input(input);
        if session.state.prompt.is_none() {
            return next_prompt(session);
        }
    }
    panic!("the rule built no answer for {:?}", session.state.prompt);
}

/// A right-click on the Angel asks the open question's seat, which answers
/// at once and leaves the question as it was: the panel says what each layer
/// did, with its rule, and links Humility.
#[test]
fn a_right_click_shows_what_the_layers_did_and_leaves_the_question_open() {
    let (mut session, ..) = session_with("devgui-why-click", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let (prompt, selection) = (session.state.prompt.clone(), session.state.selection.clone());
    let angel = on_board(&session, "Serra Angel");
    session.input(Input::Why(angel));
    assert_eq!((&session.state.prompt, &session.state.selection), (&prompt, &selection), "a why answers nothing");
    let view = answered(&mut session, "Serra Angel");
    let set = line(&view, "Power and toughness from 4/4 to 1/1");
    assert_eq!(set.depth, 1);
    assert_eq!(line(&view, "Loses flying, vigilance").depth, 1);
    let humility = on_board(&session, "Humility");
    let step = view.sections[0].lines.iter().find(|line| line.links.iter().any(|link| link.input == Input::Why(humility)));
    assert_eq!(step.and_then(|line| line.rule.as_deref()), Some("CR 613.1f"), "layer 6's step names Humility");
    assert!(!view.back && view.note.is_none());
}

/// The panel follows its object: the next question carries its why, and so
/// does the question Undo answer goes back to, on a game built again.
#[test]
fn the_panel_follows_its_object_to_the_next_question_and_through_undo() {
    let (mut session, ..) = session_with("devgui-why-follow", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let angel = on_board(&session, "Serra Angel");
    session.input(Input::Why(angel));
    let asked = answered(&mut session, "Serra Angel");
    answer_by_rule(&mut session);
    assert_eq!(session.state.why_view(), Some(asked.clone()), "the next question's why, the board unchanged");
    session.input(Input::Undo);
    assert!(session.state.replaying.is_some(), "a replay, counted");
    assert!(session.state.why_view().is_some_and(|view| view.note.is_some()), "kept, and marked as of the last question");
    next_prompt(&mut session);
    assert_eq!(session.state.why_view(), Some(asked), "asked again at the question undone to");
}

/// A link asks about the object it names, Back returns to the one before,
/// and closing the panel stops the seats answering for it.
#[test]
fn a_link_back_and_close() {
    let (mut session, ..) = session_with("devgui-why-links", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let (angel, humility) = (on_board(&session, "Serra Angel"), on_board(&session, "Humility"));
    session.input(Input::Why(angel));
    let view = answered(&mut session, "Serra Angel");
    let link = view.sections.iter().flat_map(|s| &s.lines).flat_map(|l| &l.links).find(|link| link.input == Input::Why(humility));
    session.input(link.expect("Humility is linked").input.clone());
    let view = answered(&mut session, "Humility");
    assert!(view.back, "back to the Angel");
    line(&view, "Type line from Enchantment to Enchantment Creature");
    session.input(Input::WhyBack);
    assert!(!answered(&mut session, "Serra Angel").back);
    session.input(Input::WhyClose);
    assert_eq!(session.state.why_view(), None);
    answer_by_rule(&mut session);
    assert_eq!(session.state.why_view(), None, "the next question carries no why");
}
