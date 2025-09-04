use std::process::Command;
use std::path::Path;
use std::str::FromStr;

use crate::error::GitChaiError;
use crate::types::{GitStatus, ChangeType};

#[derive(Debug, Clone)]
pub struct GitChange {
    pub status: GitStatus,
    pub change_type: ChangeType,
    pub filename: String,
}

pub fn get_changed_files(repo_path: &Path) -> Result<Vec<GitChange>, GitChaiError> {
    log::debug!("Getting changed files from {:?}", repo_path);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("status")
        .arg("--porcelain=v1")
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Git status command failed: {}", error_msg);
        return Err(GitChaiError::GitCommandError {
            command: "git status --porcelain=v1".to_string(),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    let status_output = String::from_utf8_lossy(&output.stdout);
    let mut changes = Vec::new();
    
    for line in status_output.lines() {
        if line.len() < 3 {
            continue;
        }
        
        let status_str = &line[0..2];
        let filename = line[3..].trim();
        
        if filename.is_empty() {
            continue;
        }
        
        let status = GitStatus::from_str(status_str)
            .map_err(|e| GitChaiError::ParseError(format!("Failed to parse git status: {}", e)))?;
        
        let change_type = ChangeType::from(status.clone());
        
        log::debug!("Detected change: {} - {}", status, filename);
        
        changes.push(GitChange {
            status,
            change_type,
            filename: filename.to_string(),
        });
    }
    
    log::info!("Found {} changed files", changes.len());
    Ok(changes)
}