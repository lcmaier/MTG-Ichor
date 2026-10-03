//! What the window shows and what a click means, in plain Rust.
//!
//! [`WindowState`] takes the engine's messages and the player's [`Input`]s and
//! hands back an answer once one is complete. [`BoardView`] and [`PromptView`]
//! are the board and the question as the drawing needs them: grouped, labeled,
//! and marked clickable, chosen, or the prompt's subject. `app` draws them and
//! reports clicks; nothing here knows egui.
//!
//! No case per `ChoiceKind`: a click is read off the primitive and off the
//! board things each option names (`prompt::OptionView::refs`).

use std::sync::Arc;

use mtgsim::types::ids::PlayerId;
use mtgsim::ui::auto_yield::Yield;

use crate::bridge::{Outcome, ToWindow};
use crate::prompt::{Answer, BoardRef, Primitive, Prompt, Reply};
use crate::snapshot::{CardView, PermanentView, PlayerView, Snapshot};
pub use crate::snapshot::{TypeLineView, TypeWordView};

/// Something the player did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Clicked option `i`'s button.
    OptionButton(usize),
    /// Clicked a card or a player on the board.
    Board(BoardRef),
    /// Set a `pick_number`'s number.
    Number(u64),
    /// One more into bucket `i` of an allocation.
    OneMore(usize),
    /// One fewer in bucket `i` of an allocation.
    OneFewer(usize),
    /// The confirm button.
    Done,
    /// Start the answer over.
    Reset,
    /// Build the game again from its scenario file, read again.
    Reload,
    /// Write the board at this prompt to a scenario file.
    SaveBoard,
    /// Pass at this priority prompt, and keep passing until the yield ends.
    Yield(Yield),
    /// End the seat's yield; the prompt stays open.
    StopYielding,
    /// Turn full control on or off, which `Session::input` carries to the seat.
    FullControl(bool),
    /// A shortcut key went down: a held key's repeats arrive too, and answer
    /// nothing.
    Key { key: Key, repeat: bool },
}

/// A key the window reads, as the drawing reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// `1` to `9`: that option's button.
    Digit(u8),
    Enter,
    Space,
    Escape,
    F2,
    F4,
    F6,
}

/// What each key does, for the prompt's foot.
pub const KEYS: &str = "Keys: 1–9 an option · Enter confirm · Space pass · Esc start over · F2 pass until the stack changes · F4 until end of turn · F6 until this player's next turn";

/// How long after a prompt arrives its input is dropped (`codebase-state.md`
/// item 201): egui's double-click window, so the second click of a double
/// click, or a quick click aimed at the prompt before, cannot answer one the
/// person has not seen.
pub const SETTLE_SECONDS: f64 = 0.3;

/// The answer being put together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    /// Options in click order, and a pair's first half clicked on the board
    /// with its second half still to come.
    Picks { chosen: Vec<usize>, half: Option<BoardRef> },
    Number(u64),
    Allocation(Vec<u64>),
    Order(Vec<usize>),
}

impl Selection {
    pub fn start(prompt: &Prompt) -> Selection {
        match &prompt.primitive {
            Primitive::PickN { .. } => Selection::Picks { chosen: Vec::new(), half: None },
            Primitive::Number { min, .. } => Selection::Number(*min),
            Primitive::Allocate { mins, .. } => Selection::Allocation(mins.clone()),
            Primitive::Order => Selection::Order(Vec::new()),
        }
    }

    /// Whether `input` would change the answer in progress or complete it:
    /// a click the window may offer. Run on a copy, so it is the input's own
    /// logic that answers and no second copy of the rules.
    pub fn is_live(&self, prompt: &Prompt, input: Input) -> bool {
        let mut probe = self.clone();
        probe.apply(prompt, input).is_some() || probe != *self
    }

    /// The answer, once `input` completes a legal one. Legal by `ui/ask.rs`'s
    /// validators, so the engine never asserts on a window's answer.
    pub fn apply(&mut self, prompt: &Prompt, input: Input) -> Option<Answer> {
        if input == Input::Reset {
            *self = Selection::start(prompt);
            return None;
        }
        match (&prompt.primitive, self) {
            (Primitive::PickN { min, max }, Selection::Picks { chosen, half }) => {
                pick(prompt, (*min, *max), chosen, half, input)
            }
            (Primitive::Number { min, max }, Selection::Number(value)) => match input {
                Input::Number(n) => {
                    *value = n.clamp(*min, *max);
                    None
                }
                Input::Done => Some(Answer::Number(*value)),
                _ => None,
            },
            (Primitive::Allocate { total, mins, maxs }, Selection::Allocation(amounts)) => {
                allocate(prompt, *total, mins, maxs.as_deref(), amounts, input)
            }
            (Primitive::Order, Selection::Order(order)) => {
                let item = match input {
                    Input::OptionButton(i) => Some(i),
                    Input::Board(target) => (0..prompt.options.len())
                        .find(|i| prompt.options[*i].refs.first() == Some(&target) && !order.contains(i)),
                    Input::Done => {
                        return (order.len() == prompt.options.len()).then(|| Answer::Order(order.clone()));
                    }
                    _ => None,
                };
                if let Some(i) = item
                    && i < prompt.options.len()
                    && !order.contains(&i)
                {
                    order.push(i);
                }
                None
            }
            // `start` builds the selection from the same prompt, so the shapes always agree.
            _ => None,
        }
    }
}

fn pick(
    prompt: &Prompt,
    (min, max): (usize, usize),
    chosen: &mut Vec<usize>,
    half: &mut Option<BoardRef>,
    input: Input,
) -> Option<Answer> {
    let clicked = match input {
        Input::OptionButton(i) if i < prompt.options.len() => Some(i),
        Input::Board(target) => board_option(prompt, half, target),
        Input::Done => return (min..=max).contains(&chosen.len()).then(|| Answer::Picks(chosen.clone())),
        _ => None,
    }?;
    if max == 1 {
        return Some(Answer::Picks(vec![clicked]));
    }
    if let Some(at) = chosen.iter().position(|i| *i == clicked) {
        chosen.remove(at);
    } else if chosen.len() < max {
        chosen.push(clicked);
    }
    None
}

