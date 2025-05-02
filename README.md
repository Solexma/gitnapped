# Gitnapped

![Version](https://img.shields.io/badge/version-0.1.4-blue.svg) [![Crates.io](https://img.shields.io/crates/v/gitnapped)](https://crates.io/crates/gitnapped)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)

![Downloads](https://img.shields.io/crates/d/gitnapped) ![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/Solexma/gitnapped/total?label=GH%20downloads)

![License](https://img.shields.io/badge/license-AGPL--3.0-green)

> Find out why you didn't sleep – a Git commit timeline analyzer

Gitnapped is a command-line tool that analyzes your Git commit history across multiple repositories. It provides insights into your coding patterns, helping you understand where your time is spent across projects.

## What does it mean to get Gitnapped?

Picture this: it's 5 PM, you're ready to call it a day, but then you think, "Just one more commit…" Next thing you know, it's 2 AM, and you're knee-deep in code. That's being gitnapped! Gitnapped (the tool) helps you spot these moments by analyzing your commit history—showing you when Git's magic stole your sleep.

> So, have you gitnapped yourself lately?

## Features

- Analyze commits across multiple repositories
- Group repositories by categories or projects
- Filter by time periods (relative or absolute dates)
- Filter by author
- View detailed statistics including:
  - Commit counts
  - Files changed
  - Lines added/removed
  - Most active days
  - File types modified

## Installation

### Using Cargo

```bash
cargo install gitnapped
```

### Using Homebrew

```bash
brew tap Solexma/gitnapped
brew install gitnapped
```

### From Source

```bash
git clone https://github.com/Solexma/gitnapped.git
cd gitnapped
cargo build --release
```

The compiled binary will be available at `target/release/gitnapped`.

## Usage

```bash
gitnapped [OPTIONS]
```

### Basic Examples

```bash
# Analyze commits from yesterday to now
gitnapped

# Analyze commits from the last 6 months
gitnapped -p 6M

# Analyze all repositories with detailed information
gitnapped --repo-details

# Show only active repositories grouped by project
gitnapped --active-only --projects

# Analyze a specific directory without a config file
gitnapped -d /path/to/repository
```

### Configuration

Gitnapped can be configured in two ways:

1. **Using a Config File**
   Create a `gitnapped.yaml` file in your working directory or specify a custom path with `-c`:

   ```yaml
   author: "Your Name"

   repos:
     personal:
       - /path/to/repo1 [Category][Project Name]
       - /path/to/repo2 [Category][Project Name]
     
     clients:
       - /path/to/client1 [Client][Project Name]
       
     opensource:
       - /path/to/opensource1 [OSS][Project Name]
   ```

2. **Using Current Directory**
   If no config file is found, Gitnapped will automatically use the current directory as a repository. This is useful for quick analysis of a single repository without creating a config file.

   Note: The current directory must be a valid Git repository for this fallback to work.

### Default Behavior

- If no config file is specified (`-c`), Gitnapped will look for `gitnapped.yaml` in the current directory
- If no config file is found, it will use the current directory as a repository
- If a directory is explicitly specified with `-d`, it will only analyze that directory
- The **author name** in the config file will be used to filter commits unless overridden by `-a` or `--all-authors`

### Command Line Options

```console
-c, --config <FILE>          Sets a custom config file
-d, --dir <DIRECTORY>        Sets a directory to analyze (bypasses config file)
-s, --since <DATE>           Start date for analysis (YYYY-MM-DD)
-u, --until <DATE>           End date for analysis (YYYY-MM-DD)
-p, --period <PERIOD>        Relative time period (e.g., 6M, 2Y, 5D, 12H)
    --active-only            Show only repositories with commits in the period
    --sort-by <FIELD>        Sort repositories by: commits, files, lines (default: commits)
    --categories             Show statistics by category
    --projects               Group repositories by project name
    --repo-details           Show detailed information for each repository
    --filetypes              Show file types used in the repositories
-a, --author <AUTHOR>        Filter commits by specific author
    --all-authors            Include commits from all authors
    --most-active-day        Show the most active day
    --silent                 Silent mode, no output
    --json                   Output in JSON format
    --debug                  Enable debug messages
    --working-time <TIME>    Working hours in 24-hour (HH:MM-HH:MM) or 12-hour (HAM-PM) format (default: 09:00-17:00)
    --ungitnapped            Hide gitnapped information from the output
    --most-active-repos <N>  How many most active repositories to show (default: 5)
    --show-total-stats       Show total stats across all analyzed entities
    --pretty                 Pretty print the output
```

## License

Licensed under AGPL-3.0 license.

## Full disclosure

Let's be real.

Gitnapped is a fancy wrapper for Git's command-line magic. But that's the point! It takes the heavy lifting out of analyzing your commit history, so you can see when you've been gitnapped by your code—those late-night commit sprees or weekend coding binges. Understand your coding patterns, reclaim your sleep, and maybe have a laugh along the way.

## Author/Maintainer

Marco Orlandin <marco@solexma.com>
