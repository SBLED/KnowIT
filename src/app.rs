use eframe::egui::{self, RichText};
use std::path::{PathBuf, Path};
use crate::config::UserConfig;
use crate::quiz::Quiz;
use crate::ui::{QuizUI, QuizAction};

#[derive(Debug)]
enum AppState {
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
            state: AppState::FileSelection,
        }
    }

    fn load_quiz(&mut self, path: &Path) -> Result<(), String> {
        match Quiz::load_from_csv(path) {
            Ok(quiz) => {
                println!("Quiz loaded successfully");
                self.quiz = Some(quiz);
                self.state = AppState::QuizSummary;
                Ok(())
            }
            Err(e) => {
                println!("Failed to load quiz: {}", e);
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
}

impl eframe::App for QuizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::FileSelection => {
                    if let Some(file) = self.ui.show_file_selection(
                        ui,
                        &self.config.quiz_folder,
                        &self.config.file_history,
                    ) {
                        println!("Selected file: {}", file);
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
                            ui.label(RichText::new("Failed to load quiz file").color(egui::Color32::RED));
                        }
                    }

                    if ui.button("Settings").clicked() {
                        self.ui.show_settings = !self.ui.show_settings;
                    }

                    let show_settings = self.ui.show_settings;
                    if show_settings {
                        let mut show = true;
                        egui::Window::new("Settings")
                            .open(&mut show)
                            .show(ctx, |ui| {
                                self.ui.show_settings(ui, &mut self.config.quiz_folder);
                            });
                        self.ui.show_settings = show;
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