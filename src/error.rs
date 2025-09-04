use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
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
        

    }
}