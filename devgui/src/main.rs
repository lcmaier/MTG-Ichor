//! `cargo run -- [--seed N] [--pool performance|stress]`
//!
//! Plays seat 0 against the random agent. The seed defaults to the clock; the
//! window's header shows it and the decision log records it.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use devgui::app::DevGui;
use devgui::bridge::{GameSetup, Pool, spawn_game};
use eframe::egui;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let seed = match flag("--seed") {
        Some(text) => text.parse().unwrap_or_else(|_| panic!("--seed takes a number, not {text}")),
        None => SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u64),
    };
    let pool = match flag("--pool").map(String::as_str) {
        None | Some("performance") => Pool::Performance,
        Some("stress") => Pool::Stress,
        Some(other) => panic!("--pool is performance or stress, not {other}"),
    };
    let log_path = PathBuf::from("logs").join(format!("seed-{seed}.log"));
    let setup_line = format!("seed {seed} · {pool:?} pool · decision log {}", log_path.display());
    let setup = GameSetup { seed, pool, log_path: Some(log_path) };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MTG-Ichor dev GUI",
        options,
        Box::new(move |creation| {
            let ctx = creation.egui_ctx.clone();
            let engine = spawn_game(setup, Arc::new(move || ctx.request_repaint()));
            Ok(Box::new(DevGui::new(engine, setup_line)))
        }),
    )
}