/// A board click as an option: the one option naming `target` first, or the
/// pair `target` completes. When several pairs start at `target`, it becomes
/// the half and the next click picks the pair.
fn board_option(prompt: &Prompt, half: &mut Option<BoardRef>, target: BoardRef) -> Option<usize> {
    if let Some(first) = half.take()
        && let Some(i) = prompt.options.iter().position(|o| o.refs == [first, target])
    {
        return Some(i);
    }
    let starting: Vec<usize> =
        (0..prompt.options.len()).filter(|i| prompt.options[*i].refs.first() == Some(&target)).collect();
    match starting.as_slice() {
        [] => None,
        [only] => Some(*only),
        several if several.iter().all(|i| prompt.options[*i].refs.len() == 2) => {
            *half = Some(target);
            None
        }
        // Two abilities of one permanent, say: the prompt's buttons tell them apart.
        _ => None,
    }
}

fn allocate(
    prompt: &Prompt,
    total: u64,
    mins: &[u64],
    maxs: Option<&[u64]>,
    amounts: &mut [u64],
    input: Input,
) -> Option<Answer> {
    let (bucket, more) = match input {
        Input::OneMore(bucket) | Input::OptionButton(bucket) => (bucket, true),
        Input::OneFewer(bucket) => (bucket, false),
        Input::Board(target) => (prompt.options.iter().position(|o| o.refs.first() == Some(&target))?, true),
        Input::Done => return (amounts.iter().sum::<u64>() == total).then(|| Answer::Allocation(amounts.to_vec())),
        _ => return None,
    };
    let left = total.saturating_sub(amounts.iter().sum());
    let cap = maxs.map_or(u64::MAX, |maxs| maxs[bucket]);
    match amounts.get_mut(bucket) {
        Some(amount) if more && left > 0 && *amount < cap => *amount += 1,
        Some(amount) if !more && *amount > mins[bucket] => *amount -= 1,
        _ => {}
    }
    None
}

/// Everything the window knows.
#[derive(Clone, Debug, Default)]
pub struct WindowState {
    pub board: Option<Snapshot>,
    pub prompt: Option<Prompt>,
    pub selection: Option<Selection>,
    /// Every log line so far, oldest first.
    pub log: Vec<String>,
    pub outcome: Option<Outcome>,
    pub panic: Option<String>,
    /// Why the scenario did not load.
    pub refused: Option<String>,
    /// The yield the seat holds at the open prompt.
    pub yielding: Option<Yield>,
    /// Full control is on: the seat is asked at every priority point, with
    /// its decorators and its yields off.
    pub full_control: bool,
    /// The window's clock, in seconds, as egui keeps it; `None` in a test
    /// that keeps none, where nothing settles.
    pub now: Option<f64>,
    /// When the open prompt arrived, by `now`.
    pub prompt_at: Option<f64>,
    /// Prompts received, which keys each prompt's widgets apart: focus on one
    /// prompt's button cannot pass to the next prompt's.
    pub prompts: u64,
}

impl WindowState {
    pub fn receive(&mut self, message: ToWindow) {
        match message {
            ToWindow::Prompt { snapshot, prompt, yielding } => {
                self.log.extend(snapshot.log.iter().cloned());
                self.yielding = yielding;
                self.prompt_at = self.now;
                self.prompts += 1;
                self.selection = Some(Selection::start(&prompt));
                self.prompt = Some(prompt);
                self.board = Some(snapshot);
            }
            ToWindow::Finished { snapshot, outcome } => {
                self.log.extend(snapshot.log.iter().cloned());
                self.board = Some(snapshot);
                self.prompt = None;
                self.selection = None;
                self.outcome = Some(outcome);
            }
            ToWindow::Panicked { message } => {
                self.prompt = None;
                self.selection = None;
                self.panic = Some(message);
            }
            ToWindow::Refused { message } => self.refused = Some(message),
        }
    }

    /// What to send the engine's thread, once `input` makes something: an
    /// answer or a yield, either of which closes the prompt, or "stop
    /// yielding", which leaves it open. The board stays until the engine's
    /// next message.
    pub fn input(&mut self, input: Input) -> Option<Reply> {
        let reply = match input {
            // The window's own controls, which `Session::input` acts on.
            Input::Reload | Input::SaveBoard => return None,
            Input::FullControl(on) => {
                self.full_control = on;
                return None;
            }
            // Aimed at the prompt before: it arrived too recently to be read.
            _ if self.settling_for().is_some() => return None,
            // A shortcut acts on the press; a held key's repeat answers nothing.
            Input::Key { repeat: true, .. } => return None,
            Input::Key { key, .. } => return self.click_for(key).and_then(|click| self.input(click)),
            Input::StopYielding => return self.yielding.take().map(|_| Reply::StopYielding),
            Input::Yield(until) => self.yield_is_live(until).then_some(Reply::Yield(until))?,
            input => Reply::Answer(self.selection.as_mut()?.apply(self.prompt.as_ref()?, input)?),
        };
        self.prompt = None;
        self.selection = None;
        Some(reply)
    }

    /// The click `key` stands for at the open prompt.
    fn click_for(&self, key: Key) -> Option<Input> {
        let prompt = self.prompt.as_ref()?;
        match key {
            Key::Digit(n) => Some(Input::OptionButton(usize::from(n).checked_sub(1)?)),
            Key::Enter => Some(Input::Done),
            Key::Space => prompt.pass.map(Input::OptionButton),
            Key::Escape => Some(Input::Reset),
            Key::F2 => Some(Input::Yield(Yield::UntilStackChanges)),
            Key::F4 => Some(Input::Yield(Yield::UntilEndOfTurn)),
            Key::F6 => Some(Input::Yield(Yield::UntilYourNextTurn)),
        }
    }

    /// The window's clock, which the drawing reads from egui at each frame.
    pub fn tick(&mut self, now: f64) {
        self.now = Some(now);
    }

    /// How much longer the open prompt drops input, while it does.
    pub fn settling_for(&self) -> Option<f64> {
        let left = SETTLE_SECONDS - (self.now? - self.prompt_at?);
        (left > 0.0).then_some(left)
    }

