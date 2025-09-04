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

#### Basic Usage
Run once in current directory:
```bash
git-chai
```

Check what would happen without making changes:
```bash
git-chai --dry-run
```

#### Intermediate Usage
Run with verbose output to see detailed operations:
```bash
git-chai --verbose
```

Run in specific directory and push changes:
```bash
git-chai --repo-path /path/to/repo --push --verbose
```

#### Complete Autonomy
Run continuously, monitoring for changes every 5 seconds:
```bash
git-chai --headless
```

Fully autonomous operation with push and verbose logging:
```bash
git-chai --headless --push --verbose
```

Development workflow with continuous commit:
```bash
# In one terminal - continuously commit changes
git-chai --headless

# In another terminal - work on your code
# All changes will be automatically committed and tracked
# Manually push when done; git push
```
