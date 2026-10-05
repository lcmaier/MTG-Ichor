//! The egui drawing: lays out what `view_model` and `editor` built and hands
//! each click to the `Session`. It decides nothing, so it is reviewed by
//! running it.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;

use crate::boards::{Folders, ListedFile};
use crate::editor::{CardButton, CardEdit, EditButton, EditorInput, EditorView, SearchView, SeatEdit, Stepper, TextLine, Typed};
use crate::launch::Start;
use crate::session::Session;
use crate::prompt::BoardRef;
use crate::view_model::{
    Amount, BoardView, DIVERGED, Input, Item, KEYS, Key, Mode, NO_SAVESTATES, NumberField, PromptView, SeatButton,
    ToolButton, ToolsView, TypeLineView, TypeWordView, WhyView, WindowState, ZoneView,
};

pub struct DevGui {
    session: Session,
}

impl DevGui {
    pub fn new(start: Start, wake: Arc<dyn Fn() + Send + Sync>) -> DevGui {
        DevGui { session: Session::start(start, Folders::default(), wake) }
    }
}

impl eframe::App for DevGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.session.tick(ui.ctx().input(|input| input.time));
        self.session.receive();
        let session = &self.session;
        let header = SessionHeader {
            mode: session.mode,
            line: &session.start_line,
            log: session.log_path.as_deref(),
            playing: session.setup.is_some() || session.state.refused.is_some(),
            reloadable: session.setup.as_ref().is_some_and(|setup| setup.scenario.is_some()),
            message: session.message.as_ref().map(|message| message.as_ref().map(String::as_str).map_err(String::as_str)),
            files: &session.files,
            tools: session.setup.is_some().then(|| session.state.tools_view()),
        };
        let editor = (session.mode == Mode::Edit).then(|| session.editor.view());
        for input in draw(ui, &session.state, &header, editor.as_ref()) {
            self.session.input(input);
        }
    }
}

/// What the header says of the session, beside the board.
pub struct SessionHeader<'a> {
    pub mode: Mode,
    /// The game's seed and start.
    pub line: &'a str,
    /// The game's decision log.
    pub log: Option<&'a Path>,
    /// A game has started, or was refused: the header says which.
    pub playing: bool,
    /// A scenario's game, which Reload builds again from its file.
    pub reloadable: bool,
    /// The last save or open: where it went, or why it could not.
    pub message: Option<Result<&'a str, &'a str>>,
    /// The boards and scenarios the header's list opens.
    pub files: &'a [ListedFile],
    /// The game's tools, once a game has started.
    pub tools: Option<ToolsView>,
}

/// The whole window, the game's or the editor's; the inputs the player made
/// this frame.
pub fn draw(ui: &mut egui::Ui, state: &WindowState, header: &SessionHeader, editor: Option<&EditorView>) -> Vec<Input> {
    // A shortcut answers the game's prompt, so the editor takes none.
    let mut inputs = if editor.is_none() { keys(ui.ctx()) } else { Vec::new() };
    if let Some(left) = state.settling_for() {
        // The prompt's controls come back when the beat ends, mouse or no mouse.
        ui.ctx().request_repaint_after(Duration::from_secs_f64(left));
    }
    if state.replaying.is_some() {
        // The replay's count moves with no message to wake the window.
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
    let board = editor.is_none().then(|| state.board_view()).flatten();
    egui::Panel::top("header").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (mode, label) in [(Mode::Play, "Play"), (Mode::Edit, "Edit")] {
                if ui.selectable_label(header.mode == mode, label).clicked() && header.mode != mode {
                    inputs.push(Input::Mode(mode));
                }
            }
            ui.separator();
            match editor {
                Some(view) => editor_header(ui, view, &mut inputs),
                None => game_header(ui, state, board.as_ref(), header, &mut inputs),
            }
            ui.menu_button("Open…", |ui| {
                for (i, file) in header.files.iter().enumerate() {
                    if ui.button(&file.label).clicked() {
                        inputs.push(Input::Editor(EditorInput::Open(i)));
                        ui.close();
                    }
                }
            });
            match header.message {
                Some(Ok(done)) => {
                    ui.weak(done);
                }
                Some(Err(failed)) => {
                    ui.colored_label(ui.visuals().error_fg_color, failed);
                }
                None => {}
            }
        });
    });
    match editor {
        Some(view) => editor_panels(ui, view, &mut inputs),
        None => game_panels(ui, state, board.as_ref(), header, &mut inputs),
    }
    inputs
}

