use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitChaiError {
    #[error("Git command failed: {command}: {stderr}")]
    GitCommandError {
        command: String,
        stderr: String,
        #[source]
        source: Option<std::io::Error>,
    },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("No changes detected")]
    NoChangesError,
    
    #[error("Invalid path: {0}")]
    InvalidPathError(String),
    
    #[error("Not a git repository: {0}")]
    NotGitRepoError(String),
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let git_error = GitChaiError::GitCommandError {
            command: "test".to_string(),
            stderr: "error".to_string(),
            source: None,
        };
        assert!(git_error.to_string().contains("Git command failed"));
        
        let io_error = GitChaiError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(io_error.to_string().contains("IO error"));
        
        let parse_error = GitChaiError::ParseError("test".to_string());
        assert!(parse_error.to_string().contains("Parse error"));
        
        let config_error = GitChaiError::ConfigError("test".to_string());
        assert!(config_error.to_string().contains("Configuration error"));
        
        let no_changes = GitChaiError::NoChangesError;
        assert!(no_changes.to_string().contains("No changes detected"));
        
        let invalid_path = GitChaiError::InvalidPathError("test".to_string());
        assert!(invalid_path.to_string().contains("Invalid path"));
        
        let not_git_repo = GitChaiError::NotGitRepoError("test".to_string());
        assert!(not_git_repo.to_string().contains("Not a git repository"));
    }
}