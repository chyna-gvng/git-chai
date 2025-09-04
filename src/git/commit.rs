use std::process::Command;
use std::path::Path;

use crate::error::GitChaiError;

pub fn create_commit_for_file(repo_path: &Path, filename: &str, change_type: &str) -> Result<(), GitChaiError> {
    let message = format!("{}: {}", change_type, filename);
    log::debug!("Creating commit for file: {} - {}", change_type, filename);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to commit file {}: {}", filename, error_msg);
        return Err(GitChaiError::GitCommandError {
            command: format!("git commit -m '{}'", message),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    log::debug!("Successfully committed file: {}", filename);
    Ok(())
}

pub fn create_commit_for_directory(repo_path: &Path, directory: &Path, change_type: &str) -> Result<(), GitChaiError> {
    let dir_name = directory.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| directory.to_str().unwrap_or("directory"));
    
    let message = format!("{}: {}", change_type, dir_name);
    log::debug!("Creating commit for directory: {} - {}", change_type, dir_name);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to commit directory {:?}: {}", directory, error_msg);
        return Err(GitChaiError::GitCommandError {
            command: format!("git commit -m '{}'", message),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    log::debug!("Successfully committed directory: {:?}", directory);
    Ok(())
}

pub fn push_changes(repo_path: &Path) -> Result<(), GitChaiError> {
    log::debug!("Pushing changes to remote");
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("push")
        .arg("origin")
        .arg("HEAD")
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to push changes: {}", error_msg);
        return Err(GitChaiError::GitCommandError {
            command: "git push origin HEAD".to_string(),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    log::debug!("Successfully pushed changes to remote");
    Ok(())
}