fn game_header(ui: &mut egui::Ui, state: &WindowState, board: Option<&BoardView>, header: &SessionHeader, inputs: &mut Vec<Input>) {
    if !header.playing {
        ui.weak("No game yet: the editor's Play starts its board.");
        return;
    }
    // The game's buttons first, where nothing before them changes width: a
    // replay's status and a board's line come and go, and a double click's
    // second half would land on whatever moved under it.
    if header.reloadable && ui.button("Reload").clicked() {
        inputs.push(Input::Reload);
    }
    if let Some(tools) = &header.tools {
        tool_button(ui, &tools.undo, inputs);
        tool_button(ui, &tools.savestate, inputs);
        ui.menu_button("Savestates", |ui| {
            if tools.menu.is_empty() {
                ui.weak(NO_SAVESTATES);
            }
            for entry in &tools.menu {
                if ui.add_enabled(entry.live, egui::Button::new(&entry.label)).clicked() {
                    inputs.push(entry.input.clone());
                    ui.close();
                }
            }
        });
        if let Some(why) = tools.undo_off {
            ui.weak(why);
        }
    }
    ui.separator();
    ui.strong(state.status());
    if let Some(board) = board {
        ui.separator();
        ui.label(&board.header);
    }
    ui.separator();
    match header.log {
        Some(log) => ui.weak(format!("{} · decision log {}", header.line, log.display())),
        None => ui.weak(header.line),
    };
    if let Some(failed) = &state.unwritten {
        ui.colored_label(ui.visuals().error_fg_color, failed);
    }
    let mut full_control = state.full_control;
    if ui.checkbox(&mut full_control, "Full control").changed() {
        inputs.push(Input::FullControl(full_control));
    }
    if state.board.is_some() {
        if ui.button("Save board as scenario").clicked() {
            inputs.push(Input::SaveBoard);
        }
        if ui.button("Edit this board").clicked() {
            inputs.push(Input::EditThisBoard);
        }
    }
    if header.reloadable && ui.button("Edit the scenario").clicked() {
        inputs.push(Input::EditTheScenario);
    }
}

fn game_panels(ui: &mut egui::Ui, state: &WindowState, board: Option<&BoardView>, header: &SessionHeader, inputs: &mut Vec<Input>) {
    egui::Panel::bottom("prompt").show(ui, |ui| {
        if let Some((refusal, message)) = &state.refused {
            ui.colored_label(ui.visuals().error_fg_color, format!("{}:", refusal.heading()));
            ui.monospace(message);
            ui.weak(refusal.hint());
        } else if let Some(message) = &state.panic {
            ui.colored_label(ui.visuals().error_fg_color, "The engine thread panicked:");
            ui.monospace(message);
            if let Some(log) = header.log {
                ui.weak(format!("The decision log {} holds this game's seed and every answer up to here: attach it to the report.", log.display()));
            }
            if header.reloadable {
                ui.weak("Reload starts the scenario again.");
            }
        } else if let Some(prompt) = state.prompt_view() {
            if let Some(message) = &state.diverged {
                ui.colored_label(ui.visuals().warn_fg_color, DIVERGED[0]);
                ui.monospace(message);
                ui.weak(DIVERGED[1]);
                ui.separator();
            }
            ui.push_id(prompt.serial, |ui| prompt_panel(ui, &prompt, inputs));
        } else if header.playing {
            ui.weak(state.status());
        }
    });
    egui::Panel::right("side").default_size(380.0).show(ui, |ui| {
        if let Some(board) = board {
            side_panel(ui, board, inputs);
        }
        ui.separator();
        ui.strong("Log");
        let row_height = ui.text_style_height(&egui::TextStyle::Body);
        egui::ScrollArea::vertical().stick_to_bottom(true).show_rows(ui, row_height, state.log.len(), |ui, rows| {
            // One text line a row, as `show_rows` counts them; the whole line on hover.
            for line in &state.log[rows] {
                ui.add(egui::Label::new(line).truncate());
            }
        });
    });
    if let Some(why) = state.why_view() {
        egui::Panel::left("why").default_size(400.0).show(ui, |ui| why_panel(ui, &why, inputs));
    }
    egui::CentralPanel::default().show(ui, |ui| {
        let Some(board) = board else {
            ui.weak(if header.playing { state.no_board() } else { "No game yet." });
            return;
        };
        egui::ScrollArea::vertical().show(ui, |ui| {
            for seat in &board.seats {
                // A collapsing header's id is its label, and every seat has a "Creatures".
                ui.push_id(seat.player.target, |ui| {
                    item(ui, &seat.player, inputs);
                    for zone in &seat.zones {
                        zone_view(ui, zone, inputs);
                    }
                });
                ui.separator();
            }
        });
    });
}

