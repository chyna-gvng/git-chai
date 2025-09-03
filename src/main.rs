mod error;
mod git_operations;
mod config;

use anyhow::Result;
use std::path::PathBuf;
use crate::git_operations::{get_changed_files, stage_file, create_commit_for_file, push_changes, group_changes_by_directory, stage_directory, create_commit_for_directory, ChangeGroup};
use crate::config::Config;

fn main() -> Result<()> {
    let config = Config::default();
    
    println!("git-chai: Scanning for changes...");
    
    let files = match get_changed_files(&config.repo_path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error: Failed to scan for changes: {}", e);
            std::process::exit(1);
        }
    };
    
    let change_groups = match group_changes_by_directory(&config.repo_path, &files) {
        Ok(groups) => groups,
        Err(e) => {
            eprintln!("Error: Failed to group changes by directory: {}", e);
            // Fall back to individual processing
            files.iter()
                .map(|file_entry| {
                    // Parse the file entry to extract change type for fallback
                    let change_type = if file_entry.starts_with("??") {
                        "add"
                    } else if file_entry.contains('M') {
                        "mod"
                    } else if file_entry.contains('D') {
                        "del"
                    } else if file_entry.contains('A') {
                        "add"
                    } else {
                        "mod"
                    };
                    
                    // Extract just the filename
                    let filename = if file_entry.len() >= 4 {
                        file_entry[3..].trim().to_string()
                    } else {
                        file_entry.clone()
                    };
                    
                    ChangeGroup {
                        path: PathBuf::from("."),
                        change_type: "individual".to_string(),
                        files: vec![filename],
                        file_change_types: Some(vec![change_type.to_string()]),
                    }
                })
                .collect()
        }
    };
    
    for group in change_groups {
        if group.change_type != "individual" && group.change_type != "mixed" {
            // Directory with uniform changes - commit as a group
            println!("Processing directory: {}: {}", group.change_type, group.path.display());
            
            if let Err(e) = stage_directory(&config.repo_path, &group.path) {
                eprintln!("Error: Failed to stage directory {}: {}", group.path.display(), e);
                continue;
            }
            
            if let Err(e) = create_commit_for_directory(&config.repo_path, &group.path, &group.change_type) {
                eprintln!("Error: Failed to create commit for directory {}: {}", group.path.display(), e);
                continue;
            }
            
            println!("Committed directory: {}: {}", group.change_type, group.path.display());
        } else {
            // Mixed changes or single file - process individually
            for (i, file_entry) in group.files.iter().enumerate() {
                let clean_filename = file_entry;
                
                // Get the change type for this file
                let change_type = if let Some(ref change_types) = group.file_change_types {
                    if i < change_types.len() {
                        &change_types[i]
                    } else {
                        // Fallback: determine from filename (shouldn't happen)
                        if clean_filename.ends_with("/") {
                            "add"
                        } else {
                            "mod"
                        }
                    }
                } else {
                    // Fallback for groups without change types
                    "mod"
                };
                
                println!("Processing: {}: {}", change_type, clean_filename);
                
                if let Err(e) = stage_file(&config.repo_path, clean_filename) {
                    eprintln!("Error: Failed to stage file {}: {}", clean_filename, e);
                    continue;
                }
                
                if let Err(e) = create_commit_for_file(&config.repo_path, clean_filename, change_type) {
                    eprintln!("Error: Failed to create commit for {}: {}", clean_filename, e);
                    continue;
                }
                
                println!("Committed: {}: {}", change_type, clean_filename);
            }
        }
    }
    
    println!("Successfully committed all changes!");
    
    if let Err(e) = push_changes(&config.repo_path) {
        eprintln!("Warning: Failed to push changes: {}", e);
        eprintln!("Changes were committed locally but not pushed to remote.");
    } else {
        println!("Successfully pushed changes to remote!");
    }
    
    Ok(())
}
