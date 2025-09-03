use std::fmt;

#[derive(Debug)]
pub enum GitChaiError {
    GitError(String),
    IoError(std::io::Error),
}

impl fmt::Display for GitChaiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitChaiError::GitError(e) => write!(f, "Git error: {}", e),
            GitChaiError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for GitChaiError {}

impl From<std::io::Error> for GitChaiError {
    fn from(err: std::io::Error) -> Self {
        GitChaiError::IoError(err)
    }
}