    /// Whether `until` can be set at the open prompt: a priority prompt, with
    /// full control off, since it supersedes yields, and for the yield that
    /// waits on the stack, a stack to wait on.
    fn yield_is_live(&self, until: Yield) -> bool {
        let at_priority = self.prompt.as_ref().is_some_and(|prompt| prompt.pass.is_some());
        let over_a_stack = self.board.as_ref().is_some_and(|board| !board.stack.is_empty());
        at_priority && !self.full_control && (until != Yield::UntilStackChanges || over_a_stack)
    }

    /// What the window is doing, for the header.
    pub fn status(&self) -> String {
        if self.refused.is_some() {
            "The scenario did not load".to_string()
        } else if self.panic.is_some() {
            "The engine panicked".to_string()
        } else if let Some(outcome) = &self.outcome {
            match outcome {
                Outcome::Won(player) => format!("Game over: Player {player} wins"),
                Outcome::Draw => "Game over: a draw".to_string(),
                Outcome::Error(error) => format!("The engine returned an error: {error}"),
            }
        } else if let Some(prompt) = &self.prompt {
            format!("Player {} to decide", prompt.player)
        } else {
            "The engine is playing".to_string()
        }
    }

    /// What the board's place says while there is no board.
    pub fn no_board(&self) -> &'static str {
        if self.refused.is_some() {
            "No board: the scenario did not load."
        } else if self.panic.is_some() {
            "No board: the engine panicked before its first prompt."
        } else {
            "Waiting for the engine's first prompt."
        }
    }

    pub fn board_view(&self) -> Option<BoardView> {
        let board = self.board.as_ref()?;
        let mut marks = Marks::new(self.prompt.as_ref(), self.selection.as_ref());
        if self.settling_for().is_some() {
            marks.clickable.clear();
        }
        Some(BoardView::new(board, &marks))
    }

    pub fn prompt_view(&self) -> Option<PromptView> {
        let prompt = self.prompt.as_ref()?;
        let mut view = PromptView::new(prompt, self.selection.as_ref()?, self.board.as_ref());
        view.serial = self.prompts;
        if prompt.pass.is_some() {
            view.yields = [Yield::UntilEndOfTurn, Yield::UntilStackChanges, Yield::UntilYourNextTurn]
                .into_iter()
                .map(|until| SeatButton {
                    label: format!("Pass {}", until_words(until, prompt.player)),
                    input: Input::Yield(until),
                    live: self.yield_is_live(until),
                })
                .collect();
        }
        view.yielding = self.yielding.map(|until| {
            let stop = SeatButton { label: "Stop yielding".to_string(), input: Input::StopYielding, live: true };
            (format!("Passing {}", until_words(until, prompt.player)), stop)
        });
        Some(if self.settling_for().is_some() { view.settling() } else { view })
    }
}

/// How long a yield passes for, in the person's words: a seat's yield passes
/// for that seat, so the next turn it waits for is `player`'s.
fn until_words(until: Yield, player: PlayerId) -> String {
    match until {
        Yield::UntilEndOfTurn => "until end of turn".to_string(),
        Yield::UntilStackChanges => "until the stack changes".to_string(),
        Yield::UntilYourNextTurn => format!("until Player {player}'s next turn"),
    }
}

