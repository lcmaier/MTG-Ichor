//! The egui drawing: lays out what `view_model` built and reports clicks back
//! as `Input`s. It decides nothing, so it is reviewed by running it.

use eframe::egui;

use crate::bridge::EngineHandle;
use crate::view_model::{BoardView, Input, Item, PromptView, WindowState, ZoneView};

pub struct DevGui {
    state: WindowState,
    engine: EngineHandle,
    /// The seed, the pool and the decision log's path.
    setup_line: String,
}

impl DevGui {
    pub fn new(engine: EngineHandle, setup_line: String) -> DevGui {
        DevGui { state: WindowState::default(), engine, setup_line }
    }
}

impl eframe::App for DevGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.engine.from_engine.try_recv() {
            self.state.receive(message);
        }
        for input in draw(ui, &self.state, &self.setup_line) {
            if let Some(answer) = self.state.input(input) {
                // Fails only once the engine thread has ended, and its last message said why.
                let _ = self.engine.answers.send(answer);
            }
        }
    }
}

/// The whole window; the inputs the player made this frame.
pub fn draw(ui: &mut egui::Ui, state: &WindowState, setup_line: &str) -> Vec<Input> {
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
            ui.weak(setup_line);
        });
    });
    egui::Panel::bottom("prompt").show(ui, |ui| {
        if let Some(message) = &state.panic {
            ui.colored_label(ui.visuals().error_fg_color, "The engine thread panicked:");
            ui.monospace(message);
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
            for line in &state.log[rows] {
                ui.label(line);
            }
        });
    });
    egui::CentralPanel::default().show(ui, |ui| {
        let Some(board) = &board else {
            ui.weak("Waiting for the engine's first prompt.");
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

fn side_panel(ui: &mut egui::Ui, board: &BoardView, inputs: &mut Vec<Input>) {
    let lists = [
        ("Stack, top first", &board.stack),
        ("Triggered, waiting to be put on the stack", &board.pending_triggers),
        ("Exile", &board.exile),
        ("Command zone", &board.command),
    ];
    for (name, items) in lists {
        if name.starts_with("Stack") || !items.is_empty() {
            ui.strong(name);
            if items.is_empty() {
                ui.weak("empty");
            }
            for entry in items {
                item(ui, entry, inputs);
            }
        }
    }
}

fn zone_view(ui: &mut egui::Ui, zone: &ZoneView, inputs: &mut Vec<Input>) {
    egui::CollapsingHeader::new(&zone.name).default_open(zone.open).show(ui, |ui| {
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
                Some((amount, can_lower, can_raise)) => {
                    ui.group(|ui| {
                        ui.label(&option.label);
                        if ui.add_enabled(can_lower, egui::Button::new("−")).clicked() {
                            inputs.push(Input::Adjust(i, false));
                        }
                        ui.strong(amount.to_string());
                        if ui.add_enabled(can_raise, egui::Button::new("+")).clicked() {
                            inputs.push(Input::Adjust(i, true));
                        }
                    });
                }
                None => {
                    let label = match option.place {
                        Some(place) => format!("{place}. {}", option.label),
                        None => option.label.clone(),
                    };
                    if ui.add(egui::Button::new(label).selected(option.chosen)).clicked() {
                        inputs.push(Input::Option(i));
                    }
                }
            }
        }
    });
    ui.horizontal(|ui| {
        if let Some((min, max, value)) = prompt.number {
            let mut number = value;
            ui.add(egui::DragValue::new(&mut number).range(min..=max));
            if number != value {
                inputs.push(Input::Number(number));
            }
        }
        if let Some((label, live)) = &prompt.done
            && ui.add_enabled(*live, egui::Button::new(label)).clicked()
        {
            inputs.push(Input::Done);
        }
        if prompt.can_reset && ui.button("Start over").clicked() {
            inputs.push(Input::Reset);
        }
    });
}
