# Gitnapped

> Find out why you didn't sleep – a Git commit timeline analyzer

Gitnapped is a command-line tool that analyzes your Git commit history across multiple repositories. It provides insights into your coding patterns, helping you understand where your time is spent across projects.

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

### From Source

```bash
git clone https://github.com/yourusername/gitnapped.git
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
```

### Configuration

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

Please note that the **author name** will be used to filter the commits.

### Command Line Options

```bash
-c, --config <FILE>          Sets a custom config file
-s, --since <DATE>           Start date for analysis (YYYY-MM-DD)
-u, --until <DATE>           End date for analysis (YYYY-MM-DD)
-p, --period <PERIOD>        Relative time period (e.g., 6M, 2Y, 5D, 12H)
    --active-only            Show only repositories with commits in the period
    --sort-by <FIELD>        Sort repositories by: commits, files, lines
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
```

## License

Licensed under AGPL-3.0 license.

## Author/Maintainer

Marco Orlandin <marco@solexma.com>
