use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub quiz_folder: PathBuf,
    pub file_history: Vec<(String, i64)>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            quiz_folder: PathBuf::from("."),
            file_history: Vec::new(),
        }
    }
}

impl UserConfig {
    pub fn load() -> Self {
        let config_path = "userconfig.cfg";
        match fs::read_to_string(config_path) {
            Ok(contents) => {
                serde_json::from_str(&contents).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = "userconfig.cfg";
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(config_path, contents)?;
        Ok(())
    }

    pub fn update_file_history(&mut self, filename: String) {
        let timestamp = chrono::Utc::now().timestamp();
        self.file_history.retain(|(f, _)| f != &filename);
        self.file_history.insert(0, (filename, timestamp));
        if self.file_history.len() > 10 {
            self.file_history.truncate(10);
        }
    }
} 