use egui::{RichText, Ui};
use std::path::PathBuf;
use std::fs;
use crate::quiz::{Quiz, QuestionType, Question};

pub struct QuizUI {
    pub show_settings: bool,
    pub allow_going_back: bool,
    pub shuffle_questions: bool,
    pub timer_paused: bool,
    pub start_time: Option<std::time::Instant>,
    pub paused_duration: std::time::Duration,
    pub current_answer: String,
}

#[derive(Debug)]
pub enum QuizAction {
    None,
    PreviousQuestion,
    NextQuestion,
    RestartQuiz,
    RestartShuffled,
    ReturnToFileSelection,
}

impl Default for QuizUI {
    fn default() -> Self {
        Self {
            show_settings: false,
            allow_going_back: true,
            shuffle_questions: false,
            timer_paused: false,
            start_time: None,
            paused_duration: std::time::Duration::ZERO,
            current_answer: String::new(),
        }
    }
}

impl QuizUI {
    pub fn show_file_selection(
        &mut self,
        ui: &mut Ui,
        quiz_folder: &PathBuf,
        file_history: &[(String, i64)],
    ) -> Option<String> {
        let mut selected_file = None;

        ui.heading("Select Quiz File");
        ui.separator();

        // Show file history
        if !file_history.is_empty() {
            ui.label("Recent Files:");
            for (file, _) in file_history {
                if ui.button(file).clicked() {
                    selected_file = Some(file.clone());
                }
            }
            ui.separator();
        }

        // Show current folder
        ui.label(format!("Current Folder: {}", quiz_folder.display()));
        if ui.button("Change Folder").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(quiz_folder.clone())
                .pick_folder()
            {
                return Some(path.to_string_lossy().into_owned());
            }
        }

        // Show CSV files in the current folder
        if let Ok(entries) = fs::read_dir(quiz_folder) {
            ui.add_space(10.0);
            ui.label("Available Quiz Files:");
            ui.separator();
            
            let mut files: Vec<_> = entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("csv"))
                        .unwrap_or(false)
                })
                .collect();
            
            // Sort files by name
            files.sort_by_key(|entry| entry.file_name());

            for entry in files {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if ui.button(name).clicked() {
                        selected_file = Some(entry.path().to_string_lossy().into_owned());
                    }
                }
            }
        }

        selected_file
    }

    pub fn show_settings(&mut self, ui: &mut Ui, quiz_folder: &mut PathBuf) {
        ui.heading("Settings");
        ui.separator();

        ui.label("Quiz Folder:");
        ui.horizontal(|ui| {
            ui.label(quiz_folder.display().to_string());
            if ui.button("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(quiz_folder.clone())
                    .pick_folder()
                {
                    *quiz_folder = path;
                }
            }
        });

        ui.checkbox(&mut self.allow_going_back, "Allow going back to previous questions");
        ui.checkbox(&mut self.shuffle_questions, "Shuffle questions");
    }

    pub fn show_quiz_summary(&mut self, ui: &mut Ui, quiz: &Quiz) -> bool {
        ui.heading("Quiz Summary");
        ui.separator();

        ui.label(format!("Total Questions: {}", quiz.questions.len()));
        ui.label(format!(
            "Question Type: {}",
            match quiz.question_type {
                QuestionType::ShortAnswer => "Short Answer",
                QuestionType::MultipleChoice => "Multiple Choice",
                QuestionType::Mixed => "Mixed",
            }
        ));

        ui.add_space(10.0);
        ui.checkbox(&mut self.allow_going_back, "Allow going back to previous questions");
        ui.checkbox(&mut self.shuffle_questions, "Shuffle questions");

        ui.add_space(20.0);
        ui.button("Begin Quiz").clicked()
    }

    pub fn show_question(
        &mut self,
        ui: &mut Ui,
        question: &Question,
        current_index: usize,
        total_questions: usize,
    ) -> (Option<String>, QuizAction) {
        ui.heading(format!("Question {} of {}", current_index + 1, total_questions));
        ui.separator();

        // Show timer
        if let Some(start_time) = self.start_time {
            let elapsed = if self.timer_paused {
                self.paused_duration
            } else {
                start_time.elapsed() - self.paused_duration
            };
            let minutes = elapsed.as_secs() / 60;
            let seconds = elapsed.as_secs() % 60;
            ui.label(format!("Time: {:02}:{:02}", minutes, seconds));
            
            if ui.button(if self.timer_paused { "Resume" } else { "Pause" }).clicked() {
                self.timer_paused = !self.timer_paused;
                if self.timer_paused {
                    self.paused_duration = start_time.elapsed() - self.paused_duration;
                }
            }
        }

        ui.add_space(10.0);
        ui.label(RichText::new(&question.text).size(18.0));

        let mut submit_answer = None;
        let mut action = QuizAction::None;

        if question.options.is_empty() {
            // Short answer
            ui.text_edit_singleline(&mut self.current_answer);
        } else {
            // Multiple choice
            for option in &question.options {
                if ui.radio_value(&mut self.current_answer, option.clone(), option).clicked() {
                    // Just update the current_answer, don't submit yet
                }
            }
        }

        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if current_index > 0 && self.allow_going_back {
                if ui.button("Previous").clicked() {
                    self.current_answer = String::new();
                    action = QuizAction::PreviousQuestion;
                }
            }

            if ui.button(if current_index == total_questions - 1 { "Finish" } else { "Next" }).clicked() {
                if !self.current_answer.is_empty() {
                    submit_answer = Some(self.current_answer.clone());
                    self.current_answer = String::new();
                    action = QuizAction::NextQuestion;
                }
            }
        });

        (submit_answer, action)
    }

    pub fn show_results(
        &mut self,
        ui: &mut Ui,
        total: usize,
        correct: usize,
        incorrect: &[usize],
        questions: &[Question],
    ) -> (Option<usize>, QuizAction) {
        ui.heading("Quiz Results");
        ui.separator();

        let percentage = (correct as f32 / total as f32) * 100.0;
        ui.label(format!("Score: {}/{} ({:.1}%)", correct, total, percentage));

        let mut selected_question = None;
        let mut action = QuizAction::None;

        if !incorrect.is_empty() {
            ui.add_space(10.0);
            ui.label("Incorrect Questions:");
            for &index in incorrect {
                let question = &questions[index];
                if ui
                    .button(format!(
                        "Question {}: {}",
                        question.number,
                        question.text
                    ))
                    .clicked()
                {
                    selected_question = Some(index);
                }
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Restart Quiz").clicked() {
                action = QuizAction::RestartQuiz;
            }
            if ui.button("Restart with Shuffled Questions").clicked() {
                action = QuizAction::RestartShuffled;
            }
            if ui.button("Return to File Selection").clicked() {
                action = QuizAction::ReturnToFileSelection;
            }
        });

        (selected_question, action)
    }
} 