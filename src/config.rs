use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub repo_path: PathBuf,
    pub push_by_default: bool,
    pub commit_message_template: String,
    pub min_files_for_directory_commit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from("."),
            push_by_default: true,
            commit_message_template: "{change_type}: {name}".to_string(),
            min_files_for_directory_commit: 2,
        }
    }
}