/// One card, permanent, stack object or player, as the drawing shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub target: Option<BoardRef>,
    pub title: String,
    /// P/T, counters, status; the type line for a card.
    pub detail: String,
    /// An option names it, so a click means something.
    pub clickable: bool,
    pub chosen: bool,
    /// The prompt is about it (`ChoiceKind::subject()`).
    pub subject: bool,
    pub tapped: bool,
    /// Shown on hover, what it is now; empty for none.
    pub hover: String,
    /// Shown on hover beside it: the card as printed, a face an entry.
    pub printed: Option<Arc<[String]>>,
    /// Shown on hover under the first line of `hover`, a permanent's.
    pub type_line: Option<Arc<TypeLineView>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneView {
    /// The header, with the zone's count where it has one.
    pub name: String,
    /// The zone without its count, which keys the header's open state, so a
    /// card arriving does not close a graveyard the person opened.
    pub key: &'static str,
    pub items: Vec<Item>,
    /// Shown open rather than collapsed.
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatView {
    pub player: Item,
    pub zones: Vec<ZoneView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardView {
    pub header: String,
    /// Seat 0 last, nearest the prompt, and the others above it in seat
    /// order, whichever seat is asked.
    pub seats: Vec<SeatView>,
    /// Top first.
    pub stack: Vec<Item>,
    pub pending_triggers: Vec<Item>,
    pub exile: Vec<Item>,
    pub command: Vec<Item>,
}

/// Which board things a click would move the answer along, which the answer
/// has chosen, which the prompt is about, and which player it asks.
#[derive(Default)]
struct Marks {
    clickable: Vec<BoardRef>,
    chosen: Vec<BoardRef>,
    subject: Option<BoardRef>,
    asked: Option<PlayerId>,
}

impl Marks {
    fn new(prompt: Option<&Prompt>, selection: Option<&Selection>) -> Marks {
        let (Some(prompt), Some(selection)) = (prompt, selection) else {
            return Marks::default();
        };
        let first = |i: &usize| prompt.options[*i].refs.first().copied();
        // A pair's half waiting for its second click offers only its partners.
        let candidates: Vec<BoardRef> = match selection {
            Selection::Picks { half: Some(half), .. } => {
                prompt.options.iter().filter(|o| o.refs.first() == Some(half)).filter_map(|o| o.refs.get(1).copied()).collect()
            }
            _ => prompt.options.iter().flat_map(|o| o.refs.iter().copied()).collect(),
        };
        let mut clickable: Vec<BoardRef> = Vec::new();
        for target in candidates {
            if !clickable.contains(&target) && selection.is_live(prompt, Input::Board(target)) {
                clickable.push(target);
            }
        }
        let chosen = match selection {
            Selection::Picks { chosen, half } => chosen.iter().filter_map(first).chain(*half).collect(),
            Selection::Order(order) => order.iter().filter_map(first).collect(),
            Selection::Allocation(_) | Selection::Number(_) => Vec::new(),
        };
        Marks { clickable, chosen, subject: prompt.subject.map(BoardRef::Object), asked: Some(prompt.player) }
    }

    fn item(&self, target: BoardRef, title: String, detail: String, tapped: bool) -> Item {
        Item {
            target: Some(target),
            title,
            detail,
            clickable: self.clickable.contains(&target),
            chosen: self.chosen.contains(&target),
            subject: self.subject == Some(target),
            tapped,
            hover: String::new(),
            printed: None,
            type_line: None,
        }
    }

    fn card(&self, card: &CardView) -> Item {
        let title = match &card.mana_cost {
            Some(cost) => format!("{} {cost}", card.name),
            None => card.name.clone(),
        };
        let mut item = self.item(BoardRef::Object(card.id), title, card.type_line.text.clone(), false);
        item.printed = Some(Arc::clone(&card.printed));
        item
    }

    fn permanent(&self, permanent: &PermanentView) -> Item {
        let mut detail = Vec::new();
        if let Some((power, toughness)) = permanent.power_toughness {
            detail.push(format!("{power}/{toughness}"));
        }
        if permanent.damage > 0 {
            detail.push(format!("{} damage", permanent.damage));
        }
        detail.extend(permanent.keywords.iter().cloned());
        detail.extend(permanent.counters.iter().map(|(kind, n)| format!("{kind} ×{n}")));
        let flags = [
            (permanent.tapped, "tapped"),
            (permanent.summoning_sick, "summoning sick"),
            (permanent.phased_out, "phased out"),
            (permanent.face_down, "face down"),
        ];
        detail.extend(flags.iter().filter(|(on, _)| *on).map(|(_, word)| word.to_string()));
        if let Some(target) = &permanent.attacking {
            detail.push(format!("attacking {target}"));
        }
        for attacker in &permanent.blocking {
            detail.push(format!("blocking {attacker}"));
        }
        if let Some(host) = &permanent.attached_to {
            detail.push(format!("attached to {host}"));
        }
        let title = format!("{} ({})", permanent.card.name, permanent.card.id);
        let mut item = self.item(BoardRef::Object(permanent.card.id), title, detail.join(" · "), permanent.tapped);
        item.hover = permanent.engine_text.clone();
        item.printed = Some(Arc::clone(&permanent.card.printed));
        item.type_line = Some(Arc::clone(&permanent.card.type_line));
        item
    }

    fn player(&self, player: &PlayerView) -> Item {
        let asked = if self.asked == Some(player.id) { " (to decide)" } else { "" };
        let mut detail = vec![format!("{} life", player.life)];
        if !player.mana_pool.is_empty() {
            let pool: Vec<String> = player.mana_pool.iter().map(|(symbol, n)| symbol.repeat(*n as usize)).collect();
            detail.push(format!("pool {}", pool.concat()));
        }
        detail.extend(player.counters.iter().map(|(kind, n)| format!("{kind} {n}")));
        if player.lost {
            detail.push("lost".to_string());
        }
        self.item(BoardRef::Player(player.id), format!("Player {}{asked}", player.id), detail.join(" · "), false)
    }
}

impl BoardView {
    fn new(board: &Snapshot, marks: &Marks) -> BoardView {
        // One order at every prompt, so the board does not reshuffle as the
        // asked seat changes at each pass of priority.
        let mut seats: Vec<&PlayerView> = board.players.iter().skip(1).collect();
        seats.extend(board.players.first());
        BoardView {
            header: format!("Turn {} · Player {}'s turn · {}", board.turn, board.active_player, board.phase),
            seats: seats.into_iter().map(|player| seat(player, marks)).collect(),
            stack: board
                .stack
                .iter()
                .map(|item| {
                    let what = match item.is_spell {
                        Some(true) => "",
                        Some(false) => " — ability",
                        None => " — being cast or activated",
                    };
                    let mut detail = vec![format!("Player {}'s", item.controller)];
                    if !item.targets.is_empty() {
                        detail.push(format!("targets {}", item.targets.join(", ")));
                    }
                    if let Some(x) = item.x {
                        detail.push(format!("X = {x}"));
                    }
                    let title = format!("{} ({}){what}", item.name, item.id);
                    marks.item(BoardRef::Object(item.id), title, detail.join(" · "), false)
                })
                .collect(),
            pending_triggers: board
                .pending_triggers
                .iter()
                .map(|trigger| Item {
                    target: None,
                    title: trigger.source.clone(),
                    detail: format!("Player {}'s trigger, not yet on the stack", trigger.controller),
                    clickable: false,
                    chosen: false,
                    subject: false,
                    tapped: false,
                    hover: String::new(),
                    printed: None,
                    type_line: None,
                })
                .collect(),
            exile: board.exile.iter().map(|card| owned(marks, card)).collect(),
            command: board.command.iter().map(|card| owned(marks, card)).collect(),
        }
    }
}

fn owned(marks: &Marks, card: &CardView) -> Item {
    let mut item = marks.card(card);
    item.detail = format!("{} · Player {}'s", item.detail, card.owner);
    item
}

/// A seat's zones, the battlefield grouped the way `ui/display.rs` groups it:
/// creatures, then lands, then the rest, so a land creature is a creature.
fn seat(player: &PlayerView, marks: &Marks) -> SeatView {
    let group = |keep: &dyn Fn(&PermanentView) -> bool| -> Vec<Item> {
        player.battlefield.iter().filter(|p| keep(p)).map(|p| marks.permanent(p)).collect()
    };
    let battlefield = [
        ("Creatures", group(&|p| p.is_creature)),
        ("Lands", group(&|p| !p.is_creature && p.is_land)),
        ("Other permanents", group(&|p| !p.is_creature && !p.is_land)),
    ];
    let mut zones: Vec<ZoneView> = battlefield
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(name, items)| ZoneView { name: name.to_string(), key: name, items, open: true })
        .collect();
    let counted = |name: &'static str, cards: &[CardView], open: bool| ZoneView {
        name: format!("{name} ({})", cards.len()),
        key: name,
        items: cards.iter().map(|card| marks.card(card)).collect(),
        open,
    };
    zones.push(counted("Hand", &player.hand, true));
    zones.push(counted("Graveyard", &player.graveyard, false));
    zones.push(counted("Library", &player.library, false));
    SeatView { player: marks.player(player), zones }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionButton {
    pub label: String,
    pub chosen: bool,
    /// Its place in an ordering, from 1.
    pub place: Option<usize>,
    /// An allocation's bucket, drawn as its amount between "−" and "+".
    pub amount: Option<Amount>,
    /// A click on it moves the answer along.
    pub live: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amount {
    pub value: u64,
    pub can_lower: bool,
    pub can_raise: bool,
}

/// A `pick_number`'s field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberField {
    pub min: u64,
    pub max: u64,
    pub value: u64,
}

/// A control over the seat rather than an option of the prompt: a click on
/// it is `input`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatButton {
    pub label: String,
    pub input: Input,
    pub live: bool,
}

