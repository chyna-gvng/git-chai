# Chai

A git automation tool that automatically stages and commits changes with intelligent grouping.

## Requirements

- `curl` - for downloading and installation
- `rustup` - for building Rust binaries (will be installed automatically if missing)

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/chyna-gvng/git-chai/main/installer.sh | bash
```

## Usage

Run `git-chai` in your Git repository to automatically stage and commit changes:

```bash
git-chai [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-r` | `--repo-path` | Path to git repository (default: current directory) |
| `-p` | `--push` | Push changes to remote after committing (default: false) |
| `-d` | `--dry-run` | Show what would be committed without actually committing |
| `-v` | `--verbose` | Enable verbose output |
| `-!` | `--headless` | Run continuously until interrupted (headless mode) |
| `-?` | `--version` | Show version information |

### Examples

#### Level 1: Basic Commit Operations
```bash
# Commit changes once (no push)
git-chai

# Dry-run to preview what would be committed
git-chai --dry-run
```

#### Level 2: Enhanced Commit Operations  
```bash
# Commit with verbose output for debugging
git-chai --verbose

# Commit changes in specific repository
git-chai --repo-path /path/to/repo

# Preview commits with verbose details
git-chai --dry-run --verbose
```

#### Level 3: Commit + Push Operations
```bash
# Commit and push changes once
git-chai --push

# Commit and push with verbose output
git-chai --push --verbose

# Commit and push from specific repository
git-chai --repo-path /path/to/repo --push
```

#### Level 4: Complete Autonomy
```bash
# Continuous monitoring with auto-commit (no push)
git-chai --headless

# Fully autonomous: continuous commit + push
git-chai --headless --push

# Autonomous with detailed logging
git-chai --headless --push --verbose

# Development Workflow:
# Terminal 1: git-chai --headless --push
# Terminal 2: # Keep coding - changes auto-committed & pushed
```
