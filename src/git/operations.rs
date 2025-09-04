use std::process::Command;
use std::path::Path;

use crate::error::GitChaiError;

pub fn stage_file(repo_path: &Path, filename: &str) -> Result<(), GitChaiError> {
    log::debug!("Staging file: {}", filename);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("add")
        .arg(filename)
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to stage file {}: {}", filename, error_msg);
        return Err(GitChaiError::GitCommandError {
            command: format!("git add {}", filename),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    log::debug!("Successfully staged file: {}", filename);
    Ok(())
}

pub fn stage_directory(repo_path: &Path, directory: &Path) -> Result<(), GitChaiError> {
    log::debug!("Staging directory: {:?}", directory);
    
    let dir_arg = if directory == Path::new(".") {
        "."
    } else {
        directory.to_str().unwrap_or(".")
    };
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("add")
        .arg("--all")
        .arg(dir_arg)
        .output()
        .map_err(GitChaiError::IoError)?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to stage directory {:?}: {}", directory, error_msg);
        return Err(GitChaiError::GitCommandError {
            command: format!("git add --all {}", dir_arg),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    log::debug!("Successfully staged directory: {:?}", directory);
    Ok(())
}