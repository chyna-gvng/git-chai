use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::error::GitChaiError;

pub fn get_changed_files(repo_path: &Path) -> Result<Vec<String>, GitChaiError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git status failed: {}", error_msg)));
    }
    
    let status_output = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();
    
    for line in status_output.lines() {
        if line.len() >= 4 {
            let status = &line[0..2];
            let filename = line[3..].trim();
            
            // Only process files with relevant status (added, modified, deleted, untracked)
            // Git porcelain format uses two chars: first for staging area, second for working directory
            // We care about any status that indicates changes in working directory
            // Status can be like " M" (modified in working dir), "A " (added to staging), etc.
            if status.contains('A') || status.contains('M') || status.contains('D') || status == "??" {
                // Store both status and filename for parsing later
                files.push(format!("{} {}", status, filename));
            }
        } else if line.len() >= 3 {
            // Handle short lines like "?? dir/"
            let status = &line[0..2];
            let filename = line[2..].trim();
            
            if status == "??" {
                // Store both status and filename for parsing later
                files.push(format!("{} {}", status, filename));
            }
        }
    }
    
    Ok(files)
}

pub fn stage_file(repo_path: &Path, filename: &str) -> Result<(), GitChaiError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("add")
        .arg(filename)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git add failed for {}: {}", filename, error_msg)));
    }
    
    Ok(())
}

pub fn create_commit_for_file(repo_path: &Path, filename: &str, change_type: &str) -> Result<(), GitChaiError> {
    let message = format!("{}: {}", change_type, filename);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git commit failed for {}: {}", filename, error_msg)));
    }
    
    Ok(())
}

pub fn push_changes(repo_path: &Path) -> Result<(), GitChaiError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("push")
        .arg("origin")
        .arg("HEAD")
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git push failed: {}", error_msg)));
    }
    
    Ok(())
}

#[derive(Debug)]
pub struct ChangeGroup {
    pub path: PathBuf,
    pub change_type: String,
    pub files: Vec<String>,
    pub file_change_types: Option<Vec<String>>,
}

pub fn get_all_files_in_directory(repo_path: &Path, directory: &Path) -> Result<Vec<String>, GitChaiError> {
    let dir_arg = if directory == Path::new(".") {
        "."
    } else {
        directory.to_str().unwrap_or(".")
    };
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("ls-files")
        .arg(dir_arg)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git ls-files failed for {}: {}", directory.display(), error_msg)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = output_str.lines().map(|s| s.to_string()).collect();
    
    Ok(files)
}

pub fn group_changes_by_directory(repo_path: &Path, files: &[String]) -> Result<Vec<ChangeGroup>, GitChaiError> {
    let mut directory_groups: HashMap<PathBuf, (String, Vec<String>)> = HashMap::new();
    let mut untracked_directories = Vec::new();
    
    for file_entry in files {
        // Git porcelain format: "XY filename" where X is staging area, Y is working directory
        // We need to extract the status (first 2 chars) and filename (rest)
        if file_entry.len() < 4 {
            continue;
        }
        
        let status = &file_entry[0..2];
        let filename = file_entry[3..].trim();
        
        let change_type = if status.contains("A") || status == "??" {
            "add"
        } else if status.contains("M") {
            "mod"
        } else if status.contains("D") {
            "del"
        } else {
            continue;
        };
        
        let path = PathBuf::from(filename);
        
        // Special case: if the filename ends with "/", it's a directory itself
        if filename.ends_with('/') && status == "??" {
            // This is an untracked directory, add it to special handling
            untracked_directories.push(ChangeGroup {
                path: path,
                change_type: "add".to_string(),
                files: vec![filename.to_string()],
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
        
        directory_groups
            .entry(parent_dir.clone())
            .and_modify(|(existing_type, files)| {
                if existing_type != change_type {
                    *existing_type = "mixed".to_string();
                }
                files.push(filename.to_string());
            })
            .or_insert_with(|| (change_type.to_string(), vec![filename.to_string()]));
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
                            file_change_types: None, // Not needed for directory-level commits
                        });
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get files for directory {}: {}", path.display(), e);
                    // Continue with individual processing
                }
            }
        }
        
        // Mixed changes or not all files changed - treat as individual files
        // For individual files, we need to preserve the original change types
        let mut individual_files = Vec::new();
        let mut individual_change_types = Vec::new();
        
        // Re-parse the original files to get change types for individual processing
        for file_entry in files {
            if file_entry.len() < 4 {
                continue;
            }
            
            let status = &file_entry[0..2];
            let filename = file_entry[3..].trim();
            
            let change_type = if status.contains("A") || status == "??" {
                "add"
            } else if status.contains("M") {
                "mod"
            } else if status.contains("D") {
                "del"
            } else {
                continue;
            };
            
            if changed_files.contains(&filename.to_string()) {
                individual_files.push(filename.to_string());
                individual_change_types.push(change_type.to_string());
            }
        }
        
        result.push(ChangeGroup {
            path: PathBuf::from("."), // Use current directory for individual files
            change_type: "individual".to_string(),
            files: individual_files,
            file_change_types: Some(individual_change_types),
        });
    }
    
    Ok(result)
}

pub fn stage_directory(repo_path: &Path, directory: &Path) -> Result<(), GitChaiError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("add")
        .arg("--all")
        .arg(directory)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git add failed for directory {}: {}", directory.display(), error_msg)));
    }
    
    Ok(())
}

pub fn create_commit_for_directory(repo_path: &Path, directory: &Path, change_type: &str) -> Result<(), GitChaiError> {
    let dir_name = directory.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| directory.to_str().unwrap_or("directory"));
    
    let message = format!("{}: {}", change_type, dir_name);
    
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()
        .map_err(|e| GitChaiError::IoError(e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitChaiError::GitError(format!("git commit failed for directory {}: {}", directory.display(), error_msg)));
    }
    
    Ok(())
}