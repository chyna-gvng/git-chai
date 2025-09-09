mod config;
mod error;
mod git;
mod types;

use crate::config::Config;
use crate::git::{
    ChangeGroup, create_commit_for_directory, create_commit_for_file, get_changed_files,
    group_changes_by_directory, push_changes, stage_directory, stage_file,
};
use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(about, long_about = None, disable_version_flag = true)]
struct Args {
    /// Path to git repository
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,

    /// Push changes to remote after committing
    #[arg(short, long, default_value_t = false)]
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

    /// Show version information
    #[arg(short = '?', long = "version")]
    version: bool,
}

fn process_changes(config: &Config, dry_run: bool, push: bool, verbose: bool) -> Result<()> {
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
            changes
                .iter()
                .map(|change| ChangeGroup {
                    path: PathBuf::from("."),
                    change_type: "individual".to_string(),
                    files: vec![change.filename.clone()],
                    file_change_types: Some(vec![change.change_type.to_string()]),
                })
                .collect()
        }
    };

    for group in change_groups {
        if dry_run {
            if verbose {
                log::info!(
                    "DRY RUN: Would process group - Type: {}, Path: {}, Files: {:?}",
                    group.change_type,
                    group.path.display(),
                    group.files
                );
                if let Some(ref change_types) = group.file_change_types {
                    log::info!("DRY RUN: Change types: {:?}", change_types);
                }
            } else {
                log::info!(
                    "DRY RUN: Would process {} files in {}: {}",
                    group.files.len(),
                    group.change_type,
                    group.path.display()
                );
            }
            continue;
        }

        if group.change_type != "individual" && group.change_type != "mixed" {
            if verbose {
                log::info!(
                    "Processing directory: {}: {} (would stage all files and commit)",
                    group.change_type,
                    group.path.display()
                );
            } else {
                log::info!(
                    "Processing directory: {}: {}",
                    group.change_type,
                    group.path.display()
                );
            }

            if let Err(e) = stage_directory(&config.repo_path, &group.path) {
                log::error!("Failed to stage directory {}: {}", group.path.display(), e);
                continue;
            }

            if let Err(e) =
                create_commit_for_directory(&config.repo_path, &group.path, &group.change_type)
            {
                log::error!(
                    "Failed to create commit for directory {}: {}",
                    group.path.display(),
                    e
                );
                continue;
            }

            if verbose {
                log::info!(
                    "Committed directory: {}: {} (commit message: '{}: {}')",
                    group.change_type,
                    group.path.display(),
                    group.change_type,
                    group
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("directory")
                );
            } else {
                log::info!(
                    "Committed directory: {}: {}",
                    group.change_type,
                    group.path.display()
                );
            }
        } else {
            for (i, file_entry) in group.files.iter().enumerate() {
                let clean_filename = file_entry;

                let change_type = if let Some(ref change_types) = group.file_change_types {
                    if i < change_types.len() {
                        &change_types[i]
                    } else {
                        if clean_filename.ends_with("/") {
                            "add"
                        } else {
                            "mod"
                        }
                    }
                } else {
                    "mod"
                };

                if verbose {
                    log::info!(
                        "Processing: {}: {} (would stage and commit)",
                        change_type,
                        clean_filename
                    );
                } else {
                    log::info!("Processing: {}: {}", change_type, clean_filename);
                }

                if let Err(e) = stage_file(&config.repo_path, clean_filename) {
                    log::error!("Failed to stage file {}: {}", clean_filename, e);
                    continue;
                }

                if let Err(e) =
                    create_commit_for_file(&config.repo_path, clean_filename, change_type)
                {
                    log::error!("Failed to create commit for {}: {}", clean_filename, e);
                    continue;
                }

                if verbose {
                    log::info!(
                        "Committed: {}: {} (commit message: '{}: {}')",
                        change_type,
                        clean_filename,
                        change_type,
                        clean_filename
                    );
                } else {
                    log::info!("Committed: {}: {}", change_type, clean_filename);
                }
            }
        }
    }

    log::info!("Successfully committed all changes!");

    if push && !dry_run {
        if let Err(e) = push_changes(&config.repo_path) {
            log::warn!("Failed to push changes: {}", e);
            log::warn!("Changes were committed locally but not pushed to remote.");
        } else {
            log::info!("Successfully pushed changes to remote!");
        }
    } else if push && dry_run {
        log::info!("DRY RUN: Would push changes to remote");
    } else {
        log::info!("Skipping push (--no-push specified)");
    }

    Ok(())
}

fn resolve_repo_toplevel(path: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git rev-parse: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git rev-parse --show-toplevel failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(toplevel))
}

fn main() -> Result<()> {
    let args = Args::parse();

    unsafe {
        if args.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    if args.version {
        println!("git-chai 0.1.0");
        return Ok(());
    }

    let repo_root = match resolve_repo_toplevel(&args.repo_path) {
        Ok(p) => p,
        Err(e) => {
            log::error!(
                "Failed to resolve git repo top-level for {:?}: {}",
                args.repo_path,
                e
            );
            std::process::exit(1);
        }
    };

    let config = Config {
        repo_path: repo_root,
        push_by_default: args.push,
        commit_message_template: "{change_type}: {name}".to_string(),
        min_files_for_directory_commit: 2,
    };

    if args.headless {
        use std::thread;
        use std::time::Duration;

        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, std::sync::atomic::Ordering::SeqCst);
            println!("\nReceived interrupt signal, shutting down...");
        })
        .expect("Error setting Ctrl+C handler");

        log::info!("git-chai: Starting in headless mode. Press Ctrl+C to stop.");

        while running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Err(e) = process_changes(&config, args.dry_run, args.push, args.verbose) {
                log::error!("Error processing changes: {}", e);
            }

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
        log::info!("git-chai: Running once");
        process_changes(&config, args.dry_run, args.push, args.verbose)
    }
}
