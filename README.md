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

Run once in current directory:
```bash
git-chai
```

Run in headless mode (continuous monitoring):
```bash
git-chai --headless
```

Dry run to see what would be committed:
```bash
git-chai --dry-run
```

Run in specific repository with push enabled:
```bash
git-chai --repo-path /path/to/repo --push
```