/// The confirm button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoneButton {
    pub label: String,
    pub live: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptView {
    /// The question, after the player it asks: `Player 1: Declare blockers`.
    pub question: String,
    /// Why the seat is asked again, when it is.
    pub rejected: Option<String>,
    /// What makes an answer complete.
    pub rule: String,
    pub options: Vec<OptionButton>,
    pub number: Option<NumberField>,
    /// The confirm button, when the prompt has one.
    pub done: Option<DoneButton>,
    /// "Start over" has something to undo.
    pub can_reset: bool,
    /// At a priority prompt, the yields: each passes now and keeps passing.
    pub yields: Vec<SeatButton>,
    /// The yield the seat holds, in words, and the button that ends it.
    pub yielding: Option<(String, SeatButton)>,
    /// `WindowState::prompts` when this prompt arrived.
    pub serial: u64,
}

impl PromptView {
    fn new(prompt: &Prompt, selection: &Selection, board: Option<&Snapshot>) -> PromptView {
        let live = |input: Input| selection.is_live(prompt, input);
        let options = prompt.options.iter().enumerate();
        let button = |i: usize, label: &str| OptionButton {
            label: label.to_string(),
            chosen: false,
            place: None,
            amount: None,
            live: live(Input::OptionButton(i)),
        };
        let done_button = |label: &str| DoneButton { label: label.to_string(), live: live(Input::Done) };
        let (rule, options, number, done): (String, Vec<OptionButton>, _, _) = match (&prompt.primitive, selection) {
            (Primitive::PickN { min, max }, Selection::Picks { chosen, half }) => {
                let rule = match half {
                    Some(half) => format!("now click what {} goes with", name_of(board, half)),
                    None => pick_rule(*min, *max),
                };
                // A lone option with a way out is a "may": yes or no.
                let may = *min == 0 && *max == 1 && prompt.options.len() == 1;
                let options = options
                    .map(|(i, o)| OptionButton {
                        chosen: chosen.contains(&i),
                        ..button(i, &if may { format!("Yes: {}", o.label) } else { o.label.clone() })
                    })
                    .collect();
                let done = match (*min, *max) {
                    (0, 1) => Some(done_button(if may { "No" } else { "Decline" })),
                    (_, 1) => None,
                    _ => Some(done_button("Done")),
                };
                (rule, options, None, done)
            }
            (Primitive::Number { min, max }, Selection::Number(value)) => {
                let top = if *max == u64::MAX { "any".to_string() } else { max.to_string() };
                let field = NumberField { min: *min, max: *max, value: *value };
                (format!("a number from {min} to {top}"), Vec::new(), Some(field), Some(done_button("Done")))
            }
            (Primitive::Allocate { total, .. }, Selection::Allocation(amounts)) => {
                let left = total.saturating_sub(amounts.iter().sum());
                let options = options
                    .map(|(i, o)| {
                        let amount =
                            Amount { value: amounts[i], can_lower: live(Input::OneFewer(i)), can_raise: live(Input::OneMore(i)) };
                        OptionButton { amount: Some(amount), ..button(i, &o.label) }
                    })
                    .collect();
                (format!("divide {total}: {left} left"), options, None, Some(done_button("Done")))
            }
            (Primitive::Order, Selection::Order(order)) => {
                let options = options
                    .map(|(i, o)| {
                        let place = order.iter().position(|placed| *placed == i).map(|at| at + 1);
                        OptionButton { chosen: place.is_some(), place, ..button(i, &o.label) }
                    })
                    .collect();
                let rule = format!("click them in order: {} of {} placed", order.len(), prompt.options.len());
                (rule, options, None, Some(done_button("Done")))
            }
            _ => (String::new(), Vec::new(), None, None),
        };
        let has_reset = !matches!(prompt.primitive, Primitive::PickN { max: 1, .. } | Primitive::Number { .. });
        let can_reset = has_reset && live(Input::Reset);
        PromptView {
            question: format!("Player {}: {}", prompt.player, prompt.question),
            rejected: prompt.rejected.clone(),
            rule,
            options,
            number,
            done,
            can_reset,
            yields: Vec::new(),
            yielding: None,
            serial: 0,
        }
    }

    /// Every control shown and none live: the beat after the prompt arrived.
    fn settling(mut self) -> PromptView {
        for option in &mut self.options {
            option.live = false;
            if let Some(amount) = &mut option.amount {
                (amount.can_lower, amount.can_raise) = (false, false);
            }
        }
        if let Some(done) = &mut self.done {
            done.live = false;
        }
        self.can_reset = false;
        for button in self.yields.iter_mut().chain(self.yielding.as_mut().map(|(_, stop)| stop)) {
            button.live = false;
        }
        self
    }
}

fn pick_rule(min: usize, max: usize) -> String {
    match (min, max) {
        (1, 1) => "choose one".to_string(),
        (0, 1) => "choose one, or decline".to_string(),
        (0, max) => format!("choose up to {max}"),
        (min, max) if min == max => format!("choose {min}"),
        (min, max) => format!("choose {min} to {max}"),
    }
}

/// `target` as the board titles it: `Hill Giant (#17)`, `Player 1`. A pair's
/// half is a permanent, so the battlefield is where it is looked for.
fn name_of(board: Option<&Snapshot>, target: &BoardRef) -> String {
    match target {
        BoardRef::Object(id) => board
            .and_then(|board| board.players.iter().flat_map(|p| &p.battlefield).find(|p| p.card.id == *id))
            .map_or_else(|| id.to_string(), |permanent| format!("{} ({id})", permanent.card.name)),
        BoardRef::Player(player) => format!("Player {player}"),
    }
}

#[cfg(test)]
mod tests {
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::test_support::{
        card_of_type, forest, lightning_bolt, put_in_hand, put_on_battlefield, set_attacking, set_blocking,
        setup_two_player_game, vanilla_creature,
    };
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::ids::ObjectId;

    use super::*;
    use crate::prompt::OptionView;
    use crate::snapshot::StackItem;
    use BoardRef::{Object, Player};

