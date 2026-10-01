//! `cargo run -- [--seed N] [--pool performance|stress] [--scenario FILE]`
//!
//! Plays seat 0 against the random agent. With `--scenario`, the game starts
//! from the file's board, and Reload builds it again from the file. The seed
//! defaults to the scenario's own, or else to the clock; the window's header
//! shows it and the decision log records it.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use devgui::app::DevGui;
use devgui::bridge::{GameSetup, Pool};
use eframe::egui;
use mtgsim::scenario::Scenario;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let scenario = flag("--scenario").map(PathBuf::from);
    let seed = match (flag("--seed"), &scenario) {
        (Some(text), _) => text.parse().unwrap_or_else(|_| panic!("--seed takes a number, not {text}")),
        // A file that does not parse shows its refusal in the window instead.
        (None, Some(path)) => std::fs::read_to_string(path).ok().and_then(|text| Scenario::parse(&text).ok()).map_or(0, |s| s.seed),
        (None, None) => SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u64),
    };
    let pool = match flag("--pool").map(String::as_str) {
        None | Some("performance") => Pool::Performance,
        Some("stress") => Pool::Stress,
        Some(other) => panic!("--pool is performance or stress, not {other}"),
    };
    let (log_name, setup_line) = match &scenario {
        Some(path) => {
            let stem = path.file_stem().map_or("scenario".into(), |s| s.to_string_lossy());
            (format!("{stem}-seed-{seed}.log"), format!("scenario {} · seed {seed}", path.display()))
        }
        None => (format!("seed-{seed}.log"), format!("seed {seed} · {pool:?} pool")),
    };
    let log_path = PathBuf::from("logs").join(log_name);
    let setup_line = format!("{setup_line} · decision log {}", log_path.display());
    let setup = GameSetup { seed, pool, log_path: Some(log_path), scenario };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MTG-Ichor dev GUI",
        options,
        Box::new(move |creation| {
            let ctx = creation.egui_ctx.clone();
            Ok(Box::new(DevGui::new(setup, setup_line, Arc::new(move || ctx.request_repaint()))))
        }),
    )
}
