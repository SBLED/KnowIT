mod app;
mod config;
mod quiz;
mod ui;

use app::QuizApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
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
