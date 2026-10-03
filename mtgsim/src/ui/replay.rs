//! The replay: a `DecisionProvider` that answers from a decision log, so the
//! engine cannot tell a replayed answer from a live one and nothing on the
//! event path changes (`setup-architecture.md` §7.1).
//!
//! **Exact within a build, as far as the game goes across two** (§7.2,
//! decision 6). Each line names what it chose, and the replay finds that
//! option wherever this build lists it. A question with one legal answer may
//! be asked by one build and taken by another, and its answer is the only
//! one, so a forced line no question matches is passed over and a forced
//! question no line matches is answered: neither changes the game. Anything
//! else that disagrees stops the run naming the line.
//!
//! While lines remain, every seat stops at every priority point, so the
//! engine asks the passes it would otherwise take itself, which the log
//! records too. Once they are spent the seats answer, or the run stops.

use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::scenario::position_word;
use crate::state::decision_log::{AnswerLine, Chosen, Log};
use crate::state::game_state::GameState;
use crate::state::trace::COMMIT;
use crate::types::ids::PlayerId;

use super::ask::{allocation_fits, forced_allocation, number_fits, only_number, only_order, only_pick, order_fits, pick_fits};
use super::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, position_of};
use super::decision::{DecisionProvider, SeatMode, Stop};

/// A replay of a log's answers.
pub struct Replay<'a> {
    lines: Vec<AnswerLine>,
    /// The engine that wrote the lines, which a divergence names beside this
    /// one when they differ.
    written_by: Option<String>,
    next: Cell<usize>,
    /// Who answers once the lines are spent; nobody stops the run instead.
    seats: Option<&'a dyn DecisionProvider>,
    control: Arc<ReplayControl>,
}

/// What another thread reads of a replay, and how it stops one.
#[derive(Debug, Default)]
pub struct ReplayControl {
    answered: AtomicUsize,
    superseded: AtomicBool,
}

impl ReplayControl {
    /// The lines answered so far.
    pub fn answered(&self) -> usize {
        self.answered.load(Ordering::Relaxed)
    }

    /// Stop the replay at its next answer, with `Stop::Superseded`: a newer
    /// one is starting.
    pub fn supersede(&self) {
        self.superseded.store(true, Ordering::Relaxed);
    }
}

/// Where a prompt's answer comes from.
enum Next<'l, T> {
    Line(&'l AnswerLine),
    /// The question has one legal answer and no line asks it.
    Forced(T),
    /// The lines are spent and the seats answer.
    Seats(&'l dyn DecisionProvider),
}

impl<'a> Replay<'a> {
    /// A replay of `lines` that stops the run once they are spent.
    pub fn new(lines: Vec<AnswerLine>) -> Replay<'a> {
        Replay { lines, written_by: None, next: Cell::new(0), seats: None, control: Arc::default() }
    }

    /// A replay of a log's answers, which names the engine that wrote them in
    /// a divergence.
    pub fn of(log: Log) -> Replay<'a> {
        Replay { written_by: Some(log.engine), ..Replay::new(log.answers) }
    }

    /// Hand every prompt after the last line to `seats`.
    pub fn then(self, seats: &'a dyn DecisionProvider) -> Replay<'a> {
        Replay { seats: Some(seats), ..self }
    }

    pub fn control(&self) -> Arc<ReplayControl> {
        Arc::clone(&self.control)
    }

    /// Lines not yet answered.
    pub fn remaining(&self) -> usize {
        self.lines.len() - self.next.get()
    }

    /// Where the answer to this prompt comes from, passing over the forced
    /// lines no question asks; `only` is the prompt's one legal answer, if it
    /// has one.
    fn next<T>(&self, game: &GameState, player: PlayerId, kind: &ChoiceKind, only: Option<T>) -> Next<'_, T> {
        if self.control.superseded.load(Ordering::Relaxed) {
            Stop::Superseded.raise();
        }
        loop {
            let Some(line) = self.lines.get(self.next.get()) else {
                return match self.seats {
                    Some(seats) => Next::Seats(seats),
                    None => Stop::LogSpent { answered: self.lines.len() }.raise(),
                };
            };
            let step = position_word(game.phase);
            if line.player == player && line.kind == kind.as_str() && line.turn == game.turn_number && line.step == step {
                return Next::Line(line);
            }
            if line.forced {
                self.next.set(self.next.get() + 1);
                continue;
            }
            if let Some(only) = only {
                return Next::Forced(only);
            }
            self.diverge(
                line,
                format!(
                    "it has player {} answering {} in turn {}, {}, and the engine asks player {player} {} in turn {}, {step}",
                    line.player,
                    line.kind,
                    line.turn,
                    line.step,
                    kind.as_str(),
                    game.turn_number
                ),
            );
        }
    }

    /// The line was answered.
    fn answered(&self) {
        self.next.set(self.next.get() + 1);
        self.control.answered.store(self.next.get(), Ordering::Relaxed);
    }

    fn diverge(&self, line: &AnswerLine, why: String) -> ! {
        let engines = match &self.written_by {
            Some(theirs) if theirs != COMMIT => format!(" (written by engine {theirs}; this is {COMMIT})"),
            _ => String::new(),
        };
        Stop::Diverged { line: line.number, why: format!("{why}{engines}") }.raise()
    }

    /// The options `chosen` names, found in `options` by what each is.
    fn found(&self, line: &AnswerLine, game: &GameState, options: &[ChoiceOption], chosen: &[String]) -> Vec<usize> {
        let mut found = Vec::new();
        for option in chosen {
            match position_of(options, game, option, &found) {
                Some(index) => found.push(index),
                None => self.diverge(line, format!("it chose {option}, which is not offered")),
            }
        }
        found
    }

    fn fits(&self, line: &AnswerLine, fit: Result<(), String>) {
        if let Err(why) = fit {
            self.diverge(line, format!("its answer does not fit: {why}"));
        }
    }

    fn shape(&self, line: &AnswerLine, asked: &str) -> ! {
        let had = match line.chosen {
            Chosen::Picks(_) => "picks",
            Chosen::Number(_) => "a number",
            Chosen::Allocation(_) => "an allocation",
            Chosen::Order(_) => "an order",
        };
        self.diverge(line, format!("it answers with {had}, and the engine asks for {asked}"))
    }
}

