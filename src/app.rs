use eframe::egui::{self, RichText, Color32, Vec2};
use std::path::{PathBuf, Path};
use crate::config::UserConfig;
use crate::quiz::Quiz;
use crate::ui::{QuizUI, QuizAction};

#[derive(Debug)]
enum AppState {
    Home,
    FileSelection,
    QuizSummary,
    QuizInProgress,
    QuizResults,
    QuestionReview,
}

pub struct QuizApp {
    config: UserConfig,
    ui: QuizUI,
    quiz: Option<Quiz>,
    current_file: Option<PathBuf>,
    review_index: Option<usize>,
    state: AppState,
}

impl QuizApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: UserConfig::load(),
            ui: QuizUI::default(),
            quiz: None,
            current_file: None,
            review_index: None,
            state: AppState::Home,
        }
    }

    fn load_quiz(&mut self, path: &Path) -> Result<(), String> {
        match Quiz::load_from_csv(path) {
            Ok(quiz) => {
                log::info!("Quiz loaded successfully");
                self.quiz = Some(quiz);
                self.state = AppState::QuizSummary;
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to load quiz: {}", e);
                Err(format!("Failed to load quiz: {}", e))
            }
        }
    }

    fn restart_quiz(&mut self, shuffle: bool) {
        if let Some(quiz) = &mut self.quiz {
            if shuffle {
                quiz.shuffle();
            }
            quiz.current_index = 0;
            // Reset all answers
            for question in &mut quiz.questions {
                question.user_answer = None;
            }
            self.state = AppState::QuizSummary;
            self.ui.start_time = None;
            self.ui.paused_duration = std::time::Duration::ZERO;
        }
    }

    /// Draws a home icon button in the top right. Returns true if clicked.
    fn show_home_icon(&self, ctx: &egui::Context) -> bool {
        let icon_size = Vec2::splat(32.0);
        let margin = 2.0;
        let screen_rect = ctx.screen_rect();
        let pos = egui::pos2(screen_rect.right() - icon_size.x - margin, screen_rect.top() + margin);
        let mut clicked = false;
        egui::Area::new("home_icon_area")
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(icon_size, egui::Sense::click());
                let painter = ui.painter();
                // Draw house outline
                let c = rect.center();
                let w = rect.width();
                let h = rect.height();
                let roof_top = egui::pos2(c.x, rect.top() + h * 0.25);
                let left = egui::pos2(rect.left() + w * 0.2, rect.bottom() - h * 0.2);
                let right = egui::pos2(rect.right() - w * 0.2, rect.bottom() - h * 0.2);
                let base_top_left = egui::pos2(rect.left() + w * 0.3, rect.top() + h * 0.5);
                let base_top_right = egui::pos2(rect.right() - w * 0.3, rect.top() + h * 0.5);
                // Roof
                painter.line_segment([roof_top, left], (2.0, Color32::WHITE));
                painter.line_segment([roof_top, right], (2.0, Color32::WHITE));
                // Base
                painter.line_segment([left, right], (2.0, Color32::WHITE));
                painter.line_segment([left, base_top_left], (2.0, Color32::WHITE));
                painter.line_segment([right, base_top_right], (2.0, Color32::WHITE));
                painter.line_segment([base_top_left, base_top_right], (2.0, Color32::WHITE));
                if response.clicked() {
                    clicked = true;
                }
            });
        clicked
    }
}