fn editor_header(ui: &mut egui::Ui, view: &EditorView, inputs: &mut Vec<Input>) {
    ui.weak(&view.source);
    for button in [&view.undo, &view.play, &view.save] {
        edit_button(ui, button, inputs);
    }
    if ui.button("Copy as text").clicked() {
        ui.ctx().copy_text(view.text.to_string());
    }
    if let Some(why) = view.unsaid {
        ui.colored_label(ui.visuals().warn_fg_color, why);
    }
}

/// The board editor: the names to search on the right, the loader's verdict
/// and the card being edited below, and the board between them.
fn editor_panels(ui: &mut egui::Ui, view: &EditorView, inputs: &mut Vec<Input>) {
    egui::Panel::right("search").default_size(300.0).show(ui, |ui| search_panel(ui, &view.search, inputs));
    egui::Panel::bottom("card").default_size(330.0).show(ui, |ui| {
        match &view.refusal {
            Some(refusal) => {
                ui.colored_label(ui.visuals().error_fg_color, "The loader refuses this board:");
                ui.monospace(refusal);
                ui.weak("What it names is outlined. Play waits for a board the loader accepts; Save keeps this one.");
            }
            None => {
                ui.weak("The loader accepts this board.");
            }
        }
        ui.separator();
        match &view.card {
            Some(card) => card_panel(ui, card, inputs),
            None => {
                ui.weak("Click a card to edit its words. Choose a name in the search, and a zone's + puts it there.");
            }
        }
    });
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("board").show(ui, |ui| {
            if !view.comments.is_empty() {
                egui::CollapsingHeader::new("The file's comments, kept on every save").id_salt("comments").show(ui, |ui| {
                    for line in view.comments {
                        ui.monospace(line);
                    }
                });
            }
            facts(ui, view, inputs);
            ui.separator();
            for seat in &view.seats {
                ui.push_id(("seat", seat.seat), |ui| seat_edit(ui, seat, inputs));
                ui.separator();
            }
            if !view.texts.is_empty() {
                ui.strong("Shown as text: kept on every save, changed in the file");
                for text in &view.texts {
                    text_line(ui, text, inputs);
                }
            }
        });
    });
}