impl DecisionProvider for Replay<'_> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        let line = match self.next(game, player, &context.kind, only_pick(options.len(), bounds)) {
            Next::Line(line) => line,
            Next::Forced(only) => return only,
            Next::Seats(seats) => return seats.pick_n(game, player, context, options, bounds),
        };
        let Chosen::Picks(chosen) = &line.chosen else { self.shape(line, "picks") };
        let picks = self.found(line, game, options, chosen);
        self.fits(line, pick_fits(&picks, options.len(), bounds));
        self.answered();
        picks
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        let line = match self.next(game, player, &context.kind, only_number(min, max)) {
            Next::Line(line) => line,
            Next::Forced(only) => return only,
            Next::Seats(seats) => return seats.pick_number(game, player, context, min, max),
        };
        let Chosen::Number(number) = line.chosen else { self.shape(line, "a number") };
        self.fits(line, number_fits(number, min, max));
        self.answered();
        number
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let line = match self.next(game, player, &context.kind, forced_allocation(total, per_bucket_mins, per_bucket_maxs)) {
            Next::Line(line) => line,
            Next::Forced(only) => return only,
            Next::Seats(seats) => return seats.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs),
        };
        let Chosen::Allocation(amounts) = &line.chosen else { self.shape(line, "an allocation") };
        let named: Vec<String> = amounts.iter().map(|(_, bucket)| bucket.clone()).collect();
        let at = self.found(line, game, buckets, &named);
        if at.len() != buckets.len() {
            self.diverge(line, format!("it allocates over {} buckets, and the engine offers {}", at.len(), buckets.len()));
        }
        let mut allocation = vec![0; buckets.len()];
        for (&bucket, (amount, _)) in at.iter().zip(amounts) {
            allocation[bucket] = *amount;
        }
        self.fits(line, allocation_fits(&allocation, total, per_bucket_mins, per_bucket_maxs));
        self.answered();
        allocation
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        let line = match self.next(game, player, &context.kind, only_order(items.len())) {
            Next::Line(line) => line,
            Next::Forced(only) => return only,
            Next::Seats(seats) => return seats.choose_ordering(game, player, context, items),
        };
        let Chosen::Order(chosen) = &line.chosen else { self.shape(line, "an order") };
        let order = self.found(line, game, items, chosen);
        self.fits(line, order_fits(&order, items.len()));
        self.answered();
        order
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        match self.seats {
            Some(seats) if self.remaining() == 0 => seats.seat_mode(player),
            Some(_) | None => SeatMode { stops_at_every_priority_point: true },
        }
    }
}
