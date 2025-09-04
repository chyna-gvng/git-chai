mod error;
mod git;
mod config;
mod types;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use crate::git::{get_changed_files, stage_file, create_commit_for_file, push_changes, group_changes_by_directory, stage_directory, create_commit_for_directory, ChangeGroup};
use crate::config::Config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to git repository
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,
    
    /// Push changes to remote after committing
    #[arg(short, long, default_value_t = true)]
    push: bool,
    
    /// Dry run - show what would be committed without actually committing
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
    
    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
    
    /// Headless mode - run continuously until interrupted
    #[arg(short = '!', long, default_value_t = false)]
    headless: bool,
}

fn process_changes(config: &Config, dry_run: bool, push: bool) -> Result<()> {
    log::info!("Scanning for changes in {:?}...", config.repo_path);
    
    let changes = match get_changed_files(&config.repo_path) {
        Ok(changes) => {
            if changes.is_empty() {
                log::info!("No changes detected");
                return Ok(());
            }
            changes
        }
        Err(e) => {
            log::error!("Failed to scan for changes: {}", e);
            std::process::exit(1);
        }
    };
    
    let change_groups = match group_changes_by_directory(&config.repo_path, &changes) {
        Ok(groups) => groups,
        Err(e) => {
            log::error!("Failed to group changes by directory: {}", e);
            // Fall back to individual processing
            changes.iter()
                .map(|change| {
                    ChangeGroup {
                        path: PathBuf::from("."),
                        change_type: "individual".to_string(),
                        files: vec![change.filename.clone()],
                        file_change_types: Some(vec![change.change_type.to_string()]),
                    }
                })
                .collect()
        }
    };
    
    for group in change_groups {
        if dry_run {
            log::info!("Dry run: Would process group: {:?}", group);
            continue;
        }
        
        if group.change_type != "individual" && group.change_type != "mixed" {
            // Directory with uniform changes - commit as a group
            log::info!("Processing directory: {}: {}", group.change_type, group.path.display());
            
            if let Err(e) = stage_directory(&config.repo_path, &group.path) {
                log::error!("Failed to stage directory {}: {}", group.path.display(), e);
                continue;
            }
            
            if let Err(e) = create_commit_for_directory(&config.repo_path, &group.path, &group.change_type) {
                log::error!("Failed to create commit for directory {}: {}", group.path.display(), e);
                continue;
            }
            
            log::info!("Committed directory: {}: {}", group.change_type, group.path.display());
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
                
                log::info!("Processing: {}: {}", change_type, clean_filename);
                
                if let Err(e) = stage_file(&config.repo_path, clean_filename) {
                    log::error!("Failed to stage file {}: {}", clean_filename, e);
                    continue;
                }
                
                if let Err(e) = create_commit_for_file(&config.repo_path, clean_filename, change_type) {
                    log::error!("Failed to create commit for {}: {}", clean_filename, e);
                    continue;
                }
                
                log::info!("Committed: {}: {}", change_type, clean_filename);
            }
        }
    }
    
    log::info!("Successfully committed all changes!");

    if push {
        if let Err(e) = push_changes(&config.repo_path) {
            log::warn!("Failed to push changes: {}", e);
            log::warn!("Changes were committed locally but not pushed to remote.");
        } else {
            log::info!("Successfully pushed changes to remote!");
        }
    } else {
        log::info!("Skipping push (--no-push specified)");
    }
    
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    
    let config = Config {
        repo_path: args.repo_path,
        push_by_default: args.push,
        commit_message_template: "{change_type}: {name}".to_string(),
        min_files_for_directory_commit: 2,
    };
    
    if args.headless {
        // Headless mode: run continuously until Ctrl+C
        use std::thread;
        use std::time::Duration;
        
        // Set up Ctrl+C handler
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, std::sync::atomic::Ordering::SeqCst);
            println!("\nReceived interrupt signal, shutting down...");
        }).expect("Error setting Ctrl+C handler");

        log::info!("git-chai: Starting in headless mode. Press Ctrl+C to stop.");
        
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Err(e) = process_changes(&config, args.dry_run, args.push) {
                log::error!("Error processing changes: {}", e);
            }
            
            // Wait before next scan
            log::info!("Waiting 5 seconds before next scan...");
            for _ in 0..50 {
                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }

        log::info!("git-chai stopped");
        Ok(())
    } else {
        // Normal mode: run once and exit
        log::info!("git-chai: Running once");
        process_changes(&config, args.dry_run, args.push)
    }
}