fn facts(ui: &mut egui::Ui, view: &EditorView, inputs: &mut Vec<Input>) {
    ui.horizontal_wrapped(|ui| {
        for stepper in &view.facts {
            stepper_ui(ui, stepper, inputs);
            ui.separator();
        }
        ui.label("Seed");
        let mut seed = view.seed;
        ui.push_id("seed", |ui| ui.add(egui::DragValue::new(&mut seed)));
        if seed != view.seed {
            inputs.push(Input::Editor(EditorInput::Seed(seed)));
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Active");
        for button in &view.active {
            edit_button(ui, button, inputs);
        }
        ui.separator();
        ui.label("Step");
        let current = view.steps.iter().find(|step| step.on).map_or("", |step| step.label.as_str());
        egui::ComboBox::from_id_salt("step").width(170.0).selected_text(current).show_ui(ui, |ui| {
            for step in &view.steps {
                if ui.selectable_label(step.on, &step.label).clicked() && step.live {
                    inputs.push(Input::Editor(step.input.clone()));
                }
            }
        });
    });
}

fn seat_edit(ui: &mut egui::Ui, seat: &SeatEdit, inputs: &mut Vec<Input>) {
    ui.horizontal_wrapped(|ui| {
        ui.strong(&seat.title);
        stepper_ui(ui, &seat.life, inputs);
        for word in &seat.words {
            text_line(ui, word, inputs);
        }
    });
    for zone in &seat.zones {
        ui.horizontal_wrapped(|ui| {
            ui.label(&zone.title);
            edit_button(ui, &zone.put, inputs);
            if let Some(shuffled) = &zone.shuffled {
                edit_button(ui, shuffled, inputs);
            }
            for card in &zone.cards {
                card_button(ui, card, inputs);
            }
        });
    }
}

/// A card's line: outlined when the loader's refusal names it, filled while
/// it is edited.
fn card_button(ui: &mut egui::Ui, card: &CardButton, inputs: &mut Vec<Input>) {
    let text = if card.detail.is_empty() { card.title.clone() } else { format!("{}\n{}", card.title, card.detail) };
    let visuals = ui.visuals();
    let stroke = if card.refused {
        egui::Stroke::new(2.5, visuals.error_fg_color)
    } else if card.live {
        egui::Stroke::new(1.5, visuals.selection.stroke.color)
    } else {
        egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
    };
    let sense = if card.live { egui::Sense::click() } else { egui::Sense::hover() };
    if ui.add(egui::Button::new(text).selected(card.edited).stroke(stroke).sense(sense)).clicked() && card.live {
        inputs.push(Input::Editor(card.input.clone()));
    }
}

fn card_panel(ui: &mut egui::Ui, card: &CardEdit, inputs: &mut Vec<Input>) {
    ui.monospace(&card.text);
    egui::ScrollArea::vertical().id_salt("card").show(ui, |ui| {
        for row in &card.rows {
            ui.horizontal_wrapped(|ui| {
                if !row.label.is_empty() {
                    ui.strong(row.label);
                }
                if let Some(note) = &row.note {
                    ui.label(note);
                }
                for button in &row.buttons {
                    edit_button(ui, button, inputs);
                }
                if let [stepper] = row.steppers.as_slice() {
                    stepper_ui(ui, stepper, inputs);
                }
            });
            // The counter kinds: a grid, so each stays whole.
            if row.steppers.len() > 1 {
                egui::Grid::new(row.label).show(ui, |ui| {
                    for (i, stepper) in row.steppers.iter().enumerate() {
                        ui.horizontal(|ui| stepper_ui(ui, stepper, inputs));
                        if i % 6 == 5 {
                            ui.end_row();
                        }
                    }
                });
            }
        }
    });
}

fn search_panel(ui: &mut egui::Ui, search: &SearchView, inputs: &mut Vec<Input>) {
    ui.strong("Search");
    let mut query = search.query.to_string();
    ui.add(egui::TextEdit::singleline(&mut query).id_salt("search").hint_text("part of a card's name"));
    if query != search.query {
        inputs.push(Input::Editor(EditorInput::Search(query)));
    }
    match search.chosen {
        Some(name) => ui.label(format!("{name}: a zone's + puts it there")),
        None => ui.weak("Choose a name; a zone's + puts it there."),
    };
    // One button a row, as `show_rows` counts them; a long name truncated.
    let row_height = ui.spacing().interact_size.y;
    egui::ScrollArea::vertical().id_salt("results").show_rows(ui, row_height, search.results.len(), |ui, rows| {
        for result in &search.results[rows] {
            if ui.add(egui::Button::selectable(result.on, result.label.as_str()).truncate()).clicked() && result.live {
                inputs.push(Input::Editor(result.input.clone()));
            }
        }
    });
}

/// A word or line shown as its text, with the button that removes it.
fn text_line(ui: &mut egui::Ui, text: &TextLine, inputs: &mut Vec<Input>) {
    ui.horizontal(|ui| {
        if text.refused {
            ui.colored_label(ui.visuals().error_fg_color, &text.text);
        } else {
            ui.monospace(&text.text);
        }
        if ui.small_button("×").on_hover_text("Remove").clicked() {
            inputs.push(Input::Editor(text.remove.clone()));
        }
    });
}

/// A number between "−" and "+", and a field to type it into when it has one.
fn stepper_ui(ui: &mut egui::Ui, stepper: &Stepper, inputs: &mut Vec<Input>) {
    if !stepper.label.is_empty() {
        ui.label(&stepper.label);
    }
    if ui.add_enabled(stepper.lower.is_some(), egui::Button::new("−")).clicked()
        && let Some(input) = &stepper.lower
    {
        inputs.push(Input::Editor(input.clone()));
    }
    match stepper.typed {
        Some(Typed { field, value, min, max }) => {
            let mut typed = value;
            // Kept as the board says it, even out of the field's range.
            ui.push_id(field, |ui| ui.add(egui::DragValue::new(&mut typed).range(min..=max).clamp_existing_to_range(false)));
            if typed != value {
                inputs.push(Input::Editor(EditorInput::Number(field, typed)));
            }
        }
        None => {
            ui.strong(&stepper.value);
        }
    }
    if ui.add_enabled(stepper.raise.is_some(), egui::Button::new("+")).clicked()
        && let Some(input) = &stepper.raise
    {
        inputs.push(Input::Editor(input.clone()));
    }
}

/// A button whose click sends its input while that changes something; a
/// current choice shows selected.
fn edit_button(ui: &mut egui::Ui, button: &EditButton, inputs: &mut Vec<Input>) {
    let clicked = ui.add_enabled(button.live || button.on, egui::Button::new(&button.label).selected(button.on)).clicked();
    if clicked && button.live {
        inputs.push(Input::Editor(button.input.clone()));
    }
}

/// The stack, empty or not, then each other shared zone that holds anything.
fn side_panel(ui: &mut egui::Ui, board: &BoardView, inputs: &mut Vec<Input>) {
    ui.strong("Stack, top first");
    if board.stack.is_empty() {
        ui.weak("empty");
    }
    for entry in &board.stack {
        item(ui, entry, inputs);
    }
    let others = [
        ("Triggered, waiting to be put on the stack", &board.pending_triggers),
        ("Exile", &board.exile),
        ("Command zone", &board.command),
    ];
    for (name, items) in others.into_iter().filter(|(_, items)| !items.is_empty()) {
        ui.strong(name);
        for entry in items {
            item(ui, entry, inputs);
        }
    }
}

fn zone_view(ui: &mut egui::Ui, zone: &ZoneView, inputs: &mut Vec<Input>) {
    egui::CollapsingHeader::new(&zone.name).id_salt(zone.key).default_open(zone.open).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for entry in &zone.items {
                item(ui, entry, inputs);
            }
        });
    });
}

