//! The board editor (`setup-architecture.md` §7b): a scenario built by
//! clicking, in plain Rust, which `app` draws.
//!
//! An [`Editor`] holds the board, a `Scenario`; the boards before it, for
//! Undo; and the loader's verdict on it. An [`EditorInput`] that edits makes
//! a new board, written and read back (`Scenario::parse` of its `Display`)
//! before it replaces the old one. So the editor never holds a board its own
//! text cannot say, and each line's number is its place in that text, which
//! is how the line a refusal names marks a card. The check is
//! `Scenario::build`: the editor checks nothing itself.
//!
//! [`EditorView`] is the board as the drawing needs it: every control with
//! the input a click on it sends, and whether that click changes anything.

use std::path::PathBuf;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::{
    Arrival, Attacked, CardLine, CardWord, LineKind, LineNumbered, MOST_PLAYERS, NamedCard, PlayerWord, Scenario, ScenarioError,
    Targeted, position_word, tag_letters, turn_positions,
};
use mtgsim::state::game_state::Phase;
use mtgsim::types::effects::CounterType;
use mtgsim::types::ids::PlayerId;

use crate::search::NameSearch;

/// The widest a life total or a starting life is typed, either way.
const LIFE_LIMIT: i64 = 99_999;
/// The latest turn typed: the loader begins every turn up to it.
const TURN_LIMIT: i64 = 999;
/// The most lands played, counters of a kind or commander damage typed.
const COUNT_LIMIT: i64 = 999;
/// The kinds `CounterType` groups as a player's (CR 122.1), each a control
/// in the advanced settings; another kind a player line states is shown as
/// its text.
const PLAYER_COUNTERS: [CounterType; 2] = [CounterType::Poison, CounterType::Energy];

/// A seat's zone, as the editor lists a card under one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Battlefield,
    Hand,
    Library,
    Graveyard,
    Exile,
    Command,
}

impl Zone {
    pub const ALL: [Zone; 6] = [Zone::Battlefield, Zone::Hand, Zone::Library, Zone::Graveyard, Zone::Exile, Zone::Command];

    pub fn name(self) -> &'static str {
        match self {
            Zone::Battlefield => "Battlefield",
            Zone::Hand => "Hand",
            Zone::Library => "Library",
            Zone::Graveyard => "Graveyard",
            Zone::Exile => "Exile",
            Zone::Command => "Command zone",
        }
    }

    /// The head of a line listing a card here for `seat`.
    fn kind(self, seat: PlayerId, shuffled: bool) -> LineKind {
        match self {
            Zone::Battlefield => LineKind::Battlefield,
            Zone::Hand => LineKind::Hand(seat),
            Zone::Library => LineKind::Library { player: seat, shuffled },
            Zone::Graveyard => LineKind::Graveyard(seat),
            Zone::Exile => LineKind::Exile,
            Zone::Command => LineKind::Command,
        }
    }

    /// The word saying whose card it is here: a permanent's controller, an
    /// exiled or command-zone card's owner; a seat's other zones say it in
    /// the head.
    fn seat_word(self, seat: PlayerId) -> Option<CardWord> {
        match self {
            Zone::Battlefield => Some(CardWord::Controller(seat)),
            Zone::Exile | Zone::Command => Some(CardWord::Owner(seat)),
            Zone::Hand | Zone::Library | Zone::Graveyard => None,
        }
    }
}

/// A word naming another card, set by a click on that card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    AttachedTo,
    Attacking,
    Blocking,
}

/// A word a card has or has not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flag {
    Tapped,
    Commander,
    Blocked,
    DealtFirstStrikeDamage,
}

impl Flag {
    fn word(self) -> CardWord {
        match self {
            Flag::Tapped => CardWord::Tapped,
            Flag::Commander => CardWord::Commander,
            Flag::Blocked => CardWord::Blocked,
            Flag::DealtFirstStrikeDamage => CardWord::DealtFirstStrikeDamage,
        }
    }
}

/// Where a card moves among the cards of its zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Top,
    Up,
    Down,
    Bottom,
}

/// A number the editor sets: a game's fact, a player's count, or a count on
/// the card line it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoardNumber {
    Players,
    StartingLife,
    Turn,
    Life(PlayerId),
    LandsPlayed(PlayerId),
    PlayerCounter(PlayerId, CounterType),
    /// The combat damage the player has taken from the commander on card
    /// line `from` (CR 903.10a).
    CommanderDamage { player: PlayerId, from: usize },
    Damage(usize),
    Copies(usize),
    ArrivedTurn(usize),
}

/// A word or line the editor shows as its text, which it can remove: one the
/// advanced settings have no control for, or any while they are off
/// (§7b.2's decision 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextItem {
    PlayerWord(usize),
    /// A `counters:` or `this turn:` line, by its place among the card lines.
    Line(usize),
    SetupAction(usize),
}

/// Something done in the editor. A card is named by its place among the
/// board's card lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorInput {
    Number(BoardNumber, i64),
    Seed(u64),
    Active(PlayerId),
    Step(Phase),
    /// The name chosen in the search, last in a seat's zone.
    Put(PlayerId, Zone),
    /// A seat's library shuffled from the seed, or listed top first.
    Shuffled(PlayerId, bool),
    /// The player has left the game (CR 800.4a), or is in it.
    LeftTheGame(PlayerId, bool),
    RemoveText(TextItem),
    /// A click on a card: edit it, or name it in the reference being picked.
    Card(usize),
    /// Stop editing the card.
    Close,
    Controller(usize, PlayerId),
    Owner(usize, PlayerId),
    Flag(usize, Flag, bool),
    /// When the permanent arrived; `None` is before the game.
    Arrival(usize, Option<Arrival>),
    /// A counter kind's stated count; `None` leaves the kind's default.
    Counter(usize, CounterType, Option<u32>),
    AttackPlayer(usize, PlayerId),
    /// Wait for the click on the card the reference names; `None` stops.
    Pick(Option<Reference>),
    /// The card's word for that reference, gone.
    Unreference(usize, Reference),
    /// Into its owner's zone, or onto the battlefield under its owner.
    Zone(usize, Zone),
    Move(usize, Direction),
    Remove(usize),
    Undo,
    Search(String),
    /// A name chosen, by its place in the list searched.
    Choose(usize),
    /// The advanced settings shown, or hidden (§7b.2's decision 2).
    Advanced(bool),
    /// The typed field's line, as typed.
    TypedLine(String),
    /// The typed field's line added to the board.
    AddTypedLine,
    /// The session's: save the board to its file, then play it.
    Play,
    /// The session's: save the board to its file.
    Save,
    /// The session's: open the listed file `i`.
    Open(usize),
}

impl EditorInput {
    /// The card line the input acts on, when it acts on one.
    fn card(&self) -> Option<usize> {
        match *self {
            EditorInput::Number(BoardNumber::Damage(i) | BoardNumber::Copies(i) | BoardNumber::ArrivedTurn(i), _)
            | EditorInput::Controller(i, _)
            | EditorInput::Owner(i, _)
            | EditorInput::Flag(i, ..)
            | EditorInput::Arrival(i, _)
            | EditorInput::Counter(i, ..)
            | EditorInput::AttackPlayer(i, _)
            | EditorInput::Unreference(i, _)
            | EditorInput::Zone(i, _)
            | EditorInput::Move(i, _)
            | EditorInput::Remove(i) => Some(i),
            _ => None,
        }
    }
}

/// Where the board came from, which says where it saves
/// (`setup-architecture.md` §7b, decision 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// A board begun empty, named `board-N` at its first save.
    Empty,
    /// The board a game was at, named for the game's start and turn.
    Game { start: String, turn: u32 },
    /// A file outside `boards/`, never written: its first save makes a board
    /// named for it.
    File(PathBuf),
    /// Its own file in `boards/`, saved over.
    Board(PathBuf),
}

pub struct Editor {
    board: Scenario,
    /// The board's text, `Display`'s: what Save writes and Copy copies.
    text: String,
    /// The boards before each edit, the latest last.
    undo: Vec<Scenario>,
    /// The loader's refusal of the board; `None` while it builds.
    refusal: Option<ScenarioError>,
    /// Why the last edit was not made.
    unsaid: Option<String>,
    /// The file's comment lines above its first line, kept on each save.
    pub comments: Vec<String>,
    pub source: Source,
    /// The card line being edited.
    pub editing: Option<usize>,
    /// The reference the card being edited waits to name.
    pub picking: Option<Reference>,
    /// The name chosen in the search, by its place in the list searched.
    pub chosen: Option<usize>,
    /// The advanced settings' controls and the typed field, shown.
    pub advanced: bool,
    typed_line: String,
    /// Why the typed field's line was not added: the parser's refusal, or a
    /// line that says nothing the board does not.
    typed_line_refusal: Option<String>,
    search: NameSearch,
    /// How many of the names searched are registered; the rest are in
    /// development.
    registered: usize,
    registry: CardRegistry,
}

impl Editor {
    /// The board `text` says, its leading comments kept; refused if it does
    /// not parse. Names are looked up in `registry` and its cards in
    /// development.
    pub fn open(text: &str, source: Source, registry: CardRegistry) -> Result<Editor, ScenarioError> {
        let mut comments: Vec<String> = text
            .lines()
            .take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
            .map(str::to_string)
            .collect();
        while comments.last().is_some_and(|line| line.trim().is_empty()) {
            comments.pop();
        }
        let board = Scenario::parse(&Scenario::parse(text)?.to_string())?;
        Ok(Editor::with(board, comments, source, registry))
    }

    /// An empty two-seat board, named at its first save.
    pub fn empty(registry: CardRegistry) -> Editor {
        Editor::with(Scenario::default(), Vec::new(), Source::Empty, registry)
    }

    fn with(board: Scenario, comments: Vec<String>, source: Source, registry: CardRegistry) -> Editor {
        let mut names: Vec<String> = registry.card_names().into_iter().map(str::to_string).collect();
        let registered = names.len();
        names.extend(registry.names_in_development().into_iter().map(str::to_string));
        let mut editor = Editor {
            text: board.to_string(),
            board,
            undo: Vec::new(),
            refusal: None,
            unsaid: None,
            comments,
            source,
            editing: None,
            picking: None,
            chosen: None,
            advanced: false,
            typed_line: String::new(),
            typed_line_refusal: None,
            search: NameSearch::new(names),
            registered,
            registry,
        };
        editor.check();
        editor
    }