    struct Board {
        snapshot: Snapshot,
        bear: ObjectId,
        forest: ObjectId,
        arbor: ObjectId,
        relic: ObjectId,
        bolt: ObjectId,
        their_bear: ObjectId,
        their_giant: ObjectId,
    }

    fn board() -> Board {
        let mut game = setup_two_player_game();
        let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let forest = put_on_battlefield(&mut game, forest(), 0);
        let dryad_arbor = CardDataBuilder::new("Dryad Arbor")
            .card_type(CardType::Land)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let arbor = put_on_battlefield(&mut game, dryad_arbor, 0);
        let relic = put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 0);
        let bolt = put_in_hand(&mut game, lightning_bolt(), 0);
        let their_bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
        let their_giant = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
        set_attacking(&mut game, their_bear, 0);
        set_attacking(&mut game, their_giant, 0);
        set_blocking(&mut game, arbor, vec![their_giant]);
        let snapshot = Snapshot::build(&game, 0);
        Board { snapshot, bear, forest, arbor, relic, bolt, their_bear, their_giant }
    }

    fn deciding(board: &Board, prompt: Prompt) -> WindowState {
        WindowState {
            board: Some(board.snapshot.clone()),
            selection: Some(Selection::start(&prompt)),
            prompt: Some(prompt),
            ..WindowState::default()
        }
    }

    fn prompt(primitive: Primitive, options: Vec<OptionView>) -> Prompt {
        Prompt {
            player: 0,
            kind: "Test".to_string(),
            question: String::new(),
            subject: None,
            pass: None,
            rejected: None,
            primitive,
            options,
        }
    }

    fn option(label: &str, refs: Vec<BoardRef>) -> OptionView {
        OptionView { label: label.to_string(), refs }
    }

    fn unnamed(count: usize) -> Vec<OptionView> {
        (0..count).map(|i| option(&i.to_string(), Vec::new())).collect()
    }

    fn item(view: &BoardView, id: ObjectId) -> Item {
        view.seats
            .iter()
            .flat_map(|seat| seat.zones.iter().flat_map(|zone| zone.items.iter()))
            .find(|item| item.target == Some(Object(id)))
            .cloned()
            .unwrap_or_else(|| panic!("{id} is not on the board"))
    }

    #[test]
    fn a_hover_greys_the_land_types_blood_moon_took_and_a_hand_card_prints_in_order() {
        let mut game = setup_two_player_game();
        let bayou = put_on_battlefield(&mut game, mtgsim::cards::dual_lands::bayou(), 0);
        put_on_battlefield(&mut game, mtgsim::cards::phase_ld_cards::blood_moon(), 1);
        let priest = put_in_hand(&mut game, mtgsim::cards::phase_rc_cards::containment_priest(), 0);
        let view = WindowState { board: Some(Snapshot::build(&game, 0)), ..WindowState::default() }.board_view().unwrap();

        let line = item(&view, bayou).type_line.expect("a permanent's hover has its type line");
        let words = |section: &[TypeWordView]| section.iter().map(|w| (w.text.clone(), w.faded)).collect::<Vec<_>>();
        assert_eq!(words(&line.front), [("Land".to_string(), false)]);
        assert_eq!(
            words(&line.subtypes),
            [("Swamp".to_string(), true), ("Forest".to_string(), true), ("Mountain".to_string(), false)]
        );
        assert_eq!(line.text, "Land — Mountain");
        assert_eq!(item(&view, priest).detail, "Creature — Human Cleric");
        assert_eq!(item(&view, priest).type_line, None, "a hand card's line is its detail");
    }

    #[test]
    fn the_battlefield_groups_as_the_cli_does_and_a_land_creature_is_a_creature() {
        let b = board();
        let view = WindowState { board: Some(b.snapshot.clone()), ..WindowState::default() }.board_view().unwrap();
        let mine = view.seats.last().unwrap();
        assert_eq!(mine.player.target, Some(Player(0)), "seat 0 is drawn last, nearest the prompt");
        let names: Vec<&str> = mine.zones.iter().map(|zone| zone.name.as_str()).collect();
        assert_eq!(names, ["Creatures", "Lands", "Other permanents", "Hand (1)", "Graveyard (0)", "Library (0)"]);
        let keys: Vec<&str> = mine.zones.iter().map(|zone| zone.key).collect();
        assert_eq!(keys, ["Creatures", "Lands", "Other permanents", "Hand", "Graveyard", "Library"], "no count in a key");
        let targets = |zone: &ZoneView| zone.items.iter().map(|item| item.target).collect::<Vec<_>>();
        assert_eq!(targets(&mine.zones[0]), [Some(Object(b.bear)), Some(Object(b.arbor))]);
        assert_eq!(targets(&mine.zones[1]), [Some(Object(b.forest))]);
        assert_eq!(targets(&mine.zones[2]), [Some(Object(b.relic))]);
        assert!(item(&view, b.bear).detail.starts_with("2/2"), "{}", item(&view, b.bear).detail);
        assert!(!item(&view, b.relic).detail.contains('/'), "no power or toughness off a creature");
        assert_eq!(item(&view, b.relic).printed.as_deref().unwrap_or_default(), ["Relic\nArtifact".to_string()], "the card as printed beside the hover");
        assert!(item(&view, b.their_bear).detail.contains("attacking Player 0"));
        let giant = item(&view, b.their_giant).title;
        assert!(item(&view, b.arbor).detail.ends_with(&format!("blocking {giant}")), "named as the board titles it");
    }

    #[test]
    fn a_single_pick_answers_on_the_click_on_its_button_or_on_the_board() {
        let b = board();
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        let state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, options));
        let view = state.board_view().unwrap();
        assert!(item(&view, b.bolt).clickable);
        assert!(!item(&view, b.bear).clickable, "no option names it");
        assert_eq!(state.clone().input(Input::Board(Object(b.bear))), None);
        assert_eq!(state.clone().input(Input::OptionButton(0)), Some(Reply::Answer(Answer::Picks(vec![0]))));
        let mut clicked = state.clone();
        assert_eq!(clicked.input(Input::Board(Object(b.bolt))), Some(Reply::Answer(Answer::Picks(vec![1]))));
        assert!(clicked.prompt.is_none(), "the answer closes the prompt");
    }

    #[test]
    fn a_blocker_with_two_attackers_to_choose_between_takes_a_second_click() {
        let b = board();
        let blocks = vec![
            option("bear blocks bear", vec![Object(b.bear), Object(b.their_bear)]),
            option("bear blocks giant", vec![Object(b.bear), Object(b.their_giant)]),
            option("arbor blocks bear", vec![Object(b.arbor), Object(b.their_bear)]),
        ];
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 3 }, blocks));
        assert_eq!(state.input(Input::Board(Object(b.bear))), None);
        let view = state.board_view().unwrap();
        assert!(item(&view, b.bear).chosen, "the half shows as chosen");
        assert!(item(&view, b.their_bear).clickable && item(&view, b.their_giant).clickable);
        assert!(!item(&view, b.arbor).clickable, "only the half's partners, until it is completed");
        let bear = item(&view, b.bear).title;
        assert_eq!(state.prompt_view().unwrap().rule, format!("now click what {bear} goes with"));
        state.input(Input::Board(Object(b.their_giant)));
        state.input(Input::Board(Object(b.arbor)));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Picks(vec![1, 2]))));
    }

    #[test]
    fn a_click_on_what_a_pair_ends_at_starts_nothing() {
        let b = board();
        let blocks = vec![option("bear blocks bear", vec![Object(b.bear), Object(b.their_bear)])];
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, blocks));
        assert!(!item(&state.board_view().unwrap(), b.their_bear).clickable);
        assert_eq!(state.input(Input::Board(Object(b.their_bear))), None);
        assert_eq!(state.selection, Some(Selection::Picks { chosen: Vec::new(), half: None }), "no half waits for a partner");
    }

    #[test]
    fn a_lone_option_with_a_way_out_reads_as_yes_or_no() {
        let b = board();
        let state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, vec![option("Blood Artist", Vec::new())]));
        let view = state.prompt_view().unwrap();
        assert_eq!(view.options[0].label, "Yes: Blood Artist");
        assert_eq!(view.done, Some(DoneButton { label: "No".to_string(), live: true }));
        assert_eq!(state.clone().input(Input::Done), Some(Reply::Answer(Answer::Picks(Vec::new()))));
    }

    #[test]
    fn done_is_live_only_inside_the_bounds() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 2, max: 3 }, unnamed(4)));
        state.input(Input::OptionButton(0));
        assert_eq!(state.prompt_view().unwrap().done.map(|done| done.live), Some(false));
        assert_eq!(state.input(Input::Done), None);
        for i in [3, 1, 2] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.prompt_view().unwrap().done.map(|done| done.live), Some(true));
        state.input(Input::OptionButton(3));
        assert_eq!(
            state.input(Input::Done),
            Some(Reply::Answer(Answer::Picks(vec![0, 1]))),
            "a pick past the maximum is refused, and a second click undoes one"
        );
    }

    #[test]
    fn an_allocation_stays_inside_each_bucket_and_is_done_when_the_total_is_placed() {
        let b = board();
        let buckets = vec![option("bear", vec![Object(b.their_bear)]), option("player", vec![Player(1)])];
        let split = Primitive::Allocate { total: 3, mins: vec![1, 0], maxs: Some(vec![2, 3]) };
        let mut state = deciding(&b, prompt(split, buckets));
        state.input(Input::OneFewer(0));
        for _ in 0..3 {
            state.input(Input::OneMore(0));
        }
        let amount = state.prompt_view().unwrap().options[0].amount;
        assert_eq!(amount, Some(Amount { value: 2, can_lower: true, can_raise: false }), "held between its minimum and its maximum");
        assert_eq!(state.input(Input::Done), None, "one still to place");
        state.input(Input::Board(Player(1)));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Allocation(vec![2, 1]))));
    }

    #[test]
    fn an_ordering_is_the_click_order_and_starts_over_on_reset() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Order, unnamed(3)));
        for i in [2, 0, 2] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.prompt_view().unwrap().options[2].place, Some(1));
        assert_eq!(state.input(Input::Done), None);
        state.input(Input::Reset);
        for i in [1, 2, 0] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Order(vec![1, 2, 0]))));
    }

    #[test]
    fn a_number_is_held_to_its_range() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Number { min: 2, max: 5 }, Vec::new()));
        state.input(Input::Number(9));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Number(5))));
    }

    #[test]
    fn a_click_the_window_offers_always_moves_the_answer_along() {
        let b = board();
        // Two abilities of one permanent: its buttons tell them apart, and a
        // click on the permanent could mean either, so it is no target.
        let abilities = vec![option("ability 0", vec![Object(b.forest)]), option("ability 1", vec![Object(b.forest)])];
        let state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, abilities));
        assert!(!item(&state.board_view().unwrap(), b.forest).clickable);

        let mut order = deciding(&b, prompt(Primitive::Order, unnamed(2)));
        assert!(!order.prompt_view().unwrap().can_reset, "nothing to start over yet");
        order.input(Input::OptionButton(1));
        let view = order.prompt_view().unwrap();
        assert!(view.options[0].live && !view.options[1].live, "a placed option's button is spent");
        assert!(view.can_reset);

        let mut picks = deciding(&b, prompt(Primitive::PickN { min: 0, max: 2 }, unnamed(3)));
        picks.input(Input::OptionButton(0));
        picks.input(Input::OptionButton(2));
        let live: Vec<bool> = picks.prompt_view().unwrap().options.iter().map(|o| o.live).collect();
        assert_eq!(live, [true, false, true], "a pick past the maximum is refused, and a chosen one can be undone");
    }

    #[test]
    fn the_prompts_subject_is_marked_on_the_board() {
        let b = board();
        let mut about = prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2));
        about.subject = Some(b.bear);
        let view = deciding(&b, about).board_view().unwrap();
        assert!(item(&view, b.bear).subject);
        assert!(!item(&view, b.arbor).subject);
    }

    fn at_priority(b: &Board) -> WindowState {
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        deciding(b, Prompt { pass: Some(0), ..prompt(Primitive::PickN { min: 1, max: 1 }, options) })
    }

    /// A yield answers the priority prompt it is set at; the one that waits
    /// on the stack needs a stack, and full control turns them all off.
    #[test]
    fn a_yield_answers_a_priority_prompt_and_full_control_supersedes_it() {
        let b = board();
        let state = at_priority(&b);
        let live: Vec<bool> = state.prompt_view().unwrap().yields.iter().map(|button| button.live).collect();
        assert_eq!(live, [true, false, true], "an empty stack has nothing to wait on");
        assert_eq!(state.clone().input(Input::Yield(Yield::UntilStackChanges)), None);
        let mut over_a_stack = state.clone();
        let spell = StackItem { id: b.bolt, name: "Lightning Bolt".to_string(), is_spell: Some(true), controller: 1, targets: Vec::new(), x: None };
        over_a_stack.board.as_mut().unwrap().stack.push(spell);
        assert!(over_a_stack.prompt_view().unwrap().yields[1].live);
        let mut yielded = state.clone();
        assert_eq!(yielded.input(Input::Yield(Yield::UntilEndOfTurn)), Some(Reply::Yield(Yield::UntilEndOfTurn)));
        assert!(yielded.prompt.is_none(), "the yield answered the prompt");

        let mut full = state.clone();
        full.input(Input::FullControl(true));
        assert!(full.prompt_view().unwrap().yields.iter().all(|button| !button.live));
        assert_eq!(full.input(Input::Yield(Yield::UntilEndOfTurn)), None);
    }

    /// Only a priority prompt offers a yield; while one holds, any prompt
    /// offers to stop it, which leaves the prompt open.
    #[test]
    fn stopping_a_yield_leaves_the_prompt_open() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 3 }, unnamed(3)));
        state.yielding = Some(Yield::UntilYourNextTurn);
        let view = state.prompt_view().unwrap();
        assert!(view.yields.is_empty());
        let (words, stop) = view.yielding.unwrap();
        assert_eq!(words, "Passing until Player 0's next turn");
        assert_eq!(state.input(stop.input), Some(Reply::StopYielding));
        assert!(state.prompt.is_some() && state.yielding.is_none());
        assert_eq!(state.input(Input::StopYielding), None, "nothing left to stop");
    }

    /// Item 201: a click in the moment after a prompt arrives was aimed at
    /// the one before, so it is dropped, and the prompt shows nothing live
    /// until the moment has passed. A held key's repeat never answers.
    #[test]
    fn input_in_the_beat_after_a_prompt_arrives_is_dropped() {
        let b = board();
        let mut state = WindowState::default();
        state.tick(10.0);
        state.receive(ToWindow::Prompt { snapshot: b.snapshot.clone(), prompt: prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)), yielding: None });
        state.tick(10.1);
        assert_eq!(state.input(Input::OptionButton(0)), None, "the second click of a double click");
        assert!(state.prompt_view().unwrap().options.iter().all(|option| !option.live));
        state.tick(10.0 + 2.0 * SETTLE_SECONDS);
        assert_eq!(state.input(Input::Key { key: Key::Digit(1), repeat: true }), None, "a held key");
        assert_eq!(state.input(Input::Key { key: Key::Digit(2), repeat: false }), Some(Reply::Answer(Answer::Picks(vec![1]))));
    }

    /// A key is a click the prompt already offers: Space is the pass a
    /// priority prompt offers, Enter the confirm button, F4 a yield.
    #[test]
    fn a_key_stands_for_a_click_the_prompt_offers() {
        let b = board();
        let state = at_priority(&b);
        let pressed = |key| state.clone().input(Input::Key { key, repeat: false });
        assert_eq!(pressed(Key::Space), Some(Reply::Answer(Answer::Picks(vec![0]))));
        assert_eq!(pressed(Key::F4), Some(Reply::Yield(Yield::UntilEndOfTurn)));
        assert_eq!(pressed(Key::Digit(9)), None, "no ninth option");
        assert_eq!(pressed(Key::Enter), None, "a single pick has no confirm button");
        let decline = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, unnamed(1)));
        assert_eq!(decline.clone().input(Input::Key { key: Key::Enter, repeat: false }), Some(Reply::Answer(Answer::Picks(Vec::new()))));
        assert_eq!(decline.clone().input(Input::Key { key: Key::Space, repeat: false }), None, "no pass outside priority");
    }

    #[test]
    fn a_panic_closes_the_prompt_and_says_so() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        let message = "ask_choose_priority_action: DP returned index 9".to_string();
        state.receive(ToWindow::Panicked { message });
        assert!(state.prompt.is_none() && state.prompt_view().is_none());
        assert_eq!(state.status(), "The engine panicked");
        assert_eq!(state.input(Input::OptionButton(0)), None);
        let before_any_board = WindowState { panic: Some("early".to_string()), ..WindowState::default() };
        assert_eq!(before_any_board.no_board(), "No board: the engine panicked before its first prompt.");
    }

    /// The window plays every seat (`setup-architecture.md` §7's "Seats"):
    /// a prompt says whose it is, the board marks that seat and keeps one
    /// order, and a yield names the turn it waits for by its seat.
    #[test]
    fn a_prompt_names_the_seat_it_asks_and_the_board_keeps_its_order() {
        let b = board();
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        let asking = |player| {
            let question = "You have priority".to_string();
            let priority = prompt(Primitive::PickN { min: 1, max: 1 }, options.clone());
            deciding(&b, Prompt { player, pass: Some(0), question, ..priority })
        };
        let theirs = asking(1);
        assert_eq!(theirs.status(), "Player 1 to decide");
        let view = theirs.prompt_view().unwrap();
        assert_eq!(view.question, "Player 1: You have priority");
        let next_turn = view.yields.iter().find(|button| button.input == Input::Yield(Yield::UntilYourNextTurn)).unwrap();
        assert_eq!(next_turn.label, "Pass until Player 1's next turn", "a seat's yield waits for that seat's turn");
        for (asked, state) in [(0, asking(0)), (1, theirs)] {
            let titles: Vec<String> = state.board_view().unwrap().seats.iter().map(|seat| seat.player.title.clone()).collect();
            let named = |p: PlayerId| if p == asked { format!("Player {p} (to decide)") } else { format!("Player {p}") };
            assert_eq!(titles, [named(1), named(0)], "one order whichever seat is asked");
        }
        let won = WindowState { outcome: Some(Outcome::Won(0)), ..WindowState::default() };
        assert_eq!(won.status(), "Game over: Player 0 wins");
    }
}