/// A card, permanent, stack object or player: a button when an option names
/// it, outlined when the prompt is about it, filled when chosen.
fn item(ui: &mut egui::Ui, item: &Item, inputs: &mut Vec<Input>) {
    let mut text = egui::RichText::new(if item.detail.is_empty() {
        item.title.clone()
    } else {
        format!("{}\n{}", item.title, item.detail)
    });
    if item.tapped {
        text = text.weak();
    }
    let visuals = ui.visuals();
    let stroke = if item.subject {
        egui::Stroke::new(2.5, visuals.warn_fg_color)
    } else if item.clickable {
        egui::Stroke::new(1.5, visuals.selection.stroke.color)
    } else {
        egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
    };
    let sense = if item.clickable { egui::Sense::click() } else { egui::Sense::hover() };
    let button = egui::Button::new(text).selected(item.chosen).stroke(stroke).sense(sense);
    let mut response = ui.add(button);
    // Read before the hover's tooltip takes the response: a right-click asks
    // why, on any item, clickable or not.
    let asked_why = response.hovered() && ui.input(|input| input.pointer.secondary_clicked());
    if !item.hover.is_empty() || item.printed.is_some() {
        response = response.on_hover_ui(|ui| hover(ui, item));
    }
    if response.clicked()
        && let Some(target) = item.target
    {
        inputs.push(Input::Board(target));
    }
    if asked_why && let Some(BoardRef::Object(id)) = item.target {
        inputs.push(Input::Why(id));
    }
}