    pub fn board(&self) -> &Scenario {
        &self.board
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn refusal(&self) -> Option<&ScenarioError> {
        self.refusal.as_ref()
    }

    /// The file a save writes: the comment lines kept, then the board.
    pub fn file_text(&self) -> String {
        if self.comments.is_empty() {
            self.text.clone()
        } else {
            format!("{}\n\n{}", self.comments.join("\n"), self.text)
        }
    }

    pub fn search(&self) -> &NameSearch {
        &self.search
    }

    pub fn input(&mut self, input: EditorInput) {
        self.unsaid = None;
        match input {
            EditorInput::Card(i) => match (self.picking, self.editing) {
                (Some(reference), Some(edited)) => {
                    self.edit(|draft| draft.set_reference(edited, reference, i));
                }
                _ if listed_at(&self.board, i).is_some() => (self.editing, self.picking) = (Some(i), None),
                _ => {}
            },
            EditorInput::Close => (self.editing, self.picking) = (None, None),
            EditorInput::Pick(reference) => self.picking = reference.filter(|_| self.editing.is_some()),
            EditorInput::Search(query) => self.search.set_query(query),
            EditorInput::Choose(i) => self.chosen = self.search.name(i).map(|_| i),
            EditorInput::Advanced(on) => self.advanced = on,
            EditorInput::TypedLine(line) => (self.typed_line, self.typed_line_refusal) = (line, None),
            EditorInput::AddTypedLine => self.add_typed_line(),
            EditorInput::Undo => self.step_back(),
            EditorInput::Play | EditorInput::Save | EditorInput::Open(_) => {}
            edit => {
                if edit.card().is_some_and(|i| listed_at(&self.board, i).is_none()) {
                    return;
                }
                let chosen = self.chosen.and_then(|i| self.search.name(i)).map(str::to_string);
                self.edit(|draft| draft.apply(edit, chosen.as_deref()));
            }
        }
    }

    /// An edit made to a copy of the board. The copy, renumbered through
    /// its text, replaces the board if it differs; whether it did.
    fn edit(&mut self, change: impl FnOnce(&mut Draft)) -> bool {
        let mut draft = Draft { board: self.board.clone(), edited: self.editing };
        change(&mut draft);
        self.picking = None;
        let text = draft.board.to_string();
        let unsaid = "Not made: the board's text would not read back as the board";
        match Scenario::parse(&text) {
            Ok(board) if board.to_string() != text => self.unsaid = Some(format!("{unsaid}.")),
            Ok(board) if board == self.board => {}
            Ok(board) => {
                self.undo.push(std::mem::replace(&mut self.board, board));
                self.text = text;
                self.editing = draft.edited;
                self.check();
                return true;
            }
            Err(refusal) => self.unsaid = Some(format!("{unsaid}: {}.", refusal.message)),
        }
        false
    }

    /// The typed field's line, put last in the board's text, which is read
    /// and made an edit: the field empties. A line the parser refuses, or
    /// one that says nothing the board does not, leaves the board as it was
    /// and stays in the field with why.
    fn add_typed_line(&mut self) {
        match Scenario::parse(&format!("{}{}\n", self.text, self.typed_line)) {
            Err(refusal) => self.typed_line_refusal = Some(format!("Not added: {}.", refusal.message)),
            Ok(board) => {
                if self.edit(|draft| draft.board = board) {
                    self.typed_line.clear();
                } else {
                    let says_nothing = "Not added: the line says nothing the board does not.".to_string();
                    self.typed_line_refusal = Some(self.unsaid.take().unwrap_or(says_nothing));
                }
            }
        }
    }

    /// Undo: the board before the last edit.
    fn step_back(&mut self) {
        let Some(board) = self.undo.pop() else { return };
        self.board = board;
        self.text = self.board.to_string();
        self.editing = self.editing.filter(|&i| listed_at(&self.board, i).is_some());
        self.picking = None;
        self.check();
    }

    fn check(&mut self) {
        self.refusal = self.board.build(&self.registry).err();
    }
}

/// Where a card line is listed: its seat and zone. `None` for a line the
/// editor shows as text, or none at all.
fn listed(line: &CardLine) -> Option<(PlayerId, Zone)> {
    let zone = match line.kind {
        LineKind::Battlefield => Zone::Battlefield,
        LineKind::Hand(_) => Zone::Hand,
        LineKind::Library { .. } => Zone::Library,
        LineKind::Graveyard(_) => Zone::Graveyard,
        LineKind::Exile => Zone::Exile,
        LineKind::Command => Zone::Command,
        LineKind::Counters | LineKind::ThisTurn => return None,
    };
    let seat = if zone == Zone::Battlefield { controller(line) } else { owner(line) };
    Some((seat.unwrap_or(0), zone))
}

fn listed_at(board: &Scenario, i: usize) -> Option<(PlayerId, Zone)> {
    board.cards.get(i).and_then(|line| listed(&line.value))
}

fn stated(line: &CardLine, seat_of: fn(&CardWord) -> Option<PlayerId>) -> Option<PlayerId> {
    line.words.iter().find_map(seat_of)
}

fn owner_word(word: &CardWord) -> Option<PlayerId> {
    if let CardWord::Owner(p) = word { Some(*p) } else { None }
}

fn controller_word(word: &CardWord) -> Option<PlayerId> {
    if let CardWord::Controller(p) = word { Some(*p) } else { None }
}

/// Whose card it is (CR 108.3): the seat a hand, library or graveyard line
/// names, or the `owner` word, which on the battlefield defaults to the
/// controller.
fn owner(line: &CardLine) -> Option<PlayerId> {
    match line.kind {
        LineKind::Hand(p) | LineKind::Library { player: p, .. } | LineKind::Graveyard(p) => Some(p),
        LineKind::Battlefield => stated(line, owner_word).or(stated(line, controller_word)),
        _ => stated(line, owner_word),
    }
}

/// Who controls a permanent: its `controller` word, defaulting to the owner.
fn controller(line: &CardLine) -> Option<PlayerId> {
    stated(line, controller_word).or(stated(line, owner_word))
}

/// Do two lines list their cards in one zone, as the editor orders it? The
/// battlefield is one, whoever controls each permanent: its order is CR
/// 613.7d's timestamps, which permanents of different controllers compare.
fn same_zone(a: &CardLine, b: &CardLine) -> bool {
    match (listed(a), listed(b)) {
        (Some((_, Zone::Battlefield)), Some((_, Zone::Battlefield))) => true,
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Where card `i` goes to move `direction` in its zone, when that changes
/// the board: on its way it passes a line that says something else.
fn move_target(board: &Scenario, i: usize, direction: Direction) -> Option<usize> {
    let line = &board.cards.get(i)?.value;
    let zone: Vec<usize> = (0..board.cards.len()).filter(|&k| same_zone(&board.cards[k].value, line)).collect();
    let place = zone.iter().position(|&k| k == i)?;
    let to = *match direction {
        Direction::Top => zone.first(),
        Direction::Up => place.checked_sub(1).and_then(|p| zone.get(p)),
        Direction::Down => zone.get(place + 1),
        Direction::Bottom => zone.last(),
    }?;
    let passed = if to < i { to..i } else { i + 1..to + 1 };
    board.cards[passed].iter().any(|other| other.value != *line).then_some(to)
}

/// Every word `is` matches replaced by `word` at the first one's place, or
/// last; or gone, for `None`.
fn set_word(words: &mut Vec<CardWord>, is: impl Fn(&CardWord) -> bool, word: Option<CardWord>) {
    let at = words.iter().position(&is);
    words.retain(|w| !is(w));
    if let Some(word) = word {
        words.insert(at.unwrap_or(words.len()).min(words.len()), word);
    }
}

/// A permanent's controller and owner words, the owner said only where it
/// differs.
fn set_seats(line: &mut CardLine, controller: PlayerId, owner: PlayerId) {
    line.words.retain(|w| !matches!(w, CardWord::Controller(_) | CardWord::Owner(_)));
    if owner != controller {
        line.words.insert(0, CardWord::Owner(owner));
    }
    line.words.insert(0, CardWord::Controller(controller));
}

/// The cards a reference can name (`setup-architecture.md` §4.2): the
/// permanents, for a word on the battlefield or a `counters:` or `this turn:`
/// line; the commanders, for commander damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Among {
    Permanents,
    Commanders,
}

fn is_among(line: &CardLine, among: Among) -> bool {
    match among {
        Among::Permanents => line.kind == LineKind::Battlefield,
        Among::Commanders => listed(line).is_some() && line.words.contains(&CardWord::Commander),
    }
}

/// Does `reference` name `card`, as the loader matches: an untagged reference
/// names every card of its name.
fn names(reference: &NamedCard, card: &NamedCard) -> bool {
    reference.name == card.name && (reference.tag.is_none() || reference.tag == card.tag)
}

/// Where a reference the editor keeps true is on the board.
#[derive(Clone, Copy, Debug)]
enum At {
    /// A word on a battlefield line.
    Word { line: usize, word: usize },
    /// A `counters:` or `this turn:` line's card.
    Line(usize),
    CommanderDamage(usize),
}

/// The references the editor keeps naming their cards, with the cards each
/// can name. A setup action's names stay as written: one may name a card in
/// a graveyard, or a spell (§7b.1).
fn references(board: &Scenario) -> Vec<(At, Among, &NamedCard)> {
    let mut found = Vec::new();
    for (i, line) in board.cards.iter().enumerate() {
        match line.value.kind {
            LineKind::Battlefield => {
                for (w, word) in line.value.words.iter().enumerate() {
                    if let CardWord::AttachedTo(card) | CardWord::Blocking(card) | CardWord::Attacking(Attacked::Permanent(card)) = word {
                        found.push((At::Word { line: i, word: w }, Among::Permanents, card));
                    }
                }
            }
            LineKind::Counters | LineKind::ThisTurn => found.push((At::Line(i), Among::Permanents, &line.value.card)),
            _ => {}
        }
    }
    for (i, word) in board.player_words.iter().enumerate() {
        if let PlayerWord::CommanderDamage { from, .. } = &word.value {
            found.push((At::CommanderDamage(i), Among::Commanders, from));
        }
    }
    found
}

/// The one card line `reference` names among `among`'s, a line of one copy.
fn named_line(cards: &[LineNumbered<CardLine>], reference: &NamedCard, among: Among) -> Option<usize> {
    let mut named = cards.iter().enumerate().filter(|(_, line)| is_among(&line.value, among) && names(reference, &line.value.card));
    match (named.next(), named.next()) {
        (Some((i, line)), None) if line.value.copies == 1 => Some(i),
        _ => None,
    }
}

/// Does an advanced setting show `word` as a control? A counter of a kind
/// in `PLAYER_COUNTERS`, lands played, leaving, and commander damage naming
/// one commander on the board.
fn has_control(cards: &[LineNumbered<CardLine>], word: &PlayerWord) -> bool {
    match word {
        PlayerWord::Counter { kind, .. } => PLAYER_COUNTERS.contains(kind),
        PlayerWord::LandsPlayed { .. } | PlayerWord::LeftTheGame { .. } => true,
        PlayerWord::CommanderDamage { from, .. } => named_line(cards, from, Among::Commanders).is_some(),
        PlayerWord::Life { .. } | PlayerWord::History { .. } => false,
    }
}

/// `player`'s words that `is` matches replaced by `word` at the first one's
/// place, or beside the player's other words; or gone, for `None`.
fn set_player_word(words: &mut Vec<LineNumbered<PlayerWord>>, player: PlayerId, is: impl Fn(&PlayerWord) -> bool, word: Option<PlayerWord>) {
    let at = words.iter().position(|w| is(&w.value));
    words.retain(|w| !is(&w.value));
    let Some(word) = word else { return };
    let seat = |w: &LineNumbered<PlayerWord>| player_of(&w.value);
    let beside = words.iter().rposition(|w| seat(w) == player).map(|i| i + 1);
    let before_later = words.iter().position(|w| seat(w) > player);
    let at = at.or(beside).or(before_later).unwrap_or(words.len()).min(words.len());
    words.insert(at, LineNumbered { line: 0, value: word });
}

/// A board being edited, and where the card being edited is as lines move.
struct Draft {
    board: Scenario,
    edited: Option<usize>,
}

impl Draft {
    fn line(&self, i: usize) -> &CardLine {
        &self.board.cards[i].value
    }

    fn line_mut(&mut self, i: usize) -> &mut CardLine {
        &mut self.board.cards[i].value
    }

    /// The edit `input` asks for; `chosen` is the name chosen in the search.
    fn apply(&mut self, input: EditorInput, chosen: Option<&str>) {
        match input {
            EditorInput::Number(field, value) => self.set_number(field, value),
            EditorInput::Seed(seed) => self.board.seed = seed,
            EditorInput::Active(player) => self.board.active = player,
            EditorInput::Step(step) => self.board.step = step,
            EditorInput::Put(seat, zone) => {
                if let Some(name) = chosen {
                    let words = zone.seat_word(seat).into_iter().collect();
                    let card = NamedCard { name: name.to_string(), tag: None };
                    self.arrive(CardLine { kind: zone.kind(seat, self.shuffled(seat)), card, copies: 1, words });
                }
            }
            EditorInput::Shuffled(seat, on) => {
                for line in &mut self.board.cards {
                    if let LineKind::Library { player, shuffled } = &mut line.value.kind
                        && *player == seat
                    {
                        *shuffled = on;
                    }
                }
            }
            EditorInput::LeftTheGame(player, on) => {
                let left = PlayerWord::LeftTheGame { player };
                set_player_word(&mut self.board.player_words, player, |word| *word == left, on.then(|| left.clone()));
            }
            EditorInput::RemoveText(item) => self.remove_text(item),
            EditorInput::Controller(i, player) => {
                let owned_by = owner(self.line(i)).unwrap_or(player);
                set_seats(self.line_mut(i), player, owned_by);
            }
            EditorInput::Owner(i, player) => self.set_owner(i, player),
            EditorInput::Flag(i, Flag::Commander, on) => self.set_commander(i, on),
            EditorInput::Flag(i, flag, on) => {
                let word = flag.word();
                set_word(&mut self.line_mut(i).words, |w| *w == word, on.then(|| word.clone()));
            }
            EditorInput::Arrival(i, arrival) => {
                set_word(&mut self.line_mut(i).words, |w| matches!(w, CardWord::Arrived(_)), arrival.map(CardWord::Arrived));
            }
            EditorInput::Counter(i, kind, count) => {
                let is_kind = |w: &CardWord| matches!(w, CardWord::Counter(k, _) if *k == kind);
                set_word(&mut self.line_mut(i).words, is_kind, count.map(|n| CardWord::Counter(kind, n)));
            }
            EditorInput::AttackPlayer(i, player) => {
                let attacking = Some(CardWord::Attacking(Attacked::Player(player)));
                set_word(&mut self.line_mut(i).words, |w| matches!(w, CardWord::Attacking(_)), attacking);
            }
            EditorInput::Unreference(i, reference) => {
                self.line_mut(i).words.retain(|word| !matches!(
                    (reference, word),
                    (Reference::AttachedTo, CardWord::AttachedTo(_))
                        | (Reference::Attacking, CardWord::Attacking(_))
                        | (Reference::Blocking, CardWord::Blocking(_))
                ));
            }
            EditorInput::Zone(i, zone) => self.move_to(i, zone),
            EditorInput::Move(i, direction) => self.step(i, direction),
            EditorInput::Remove(i) => self.remove(i),
            // The selection's, the search's, the typed field's and the session's.
            EditorInput::Card(_)
            | EditorInput::Close
            | EditorInput::Pick(_)
            | EditorInput::Undo
            | EditorInput::Search(_)
            | EditorInput::Choose(_)
            | EditorInput::Advanced(_)
            | EditorInput::TypedLine(_)
            | EditorInput::AddTypedLine
            | EditorInput::Play
            | EditorInput::Save
            | EditorInput::Open(_) => {}
        }
    }

    fn set_number(&mut self, field: BoardNumber, value: i64) {
        let count = |value: i64| u32::try_from(value).unwrap_or(0);
        match field {
            BoardNumber::Players => self.board.players = usize::try_from(value).unwrap_or(2),
            BoardNumber::StartingLife => self.board.starting_life = value,
            BoardNumber::Turn => self.board.turn = count(value),
            BoardNumber::Life(player) => {
                let life = (value != self.board.starting_life).then_some(PlayerWord::Life { player, life: value });
                set_player_word(&mut self.board.player_words, player, |w| matches!(w, PlayerWord::Life { player: p, .. } if *p == player), life);
            }
            BoardNumber::LandsPlayed(player) => {
                let lands = (value > 0).then(|| PlayerWord::LandsPlayed { player, count: count(value) });
                set_player_word(&mut self.board.player_words, player, |w| matches!(w, PlayerWord::LandsPlayed { player: p, .. } if *p == player), lands);
            }
            BoardNumber::PlayerCounter(player, kind) => {
                let counter = (value > 0).then(|| PlayerWord::Counter { player, kind, count: count(value) });
                let is_kind = |w: &PlayerWord| matches!(w, PlayerWord::Counter { player: p, kind: k, .. } if *p == player && *k == kind);
                set_player_word(&mut self.board.player_words, player, is_kind, counter);
            }
            BoardNumber::CommanderDamage { player, from } => {
                let Some(commander) = self.board.cards.get(from).filter(|line| is_among(&line.value, Among::Commanders)) else { return };
                let damage = (value > 0).then(|| PlayerWord::CommanderDamage { player, damage: count(value), from: commander.value.card.clone() });
                let cards = &self.board.cards;
                let is_from = |w: &PlayerWord| {
                    matches!(w, PlayerWord::CommanderDamage { player: p, from: named, .. } if *p == player && named_line(cards, named, Among::Commanders) == Some(from))
                };
                set_player_word(&mut self.board.player_words, player, is_from, damage);
            }
            BoardNumber::Damage(i) => {
                let damage = (value > 0).then(|| CardWord::Damage(count(value)));
                set_word(&mut self.line_mut(i).words, |w| matches!(w, CardWord::Damage(_)), damage);
            }
            BoardNumber::Copies(i) => self.line_mut(i).copies = count(value).max(1),
            BoardNumber::ArrivedTurn(i) => {
                let arrived = Some(CardWord::Arrived(Arrival::Turn(count(value))));
                set_word(&mut self.line_mut(i).words, |w| matches!(w, CardWord::Arrived(_)), arrived);
            }
        }
    }

    fn remove_text(&mut self, item: TextItem) {
        match item {
            TextItem::PlayerWord(i) if i < self.board.player_words.len() => {
                self.board.player_words.remove(i);
            }
            TextItem::Line(i) if self.board.cards.get(i).is_some_and(|line| listed(&line.value).is_none()) => self.remove_line(i),
            TextItem::SetupAction(i) if i < self.board.setup_actions.len() => {
                self.board.setup_actions.remove(i);
            }
            _ => {}
        }
    }

    /// Whether `seat`'s library is shuffled: its lines share one answer.
    fn shuffled(&self, seat: PlayerId) -> bool {
        self.board.cards.iter().any(|line| line.value.kind == LineKind::Library { player: seat, shuffled: true })
    }

    fn set_owner(&mut self, i: usize, player: PlayerId) {
        let shuffled = self.shuffled(player);
        let line = self.line_mut(i);
        match line.kind {
            LineKind::Hand(_) => line.kind = LineKind::Hand(player),
            LineKind::Library { .. } => line.kind = LineKind::Library { player, shuffled },
            LineKind::Graveyard(_) => line.kind = LineKind::Graveyard(player),
            LineKind::Battlefield => {
                let controlled_by = controller(line).unwrap_or(player);
                set_seats(line, controlled_by, player);
            }
            LineKind::Exile | LineKind::Command => {
                set_word(&mut line.words, |w| matches!(w, CardWord::Owner(_)), Some(CardWord::Owner(player)));
            }
            LineKind::Counters | LineKind::ThisTurn => {}
        }
    }

    /// A commander sharing its name with another is tagged, and so is the
    /// other; one no longer a commander takes the commander damage that
    /// named only it with it.
    fn set_commander(&mut self, i: usize, on: bool) {
        let card = self.line(i).card.clone();
        if on {
            let shared = self.count(&card.name, Among::Commanders) > 0;
            if shared {
                self.tag_all(&card.name, Among::Commanders);
            }
            set_word(&mut self.line_mut(i).words, |w| *w == CardWord::Commander, Some(CardWord::Commander));
            if shared && card.tag.is_none() && self.line(i).copies == 1 {
                self.line_mut(i).card.tag = Some(self.unused_tag(&card.name));
            }
        } else {
            set_word(&mut self.line_mut(i).words, |w| *w == CardWord::Commander, None);
            self.drop_references(&card, Among::Commanders);
        }
    }

    /// `line` put last, the card edited next. A permanent sharing its name
    /// with another takes the first unused tag, and so does the other if it
    /// has none, with every reference to it (§7b.1).
    fn arrive(&mut self, mut line: CardLine) {
        if line.kind == LineKind::Battlefield && self.count(&line.card.name, Among::Permanents) > 0 {
            self.tag_all(&line.card.name, Among::Permanents);
            if line.card.tag.is_none() && line.copies == 1 {
                line.card.tag = Some(self.unused_tag(&line.card.name));
            }
        }
        self.board.cards.push(LineNumbered { line: 0, value: line });
        self.edited = Some(self.board.cards.len() - 1);
    }

    /// Card `i` into its owner's `zone`, or onto the battlefield under its
    /// owner; one copy of a line of several. It keeps only the words that
    /// zone's line has, and a card leaving the battlefield takes the
    /// references that named only it with it (§7b.1).
    fn move_to(&mut self, i: usize, zone: Zone) {
        let Some((_, from)) = listed(self.line(i)) else { return };
        if from == zone {
            return;
        }
        let seat = owner(self.line(i)).unwrap_or(0);
        let mut card = self.line(i).clone();
        if card.copies > 1 {
            self.line_mut(i).copies -= 1;
            card.copies = 1;
        } else {
            self.remove_line(i);
        }
        let commander = card.words.contains(&CardWord::Commander).then_some(CardWord::Commander);
        let words = zone.seat_word(seat).into_iter().chain(commander).collect();
        let named = card.card.clone();
        self.arrive(CardLine { kind: zone.kind(seat, self.shuffled(seat)), words, ..card });
        if from == Zone::Battlefield {
            self.drop_references(&named, Among::Permanents);
        }
    }

    fn remove(&mut self, i: usize) {
        let line = self.line(i).clone();
        self.remove_line(i);
        if line.kind == LineKind::Battlefield {
            self.drop_references(&line.card, Among::Permanents);
        }
        if line.words.contains(&CardWord::Commander) {
            self.drop_references(&line.card, Among::Commanders);
        }
    }

    /// Card `i` to the top or bottom of its zone, or past the next card in it.
    fn step(&mut self, i: usize, direction: Direction) {
        if let Some(to) = move_target(&self.board, i, direction) {
            let mut order: Vec<usize> = (0..self.board.cards.len()).filter(|&k| k != i).collect();
            order.insert(to, i);
            self.reorder(order);
        }
    }

    /// Card `i`'s word naming another card names card `target` now: one
    /// such word a line, since it takes the rest of the line, so it replaces
    /// any other. An Aura attached to a permanent listed below it moves to
    /// just below it: the loader needs the host first, and CR 613.7e gives an
    /// attached Aura or Equipment a new timestamp anyway.
    fn set_reference(&mut self, i: usize, reference: Reference, target: usize) {
        if target == i || listed_at(&self.board, target).map(|(_, zone)| zone) != Some(Zone::Battlefield) {
            return;
        }
        let name = self.line(target).card.name.clone();
        if self.count(&name, Among::Permanents) > 1 {
            self.tag_all(&name, Among::Permanents);
        }
        let card = self.line(target).card.clone();
        let words = &mut self.line_mut(i).words;
        words.retain(|w| !(w.names_a_card() || (reference == Reference::Attacking && matches!(w, CardWord::Attacking(_)))));
        words.push(match reference {
            Reference::AttachedTo => CardWord::AttachedTo(card),
            Reference::Attacking => CardWord::Attacking(Attacked::Permanent(card)),
            Reference::Blocking => CardWord::Blocking(card),
        });
        if reference == Reference::AttachedTo && target > i {
            self.move_below(i, target);
        }
    }

    /// Card `i` just below `host`, with each line between them that needs it
    /// above: one attached to it, or its `counters:` line.
    fn move_below(&mut self, i: usize, host: usize) {
        let mut moving = vec![i];
        for k in i + 1..host {
            let line = self.line(k);
            let needs = |reference: &NamedCard| moving.iter().any(|&m| names(reference, &self.line(m).card));
            let depends = match line.kind {
                LineKind::Counters => needs(&line.card),
                LineKind::Battlefield => line.words.iter().any(|w| matches!(w, CardWord::AttachedTo(named) if needs(named))),
                _ => false,
            };
            if depends {
                moving.push(k);
            }
        }
        let mut order: Vec<usize> = (0..self.board.cards.len()).filter(|k| !moving.contains(k)).collect();
        let at = order.iter().position(|&k| k == host).map_or(order.len(), |place| place + 1);
        order.splice(at..at, moving);
        self.reorder(order);
    }

    /// The lines in `order`, each named by its old place; the card being
    /// edited followed to its new one.
    fn reorder(&mut self, order: Vec<usize>) {
        let mut old: Vec<Option<LineNumbered<CardLine>>> = std::mem::take(&mut self.board.cards).into_iter().map(Some).collect();
        self.edited = self.edited.and_then(|e| order.iter().position(|&k| k == e));
        self.board.cards = order.into_iter().filter_map(|k| old.get_mut(k).and_then(Option::take)).collect();
    }

    fn remove_line(&mut self, i: usize) {
        self.board.cards.remove(i);
        self.edited = self.edited.filter(|&e| e != i).map(|e| if e > i { e - 1 } else { e });
    }

    /// How many cards of `name` are among `among`'s.
    fn count(&self, name: &str, among: Among) -> u32 {
        let lines = self.board.cards.iter().map(|line| &line.value);
        lines.filter(|line| is_among(line, among) && line.card.name == name).map(|line| line.copies).sum()
    }

    /// Each card of `name` among `among`'s with no tag takes the first unused
    /// one, and so does each reference that named only it. A tag never
    /// changes once given, so a reference keeps naming its card.
    fn tag_all(&mut self, name: &str, among: Among) {
        for i in 0..self.board.cards.len() {
            let line = self.line(i);
            if !is_among(line, among) || line.card.name != name || line.card.tag.is_some() || line.copies != 1 {
                continue;
            }
            let tag = self.unused_tag(name);
            let named_only_it: Vec<At> = references(&self.board)
                .into_iter()
                .filter(|(_, kind, reference)| reference.tag.is_none() && named_line(&self.board.cards, reference, *kind) == Some(i))
                .map(|(at, ..)| at)
                .collect();
            for at in named_only_it {
                if let Some(reference) = self.reference_mut(at) {
                    reference.tag = Some(tag.clone());
                }
            }
            self.line_mut(i).card.tag = Some(tag);
        }
    }

    /// The first tag in the writer's spelling that no card or reference of
    /// `name` holds, setup actions' included.
    fn unused_tag(&self, name: &str) -> String {
        let board = &self.board;
        let actions = board.setup_actions.iter().flat_map(|action| {
            let targets = action.value.targets.iter().filter_map(|target| match target {
                Targeted::Card(card) => Some(card),
                Targeted::Player(_) => None,
            });
            std::iter::once(&action.value.card).chain(targets)
        });
        let cards = board.cards.iter().map(|line| &line.value.card);
        let held: Vec<&NamedCard> = cards.chain(references(board).into_iter().map(|(.., card)| card)).chain(actions).collect();
        let used: Vec<&str> = held.into_iter().filter(|card| card.name == name).filter_map(|card| card.tag.as_deref()).collect();
        (0..).map(tag_letters).find(|tag| !used.contains(&tag.as_str())).unwrap_or_default()
    }

    fn reference_mut(&mut self, at: At) -> Option<&mut NamedCard> {
        match at {
            At::Word { line, word } => match self.board.cards.get_mut(line)?.value.words.get_mut(word)? {
                CardWord::AttachedTo(card) | CardWord::Blocking(card) | CardWord::Attacking(Attacked::Permanent(card)) => Some(card),
                _ => None,
            },
            At::Line(line) => Some(&mut self.board.cards.get_mut(line)?.value.card),
            At::CommanderDamage(i) => match &mut self.board.player_words.get_mut(i)?.value {
                PlayerWord::CommanderDamage { from, .. } => Some(from),
                _ => None,
            },
        }
    }

    /// Drop each reference that named `card` among `among`'s and names
    /// nothing now that `card` is not one of them.
    fn drop_references(&mut self, card: &NamedCard, among: Among) {
        let still_names = |reference: &NamedCard| self.board.cards.iter().any(|line| is_among(&line.value, among) && names(reference, &line.value.card));
        let gone: Vec<At> = references(&self.board)
            .into_iter()
            .filter(|(_, kind, reference)| *kind == among && names(reference, card) && !still_names(reference))
            .map(|(at, ..)| at)
            .collect();
        // From the last, so each place still holds what it named.
        for at in gone.into_iter().rev() {
            match at {
                At::Word { line, word } => {
                    self.line_mut(line).words.remove(word);
                }
                At::Line(line) => self.remove_line(line),
                At::CommanderDamage(i) => {
                    self.board.player_words.remove(i);
                }
            }
        }
    }
}

fn player_of(word: &PlayerWord) -> PlayerId {
    match *word {
        PlayerWord::Life { player, .. }
        | PlayerWord::Counter { player, .. }
        | PlayerWord::LandsPlayed { player, .. }
        | PlayerWord::LeftTheGame { player }
        | PlayerWord::CommanderDamage { player, .. }
        | PlayerWord::History { player, .. } => player,
    }
}

// The view.

/// A button: a click on it sends `input`, and changes something while `live`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditButton {
    pub label: String,
    pub input: EditorInput,
    pub live: bool,
    /// Drawn selected: a word the card has, the current choice.
    pub on: bool,
}

/// A number between "−" and "+", each a button of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stepper {
    /// Empty where its row's label says it.
    pub label: String,
    /// The number as shown: a counter kind the line does not state is `—`.
    pub value: String,
    pub lower: Option<EditorInput>,
    pub raise: Option<EditorInput>,
    /// A field to type it into, beside the buttons.
    pub typed: Option<Typed>,
}

