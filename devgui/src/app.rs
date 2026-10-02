//! The egui drawing: lays out what `view_model` built and hands each click to
//! the `Session`. It decides nothing, so it is reviewed by running it.

use std::path::Path;
use std::sync::Arc;

use eframe::egui;

use crate::bridge::GameSetup;
use crate::session::Session;
use crate::view_model::{Amount, BoardView, Input, Item, NumberField, PromptView, WindowState, ZoneView};

pub struct DevGui {
    session: Session,
    /// The seed and the start.
    setup_line: String,
}

impl DevGui {
    pub fn new(setup: GameSetup, setup_line: String, wake: Arc<dyn Fn() + Send + Sync>) -> DevGui {
        DevGui { session: Session::start(setup, wake), setup_line }
    }
}

impl eframe::App for DevGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.session.receive();
        let session = &self.session;
        let header = SessionHeader {
            line: &self.setup_line,
            log: session.log_path.as_deref(),
            reloadable: session.setup.scenario.is_some(),
            saved: session.saved.as_ref().map(|saved| saved.as_ref().map(String::as_str).map_err(String::as_str)),
        };
        for input in draw(ui, &session.state, &header) {
            self.session.input(input);
        }
    }
}

/// What the header says of the session, beside the board.
pub struct SessionHeader<'a> {
    /// The seed and the start.
    pub line: &'a str,
    /// This game's decision log.
    pub log: Option<&'a Path>,
    /// A scenario's game, which Reload builds again from its file.
    pub reloadable: bool,
    /// The last save: where it went, or why it could not.
    pub saved: Option<Result<&'a str, &'a str>>,
}

/// The whole window; the inputs the player made this frame.
pub fn draw(ui: &mut egui::Ui, state: &WindowState, header: &SessionHeader) -> Vec<Input> {
    let mut inputs = Vec::new();
    let board = state.board_view();
    egui::Panel::top("header").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong(state.status());
            if let Some(board) = &board {
                ui.separator();
                ui.label(&board.header);
            }
            ui.separator();
            match header.log {
                Some(log) => ui.weak(format!("{} · decision log {}", header.line, log.display())),
                None => ui.weak(header.line),
            };
            if header.reloadable && ui.button("Reload").clicked() {
                inputs.push(Input::Reload);
            }
            if state.board.is_some() && ui.button("Save board as scenario").clicked() {
                inputs.push(Input::SaveBoard);
            }
            match header.saved {
                Some(Ok(saved)) => {
                    ui.weak(saved);
                }
                Some(Err(failed)) => {
                    ui.colored_label(ui.visuals().error_fg_color, failed);
                }
                None => {}
            }
        });
    });
    egui::Panel::bottom("prompt").show(ui, |ui| {
        if let Some(message) = &state.refused {
            ui.colored_label(ui.visuals().error_fg_color, "The scenario did not load:");
            ui.monospace(message);
            ui.weak("Fix the file, then click Reload.");
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
            prompt_panel(ui, &prompt, &mut inputs);
        } else {
            ui.weak(state.status());
        }
    });
    egui::Panel::right("side").default_size(380.0).show(ui, |ui| {
        if let Some(board) = &board {
            side_panel(ui, board, &mut inputs);
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
    egui::CentralPanel::default().show(ui, |ui| {
        let Some(board) = &board else {
            ui.weak(state.no_board());
            return;
        };
        egui::ScrollArea::vertical().show(ui, |ui| {
            for seat in &board.seats {
                // A collapsing header's id is its label, and every seat has a "Creatures".
                ui.push_id(seat.player.target, |ui| {
                    item(ui, &seat.player, &mut inputs);
                    for zone in &seat.zones {
                        zone_view(ui, zone, &mut inputs);
                    }
                });
                ui.separator();
            }
        });
    });
    inputs
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
    if !item.hover.is_empty() {
        response = response.on_hover_text(&item.hover);
    }
    if response.clicked()
        && let Some(target) = item.target
    {
        inputs.push(Input::Board(target));
    }
}

fn prompt_panel(ui: &mut egui::Ui, prompt: &PromptView, inputs: &mut Vec<Input>) {
    ui.heading(&prompt.question);
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
}
