use crate::models::{CategoryStats, Config, ProjectStats, RepoInfo, RepoStats};
use crate::parser::{group_repos_by_vanity, parse_repo_string};
use crate::utils::{
    aggregate_stats, count_files_and_lines, debug, debug_git_command, is_repo_active, log,
};
use chrono::NaiveDate;
use colored::*;
use std::collections::HashMap;
use std::process::Command;

/// Analyzes a single repository and returns its statistics.
///
/// # Arguments
/// * `repo` - Path to the Git repository
/// * `author` - Optional author name to filter commits
/// * `since` - Start date for commit analysis (YYYY-MM-DD format)
/// * `until` - End date for commit analysis (YYYY-MM-DD format)
/// * `show_details` - Whether to print detailed information about the repository
/// * `show_filetypes` - Whether to analyze and show file type statistics
/// * `working_hours` - Optional working hours to track out-of-hours commits
///
/// # Returns
/// * `RepoStats` - Statistics about the repository's commits, files, and lines
///
/// This function will:
/// - Count commits in the specified date range
/// - Handle submodules if present
/// - Count files and lines in the repository
/// - Analyze file types if requested
/// - Track out-of-hours commits
pub fn analyze_repo(
    repo: &str,
    author: &Option<String>,
    since: &str,
    until: &str,
    show_details: bool,
    show_filetypes: bool,
    working_hours: Option<(u32, u32, u32, u32)>,
) -> RepoStats {
    let mut stats = RepoStats::default();

    // Get commit history
    let mut cmd = Command::new("git");
    cmd.args([
        "-C",
        repo,
        "log",
        "--pretty=format:%h %ad %s",
        "--date=iso-strict",
    ]);

    if let Some(a) = author {
        cmd.arg(format!("--author={}", a));
    }

    cmd.arg(format!("--since={}", since));
    cmd.arg(format!("--until={}", until));

    debug(&format!("Executing git command on repo: {}", repo));

    let output = match cmd.output() {
        Ok(out) => {
            debug_git_command(repo, &cmd, &out);
            out
        }
        Err(e) => {
            debug(&format!("Error executing git command: {}", e));
            return stats;
        }
    };

    if !output.status.success() {
        debug(&format!(
            "Git command failed with status: {}",
            output.status
        ));
        debug(&format!(
            "Error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits: Vec<String> = stdout.lines().map(String::from).collect();

    // Check for submodules automatically
    debug(&format!("Checking for submodules in repository: {}", repo));

    // Get submodule status
    let mut submodule_cmd = Command::new("git");
    submodule_cmd.args(["-C", repo, "submodule", "status"]);

    let submodule_output = match submodule_cmd.output() {
        Ok(out) => {
            debug_git_command(repo, &submodule_cmd, &out);
            out
        }
        Err(e) => {
            debug(&format!("Error executing git submodule command: {}", e));
            // Continue without submodule info
            stats.commit_count = commits.len();
            return stats;
        }
    };

    // Process submodules only if the command was successful and returned output
    let has_submodules = submodule_output.status.success() && !submodule_output.stdout.is_empty();

    if has_submodules {
        let submodule_stdout = String::from_utf8_lossy(&submodule_output.stdout);
        let submodule_lines: Vec<&str> = submodule_stdout.lines().collect();

        debug(&format!("Found {} submodules", submodule_lines.len()));

        for line in submodule_lines {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                // Extract submodule path (2nd element)
                let submodule_path = parts[1];
                let full_path = format!("{}/{}", repo, submodule_path);

                debug(&format!("Found submodule: {}", full_path));

                // Get commit history for this submodule
                let mut sub_cmd = Command::new("git");
                sub_cmd.args([
                    "-C",
                    &full_path,
                    "log",
                    "--pretty=format:[SUBMODULE %s] %h %ad %s",
                    "--date=short",
                ]);

                if let Some(a) = author {
                    sub_cmd.arg(format!("--author={}", a));
                }

                sub_cmd.arg(format!("--since={}", since));
                sub_cmd.arg(format!("--until={}", until));

                debug(&format!(
                    "Executing git command on submodule: {}",
                    full_path
                ));

                let sub_output = match sub_cmd.output() {
                    Ok(out) => {
                        debug_git_command(&full_path, &sub_cmd, &out);
                        out
                    }
                    Err(e) => {
                        debug(&format!("Error executing git command on submodule: {}", e));
                        continue;
                    }
                };

                if !sub_output.status.success() {
                    debug(&format!(
                        "Git command failed on submodule with status: {}",
                        sub_output.status
                    ));
                } else {
                    let sub_stdout = String::from_utf8_lossy(&sub_output.stdout);
                    let sub_commit_count = sub_stdout.lines().count();

                    // Add submodule commits to the list (convert to owned Strings)
                    for commit in sub_stdout.lines() {
                        commits.push(format!("{}", commit));
                    }

                    debug(&format!(
                        "Added {} commits from submodule {}",
                        sub_commit_count, submodule_path
                    ));
                }
            }
        }
    }

    stats.commit_count = commits.len();

    // Display information about found commits
    debug(&format!(
        "Found {} commits in repository {}",
        commits.len(),
        repo
    ));

    // Parse commits by date and check for out-of-hours commits
    for commit in &commits {
        if let Some(date_part) = commit.split_whitespace().nth(1) {
            debug(&format!("Processing commit date: {}", date_part));

            // Extract just the date part from ISO format (YYYY-MM-DD)
            let date = date_part.split('T').next().unwrap_or(date_part);
            *stats.commits_by_date.entry(date.to_string()).or_insert(0) += 1;

            // Check if commit is outside working hours
            if let Some((start_hour, start_min, end_hour, end_min)) = working_hours {
                if let Some(time_part) = date_part.split('T').nth(1) {
                    debug(&format!("Found time part: {}", time_part));
                    if let Some((hour, minute)) = parse_commit_time(time_part) {
                        debug(&format!(
                            "Parsed commit time: {:02}:{:02} (working hours: {:02}:{:02}-{:02}:{:02})",
                            hour, minute, start_hour, start_min, end_hour, end_min
                        ));
                        if !is_within_working_hours(
                            hour, minute, start_hour, start_min, end_hour, end_min,
                        ) {
                            stats.out_of_hours_commits += 1;
                            debug(&format!(
                                "Found out-of-hours commit at {:02}:{:02}",
                                hour, minute
                            ));
                        }
                    } else {
                        debug(&format!("Failed to parse time: {}", time_part));
                    }
                } else {
                    debug("No time part found in commit date");
                }
            }
        }
    }

    // Count files and lines
    let (file_count, line_count, file_types) = count_files_and_lines(repo);
    stats.file_count = file_count;
    stats.line_count = line_count;
    stats.file_types = file_types;

    debug(&format!(
        "Counted {} files, {} lines in repository {}",
        file_count, line_count, repo
    ));

    if show_details {
        // Print repo stats with colors
        log(&format!("\n{} {}", "Repo:".bright_blue(), repo.green()));
        log(&format!(
            "{}: {}",
            "Commits".yellow(),
            stats.commit_count.to_string().cyan()
        ));
        if let Some(_) = working_hours {
            log(&format!(
                "{}: {}",
                "Out-of-hours commits".yellow(),
                stats.out_of_hours_commits.to_string().cyan()
            ));
        }
        log(&format!(
            "{}: {}",
            "Files".yellow(),
            stats.file_count.to_string().cyan()
        ));
        log(&format!(
            "{}: {}",
            "Lines of code".yellow(),
            stats.line_count.to_string().cyan()
        ));

        // Show commit history
        if !commits.is_empty() {
            log(&format!("\n{}", "Commit history:".bright_magenta()));
            for commit in commits {
                log(&format!("{}", commit));
            }

            // Show commits by date (sorted)
            log(&format!("\n{}", "Commits by date:".bright_magenta()));
            let mut dates: Vec<(String, usize)> = stats
                .commits_by_date
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();

            // Sort by date (descending)
            dates.sort_by(|a, b| {
                NaiveDate::parse_from_str(&b.0, "%Y-%m-%d")
                    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                    .cmp(
                        &NaiveDate::parse_from_str(&a.0, "%Y-%m-%d")
                            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
                    )
            });

            for (date, count) in dates {
                log(&format!(
                    "  {} - {} {}",
                    date.bright_cyan(),
                    count,
                    "commits".green()
                ));
            }

            // Show file types
            if show_filetypes {
                if !stats.file_types.is_empty() {
                    log(&format!("\n{}", "File types:".bright_magenta()));
                    let mut types: Vec<(String, usize)> = stats
                        .file_types
                        .iter()
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();

                    // Sort by count (descending)
                    types.sort_by(|a, b| b.1.cmp(&a.1));

                    for (ext, count) in types {
                        log(&format!(
                            "  {} - {} {}",
                            ext.bright_yellow(),
                            count,
                            "files".green()
                        ));
                    }
                }
            }
        }
    }

    stats
}

/// Parses a time string in ISO format (HH:MM:SS+HHMM) and returns the hour and minute
fn parse_commit_time(time: &str) -> Option<(u32, u32)> {
    // Split on the timezone offset
    let time_part = time.split('+').next()?;
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;

    debug(&format!("Parsed commit time: {:02}:{:02}", hour, minute));

    Some((hour, minute))
}

/// Checks if a given time is within working hours
fn is_within_working_hours(
    hour: u32,
    minute: u32,
    start_hour: u32,
    start_min: u32,
    end_hour: u32,
    end_min: u32,
) -> bool {
    let commit_time = hour * 60 + minute;
    let start_time = start_hour * 60 + start_min;
    let end_time = end_hour * 60 + end_min;

    // If commit time is before start time, it's out of hours
    if commit_time < start_time {
        return false;
    }

    // If commit time is after end time, it's out of hours
    if commit_time > end_time {
        return false;
    }

    // Otherwise it's within working hours
    true
}

/// Creates a mapping between original repository paths from the config file
/// and their cleaned versions.
///
/// # Arguments
/// * `config` - The configuration structure
///
/// # Returns
/// * `HashMap<String, String>` - Map of original paths to cleaned paths
pub fn create_repo_path_map(config: &Config) -> HashMap<String, String> {
    let mut repo_path_map: HashMap<String, String> = HashMap::new();

    for (_, repos) in &config.repos {
        for repo_str in repos {
            let repo_info = parse_repo_string(repo_str);
            repo_path_map.insert(repo_str.clone(), repo_info.path);
        }
    }

    repo_path_map
}

/// Analyzes all categories defined in the configuration and returns their statistics.
///
/// # Arguments
/// * `config` - The configuration structure
/// * `repo_path_map` - Mapping of repository paths
/// * `author_filter` - Optional author name to filter commits
/// * `since` - Start date for analysis (YYYY-MM-DD format)
/// * `until` - End date for analysis (YYYY-MM-DD format)
/// * `active_only` - Whether to include only repositories with commits
/// * `show_repo_details` - Whether to show detailed repository information
/// * `show_filetypes` - Whether to analyze and show file type statistics
/// * `working_hours` - Optional working hours to filter out-of-hours commits
///
/// # Returns
/// * `(Vec<CategoryStats>, Vec<(String, RepoStats)>)` - Tuple containing:
///   - Vector of category statistics
///   - Vector of all repository statistics
pub fn analyze_all_categories(
    config: &Config,
    repo_path_map: &HashMap<String, String>,
    author_filter: &Option<String>,
    since: &str,
    until: &str,
    active_only: bool,
    show_repo_details: bool,
    show_filetypes: bool,
    working_hours: Option<(u32, u32, u32, u32)>,
) -> (Vec<CategoryStats>, Vec<(String, RepoStats)>) {
    let mut categories = Vec::new();
    let mut all_repo_stats = Vec::new();

    for (category_name, repos) in &config.repos {
        let mut category_stats = CategoryStats {
            name: category_name.to_string(),
            repos: Vec::new(),
            total: RepoStats::default(),
        };

        let mut category_repo_stats = Vec::new();

        for repo_str in repos {
            // Get the parsed path for this repository
            let repo_path = repo_path_map.get(repo_str).unwrap_or(repo_str);

            // Check if we've already analyzed this repo
            let repo_stats = analyze_repo(
                repo_path,
                author_filter,
                since,
                until,
                show_repo_details,
                show_filetypes,
                working_hours,
            );

            // Skip inactive repositories if active-only flag is set
            if active_only && !is_repo_active(&repo_stats) {
                continue;
            }

            category_stats
                .repos
                .push((repo_path.clone(), repo_stats.clone()));
            category_repo_stats.push(repo_stats.clone());
            all_repo_stats.push((repo_path.clone(), repo_stats));
        }

        // Aggregate statistics for this category
        category_stats.total = aggregate_stats(&category_repo_stats);
        categories.push(category_stats);
    }

    // Filter only active repositories if needed
    if active_only {
        all_repo_stats.retain(|(_, stats)| is_repo_active(stats));
    }

    (categories, all_repo_stats)
}

/// Analyzes all projects by grouping repositories with the same vanity name.
///
/// # Arguments
/// * `repo_infos` - Vector of repository information
/// * `repo_stats_map` - Map of repository paths to their statistics
/// * `author_filter` - Optional author name to filter commits
/// * `since` - Start date for analysis (YYYY-MM-DD format)
/// * `until` - End date for analysis (YYYY-MM-DD format)
/// * `active_only` - Whether to include only repositories with commits
/// * `show_repo_details` - Whether to show detailed repository information
/// * `show_filetypes` - Whether to analyze and show file type statistics
/// * `working_hours` - Optional working hours to filter out-of-hours commits
///
/// # Returns
/// * `Vec<ProjectStats>` - Vector of project statistics
pub fn analyze_all_projects(
    repo_infos: &[RepoInfo],
    repo_stats_map: &HashMap<String, RepoStats>,
    author_filter: &Option<String>,
    since: &str,
    until: &str,
    active_only: bool,
    show_repo_details: bool,
    show_filetypes: bool,
    working_hours: Option<(u32, u32, u32, u32)>,
) -> Vec<ProjectStats> {
    let grouped_repos = group_repos_by_vanity(repo_infos);
    let mut project_list = Vec::new();

    for (vanity_name, repo_group) in grouped_repos {
        debug(&format!("\nProcessing project: {}", vanity_name));
        let mut project_stats = ProjectStats {
            name: vanity_name,
            group: repo_group.first().and_then(|r| r.group.clone()),
            repos: Vec::new(),
            stats: RepoStats::default(),
        };

        let mut project_repo_stats = Vec::new();
        let mut active_repos_in_project = 0;

        for repo_info in repo_group {
            let repo_path = &repo_info.path;
            project_stats.repos.push(repo_path.clone());

            // Use already calculated statistics for this repo or analyze it
            let repo_stats = if let Some(stats) = repo_stats_map.get(repo_path) {
                stats.clone()
            } else {
                analyze_repo(
                    repo_path,
                    author_filter,
                    since,
                    until,
                    show_repo_details,
                    show_filetypes,
                    working_hours,
                )
            };

            debug(&format!(
                "  Repository: {} - {} commits",
                repo_path, repo_stats.commit_count
            ));

            // Skip inactive repositories if active-only flag is set
            if active_only && !is_repo_active(&repo_stats) {
                continue;
            }

            if is_repo_active(&repo_stats) {
                active_repos_in_project += 1;
            }

            project_repo_stats.push(repo_stats);
        }

        debug(&format!(
            "  Active repositories in project: {}",
            active_repos_in_project
        ));

        // Aggregate statistics for this project
        project_stats.stats = aggregate_stats(&project_repo_stats);
        project_list.push(project_stats);
    }

    project_list
}