/// A number typed into a field, in `min..=max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Typed {
    pub field: BoardNumber,
    pub value: i64,
    pub min: i64,
    pub max: i64,
}

fn stepper(label: &str, field: BoardNumber, value: i64, (min, max): (i64, i64), typed: bool) -> Stepper {
    let to = |value: Option<i64>| value.filter(|v| (min..=max).contains(v)).map(|v| EditorInput::Number(field, v));
    Stepper {
        label: label.to_string(),
        value: value.to_string(),
        lower: to(value.checked_sub(1)),
        raise: to(value.checked_add(1)),
        typed: typed.then_some(Typed { field, value, min, max }),
    }
}

/// A card's line, as its button shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardButton {
    /// Its name and tag; on the battlefield, after its place in the order
    /// of arrival.
    pub title: String,
    /// Its words as the file spells them, but the one its place already says.
    pub detail: String,
    pub input: EditorInput,
    pub live: bool,
    pub edited: bool,
    /// The loader's refusal names its line.
    pub refused: bool,
}

/// A word or line shown as its text, and the input that removes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLine {
    pub text: String,
    pub remove: EditorInput,
    pub refused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneEdit {
    pub zone: Zone,
    pub title: String,
    /// "+": the name chosen in the search, put last here.
    pub put: EditButton,
    /// A library's: shuffled from the seed, or listed top first.
    pub shuffled: Option<EditButton>,
    pub cards: Vec<CardButton>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatEdit {
    pub seat: PlayerId,
    pub title: String,
    /// The loader's refusal names a line of the player's that a control
    /// shows, rather than its text.
    pub refused: bool,
    pub life: Stepper,
    /// The player's other words, as text.
    pub words: Vec<TextLine>,
    /// The advanced settings' controls for the player's words, while they
    /// show.
    pub rows: Vec<ControlRow>,
    pub zones: Vec<ZoneEdit>,
}

/// One row of the card's controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlRow {
    pub label: &'static str,
    /// What the row says now, where its buttons do not: the card a
    /// reference names.
    pub note: Option<String>,
    pub buttons: Vec<EditButton>,
    pub steppers: Vec<Stepper>,
}