/// The why panel: the engine's lines, each indented under the one it says
/// more about, its rule beside it and a link for each object it names.
fn why_panel(ui: &mut egui::Ui, why: &WhyView, inputs: &mut Vec<Input>) {
    ui.horizontal(|ui| {
        if ui.add_enabled(why.back, egui::Button::new("Back")).clicked() {
            inputs.push(Input::WhyBack);
        }
        if ui.button("×").on_hover_text("Close").clicked() {
            inputs.push(Input::WhyClose);
        }
        ui.strong(format!("Why: {}", why.title));
    });
    if let Some(note) = why.note {
        ui.weak(note);
    }
    egui::ScrollArea::vertical().id_salt("why").show(ui, |ui| {
        for section in &why.sections {
            ui.separator();
            ui.strong(&section.heading);
            for line in &section.lines {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(18.0 * f32::from(line.depth));
                    ui.label(&line.text);
                    if let Some(rule) = &line.rule {
                        ui.weak(rule);
                    }
                    for link in &line.links {
                        if ui.add_enabled(link.live, egui::Link::new(&link.label)).clicked() {
                            inputs.push(link.input.clone());
                        }
                    }
                });
            }
        }
    });
}

/// What a hover shows: the item as it is now, its type line under its first
/// line as a card prints one, beside each face as printed.
fn hover(ui: &mut egui::Ui, item: &Item) {
    ui.horizontal_top(|ui| {
        if !item.hover.is_empty() {
            ui.vertical(|ui| {
                ui.strong("Now");
                let (first, rest) = item.hover.split_once('\n').unwrap_or((&item.hover, ""));
                ui.label(first);
                if let Some(line) = &item.type_line {
                    type_line(ui, line);
                }
                if !rest.is_empty() {
                    ui.label(rest);
                }
            });
        }
        for face in item.printed.iter().flat_map(|faces| faces.iter()) {
            ui.vertical(|ui| {
                ui.strong("As printed");
                ui.label(face);
            });
        }
    });
    if let Some(hint) = item.why_hint {
        ui.weak(hint);
    }
}

/// A type line a word at a time, a word an effect took away greyed in place.
fn type_line(ui: &mut egui::Ui, line: &TypeLineView) {
    ui.horizontal_wrapped(|ui| {
        // About a space's width, so the words read as one line.
        ui.spacing_mut().item_spacing.x = 4.0;
        let word = |ui: &mut egui::Ui, word: &TypeWordView| {
            if word.faded {
                ui.weak(&word.text);
            } else {
                ui.label(&word.text);
            }
        };
        line.front.iter().for_each(|w| word(ui, w));
        if !line.subtypes.is_empty() {
            ui.label("—");
            line.subtypes.iter().for_each(|w| word(ui, w));
        }
    });
}

