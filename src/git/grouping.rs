use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::error::GitChaiError;
use crate::git::status::GitChange;

#[derive(Debug)]
pub struct ChangeGroup {
    pub path: PathBuf,
    pub change_type: String,
    pub files: Vec<String>,
    pub file_change_types: Option<Vec<String>>,
}

pub fn get_all_files_in_directory(repo_path: &Path, directory: &Path) -> Result<Vec<String>, GitChaiError> {
    log::debug!("Getting all files in directory: {:?}", directory);
    
    let dir_arg = if directory == Path::new(".") {
        "."
    } else {
        directory.to_str().unwrap_or(".")
    };
    
    let output = std::process::Command::new("git")
        .current_dir(repo_path)
        .arg("ls-files")
        .arg(dir_arg)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to get files in directory {:?}: {}", directory, error_msg);
        return Err(GitChaiError::GitCommandError {
            command: format!("git ls-files {}", dir_arg),
            stderr: error_msg.to_string(),
            source: None,
        });
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = output_str.lines().map(|s| s.to_string()).collect();
    
    log::debug!("Found {} files in directory: {:?}", files.len(), directory);
    Ok(files)
}

pub fn group_changes_by_directory(repo_path: &Path, changes: &[GitChange]) -> Result<Vec<ChangeGroup>, GitChaiError> {
    let mut directory_groups: HashMap<PathBuf, (String, Vec<String>)> = HashMap::new();
    let mut untracked_directories = Vec::new();
    
    for change in changes {
        let path = PathBuf::from(&change.filename);
        
        // Special case: if the filename ends with "/", it's a directory itself
        if change.filename.ends_with('/') && change.status == crate::types::GitStatus::Untracked {
            untracked_directories.push(ChangeGroup {
                path: path.clone(),
                change_type: "add".to_string(),
                files: vec![change.filename.clone()],
                file_change_types: Some(vec!["add".to_string()]),
            });
            continue;
        }
        
        let parent_dir = if let Some(parent) = path.parent() {
            if parent == Path::new("") {
                PathBuf::from(".")
            } else {
                parent.to_path_buf()
            }
        } else {
            PathBuf::from(".")
        };
        
        let change_type_str = change.change_type.to_string();
        
        directory_groups
            .entry(parent_dir.clone())
            .and_modify(|(existing_type, files)| {
                if existing_type != &change_type_str {
                    *existing_type = "mixed".to_string();
                }
                files.push(change.filename.clone());
            })
            .or_insert_with(|| (change_type_str, vec![change.filename.clone()]));
    }
    
    let mut result = Vec::new();
    
    // Add untracked directories first
    result.extend(untracked_directories);
    
    for (path, (change_type, changed_files)) in directory_groups {
        if change_type != "mixed" {
            // Check if ALL files in this directory are changed
            match get_all_files_in_directory(repo_path, &path) {
                Ok(all_files) => {
                    if changed_files.len() == all_files.len() {
                        // All files in directory are changed with uniform type
                        result.push(ChangeGroup {
                            path,
                            change_type,
                            files: changed_files,
                            file_change_types: None,
                        });
                        continue;
                    }
                }
                Err(e) => {
                    // Continue with individual processing
                    eprintln!("Warning: Failed to get files for directory {}: {}", path.display(), e);
                }
            }
        }
        
        // Mixed changes or not all files changed - treat as individual files
        let mut individual_files = Vec::new();
        let mut individual_change_types = Vec::new();
        
        for change in changes {
            if changed_files.contains(&change.filename) {
                individual_files.push(change.filename.clone());
                individual_change_types.push(change.change_type.to_string());
            }
        }
        
        result.push(ChangeGroup {
            path: PathBuf::from("."),
            change_type: "individual".to_string(),
            files: individual_files,
            file_change_types: Some(individual_change_types),
        });
    }
    
    Ok(result)
}