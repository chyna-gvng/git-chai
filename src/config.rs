use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub repo_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from("."),
        }
    }
}