fn prompt_panel(ui: &mut egui::Ui, prompt: &PromptView, inputs: &mut Vec<Input>) {
    ui.heading(&prompt.question);
    if let Some(rejected) = &prompt.rejected {
        ui.colored_label(ui.visuals().warn_fg_color, rejected);
    }
    ui.weak(&prompt.rule);
    ui.horizontal_wrapped(|ui| {
        for (i, option) in prompt.options.iter().enumerate() {
            match option.amount {
                Some(Amount { value, can_lower, can_raise }) => {
                    ui.group(|ui| {
                        ui.label(&option.label);
                        if ui.add_enabled(can_lower, egui::Button::new("−")).clicked() {
                            inputs.push(Input::OneFewer(i));
                        }
                        ui.strong(value.to_string());
                        if ui.add_enabled(can_raise, egui::Button::new("+")).clicked() {
                            inputs.push(Input::OneMore(i));
                        }
                    });
                }
                None => {
                    let label = match option.place {
                        Some(place) => format!("{place}. {}", option.label),
                        None => option.label.clone(),
                    };
                    if ui.add_enabled(option.live, egui::Button::new(label).selected(option.chosen)).clicked() {
                        inputs.push(Input::OptionButton(i));
                    }
                }
            }
        }
    });
    ui.horizontal(|ui| {
        if let Some(NumberField { min, max, value }) = prompt.number {
            let mut number = value;
            ui.add(egui::DragValue::new(&mut number).range(min..=max));
            if number != value {
                inputs.push(Input::Number(number));
            }
        }
        if let Some(done) = &prompt.done
            && ui.add_enabled(done.live, egui::Button::new(&done.label)).clicked()
        {
            inputs.push(Input::Done);
        }
        if prompt.can_reset && ui.button("Start over").clicked() {
            inputs.push(Input::Reset);
        }
    });
    if !prompt.yields.is_empty() || prompt.yielding.is_some() {
        ui.horizontal_wrapped(|ui| {
            for button in &prompt.yields {
                seat_button(ui, button, inputs);
            }
            if let Some((words, stop)) = &prompt.yielding {
                ui.weak(words);
                seat_button(ui, stop, inputs);
            }
        });
    }
    ui.weak(KEYS);
}

/// The shortcut keys pressed this frame, taken out of egui's input so that a
/// focused button does not act on them too; none while a field is typed into.
fn keys(ctx: &egui::Context) -> Vec<Input> {
    if ctx.egui_wants_keyboard_input() {
        return Vec::new();
    }
    ctx.input_mut(|input| {
        let mut keys = Vec::new();
        input.events.retain(|event| match event {
            egui::Event::Key { key, pressed: true, repeat, modifiers, .. } if modifiers.is_none() => match shortcut(*key) {
                Some(key) => {
                    keys.push(Input::Key { key, repeat: *repeat });
                    false
                }
                None => true,
            },
            _ => true,
        });
        keys
    })
}

fn shortcut(key: egui::Key) -> Option<Key> {
    let digit = [
        egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4, egui::Key::Num5,
        egui::Key::Num6, egui::Key::Num7, egui::Key::Num8, egui::Key::Num9,
    ]
    .iter()
    .position(|digit| *digit == key);
    Some(match key {
        _ if digit.is_some() => Key::Digit(digit.map_or(0, |at| at as u8 + 1)),
        egui::Key::Enter => Key::Enter,
        egui::Key::Space => Key::Space,
        egui::Key::Escape => Key::Escape,
        egui::Key::F2 => Key::F2,
        egui::Key::F4 => Key::F4,
        egui::Key::F6 => Key::F6,
        _ => return None,
    })
}

fn tool_button(ui: &mut egui::Ui, button: &ToolButton, inputs: &mut Vec<Input>) {
    if ui.add_enabled(button.live, egui::Button::new(&button.label)).clicked() {
        inputs.push(button.input.clone());
    }
}

fn seat_button(ui: &mut egui::Ui, button: &SeatButton, inputs: &mut Vec<Input>) {
    if ui.add_enabled(button.live, egui::Button::new(&button.label)).clicked() {
        inputs.push(button.input.clone());
    }
}
