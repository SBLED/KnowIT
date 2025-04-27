#![windows_subsystem = "windows"]

mod app;
mod config;
mod quiz;
mod ui;

use app::QuizApp;
use eframe::egui;
use std::path::PathBuf;
use simplelog::*;
use std::fs::{self, File};

fn main() -> Result<(), eframe::Error> {
    // Get the user's Documents/KnowIT directory
    let mut log_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
    log_dir.push("KnowIT");
    let _ = fs::create_dir_all(&log_dir); // Create the directory if it doesn't exist
    let log_path = log_dir.join("quiz_app.log");

    // Overwrite the log file on each launch
    let _ = File::create(&log_path); // Truncate the file

    // Set up logging
    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, Config::default(), File::create(&log_path).unwrap()),
    ]).unwrap();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Quiz App",
        options,
        Box::new(|cc| {
            // Set dark mode
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(QuizApp::new(cc))
        }),
    )
}
