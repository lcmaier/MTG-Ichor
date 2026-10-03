//! `cargo run -- [--seed N] [--pool performance|stress] [--scenario FILE]`
//!
//! Plays every seat, a prompt naming the seat it asks. With `--scenario`, the
//! game starts from the file's board, and Reload builds it again from the
//! file. `launch` reads the command line; this starts the window over it.
//!
//! A debug build checks every layer-memo hit against a fresh walk, which is
//! what to test cards under and costs a prompt about 30 ms on a large board,
//! with the engine lightly optimized (`Cargo.toml`); `cargo run --release`
//! costs it a fraction of a millisecond (`engineering-practices.md` §10.4).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use devgui::app::DevGui;
use devgui::launch;
use eframe::egui;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let clock = || SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u64);
    let launch = launch::read(&args, clock).unwrap_or_else(|problem| {
        eprintln!("{problem}\n{}", launch::USAGE);
        std::process::exit(2)
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MTG-Ichor dev GUI",
        options,
        Box::new(move |creation| {
            let ctx = creation.egui_ctx.clone();
            Ok(Box::new(DevGui::new(launch.setup, launch.setup_line, Arc::new(move || ctx.request_repaint()))))
        }),
    )
}