fn row(label: &'static str, buttons: Vec<EditButton>) -> ControlRow {
    ControlRow { label, note: None, buttons, steppers: Vec::new() }
}

/// The card being edited: its line, and its words as controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardEdit {
    pub text: String,
    pub rows: Vec<ControlRow>,
}

/// The typed field: a line of the file, added to the board as an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedLineView<'e> {
    pub line: &'e str,
    /// Live while the field holds a line not yet refused as it stands.
    pub add: EditButton,
    /// Why the line was not added.
    pub refusal: Option<&'e str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchView<'e> {
    pub query: &'e str,
    /// Each name holding the query, a click choosing it.
    pub results: Vec<EditButton>,
    pub chosen: Option<&'e str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorView<'e> {
    /// Where the board came from, and where it saves.
    pub source: String,
    pub comments: &'e [String],
    pub text: &'e str,
    pub facts: Vec<Stepper>,
    pub seed: u64,
    pub active: Vec<EditButton>,
    pub steps: Vec<EditButton>,
    /// The other seats in seat order, then seat 0, as the game shows them.
    pub seats: Vec<SeatEdit>,
    /// The `counters:` and `this turn:` lines and the setup actions.
    pub texts: Vec<TextLine>,
    pub card: Option<CardEdit>,
    /// The loader's refusal, or `None` while it builds the board.
    pub refusal: Option<String>,
    /// Why the last edit was not made.
    pub unsaid: Option<&'e str>,
    pub undo: EditButton,
    pub play: EditButton,
    pub save: EditButton,
    /// The switch that shows the advanced settings.
    pub advanced: EditButton,
    /// While they show.
    pub typed_line: Option<TypedLineView<'e>>,
    pub search: SearchView<'e>,
}

impl Editor {
    pub fn view(&self) -> EditorView<'_> {
        let board = &self.board;
        let refused = |line: usize| self.refusal.as_ref().and_then(|refusal| refusal.line) == Some(line);
        let seated = board.cards.iter().filter_map(|line| listed(&line.value)).map(|(seat, _)| seat);
        let shown = seated.chain(board.player_words.iter().map(|word| player_of(&word.value))).map(|seat| seat + 1).fold(board.players, usize::max);
        let mut order: Vec<PlayerId> = (1..shown).collect();
        order.push(0);
        let button = |label: String, input: EditorInput, on: bool| EditButton { label, input, live: !on, on };
        let texts = board.cards.iter().enumerate().filter(|(_, line)| listed(&line.value).is_none()).map(|(i, line)| TextLine {
            text: line.value.to_string(),
            remove: EditorInput::RemoveText(TextItem::Line(i)),
            refused: refused(line.line),
        });
        let actions = board.setup_actions.iter().enumerate().map(|(i, action)| TextLine {
            text: action.value.to_string(),
            remove: EditorInput::RemoveText(TextItem::SetupAction(i)),
            refused: refused(action.line),
        });
        let action = |label: &str, input: EditorInput, live: bool| EditButton { label: label.to_string(), input, live, on: false };
        EditorView {
            source: self.source_words(),
            comments: &self.comments,
            text: &self.text,
            facts: vec![
                stepper("Players", BoardNumber::Players, board.players as i64, (2, MOST_PLAYERS as i64), false),
                stepper("Starting life", BoardNumber::StartingLife, board.starting_life, (-LIFE_LIMIT, LIFE_LIMIT), true),
                stepper("Turn", BoardNumber::Turn, i64::from(board.turn), (1, TURN_LIMIT), true),
            ],
            seed: board.seed,
            active: (0..board.players).map(|p| button(format!("Player {p}"), EditorInput::Active(p), p == board.active)).collect(),
            steps: turn_positions().map(|step| button(position_word(step), EditorInput::Step(step), step == board.step)).collect(),
            seats: order.into_iter().map(|seat| self.seat_edit(seat, &refused)).collect(),
            texts: texts.chain(actions).collect(),
            card: self.editing.and_then(|i| self.card_edit(i)),
            refusal: self.refusal.as_ref().map(|refusal| refusal.message.clone()),
            unsaid: self.unsaid.as_deref(),
            undo: action("Undo", EditorInput::Undo, !self.undo.is_empty()),
            play: action("Play this board", EditorInput::Play, self.refusal.is_none()),
            save: action("Save", EditorInput::Save, true),
            advanced: EditButton { label: "Advanced".to_string(), input: EditorInput::Advanced(!self.advanced), live: true, on: self.advanced },
            typed_line: self.advanced.then(|| TypedLineView {
                line: &self.typed_line,
                add: action("Add", EditorInput::AddTypedLine, !self.typed_line.trim().is_empty() && self.typed_line_refusal.is_none()),
                refusal: self.typed_line_refusal.as_deref(),
            }),
            search: SearchView {
                query: self.search.query(),
                results: self.search.matches().iter().map(|&i| self.search_result(i)).collect(),
                chosen: self.chosen.and_then(|i| self.search.name(i)),
            },
        }
    }

    fn source_words(&self) -> String {
        let first_save = "named at its first Save or Play";
        match &self.source {
            Source::Empty => format!("A new board, {first_save}"),
            Source::Game { start, turn } => format!("The board {start}'s game was at on turn {turn}, {first_save}"),
            Source::File(path) => format!("{}: Save makes a board of its own in boards/, never writing this file", path.display()),
            Source::Board(path) => path.display().to_string(),
        }
    }

    fn search_result(&self, i: usize) -> EditButton {
        let name = self.search.name(i).unwrap_or_default();
        let label = if i < self.registered { name.to_string() } else { format!("{name} (in development)") };
        EditButton { label, input: EditorInput::Choose(i), live: self.chosen != Some(i), on: self.chosen == Some(i) }
    }

    fn seat_edit(&self, seat: PlayerId, refused: &dyn Fn(usize) -> bool) -> SeatEdit {
        let board = &self.board;
        // The loader reads a player's life words in order, so the last stands.
        let life = board.player_words.iter().rev().find_map(|word| match word.value {
            PlayerWord::Life { player, life } if player == seat => Some(life),
            _ => None,
        });
        let as_control = |word: &PlayerWord| matches!(word, PlayerWord::Life { .. }) || (self.advanced && has_control(&board.cards, word));
        let (controlled, words): (Vec<_>, Vec<_>) =
            board.player_words.iter().enumerate().filter(|(_, word)| player_of(&word.value) == seat).partition(|(_, word)| as_control(&word.value));
        let title = if seat < board.players {
            format!("Player {seat}")
        } else {
            format!("Player {seat}, not in this {}-player game", board.players)
        };
        SeatEdit {
            seat,
            title,
            refused: controlled.iter().any(|(_, word)| refused(word.line)),
            life: stepper("Life", BoardNumber::Life(seat), life.unwrap_or(board.starting_life), (-LIFE_LIMIT, LIFE_LIMIT), true),
            words: words
                .into_iter()
                .map(|(i, word)| TextLine { text: word.value.to_string(), remove: EditorInput::RemoveText(TextItem::PlayerWord(i)), refused: refused(word.line) })
                .collect(),
            rows: if self.advanced { self.player_rows(seat) } else { Vec::new() },
            zones: Zone::ALL.into_iter().map(|zone| self.zone_edit(seat, zone, refused)).collect(),
        }
    }

    /// The advanced settings' rows for `seat`'s words, each control showing
    /// what the loader makes of them: it sets lands played and commander
    /// damage, so the last word stands, and adds counters.
    fn player_rows(&self, seat: PlayerId) -> Vec<ControlRow> {
        let board = &self.board;
        let words = || board.player_words.iter().map(|word| &word.value).filter(move |word| player_of(word) == seat);
        let left = words().any(|word| *word == PlayerWord::LeftTheGame { player: seat });
        let left = EditButton { label: "left the game".to_string(), input: EditorInput::LeftTheGame(seat, !left), live: true, on: left };
        let lands = words().rev().find_map(|word| if let PlayerWord::LandsPlayed { count, .. } = word { Some(*count) } else { None });
        let mut counts = vec![stepper("lands played", BoardNumber::LandsPlayed(seat), lands.unwrap_or(0).into(), (0, COUNT_LIMIT), true)];
        for kind in PLAYER_COUNTERS {
            let of_kind = words().filter_map(|word| match word {
                PlayerWord::Counter { kind: k, count, .. } if *k == kind => Some(*count),
                _ => None,
            });
            counts.push(stepper(kind.name(), BoardNumber::PlayerCounter(seat, kind), of_kind.fold(0, u32::saturating_add).into(), (0, COUNT_LIMIT), true));
        }
        let mut rows = vec![ControlRow { steppers: counts, ..row("", vec![left]) }];
        let commanders = board.cards.iter().enumerate().filter(|(_, line)| is_among(&line.value, Among::Commanders) && line.value.copies == 1);
        let damage: Vec<Stepper> = commanders
            .map(|(from, line)| {
                let taken = words().rev().find_map(|word| match word {
                    PlayerWord::CommanderDamage { damage, from: named, .. } if named_line(&board.cards, named, Among::Commanders) == Some(from) => Some(*damage),
                    _ => None,
                });
                let field = BoardNumber::CommanderDamage { player: seat, from };
                stepper(&format!("from {}", line.value.card), field, taken.unwrap_or(0).into(), (0, COUNT_LIMIT), true)
            })
            .collect();
        if !damage.is_empty() {
            rows.push(ControlRow { steppers: damage, ..row("Commander damage", Vec::new()) });
        }
        rows
    }

    fn zone_edit(&self, seat: PlayerId, zone: Zone, refused: &dyn Fn(usize) -> bool) -> ZoneEdit {
        let board = &self.board;
        let here: Vec<usize> = (0..board.cards.len()).filter(|&i| listed_at(board, i) == Some((seat, zone))).collect();
        let count: u32 = here.iter().map(|&i| board.cards[i].value.copies).sum();
        let title = match zone {
            Zone::Battlefield => format!("Battlefield ({count}), by arrival"),
            Zone::Library => format!("Library ({count}), top first"),
            Zone::Graveyard => format!("Graveyard ({count}), bottom first"),
            _ => format!("{} ({count})", zone.name()),
        };
        let library = here.first().map(|&i| board.cards[i].value.kind);
        let shuffled = match library {
            Some(LineKind::Library { shuffled, .. }) if zone == Zone::Library => {
                Some(EditButton { label: "shuffled".to_string(), input: EditorInput::Shuffled(seat, !shuffled), live: true, on: shuffled })
            }
            _ => None,
        };
        let put = EditButton { label: "+".to_string(), input: EditorInput::Put(seat, zone), live: self.chosen.is_some(), on: false };
        let cards = here.into_iter().map(|i| self.card_button(i, seat, zone, refused)).collect();
        ZoneEdit { zone, title, put, shuffled, cards }
    }

    fn card_button(&self, i: usize, seat: PlayerId, zone: Zone, refused: &dyn Fn(usize) -> bool) -> CardButton {
        let line = &self.board.cards[i];
        let card = &line.value;
        let placed_by = |word: &CardWord| match zone {
            Zone::Battlefield => matches!(word, CardWord::Controller(_)) || *word == CardWord::Owner(seat),
            Zone::Exile | Zone::Command => matches!(word, CardWord::Owner(_)),
            Zone::Hand | Zone::Library | Zone::Graveyard => false,
        };
        let (references, plain): (Vec<&CardWord>, Vec<&CardWord>) = card.words.iter().filter(|w| !placed_by(w)).partition(|w| w.names_a_card());
        let copies = (card.copies > 1).then(|| format!("x{}", card.copies));
        let detail: Vec<String> = copies.into_iter().chain(plain.into_iter().chain(references).map(CardWord::to_string)).collect();
        let title = match zone {
            Zone::Battlefield => {
                let place = self.board.cards[..i].iter().filter(|other| other.value.kind == LineKind::Battlefield).count() + 1;
                format!("{place}. {}", card.card)
            }
            _ => card.card.to_string(),
        };
        let edited = self.editing == Some(i);
        let live = match self.picking {
            Some(_) => zone == Zone::Battlefield && !edited,
            None => !edited,
        };
        CardButton { title, detail: detail.join(" · "), input: EditorInput::Card(i), live, edited, refused: refused(line.line) }
    }

    fn card_edit(&self, i: usize) -> Option<CardEdit> {
        let board = &self.board;
        let line = &board.cards.get(i)?.value;
        let (_, zone) = listed(line)?;
        let seats = |current: Option<PlayerId>, input: fn(usize, PlayerId) -> EditorInput| -> Vec<EditButton> {
            (0..board.players)
                .map(|p| EditButton { label: format!("Player {p}"), input: input(i, p), live: current != Some(p), on: current == Some(p) })
                .collect()
        };
        let has = |word: &CardWord| line.words.contains(word);
        let toggle = |flag: Flag, label: &str| {
            let on = has(&flag.word());
            EditButton { label: label.to_string(), input: EditorInput::Flag(i, flag, !on), live: true, on }
        };
        let mut rows = Vec::new();
        if zone == Zone::Battlefield {
            rows.push(row("Controller", seats(controller(line), EditorInput::Controller)));
        }
        rows.push(row("Owner", seats(owner(line), EditorInput::Owner)));
        if zone == Zone::Battlefield {
            rows.push(row(
                "Words",
                vec![
                    toggle(Flag::Tapped, "tapped"),
                    toggle(Flag::Commander, "commander"),
                    toggle(Flag::Blocked, "blocked"),
                    toggle(Flag::DealtFirstStrikeDamage, "dealt first-strike damage"),
                ],
            ));
            rows.extend(self.permanent_rows(i, line));
        } else {
            rows.push(row("Words", vec![toggle(Flag::Commander, "commander")]));
            let mut copies = stepper("", BoardNumber::Copies(i), i64::from(line.copies), (1, i64::from(u32::MAX)), false);
            // A tag names one card, so a tagged line is one (§5.1's `xN`).
            if line.card.tag.is_some() {
                copies.raise = None;
            }
            rows.push(ControlRow { steppers: vec![copies], ..row("Copies", Vec::new()) });
        }
        let zones = Zone::ALL.into_iter().map(|z| EditButton { label: z.name().to_string(), input: EditorInput::Zone(i, z), live: z != zone, on: z == zone });
        rows.push(row("Zone", zones.collect()));
        let moves = [("top", Direction::Top), ("up", Direction::Up), ("down", Direction::Down), ("bottom", Direction::Bottom)];
        let moves = moves.map(|(label, direction)| {
            let live = move_target(board, i, direction).is_some();
            EditButton { label: label.to_string(), input: EditorInput::Move(i, direction), live, on: false }
        });
        rows.push(row("Order", moves.into()));
        let done = [("Remove", EditorInput::Remove(i)), ("Close", EditorInput::Close)];
        rows.push(row("", done.map(|(label, input)| EditButton { label: label.to_string(), input, live: true, on: false }).into()));
        Some(CardEdit { text: line.to_string(), rows })
    }

    /// The rows only a permanent has: arrival, counters, damage, and the
    /// words that name another card, each set by a click on that card.
    fn permanent_rows(&self, i: usize, line: &CardLine) -> Vec<ControlRow> {
        let turn = self.board.turn;
        let arrived = line.words.iter().find_map(|w| if let CardWord::Arrived(a) = w { Some(*a) } else { None });
        let earlier = match arrived {
            Some(Arrival::Turn(n)) => n,
            _ => turn.saturating_sub(1).max(1),
        };
        let arrivals = [("before the game", None), ("this turn", Some(Arrival::ThisTurn)), ("an earlier turn", Some(Arrival::Turn(earlier)))];
        let arrivals = arrivals.map(|(label, to)| {
            let on = match (to, arrived) {
                (Some(Arrival::Turn(_)), Some(Arrival::Turn(_))) => true,
                (to, arrived) => to == arrived,
            };
            EditButton { label: label.to_string(), input: EditorInput::Arrival(i, to), live: !on, on }
        });
        let mut arrival = row("Arrived", arrivals.into());
        if let Some(Arrival::Turn(n)) = arrived {
            arrival.steppers.push(stepper("turn", BoardNumber::ArrivedTurn(i), i64::from(n), (1, i64::from(turn.max(1))), false));
        }
        let counters = CounterType::ALL.into_iter().map(|kind| {
            let count = line.words.iter().find_map(|w| match w {
                CardWord::Counter(k, n) if *k == kind => Some(*n),
                _ => None,
            });
            Stepper {
                label: kind.name().to_string(),
                value: count.map_or("—".to_string(), |n| n.to_string()),
                lower: count.map(|n| EditorInput::Counter(i, kind, n.checked_sub(1))),
                raise: Some(EditorInput::Counter(i, kind, Some(count.map_or(1, |n| n.saturating_add(1))))),
                typed: None,
            }
        });
        let damage = line.words.iter().find_map(|w| if let CardWord::Damage(n) = w { Some(*n) } else { None }).unwrap_or(0);
        let mut rows = vec![
            arrival,
            ControlRow { steppers: counters.collect(), ..row("Counters", Vec::new()) },
            ControlRow { steppers: vec![stepper("", BoardNumber::Damage(i), i64::from(damage), (0, i64::from(u32::MAX)), false)], ..row("Damage", Vec::new()) },
        ];
        for (label, reference) in [("Attached to", Reference::AttachedTo), ("Attacking", Reference::Attacking), ("Blocking", Reference::Blocking)] {
            let named = line.words.iter().find_map(|w| match (reference, w) {
                (Reference::AttachedTo, CardWord::AttachedTo(card)) | (Reference::Blocking, CardWord::Blocking(card)) => Some(card.to_string()),
                (Reference::Attacking, CardWord::Attacking(Attacked::Permanent(card))) => Some(card.to_string()),
                (Reference::Attacking, CardWord::Attacking(Attacked::Player(p))) => Some(format!("player {p}")),
                _ => None,
            });
            let mut buttons = Vec::new();
            if reference == Reference::Attacking {
                let attacked = |p: PlayerId| line.words.contains(&CardWord::Attacking(Attacked::Player(p)));
                let players = (0..self.board.players).map(|p| EditButton { label: format!("player {p}"), input: EditorInput::AttackPlayer(i, p), live: !attacked(p), on: attacked(p) });
                buttons.extend(players);
            }
            let picking = self.picking == Some(reference);
            let pick = match picking {
                true => "click a permanent, or here to stop",
                false if reference == Reference::Attacking => "a planeswalker or battle…",
                false => "a permanent…",
            };
            buttons.push(EditButton { label: pick.to_string(), input: EditorInput::Pick((!picking).then_some(reference)), live: true, on: picking });
            buttons.push(EditButton { label: "none".to_string(), input: EditorInput::Unreference(i, reference), live: named.is_some(), on: false });
            rows.push(ControlRow { note: Some(named.unwrap_or_else(|| "nothing".to_string())), ..row(label, buttons) });
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtgsim::objects::card_data::CardDataBuilder;

    fn opened(text: &str) -> Editor {
        Editor::open(text, Source::Empty, CardRegistry::default_registry()).unwrap_or_else(|refusal| panic!("{refusal}"))
    }

    /// The card line holding `name` with `tag`.
    fn line_of(editor: &Editor, name: &str) -> usize {
        let named = |line: &&LineNumbered<CardLine>| line.value.card.to_string() == name;
        editor.board().cards.iter().position(|line| named(&line)).unwrap_or_else(|| panic!("{name} is not on the board:\n{}", editor.text()))
    }

    fn choose(editor: &mut Editor, name: &str) {
        editor.input(EditorInput::Search(name.to_string()));
        let i = editor.search().matches().iter().copied().find(|&i| editor.search().name(i) == Some(name)).unwrap();
        editor.input(EditorInput::Choose(i));
    }

    fn put(editor: &mut Editor, name: &str, seat: PlayerId, zone: Zone) -> usize {
        choose(editor, name);
        editor.input(EditorInput::Put(seat, zone));
        editor.editing.unwrap()
    }

    fn seat(editor: &Editor, seat: PlayerId) -> SeatEdit {
        editor.view().seats.into_iter().find(|shown| shown.seat == seat).unwrap()
    }

    /// The advanced settings' count labeled `label` on `seat`.
    fn count(editor: &Editor, on: PlayerId, label: &str) -> Stepper {
        let steppers = seat(editor, on).rows.into_iter().flat_map(|row| row.steppers);
        steppers.into_iter().find(|stepper| stepper.label == label).unwrap_or_else(|| panic!("player {on} has no {label}"))
    }

    fn typed(editor: &Editor) -> (String, bool, Option<String>) {
        let typed = editor.view().typed_line.unwrap();
        (typed.line.to_string(), typed.add.live, typed.refusal.map(str::to_string))
    }

    /// A click edits a copy, renumbered through its text: the file's comment
    /// lines are kept for the save but number nothing, and undo walks back.
    #[test]
    fn a_click_edits_a_copy_renumbered_through_its_text() {
        let mut editor = opened("# Two bears\n\nturn 3\n\n# the board\nbattlefield: Grizzly Bears | controller 0\n");
        assert_eq!(editor.comments, ["# Two bears"]);
        assert_eq!(editor.board().cards[0].line, 2, "the Bears' line in the board's own text");
        editor.input(EditorInput::Flag(0, Flag::Tapped, true));
        assert_eq!(editor.text(), "turn 3\nbattlefield: Grizzly Bears | controller 0, tapped\n");
        assert_eq!(editor.file_text(), "# Two bears\n\nturn 3\nbattlefield: Grizzly Bears | controller 0, tapped\n");
        editor.input(EditorInput::Flag(0, Flag::Tapped, true));
        assert!(editor.view().undo.live, "the click that changed nothing pushed nothing");
        editor.input(EditorInput::Undo);
        assert!(!editor.view().undo.live);
        assert_eq!(editor.text(), "turn 3\nbattlefield: Grizzly Bears | controller 0\n");
    }

    /// The loader's refusal is the check, and it marks the card it names.
    #[test]
    fn the_loaders_refusal_marks_the_card_it_names() {
        let mut editor = opened("turn 3\nbattlefield: Grizzly Bears | controller 0");
        assert!(editor.refusal().is_none() && editor.view().play.live);
        editor.input(EditorInput::AttackPlayer(0, 1));
        let view = editor.view();
        assert!(view.refusal.as_deref().unwrap().contains("attacking in the precombat main phase"), "{:?}", view.refusal);
        assert!(!view.play.live, "Play waits for a board the loader accepts");
        let seat = view.seats.iter().find(|seat| seat.seat == 0).unwrap();
        assert!(seat.zones[0].cards[0].refused);
        let combat = turn_positions().find(|step| position_word(*step) == "declare attackers").unwrap();
        editor.input(EditorInput::Step(combat));
        editor.input(EditorInput::Number(BoardNumber::Life(1), 18));
        assert_eq!(editor.refusal().map(|r| r.message.as_str()), None, "{}", editor.text());
    }

    /// §7b.1's tags: a permanent sharing a name is tagged, and so is the one
    /// it shares it with, with the words naming it; a setup action is left as
    /// written, so the loader refuses it as ambiguous, naming its line.
    #[test]
    fn a_shared_name_is_tagged_with_every_reference_to_the_first() {
        let mut editor = opened(
            "turn 3\nhand 0: Lightning Bolt\nbattlefield: Mountain | controller 0\n\
             battlefield: Grizzly Bears | controller 1\nbattlefield: Holy Strength | controller 1, attached to Grizzly Bears\n\
             counters: Grizzly Bears | flying 1\nthen: player 0 casts Lightning Bolt | targeting Grizzly Bears",
        );
        assert!(editor.refusal().is_none(), "{:?}", editor.refusal());
        put(&mut editor, "Grizzly Bears", 0, Zone::Battlefield);
        let text = editor.text().to_string();
        for line in [
            "battlefield: Grizzly Bears [a] | controller 1\n",
            "battlefield: Holy Strength | controller 1, attached to Grizzly Bears [a]\n",
            "counters: Grizzly Bears [a] | flying 1\n",
            "then: player 0 casts Lightning Bolt | targeting Grizzly Bears\n",
            "battlefield: Grizzly Bears [b] | controller 0\n",
        ] {
            assert!(text.contains(line), "{line}in\n{text}");
        }
        let refusal = editor.refusal().unwrap();
        assert!(refusal.message.contains("names two objects"), "{refusal}");
        assert!(editor.view().texts.iter().any(|text| text.refused && text.text.starts_with("then:")));
    }

    /// Attaching to a permanent listed below moves the Aura just below its
    /// host, and the line that needs the Aura above it moves with it.
    #[test]
    fn attaching_moves_the_aura_below_its_host() {
        let mut editor = opened(
            "battlefield: Holy Strength | controller 0\ncounters: Holy Strength | shield 1\n\
             battlefield: Glorious Anthem | controller 0\nbattlefield: Grizzly Bears | controller 0",
        );
        editor.input(EditorInput::Card(0));
        editor.input(EditorInput::Pick(Some(Reference::AttachedTo)));
        editor.input(EditorInput::Card(3));
        let lines: Vec<String> = editor.board().cards.iter().map(|line| line.value.to_string()).collect();
        assert_eq!(
            lines,
            [
                "battlefield: Glorious Anthem | controller 0",
                "battlefield: Grizzly Bears | controller 0",
                "battlefield: Holy Strength | controller 0, attached to Grizzly Bears",
                "counters: Holy Strength | shield 1",
            ]
        );
        assert_eq!(editor.editing, Some(2), "the Aura is still the card edited");
        assert!(editor.refusal().is_none(), "{:?}", editor.refusal());
    }

    /// §7b.1's example: the Bears, tapped and attacking with Holy Strength on
    /// them, moved to the hand, lose what only a permanent has; the Aura's
    /// word naming them goes, and so does their `counters:` line. Undo brings
    /// all of it back.
    #[test]
    fn a_card_leaving_the_battlefield_takes_the_words_that_no_longer_fit() {
        let before = "turn 3\nstep declare attackers\nbattlefield: Grizzly Bears | controller 0, tapped, +1/+1 1, attacking player 1\n\
                      battlefield: Holy Strength | controller 0, attached to Grizzly Bears\ncounters: Grizzly Bears | flying 1\n";
        let mut editor = opened(before);
        editor.input(EditorInput::Zone(0, Zone::Hand));
        assert_eq!(
            editor.text(),
            "turn 3\nstep declare attackers\nbattlefield: Holy Strength | controller 0\nhand 0: Grizzly Bears\n"
        );
        assert_eq!(editor.editing, Some(1), "the Bears, in the hand");
        editor.input(EditorInput::Undo);
        assert_eq!(editor.text(), before);
    }

    /// The battlefield is one zone in CR 613.7d's order, whoever controls
    /// each permanent, so Humility can arrive before Opalescence; one copy
    /// of a line of several moves at a time.
    #[test]
    fn a_card_moves_in_its_zone_and_one_copy_moves_out_of_it() {
        let mut editor = opened("library 0: Forest | x3\nbattlefield: Opalescence | controller 0\nbattlefield: Humility | controller 1");
        editor.input(EditorInput::Move(2, Direction::Up));
        assert_eq!(line_of(&editor, "Humility"), 1);
        let seat_1 = editor.view().seats.into_iter().find(|seat| seat.seat == 1).unwrap();
        assert_eq!(seat_1.zones[0].cards[0].title, "1. Humility");
        editor.input(EditorInput::Zone(0, Zone::Battlefield));
        assert!(editor.text().starts_with("library 0: Forest | x2\n"), "{}", editor.text());
        assert!(editor.text().ends_with("battlefield: Forest | controller 0\n"));
    }

    /// A seat's controller and owner: the owner is said where it differs,
    /// and a hand card's owner is the hand it is in.
    #[test]
    fn a_permanents_controller_changes_and_its_owner_stays() {
        let mut editor = opened("players 3\nhand 2: Grizzly Bears\nbattlefield: Wall of Stone | controller 0");
        editor.input(EditorInput::Controller(1, 1));
        editor.input(EditorInput::Owner(0, 1));
        assert_eq!(editor.text(), "players 3\nhand 1: Grizzly Bears\nbattlefield: Wall of Stone | controller 1, owner 0\n");
        editor.input(EditorInput::Owner(1, 1));
        assert!(editor.text().ends_with("battlefield: Wall of Stone | controller 1\n"));
    }

    /// Commander damage names a commander, and goes when its commander is no
    /// longer one; a second commander of one name is tagged.
    #[test]
    fn commander_damage_follows_its_commander() {
        let mut editor = opened("players 3\nplayer 1: commander damage 5 from Isamaru, Hound of Konda\ncommand: Isamaru, Hound of Konda | owner 0, commander");
        let second = put(&mut editor, "Isamaru, Hound of Konda", 2, Zone::Command);
        editor.input(EditorInput::Flag(second, Flag::Commander, true));
        assert!(editor.text().contains("player 1: commander damage 5 from Isamaru, Hound of Konda [a]\n"), "{}", editor.text());
        assert!(editor.text().ends_with("command: Isamaru, Hound of Konda [b] | owner 2, commander\n"));
        editor.input(EditorInput::Flag(0, Flag::Commander, false));
        assert!(!editor.text().contains("commander damage"), "{}", editor.text());
    }

    /// Off, a player's words but life are text. On, each the advanced
    /// settings have a control for leaves the text, and the typed field shows.
    #[test]
    fn the_advanced_switch_shows_controls_in_place_of_the_text() {
        let mut editor = opened("players 3\nplayer 1: left the game, poison 2, flying 1\nplayer 1 this turn: spells cast 1");
        let texts = |editor: &Editor| seat(editor, 1).words.into_iter().map(|word| word.text).collect::<Vec<_>>();
        assert_eq!(texts(&editor).len(), 4);
        assert!(seat(&editor, 1).rows.is_empty() && editor.view().typed_line.is_none());
        let switch = editor.view().advanced.input;
        editor.input(switch);
        assert_eq!(texts(&editor), ["player 1: flying 1", "player 1 this turn: spells cast 1"]);
        assert!(seat(&editor, 1).rows[0].buttons[0].on, "left the game");
        assert_eq!(count(&editor, 1, "poison").value, "2");
        assert!(editor.view().typed_line.is_some());
    }

    /// A typed line goes last in the board's text and is made an edit, which
    /// Undo takes back. One the parser refuses, or that says nothing new,
    /// leaves the board and stays in the field with why, until it changes.
    #[test]
    fn a_typed_line_is_added_as_an_edit_and_a_refused_one_stays_in_the_field() {
        let mut editor = opened("turn 3\nbattlefield: Grizzly Bears | controller 0");
        editor.input(EditorInput::Advanced(true));
        assert_eq!(typed(&editor), (String::new(), false, None), "nothing to add yet");
        editor.input(EditorInput::TypedLine("player 1: poison 3".to_string()));
        editor.input(EditorInput::AddTypedLine);
        let with_poison = "turn 3\nplayer 1: poison 3\nbattlefield: Grizzly Bears | controller 0\n";
        assert_eq!((editor.text(), typed(&editor)), (with_poison, (String::new(), false, None)));
        for (line, says) in [
            ("player 1: poison three", "`poison three` is not a word a player line has"),
            ("turn 4", "`turn` is stated twice"),
            ("# a comment", "the line says nothing the board does not"),
        ] {
            editor.input(EditorInput::TypedLine(line.to_string()));
            editor.input(EditorInput::AddTypedLine);
            let (kept, live, refusal) = typed(&editor);
            assert!(kept == line && !live && refusal.as_deref().is_some_and(|why| why.contains(says)), "{line}: {refusal:?}");
            assert_eq!(editor.text(), with_poison);
        }
        editor.input(EditorInput::TypedLine("# a comment, typed again".to_string()));
        assert_eq!(typed(&editor).2, None, "a changed line is not refused yet");
        editor.input(EditorInput::Undo);
        assert_eq!(editor.text(), "turn 3\nbattlefield: Grizzly Bears | controller 0\n");
    }

    /// The loader adds a player's counters, so the count is their sum; a
    /// click leaves one word of the kind, and zero none.
    #[test]
    fn a_player_counter_counts_what_the_loader_adds() {
        let mut editor = opened("player 1: poison 2, poison 1\nplayer 1: energy 4, flying 1");
        editor.input(EditorInput::Advanced(true));
        assert_eq!((count(&editor, 1, "poison").value, count(&editor, 1, "energy").value), ("3".to_string(), "4".to_string()));
        editor.input(count(&editor, 1, "poison").raise.unwrap());
        assert_eq!(editor.text(), "player 1: poison 4, energy 4, flying 1\n");
        editor.input(EditorInput::Number(count(&editor, 1, "energy").typed.unwrap().field, 0));
        assert_eq!(editor.text(), "player 1: poison 4, flying 1\n");
        // A typed word that sums past what a count holds is checked on the
        // window's thread, and the loader keeps the most.
        editor.input(EditorInput::TypedLine("player 1: poison 4294967295".to_string()));
        editor.input(EditorInput::AddTypedLine);
        assert_eq!((count(&editor, 1, "poison").value, editor.refusal()), ("4294967295".to_string(), None));
    }

    /// The loader sets lands played, so the last word stands; zero is where
    /// a turn starts, and says nothing.
    #[test]
    fn lands_played_counts_the_last_word_and_zero_says_nothing() {
        let mut editor = opened("player 0: life 14, lands played 2, lands played 1");
        editor.input(EditorInput::Advanced(true));
        assert_eq!(count(&editor, 0, "lands played").value, "1");
        editor.input(count(&editor, 0, "lands played").lower.unwrap());
        assert_eq!(editor.text(), "player 0: life 14\n");
        editor.input(count(&editor, 1, "lands played").raise.unwrap());
        assert_eq!(editor.text(), "player 0: life 14\nplayer 1: lands played 1\n");
    }

    /// Leaving the game is a toggle; the loader's refusal of it marks the
    /// player, since no text shows the word.
    #[test]
    fn leaving_the_game_is_a_toggle_and_its_refusal_marks_the_player() {
        let mut editor = opened("players 3\nhand 0: Forest");
        editor.input(EditorInput::Advanced(true));
        let left = |editor: &Editor, on: PlayerId| seat(editor, on).rows[0].buttons[0].clone();
        editor.input(left(&editor, 2).input);
        assert_eq!(editor.text(), "players 3\nplayer 2: left the game\nhand 0: Forest\n");
        assert!(left(&editor, 2).on && editor.refusal().is_none(), "{:?}", editor.refusal());
        editor.input(left(&editor, 0).input);
        assert!(editor.refusal().is_some_and(|refusal| refusal.message.contains("player 0 has left")), "{:?}", editor.refusal());
        assert!(seat(&editor, 0).refused && !seat(&editor, 2).refused);
        editor.input(left(&editor, 0).input);
        assert_eq!(editor.text(), "players 3\nplayer 2: left the game\nhand 0: Forest\n");
    }

    /// Each player has a count for each commander on the board, read from
    /// the word naming it, and a click writes its name and tag; a word naming
    /// no commander stays text.
    #[test]
    fn commander_damage_has_a_count_for_each_commander() {
        let mut editor = opened(
            "players 3\nplayer 1: commander damage 5 from Isamaru, Hound of Konda\nplayer 2: commander damage 3 from Grizzly Bears\n\
             command: Isamaru, Hound of Konda | owner 0, commander\nbattlefield: Thalia, Guardian of Thraben | controller 2, commander\n\
             battlefield: Grizzly Bears | controller 2",
        );
        editor.input(EditorInput::Advanced(true));
        assert_eq!(count(&editor, 1, "from Isamaru, Hound of Konda").value, "5");
        editor.input(EditorInput::Number(count(&editor, 1, "from Thalia, Guardian of Thraben").typed.unwrap().field, 21));
        let damage = "player 1: commander damage 5 from Isamaru, Hound of Konda\nplayer 1: commander damage 21 from Thalia, Guardian of Thraben\n";
        assert!(editor.text().contains(damage), "{}", editor.text());
        assert_eq!(seat(&editor, 2).words.into_iter().map(|word| word.text).collect::<Vec<_>>(), ["player 2: commander damage 3 from Grizzly Bears"]);
        let second = put(&mut editor, "Isamaru, Hound of Konda", 2, Zone::Command);
        editor.input(EditorInput::Flag(second, Flag::Commander, true));
        assert_eq!(count(&editor, 1, "from Isamaru, Hound of Konda [a]").value, "5", "the tag the word took names its commander");
        assert_eq!(count(&editor, 1, "from Isamaru, Hound of Konda [b]").value, "0");
    }

    /// The search lists every name a scenario can use, the cards in
    /// development after the registered ones and marked; a name whose text
    /// would not read back is refused with the parser's message.
    #[test]
    fn the_search_offers_the_cards_in_development_and_a_name_that_does_not_read_back_is_refused() {
        let mut registry = CardRegistry::default_registry();
        let card = || CardDataBuilder::new("Test # Bear").build();
        registry.register_in_development("Test # Bear", card);
        let mut editor = Editor::open("", Source::Empty, registry).unwrap();
        editor.input(EditorInput::Search("test #".to_string()));
        let results = editor.view().search.results;
        assert_eq!(results.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Test # Bear (in development)"]);
        editor.input(results[0].input.clone());
        editor.input(EditorInput::Put(0, Zone::Hand));
        assert!(editor.board().cards.is_empty());
        assert!(editor.view().unsaid.is_some_and(|why| why.contains("would not read back")));
    }
}
