//! The why panel with no window (`setup-architecture.md` §7c, SU-6 and
//! SU-7): a right-click asks the open question's seat, the answer shows
//! beside the board, and the panel follows its object to each question after,
//! through Undo answer too. A player's line asks too, and the first section
//! says whether the open question offers what was clicked, and if not, why.
//! Each test plays a board through the session, as clicks do.

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
use devgui::prompt::BoardRef;
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
    session.input(Input::Why(BoardRef::Object(angel)));
    assert_eq!((&session.state.prompt, &session.state.selection), (&prompt, &selection), "a why answers nothing");
    let view = answered(&mut session, "Serra Angel");
    let set = line(&view, "Power and toughness from 4/4 to 1/1");
    assert_eq!(set.depth, 1);
    assert_eq!(line(&view, "Loses flying, vigilance").depth, 1);
    let humility = on_board(&session, "Humility");
    let lines = view.sections.iter().flat_map(|section| &section.lines);
    let step = lines.into_iter().find(|line| line.links.iter().any(|link| link.input == Input::Why(BoardRef::Object(humility))));
    assert_eq!(step.and_then(|line| line.rule.as_deref()), Some("CR 613.1f"), "layer 6's step names Humility");
    assert!(!view.back && view.note.is_none());
}

/// The panel follows its object: the next question carries its why, its
/// first section about that question, and so does the question Undo answer
/// goes back to, on a game built again.
#[test]
fn the_panel_follows_its_object_to_the_next_question_and_through_undo() {
    let (mut session, ..) = session_with("devgui-why-follow", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let angel = on_board(&session, "Serra Angel");
    session.input(Input::Why(BoardRef::Object(angel)));
    let asked = answered(&mut session, "Serra Angel");
    answer_by_rule(&mut session);
    let next = session.state.why_view().expect("the next question carries its why");
    assert_eq!(next.sections[1..], asked.sections[1..], "the layers' sections, the board unchanged");
    let (was, now) = (&asked.sections[0].lines[0].text, &next.sections[0].lines[0].text);
    assert_ne!(was, now, "the question's section follows the question: {was} then {now}");
    session.input(Input::Undo);
    assert!(session.state.replaying.is_some(), "a replay, counted");
    assert!(session.state.why_view().is_some_and(|view| view.note.is_some()), "kept, and marked as of the last question");
    next_prompt(&mut session);
    assert_eq!(session.state.why_view(), Some(asked), "asked again at the question undone to");
}

/// The panel follows an id, and another start may give the id another card:
/// a Reload of the same board keeps the panel, and one of a board whose two
/// lines were swapped closes it.
#[test]
fn a_reload_keeps_the_panel_only_on_the_same_board() {
    let (mut session, _, file) = session_with("devgui-why-reload", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let humility = on_board(&session, "Humility");
    session.input(Input::Why(BoardRef::Object(humility)));
    let asked = answered(&mut session, "Humility");
    session.input(Input::Reload);
    next_prompt(&mut session);
    assert_eq!(session.state.why_view(), Some(asked), "the same board numbers Humility alike");
    let (opalescence, humility_line) = ("battlefield: Opalescence | controller 0", "battlefield: Humility | controller 1");
    let swapped = humility_board().replace(opalescence, "\u{0}").replace(humility_line, opalescence).replace('\u{0}', humility_line);
    std::fs::write(&file, swapped).unwrap();
    session.input(Input::Reload);
    next_prompt(&mut session);
    assert_eq!(on_board(&session, "Opalescence"), humility, "the id the panel showed is Opalescence's now");
    assert_eq!(session.state.why_view(), None, "so the panel closed");
}

/// A link asks about the object it names, Back returns to the one before,
/// and closing the panel stops the seats answering for it.
#[test]
fn a_link_back_and_close() {
    let (mut session, ..) = session_with("devgui-why-links", &humility_board(), scenario_game);
    next_prompt(&mut session);
    let (angel, humility) = (on_board(&session, "Serra Angel"), on_board(&session, "Humility"));
    session.input(Input::Why(BoardRef::Object(angel)));
    let view = answered(&mut session, "Serra Angel");
    let link = view.sections.iter().flat_map(|s| &s.lines).flat_map(|l| &l.links).find(|link| link.input == Input::Why(BoardRef::Object(humility)));
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

/// The review board `blocks.scenario` with a Serra Angel attacking too, as
/// the click script edits it in.
fn blocks_with_an_angel() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scenarios/blocks.scenario");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    format!("{text}battlefield: Serra Angel | controller 1, attacking player 0\n")
}

/// At the declare-blockers question, Wall of Stone's why says what it is
/// offered and why it is never offered the Angel, with the rule; a player's
/// line asks about the player, who is not among the options.
#[test]
fn the_first_section_says_what_the_question_offers_and_why_not() {
    let (mut session, ..) = session_with("devgui-why-options", &blocks_with_an_angel(), scenario_game);
    next_prompt(&mut session);
    assert_eq!(session.state.prompt.as_ref().map(|prompt| prompt.kind.as_str()), Some("DeclareBlockers"));
    let wall = on_board(&session, "Wall of Stone");
    session.input(Input::Why(BoardRef::Object(wall)));
    let view = answered(&mut session, "Wall of Stone");
    assert_eq!(view.sections[0].heading, "At this question");
    let never = view.sections[0].lines.iter().find(|line| line.text.starts_with("To block Serra Angel")).expect("the Angel's line");
    assert_eq!(never.rule.as_deref(), Some("CR 702.9b"));
    assert!(never.links.iter().any(|link| link.label.starts_with("Serra Angel")), "the Angel is linked");

    session.input(Input::Why(BoardRef::Player(0)));
    let view = answered(&mut session, "Player 0");
    assert_eq!(view.sections.len(), 1, "a player has only the question's section");
    line(&view, "Not among the options: the question ranges over the creatures Player 0 controls that can block, and what each can block.");
    assert!(view.back, "back to Wall of Stone");
}
