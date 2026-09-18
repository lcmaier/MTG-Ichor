//! The trace sink — row A4c, tier 2 of `engineering-practices.md` §7.1.
//!
//! Four things a sink has to be, each a test below:
//!
//! - **A spine.** `plans/traces/rd-2-a-decision-is-per-subject.html`'s
//!   Trace A, hand-authored, walks two shield counters under two blockers:
//!   one allocation prompt, one batch of four, three subject groups, one
//!   candidate over two members applied unasked, both damages dropped, one
//!   rider. The same board with a sink attached has to write every one of
//!   those steps, in that order, from the five emit points.
//! - **An observer.** A board traced and the same board untraced perform the
//!   same events; the sink draws nothing, asks nothing, and decides nothing.
//! - **A tree.** `GameState` derives `Clone`, so a fork clones the handle,
//!   and the clone's records carry their own branch after one `fork` record.
//! - **Readable back.** Every line is one JSON object, and the same board
//!   writes the same bytes twice — `seq`, `branch` and ids are all the
//!   game's, never the process's.
//!
//! And the question the row was scheduled to answer (A4h): a priority
//! re-ask. The `priority_rejected` record sits between the `decision` that
//! offered the action and the `decision` that no longer does.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::CounterSubject;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    install_trace, place_vanilla_creature, set_attacking, set_blocked_by, set_blocking,
    setup_two_player_game, test_ctx, TraceBuffer,
};
use mtgsim::types::effects::CounterType;
use mtgsim::types::ids::ObjectId;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

// ---------------------------------------------------------------------------
// The board: rd-2's Trace A
// ---------------------------------------------------------------------------

/// Player 0's 3/3 with two shield counters attacks; player 1 blocks with two
/// 2/2s. CR 510.2 makes the four damage assignments one batch.
fn shield_board() -> (GameState, ObjectId, ObjectId, ObjectId) {
    let mut game = setup_two_player_game();
    game.active_player = 0;
    let attacker = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    game.execute_action(
        GameAction::AddCounters {
            subject: CounterSubject::Object(attacker),
            counter: CounterType::Shield,
            n: 2,
            by: 0,
        },
        &test_ctx(),
    )
    .unwrap();
    let first = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    let second = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![first, second]);
    set_blocking(&mut game, first, vec![attacker]);
    set_blocking(&mut game, second, vec![attacker]);
    (game, attacker, first, second)
}

/// Combat damage on the shield board, traced. Returns the buffer and the ids.
fn traced_combat(label: &str) -> (TraceBuffer, GameState, ObjectId, ObjectId, ObjectId) {
    let (mut game, attacker, first, second) = shield_board();
    let trace = install_trace(&mut game, label);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id: attacker }, vec![2, 1]);
    game.process_combat_damage(&dp, false).unwrap();
    game.trace_objects();
    (trace, game, attacker, first, second)
}

fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

// ---------------------------------------------------------------------------
// A spine
// ---------------------------------------------------------------------------

