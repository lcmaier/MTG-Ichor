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

use crate::bridge::{Outcome, ToWindow, WINDOW_SEAT};
use crate::prompt::{Answer, BoardRef, Primitive, Prompt};
use crate::snapshot::{CardView, PermanentView, PlayerView, Snapshot};

/// Something the player did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Clicked option `i` of the prompt.
    Option(usize),
    /// Clicked a card or a player on the board.
    Board(BoardRef),
    /// Set a `pick_number`'s number.
    Number(u64),
    /// One more (`true`) or one fewer into bucket `i` of an allocation.
    Adjust(usize, bool),
    /// The confirm button.
    Done,
    /// Start the answer over.
    Reset,
}

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
                    Input::Option(i) => Some(i),
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
        Input::Option(i) if i < prompt.options.len() => Some(i),
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
        Input::Adjust(bucket, more) => (bucket, more),
        Input::Option(bucket) => (bucket, true),
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
}

impl WindowState {
    pub fn receive(&mut self, message: ToWindow) {
        match message {
            ToWindow::Prompt { snapshot, prompt } => {
                self.log.extend(snapshot.log.iter().cloned());
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
        }
    }

    /// The answer to send, once `input` completes one. The prompt closes with
    /// it; the board stays until the engine's next message.
    pub fn input(&mut self, input: Input) -> Option<Answer> {
        let answer = self.selection.as_mut()?.apply(self.prompt.as_ref()?, input)?;
        self.prompt = None;
        self.selection = None;
        Some(answer)
    }

    /// What the window is doing, for the header.
    pub fn status(&self) -> String {
        if self.panic.is_some() {
            "The engine panicked".to_string()
        } else if let Some(outcome) = &self.outcome {
            match outcome {
                Outcome::Won(player) if *player == WINDOW_SEAT => "Game over: you win".to_string(),
                Outcome::Won(player) => format!("Game over: Player {player} wins"),
                Outcome::Draw => "Game over: a draw".to_string(),
                Outcome::Error(error) => format!("The engine returned an error: {error}"),
            }
        } else if self.prompt.is_some() {
            "Your decision".to_string()
        } else {
            "The engine is playing".to_string()
        }
    }

    pub fn board_view(&self) -> Option<BoardView> {
        let board = self.board.as_ref()?;
        Some(BoardView::new(board, &Marks::new(self.prompt.as_ref(), self.selection.as_ref())))
    }

    pub fn prompt_view(&self) -> Option<PromptView> {
        Some(PromptView::new(self.prompt.as_ref()?, self.selection.as_ref()?))
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
    /// Shown on hover; empty for none.
    pub hover: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneView {
    pub name: String,
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
    /// Opponents first and the window's seat last, nearest the prompt.
    pub seats: Vec<SeatView>,
    /// Top first.
    pub stack: Vec<Item>,
    pub pending_triggers: Vec<Item>,
    pub exile: Vec<Item>,
    pub command: Vec<Item>,
}

/// Which board things the open prompt makes clickable, which it has chosen,
/// and which it is about.
#[derive(Default)]
struct Marks {
    clickable: Vec<BoardRef>,
    chosen: Vec<BoardRef>,
    subject: Option<BoardRef>,
}

impl Marks {
    fn new(prompt: Option<&Prompt>, selection: Option<&Selection>) -> Marks {
        let (Some(prompt), Some(selection)) = (prompt, selection) else {
            return Marks::default();
        };
        let first = |i: &usize| prompt.options[*i].refs.first().copied();
        let all: Vec<usize> = (0..prompt.options.len()).collect();
        let (clickable, chosen) = match selection {
            Selection::Picks { chosen, half: Some(half) } => (
                prompt.options.iter().filter(|o| o.refs.first() == Some(half)).filter_map(|o| o.refs.get(1).copied()).collect(),
                chosen.iter().filter_map(first).chain([*half]).collect(),
            ),
            Selection::Picks { chosen, half: None } => {
                (all.iter().filter_map(first).collect(), chosen.iter().filter_map(first).collect())
            }
            Selection::Order(order) => {
                (all.iter().filter(|i| !order.contains(i)).filter_map(first).collect(), order.iter().filter_map(first).collect())
            }
            Selection::Allocation(_) => (all.iter().filter_map(first).collect(), Vec::new()),
            Selection::Number(_) => (Vec::new(), Vec::new()),
        };
        Marks { clickable, chosen, subject: prompt.subject.map(BoardRef::Object) }
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
        }
    }

    fn card(&self, card: &CardView) -> Item {
        let title = match &card.mana_cost {
            Some(cost) => format!("{} {cost}", card.name),
            None => card.name.clone(),
        };
        self.item(BoardRef::Object(card.id), title, card.type_line.clone(), false)
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
        if let Some(host) = permanent.attached_to {
            detail.push(format!("attached to {host}"));
        }
        let title = format!("{} ({})", permanent.card.name, permanent.card.id);
        let mut item = self.item(BoardRef::Object(permanent.card.id), title, detail.join(" · "), permanent.tapped);
        item.hover = permanent.engine_text.clone();
        item
    }

    fn player(&self, player: &PlayerView) -> Item {
        let you = if player.id == WINDOW_SEAT { " (you)" } else { "" };
        let mut detail = vec![format!("{} life", player.life)];
        if !player.mana_pool.is_empty() {
            let pool: Vec<String> = player.mana_pool.iter().map(|(symbol, n)| symbol.repeat(*n as usize)).collect();
            detail.push(format!("pool {}", pool.concat()));
        }
        detail.extend(player.counters.iter().map(|(kind, n)| format!("{kind} {n}")));
        if player.lost {
            detail.push("lost".to_string());
        }
        self.item(BoardRef::Player(player.id), format!("Player {}{you}", player.id), detail.join(" · "), false)
    }
}

impl BoardView {
    fn new(board: &Snapshot, marks: &Marks) -> BoardView {
        let mut seats: Vec<&PlayerView> = board.players.iter().filter(|p| p.id != WINDOW_SEAT).collect();
        seats.extend(board.players.iter().filter(|p| p.id == WINDOW_SEAT));
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
        .map(|(name, items)| ZoneView { name: name.to_string(), items, open: true })
        .collect();
    let counted = |name: &str, cards: &[CardView], open: bool| ZoneView {
        name: format!("{name} ({})", cards.len()),
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
    /// An allocation's amount in this bucket, and whether it can go down and up.
    pub amount: Option<(u64, bool, bool)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptView {
    pub question: String,
    /// What makes an answer complete.
    pub rule: String,
    pub options: Vec<OptionButton>,
    /// A number's range and its value now.
    pub number: Option<(u64, u64, u64)>,
    /// The confirm button, when the prompt has one, and whether it is live.
    pub done: Option<(String, bool)>,
    pub can_reset: bool,
}

impl PromptView {
    fn new(prompt: &Prompt, selection: &Selection) -> PromptView {
        let options = prompt.options.iter().enumerate();
        let button = |label: &str| OptionButton { label: label.to_string(), chosen: false, place: None, amount: None };
        let (rule, options, number, done): (String, Vec<OptionButton>, _, _) = match (&prompt.primitive, selection) {
            (Primitive::PickN { min, max }, Selection::Picks { chosen, half }) => {
                let rule = match half {
                    Some(half) => format!("now click what {} goes with", name_of(half)),
                    None => pick_rule(*min, *max),
                };
                // A lone option with a way out is a "may": yes or no.
                let may = *min == 0 && *max == 1 && prompt.options.len() == 1;
                let options = options
                    .map(|(i, o)| OptionButton {
                        chosen: chosen.contains(&i),
                        ..button(&if may { format!("Yes: {}", o.label) } else { o.label.clone() })
                    })
                    .collect();
                let done = match (*min, *max) {
                    (0, 1) => Some((if may { "No" } else { "Decline" }.to_string(), true)),
                    (_, 1) => None,
                    _ => Some(("Done".to_string(), (*min..=*max).contains(&chosen.len()))),
                };
                (rule, options, None, done)
            }
            (Primitive::Number { min, max }, Selection::Number(value)) => {
                let top = if *max == u64::MAX { "any".to_string() } else { max.to_string() };
                (format!("a number from {min} to {top}"), Vec::new(), Some((*min, *max, *value)), Some(("Done".to_string(), true)))
            }
            (Primitive::Allocate { total, mins, maxs }, Selection::Allocation(amounts)) => {
                let left = total.saturating_sub(amounts.iter().sum());
                let options = options
                    .map(|(i, o)| {
                        let cap = maxs.as_ref().map_or(u64::MAX, |maxs| maxs[i]);
                        OptionButton { amount: Some((amounts[i], amounts[i] > mins[i], left > 0 && amounts[i] < cap)), ..button(&o.label) }
                    })
                    .collect();
                (format!("divide {total}: {left} left"), options, None, Some(("Done".to_string(), left == 0)))
            }
            (Primitive::Order, Selection::Order(order)) => {
                let options = options
                    .map(|(i, o)| {
                        let place = order.iter().position(|placed| *placed == i).map(|at| at + 1);
                        OptionButton { chosen: place.is_some(), place, ..button(&o.label) }
                    })
                    .collect();
                let rule = format!("click them in order: {} of {} placed", order.len(), prompt.options.len());
                (rule, options, None, Some(("Done".to_string(), order.len() == prompt.options.len())))
            }
            _ => (String::new(), Vec::new(), None, None),
        };
        let can_reset = !matches!(prompt.primitive, Primitive::PickN { max: 1, .. } | Primitive::Number { .. });
        PromptView { question: prompt.question.clone(), rule, options, number, done, can_reset }
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

fn name_of(target: &BoardRef) -> String {
    match target {
        BoardRef::Object(id) => id.to_string(),
        BoardRef::Player(player) => format!("Player {player}"),
    }
}

#[cfg(test)]
mod tests {
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::test_support::{
        card_of_type, forest, lightning_bolt, put_in_hand, put_on_battlefield, set_attacking, setup_two_player_game,
        vanilla_creature,
    };
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::ids::ObjectId;

    use super::*;
    use crate::prompt::OptionView;
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
        Prompt { kind: "Test".to_string(), question: String::new(), subject: None, primitive, options }
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
    fn the_battlefield_groups_as_the_cli_does_and_a_land_creature_is_a_creature() {
        let b = board();
        let view = WindowState { board: Some(b.snapshot.clone()), ..WindowState::default() }.board_view().unwrap();
        let mine = view.seats.last().unwrap();
        assert_eq!(mine.player.target, Some(Player(WINDOW_SEAT)), "the window's seat is drawn last");
        let names: Vec<&str> = mine.zones.iter().map(|zone| zone.name.as_str()).collect();
        assert_eq!(names, ["Creatures", "Lands", "Other permanents", "Hand (1)", "Graveyard (0)", "Library (0)"]);
        let targets = |zone: &ZoneView| zone.items.iter().map(|item| item.target).collect::<Vec<_>>();
        assert_eq!(targets(&mine.zones[0]), [Some(Object(b.bear)), Some(Object(b.arbor))]);
        assert_eq!(targets(&mine.zones[1]), [Some(Object(b.forest))]);
        assert_eq!(targets(&mine.zones[2]), [Some(Object(b.relic))]);
        assert!(item(&view, b.bear).detail.starts_with("2/2"), "{}", item(&view, b.bear).detail);
        assert!(!item(&view, b.relic).detail.contains('/'), "no power or toughness off a creature");
        assert!(item(&view, b.their_bear).detail.contains("attacking Player 0"));
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
        assert_eq!(state.clone().input(Input::Option(0)), Some(Answer::Picks(vec![0])));
        let mut clicked = state.clone();
        assert_eq!(clicked.input(Input::Board(Object(b.bolt))), Some(Answer::Picks(vec![1])));
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
        assert!(state.prompt_view().unwrap().rule.starts_with("now click"));
        state.input(Input::Board(Object(b.their_giant)));
        state.input(Input::Board(Object(b.arbor)));
        assert_eq!(state.input(Input::Done), Some(Answer::Picks(vec![1, 2])));
    }

    #[test]
    fn a_lone_option_with_a_way_out_reads_as_yes_or_no() {
        let b = board();
        let state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, vec![option("Blood Artist", Vec::new())]));
        let view = state.prompt_view().unwrap();
        assert_eq!(view.options[0].label, "Yes: Blood Artist");
        assert_eq!(view.done, Some(("No".to_string(), true)));
        assert_eq!(state.clone().input(Input::Done), Some(Answer::Picks(Vec::new())));
    }

    #[test]
    fn done_is_live_only_inside_the_bounds() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 2, max: 3 }, unnamed(4)));
        state.input(Input::Option(0));
        assert_eq!(state.prompt_view().unwrap().done, Some(("Done".to_string(), false)));
        assert_eq!(state.input(Input::Done), None);
        for i in [3, 1, 2] {
            state.input(Input::Option(i));
        }
        assert_eq!(state.prompt_view().unwrap().done, Some(("Done".to_string(), true)));
        state.input(Input::Option(3));
        assert_eq!(
            state.input(Input::Done),
            Some(Answer::Picks(vec![0, 1])),
            "a pick past the maximum is refused, and a second click undoes one"
        );
    }

    #[test]
    fn an_allocation_stays_inside_each_bucket_and_is_done_when_the_total_is_placed() {
        let b = board();
        let buckets = vec![option("bear", vec![Object(b.their_bear)]), option("player", vec![Player(1)])];
        let split = Primitive::Allocate { total: 3, mins: vec![1, 0], maxs: Some(vec![2, 3]) };
        let mut state = deciding(&b, prompt(split, buckets));
        state.input(Input::Adjust(0, false));
        for _ in 0..3 {
            state.input(Input::Adjust(0, true));
        }
        let amount = state.prompt_view().unwrap().options[0].amount;
        assert_eq!(amount, Some((2, true, false)), "held between its minimum and its maximum");
        assert_eq!(state.input(Input::Done), None, "one still to place");
        state.input(Input::Board(Player(1)));
        assert_eq!(state.input(Input::Done), Some(Answer::Allocation(vec![2, 1])));
    }

    #[test]
    fn an_ordering_is_the_click_order_and_starts_over_on_reset() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Order, unnamed(3)));
        for i in [2, 0, 2] {
            state.input(Input::Option(i));
        }
        assert_eq!(state.prompt_view().unwrap().options[2].place, Some(1));
        assert_eq!(state.input(Input::Done), None);
        state.input(Input::Reset);
        for i in [1, 2, 0] {
            state.input(Input::Option(i));
        }
        assert_eq!(state.input(Input::Done), Some(Answer::Order(vec![1, 2, 0])));
    }

    #[test]
    fn a_number_is_held_to_its_range() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Number { min: 2, max: 5 }, Vec::new()));
        state.input(Input::Number(9));
        assert_eq!(state.input(Input::Done), Some(Answer::Number(5)));
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

    #[test]
    fn a_panic_closes_the_prompt_and_says_so() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        let message = "ask_choose_priority_action: DP returned index 9".to_string();
        state.receive(ToWindow::Panicked { message });
        assert!(state.prompt.is_none() && state.prompt_view().is_none());
        assert_eq!(state.status(), "The engine panicked");
        assert_eq!(state.input(Input::Option(0)), None);
    }
}