impl eframe::App for QuizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show home icon in top right for all screens except Home
        if !matches!(self.state, AppState::Home) {
            if self.show_home_icon(ctx) {
                self.state = AppState::Home;
                return;
            }
        }
        ctx.set_pixels_per_point(self.ui.ui_scale);
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Home => {
                    // Vertically center content responsively
                    let available_height = ui.available_height();
                    let content_height = 250.0; // Estimate of content height
                    let top_space = (available_height - content_height).max(0.0) / 2.0;
                    ui.add_space(top_space);
                    ui.vertical_centered(|ui| {
                        // Title
                        ui.label(RichText::new("KnowIT")
                            .size(72.0)
                            .color(Color32::WHITE)
                            .strong());
                        ui.add_space(30.0);
                        // Start button
                        let button_response = ui.add_sized(
                            [200.0, 50.0],
                            egui::Button::new(
                                RichText::new("Start")
                                    .size(24.0)
                                    .color(Color32::WHITE)
                            )
                            .fill(Color32::from_rgb(48, 86, 148))
                        );
                        if button_response.clicked() {
                            self.state = AppState::FileSelection;
                        }
                        ui.add_space(20.0);
                        // Settings text
                        let settings_response = ui.add(
                            egui::Label::new(
                                RichText::new("Settings")
                                    .size(16.0)
                                    .color(Color32::WHITE)
                            )
                            .sense(egui::Sense::click())
                        );
                        if settings_response.clicked() {
                            self.ui.show_settings = true;
                        }
                        // Show settings window if enabled
                        if self.ui.show_settings {
                            let mut show = true;
                            egui::Window::new("Settings")
                                .open(&mut show)
                                .show(ctx, |ui| {
                                    self.ui.show_settings(ui, &mut self.config.quiz_folder);
                                });
                            self.ui.show_settings = show;
                        }
                    });
                }
                AppState::FileSelection => {
                    if let Some(file) = self.ui.show_file_selection(
                        ui,
                        &self.config.quiz_folder,
                        &self.config.file_history,
                    ) {
                        log::info!("Selected file: {}", file);
                        let path = if Path::new(&file).is_absolute() {
                            PathBuf::from(file)
                        } else {
                            self.config.quiz_folder.join(file)
                        };
                        
                        self.current_file = Some(path.clone());
                        if let Ok(()) = self.load_quiz(&path) {
                            if let Some(file_str) = path.to_str() {
                                self.config.update_file_history(file_str.to_string());
                                self.config.save().unwrap_or_default();
                            }
                        } else {
                            ui.label(RichText::new("Failed to load quiz file").color(Color32::RED));
                        }
                    }
                }
                AppState::QuizSummary => {
                    if let Some(quiz) = &mut self.quiz {
                        if self.ui.show_quiz_summary(ui, quiz) {
                            if self.ui.shuffle_questions {
                                quiz.shuffle();
                            }
                            self.ui.start_time = Some(std::time::Instant::now());
                            self.state = AppState::QuizInProgress;
                        }
                    }
                }
                AppState::QuizInProgress => {
                    if let Some(quiz) = &mut self.quiz {
                        if let Some(question) = quiz.current_question() {
                            let (answer, action) = self.ui.show_question(
                                ui,
                                question,
                                quiz.current_index,
                                quiz.questions.len(),
                            );
                            
                            match action {
                                QuizAction::PreviousQuestion => {
                                    quiz.previous_question();
                                }
                                QuizAction::NextQuestion => {
                                    if let Some(ans) = answer {
                                        quiz.submit_answer(ans);
                                        if quiz.current_index == quiz.questions.len() - 1 {
                                            self.state = AppState::QuizResults;
                                        } else {
                                            quiz.next_question();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppState::QuizResults => {
                    if let Some(quiz) = &self.quiz {
                        let (total, correct, incorrect) = quiz.get_results();
                        let (selected_question, action) = self.ui.show_results(ui, total, correct, &incorrect, &quiz.questions);
                        
                        match action {
                            QuizAction::RestartQuiz => {
                                self.restart_quiz(false);
                            }
                            QuizAction::RestartShuffled => {
                                self.restart_quiz(true);
                            }
                            QuizAction::ReturnToFileSelection => {
                                self.state = AppState::FileSelection;
                                self.quiz = None;
                                self.current_file = None;
                            }
                            _ => {}
                        }

                        if let Some(index) = selected_question {
                            self.review_index = Some(index);
                            self.state = AppState::QuestionReview;
                        }
                    }
                }
                AppState::QuestionReview => {
                    if let Some(quiz) = &self.quiz {
                        if let Some(review_index) = self.review_index {
                            if let Some(question) = quiz.questions.get(review_index) {
                                ui.heading("Review Question");
                                ui.separator();
                                ui.label(RichText::new(&question.text).size(18.0));
                                ui.add_space(10.0);
                                ui.label(format!("Your answer: {}", question.user_answer.as_ref().unwrap_or(&String::new())));
                                ui.label(format!("Correct answer: {}", question.correct_answer));
                                if ui.button("Back to Results").clicked() {
                                    self.review_index = None;
                                    self.state = AppState::QuizResults;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
} 