/// Trace A's steps, as the sink writes them — compared row by row against the
/// hand-authored page in the PR that landed this file.
#[test]
fn rd_2_trace_a_comes_out_of_the_five_emit_points() {
    let (trace, game, attacker, first, second) = traced_combat("rd-2 trace A");
    let lines = trace.lines();

    // The header: who wrote it, from what.
    assert!(lines[0].contains(r#""kind":"game""#), "{}", lines[0]);
    assert!(lines[0].contains(r#""label":"rd-2 trace A""#));
    assert!(lines[0].contains(r#""seed":null"#));
    assert!(lines[0].contains(r#""players":2"#));
    assert!(lines[0].contains(r#""commit":""#));

    // Step 3 of the page: the one prompt, the attacking player's split.
    let decisions = trace.of_kind("decision");
    assert_eq!(decisions.len(), 1, "{decisions:?}");
    let d = &decisions[0];
    assert!(d.contains(r#""player":0"#), "{d}");
    assert!(d.contains(r#""prompt":"allocate""#), "{d}");
    assert!(d.contains(r#""choice":"AssignCombatDamage""#), "{d}");
    assert!(d.contains(&format!(r#""subject":{}"#, attacker.raw())), "{d}");
    assert!(d.contains(&format!(r#""options":["{}","{}"]"#, first, second)), "{d}");
    assert!(d.contains(r#""total":3"#), "{d}");
    assert!(d.contains(r#""answer":[2,1]"#), "{d}");

    // Steps 4–6: one batch, four members, three groups with the attacker's
    // first (APNAP: player 0 is active). The page numbered the blockers'
    // assignments 0 and 1; the engine's batch has the attacker's there and
    // the blockers' at 2 and 3 — the first thing the spine diff found.
    let batches = trace.of_kind("batch");
    let combat = &batches[0];
    assert!(combat.contains(r#""depth":1"#), "{combat}");
    assert!(combat.contains(r#""inherited":0"#), "{combat}");
    assert_eq!(count(combat, r#""action":"DealDamage {"#), 4, "{combat}");
    assert!(
        combat.contains(&format!(r#""groups":[{{"subject":"Object({})","members":[2,3]}}"#, attacker)),
        "{combat}"
    );
    assert!(combat.contains(&format!(r#"{{"subject":"Object({})","members":[0]}}"#, first)), "{combat}");
    assert!(combat.contains(&format!(r#"{{"subject":"Object({})","members":[1]}}"#, second)), "{combat}");

    // Steps 7–13: the attacker's group, one iteration — no frame (not an
    // entry), one candidate over both members, decided without a prompt,
    // Prevent applied, both members gone. Then the two blockers' groups,
    // where the gather found nothing.
    // Four: the three groups, and the rider's own batch of one at step 14.
    let pipelines = trace.of_kind("pipeline");
    assert_eq!(pipelines.len(), 4, "{pipelines:?}");
    assert!(pipelines[3].contains(r#""event":"RemoveCounters {"#), "{}", pipelines[3]);
    let p = &pipelines[0];
    assert!(p.contains(r#""iteration":1"#), "{p}");
    assert!(p.contains(&format!(r#""subject":"Object({})""#, attacker)), "{p}");
    assert!(p.contains(r#""frame_computed":false"#), "{p}");
    assert!(
        p.contains(&format!(
            r#""candidates":[{{"id":"Counter({}, Shield, Prevention)","source":{},"class":"#,
            attacker,
            attacker.raw()
        )),
        "{p}"
    );
    assert!(p.contains(r#""members":[2,3],"bucket":true}]"#), "{p}");
    assert!(p.contains(r#""chooser":0"#), "{p}");
    assert!(p.contains(r#""decided":"single""#), "{p}");
    assert!(p.contains(r#""optional":null"#), "{p}");
    assert!(p.contains(r#""rewrite":"Prevent""#), "{p}");
    assert!(p.contains(r#""results":[{"i":2,"event":null},{"i":3,"event":null}]"#), "{p}");
    for (q, member) in pipelines[1..3].iter().zip([0u64, 1]) {
        assert!(q.contains(r#""candidates":[]"#), "{q}");
        assert!(q.contains(r#""decided":"none""#), "{q}");
        assert!(q.contains(&format!(r#""results":[{{"i":{},"event":"DealDamage {{"#, member)), "{q}");
    }

    // Step 14: members 0 and 1 performed, 2 and 3 not, one rider queued.
    let ends = trace.of_kind("batch_end");
    let end = &ends[0];
    assert!(end.contains(r#"{"i":0,"performed":"DealDamage {"#), "{end}");
    assert!(end.contains(r#"{"i":2,"performed":null},{"i":3,"performed":null}]"#), "{end}");
    assert!(end.contains(r#""riders":1"#), "{end}");

    // The rider is its own batch of one, nested, and its event is the
    // counter's removal; the damage events are the blockers' only.
    assert!(batches.iter().any(|b| b.contains(r#""depth":2"#) && b.contains("RemoveCounters")), "{batches:?}");
    let events = trace.of_kind("event");
    assert_eq!(count(&events.join("\n"), "DamageDealt"), 2, "{events:?}");
    assert!(events.iter().any(|e| e.contains("Shield")), "{events:?}");

    // The names table closes it, one row per object, by id.
    let objects = trace.of_kind("object");
    assert_eq!(objects.len(), game.objects.len());
    assert!(objects[0].contains(r#""id":1,"#), "{}", objects[0]);
    assert!(lines.last().unwrap().contains(r#""kind":"object""#));

    // The order the page walks: prompt, batch, the three loops, the end.
    let kinds: Vec<&str> = lines
        .iter()
        .map(|l| {
            let at = l.find(r#""kind":""#).unwrap() + 8;
            &l[at..l[at..].find('"').unwrap() + at]
        })
        .collect();
    let spine: Vec<&str> = kinds
        .iter()
        .copied()
        .filter(|k| matches!(*k, "decision" | "batch" | "pipeline" | "batch_end"))
        .collect();
    assert_eq!(
        spine,
        ["decision", "batch", "pipeline", "pipeline", "pipeline", "batch_end", "batch", "pipeline", "batch_end"],
    );
}

// ---------------------------------------------------------------------------
// An observer
// ---------------------------------------------------------------------------

/// The traced board and the untraced board perform the same events, and the
/// untraced one writes nothing at all.
#[test]
fn the_sink_changes_nothing_the_game_does() {
    let (_, traced, ..) = traced_combat("observer");

    let (mut game, attacker, ..) = shield_board();
    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id: attacker }, vec![2, 1]);
    game.process_combat_damage(&dp, false).unwrap();
    assert!(!game.trace_on());

    let rendered = |g: &GameState| mtgsim::ui::display::format_event_log(g);
    assert_eq!(rendered(&traced), rendered(&game));
    assert_eq!(traced.diagnostics.layer_walks(), game.diagnostics.layer_walks());
    assert_eq!(traced.diagnostics.decisions(), game.diagnostics.decisions());
}

// ---------------------------------------------------------------------------
// A tree
// ---------------------------------------------------------------------------

/// A clone takes its own branch: one `fork` record from the parent, and the
/// clone's own records carry the new number.
#[test]
fn a_clone_forks_the_trace_and_writes_as_its_own_branch() {
    let mut game = setup_two_player_game();
    let trace = install_trace(&mut game, "fork");
    let bears = place_vanilla_creature(&mut game, 0, 2, 2, &[]);

    let mut fork = game.clone();
    fork.execute_action(GameAction::Tap { object: bears }, &test_ctx()).unwrap();
    game.execute_action(GameAction::Tap { object: bears }, &test_ctx()).unwrap();

    let forks = trace.of_kind("fork");
    assert_eq!(forks.len(), 1, "{forks:?}");
    assert!(forks[0].contains(r#""branch":0,"kind":"fork","from":0,"to":1"#), "{}", forks[0]);

    let batches = trace.of_kind("batch");
    assert_eq!(batches.len(), 2, "{batches:?}");
    assert!(batches[0].contains(r#""branch":1,"#), "the fork's batch first: {}", batches[0]);
    assert!(batches[1].contains(r#""branch":0,"#), "then the parent's: {}", batches[1]);

    // A second clone of the parent is branch 2, not a reuse of 1.
    let _again = game.clone();
    let forks = trace.of_kind("fork");
    assert!(forks[1].contains(r#""from":0,"to":2"#), "{}", forks[1]);
}

// ---------------------------------------------------------------------------
// Readable back
// ---------------------------------------------------------------------------

/// The same board writes the same bytes twice.
#[test]
fn the_same_board_writes_the_same_trace() {
    let (a, ..) = traced_combat("twice");
    let (b, ..) = traced_combat("twice");
    assert_eq!(a.lines(), b.lines());
}

/// Every line is one JSON object, checked by a parser that accepts nothing
/// else. No `serde` in the tree, so the check is a hand-written one too.
#[test]
fn every_line_is_one_json_object() {
    let (trace, ..) = traced_combat("json");
    let lines = trace.lines();
    assert!(lines.len() >= 15, "{}", lines.len());
    for (n, line) in lines.iter().enumerate() {
        assert!(line.starts_with(r#"{"seq":"#), "line {n}: {line}");
        assert!(line.contains(&format!(r#"{{"seq":{},"branch":0,"kind":""#, n + 1)), "line {n}: {line}");
        json::parse_object(line).unwrap_or_else(|e| panic!("line {n} is not JSON ({e}): {line}"));
    }
}

/// A string with every character JSON escapes survives the writer.
#[test]
fn strings_are_escaped() {
    let mut game = setup_two_player_game();
    let trace = install_trace(&mut game, "quote \" backslash \\ newline \n tab \t bell \u{7}");
    let header = &trace.lines()[0];
    assert!(
        header.contains(r#""label":"quote \" backslash \\ newline \n tab \t bell \u0007""#),
        "{header:?}"
    );
    json::parse_object(header).unwrap();
}

// ---------------------------------------------------------------------------
// The A4h question: a re-ask, answerable from the trace
// ---------------------------------------------------------------------------

/// Play seeded random games until the engine rejects an action a priority
/// prompt offered; every rejection has to sit between the `decision` that
/// offered it and a `decision` for the same player that does not.
///
/// Rejections are what the retry loop exists for — the candidate list is an
/// overapproximation by contract — so a handful of seeds reach one. The
/// property is asserted for every rejection found, and the test fails if the
/// sweep found none, because then it proved nothing.
#[test]
fn a_re_ask_is_explained_by_the_rejection_between_two_decisions() {
    let registry = CardRegistry::default_registry();
    let deck: Vec<_> = registry
        .card_names()
        .iter()
        .cycle()
        .take(60)
        .filter_map(|name| registry.create(name).ok())
        .collect();

    let mut rejections = 0;
    for seed in 1..=24u64 {
        let mut game = Game::new(GameConfig::test(), vec![deck.clone(); 2]).expect("game");
        game.reseed(seed);
        let trace = install_trace(&mut game.state, &format!("re-ask seed {seed}"));
        let dp = ManaWindowStop::new(RandomDecisionProvider::seeded(seed));
        game.setup(&dp).expect("setup");
        let mut turns = 0;
        while !game.is_over() && turns < 12 {
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }

        let lines = trace.lines();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains(r#""kind":"priority_rejected""#) {
                continue;
            }
            rejections += 1;
            let player = &line[line.find(r#""player":"#).unwrap() + 9..];
            let player = &player[..player.find(',').unwrap()];
            let action = &line[line.find(r#""action":""#).unwrap() + 10..];
            let action = &action[..action.find('"').unwrap()];
            let is_priority_prompt = |l: &String| {
                l.contains(r#""kind":"decision""#)
                    && l.contains(r#""choice":"PriorityAction""#)
                    && l.contains(&format!(r#""player":{player},"#))
            };
            let before = lines[..i].iter().rev().find(|l| is_priority_prompt(l)).expect("an offer before");
            let after = lines[i + 1..].iter().find(|l| is_priority_prompt(l)).expect("a re-ask after");
            let quoted = format!("\"{action}\"");
            assert!(before.contains(&quoted), "seed {seed}: offered list lacks {action}: {before}");
            assert!(!after.contains(&quoted), "seed {seed}: re-ask still offers {action}: {after}");
        }
    }
    assert!(rejections > 0, "no seed reached a rejected priority action, so nothing was checked");
}

// ---------------------------------------------------------------------------
// How a page's spine is regenerated
// ---------------------------------------------------------------------------

/// The file-backed helper a `// COVERS:` test uses to write its own trace:
/// the same board to `target/traces/`, byte for byte what the buffer holds.
/// `python plans/trace_spine.py mtgsim/target/traces/rd-2-trace-a.jsonl`
/// then renders it as the page's spine; the checked-in copy is
/// `plans/traces/examples/rd-2-trace-a.jsonl`.
#[test]
fn a_test_writes_its_trace_to_a_file_for_the_spine_script() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/traces/rd-2-trace-a.jsonl");
    let (mut game, attacker, ..) = shield_board();
    let sink = mtgsim::test_support::install_trace_file(&mut game, &path, "rd-2 trace A");
    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id: attacker }, vec![2, 1]);
    game.process_combat_damage(&dp, false).unwrap();
    game.trace_objects();
    sink.flush().unwrap();

    let written: Vec<String> = std::fs::read_to_string(&path).unwrap().lines().map(str::to_string).collect();
    let (buffer, ..) = traced_combat("rd-2 trace A");
    assert_eq!(written, buffer.lines());
}

// ---------------------------------------------------------------------------
// A JSON reader, enough to check the writer
// ---------------------------------------------------------------------------

mod json {
    struct Parser<'a> {
        s: &'a [u8],
        i: usize,
    }

    pub fn parse_object(line: &str) -> Result<(), String> {
        let mut p = Parser { s: line.as_bytes(), i: 0 };
        p.ws();
        if p.peek() != Some(b'{') {
            return Err("not an object".into());
        }
        p.value()?;
        p.ws();
        if p.i != p.s.len() {
            return Err(format!("trailing bytes at {}", p.i));
        }
        Ok(())
    }

    impl Parser<'_> {
        fn peek(&self) -> Option<u8> {
            self.s.get(self.i).copied()
        }

        fn ws(&mut self) {
            while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
                self.i += 1;
            }
        }

        fn expect(&mut self, b: u8) -> Result<(), String> {
            if self.peek() == Some(b) {
                self.i += 1;
                Ok(())
            } else {
                Err(format!("expected {:?} at {}", b as char, self.i))
            }
        }

        fn value(&mut self) -> Result<(), String> {
            self.ws();
            match self.peek() {
                Some(b'{') => {
                    self.i += 1;
                    self.ws();
                    if self.peek() == Some(b'}') {
                        self.i += 1;
                        return Ok(());
                    }
                    loop {
                        self.ws();
                        self.string()?;
                        self.ws();
                        self.expect(b':')?;
                        self.value()?;
                        self.ws();
                        match self.peek() {
                            Some(b',') => self.i += 1,
                            Some(b'}') => {
                                self.i += 1;
                                return Ok(());
                            }
                            _ => return Err(format!("bad object at {}", self.i)),
                        }
                    }
                }
                Some(b'[') => {
                    self.i += 1;
                    self.ws();
                    if self.peek() == Some(b']') {
                        self.i += 1;
                        return Ok(());
                    }
                    loop {
                        self.value()?;
                        self.ws();
                        match self.peek() {
                            Some(b',') => self.i += 1,
                            Some(b']') => {
                                self.i += 1;
                                return Ok(());
                            }
                            _ => return Err(format!("bad array at {}", self.i)),
                        }
                    }
                }
                Some(b'"') => self.string(),
                Some(b't') => self.literal("true"),
                Some(b'f') => self.literal("false"),
                Some(b'n') => self.literal("null"),
                Some(b'-' | b'0'..=b'9') => self.number(),
                other => Err(format!("unexpected {:?} at {}", other.map(|b| b as char), self.i)),
            }
        }

        fn string(&mut self) -> Result<(), String> {
            self.expect(b'"')?;
            loop {
                match self.peek() {
                    None => return Err("unterminated string".into()),
                    Some(b'"') => {
                        self.i += 1;
                        return Ok(());
                    }
                    Some(b'\\') => {
                        self.i += 1;
                        match self.peek() {
                            Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => self.i += 1,
                            Some(b'u') => {
                                self.i += 1;
                                for _ in 0..4 {
                                    if !self.peek().is_some_and(|b| b.is_ascii_hexdigit()) {
                                        return Err(format!("bad \\u escape at {}", self.i));
                                    }
                                    self.i += 1;
                                }
                            }
                            _ => return Err(format!("bad escape at {}", self.i)),
                        }
                    }
                    Some(b) if b < 0x20 => return Err(format!("control byte in string at {}", self.i)),
                    Some(_) => self.i += 1,
                }
            }
        }

        fn literal(&mut self, word: &str) -> Result<(), String> {
            if self.s[self.i..].starts_with(word.as_bytes()) {
                self.i += word.len();
                Ok(())
            } else {
                Err(format!("expected {word} at {}", self.i))
            }
        }

        fn number(&mut self) -> Result<(), String> {
            if self.peek() == Some(b'-') {
                self.i += 1;
            }
            let start = self.i;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.i += 1;
            }
            if self.i == start {
                return Err(format!("bad number at {}", self.i));
            }
            Ok(())
        }
    }
}
