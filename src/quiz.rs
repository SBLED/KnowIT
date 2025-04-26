use std::path::Path;
use csv::ReaderBuilder;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
pub struct Question {
    pub number: u32,
    pub text: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub user_answer: Option<String>,
}

#[derive(Debug, Clone)]
pub enum QuestionType {
    ShortAnswer,
    MultipleChoice,
    Mixed,
}

#[derive(Debug)]
pub struct Quiz {
    pub questions: Vec<Question>,
    pub question_type: QuestionType,
    pub current_index: usize,
    pub shuffled: bool,
}

impl Quiz {
    pub fn load_from_csv(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Loading quiz from: {}", path.display());
        
        // Create a CSV reader with flexible options
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(path)?;

        let mut questions = Vec::new();
        let mut has_short_answer = false;
        let mut has_multiple_choice = false;

        for (i, result) in reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    println!("Error reading row {}: {}", i + 1, e);
                    return Err(format!("Error reading row {}: {}", i + 1, e).into());
                }
            };

            println!("Processing row {}: {:?}", i + 1, record);
            
            // We need at least 3 columns: number, question, and correct answer
            if record.len() < 3 {
                let error = format!("Error: Row {} must have at least 3 columns (number, question, answer), found {}", i + 1, record.len());
                println!("{}", error);
                return Err(error.into());
            }

            let number = match record.get(0).and_then(|s| s.trim().parse::<u32>().ok()) {
                Some(n) => n,
                None => {
                    let error = format!("Invalid question number in row {}", i + 1);
                    println!("{}", error);
                    return Err(error.into());
                }
            };

            let text = match record.get(1) {
                Some(t) => t.trim().to_string(),
                None => {
                    let error = format!("Missing question text in row {}", i + 1);
                    println!("{}", error);
                    return Err(error.into());
                }
            };

            let correct_answer = match record.get(2) {
                Some(a) => a.trim().to_string(),
                None => {
                    let error = format!("Missing correct answer in row {}", i + 1);
                    println!("{}", error);
                    return Err(error.into());
                }
            };

            // Get multiple choice options if they exist (columns 4 and beyond)
            let mut options = Vec::new();
            if record.len() > 3 {
                // For multiple choice, include the correct answer in the options
                options.push(correct_answer.clone());
                // Add the additional options
                options.extend(record.iter().skip(3).map(|s| s.trim().to_string()));
            }

            if options.is_empty() {
                has_short_answer = true;
                println!("Row {} is a short answer question", i + 1);
            } else {
                has_multiple_choice = true;
                println!("Row {} is a multiple choice question with {} options", i + 1, options.len());
            }

            questions.push(Question {
                number,
                text,
                correct_answer,
                options,
                user_answer: None,
            });
        }

        if questions.is_empty() {
            let error = "No questions found in the CSV file";
            println!("{}", error);
            return Err(error.into());
        }

        let question_type = match (has_short_answer, has_multiple_choice) {
            (true, false) => QuestionType::ShortAnswer,
            (false, true) => QuestionType::MultipleChoice,
            _ => QuestionType::Mixed,
        };

        println!("Loaded {} questions", questions.len());
        println!("Question type: {:?}", question_type);

        Ok(Self {
            questions,
            question_type,
            current_index: 0,
            shuffled: false,
        })
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.questions.shuffle(&mut rng);
        for question in &mut self.questions {
            if !question.options.is_empty() {
                question.options.shuffle(&mut rng);
            }
        }
        self.shuffled = true;
    }

    pub fn current_question(&self) -> Option<&Question> {
        self.questions.get(self.current_index)
    }

    pub fn next_question(&mut self) -> bool {
        if self.current_index < self.questions.len() - 1 {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn previous_question(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn submit_answer(&mut self, answer: String) {
        if let Some(question) = self.questions.get_mut(self.current_index) {
            question.user_answer = Some(answer);
        }
    }

    pub fn get_results(&self) -> (usize, usize, Vec<usize>) {
        let total = self.questions.len();
        let mut correct = 0;
        let mut incorrect = Vec::new();

        for (i, question) in self.questions.iter().enumerate() {
            if let Some(user_answer) = &question.user_answer {
                if user_answer.trim().to_lowercase() == question.correct_answer.trim().to_lowercase() {
                    correct += 1;
                } else {
                    incorrect.push(i);
                }
            }
        }

        (total, correct, incorrect)
    }
} 