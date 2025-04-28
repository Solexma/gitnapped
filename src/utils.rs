use crate::models::RepoStats;
use chrono::{DateTime, Duration, Local};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

static mut DEBUG_MODE: bool = false;
static mut SILENT_MODE: bool = false;

/// Initializes the debug mode for the application.
/// When enabled, detailed debug information will be printed during execution.
///
/// # Arguments
/// * `debug` - A boolean flag to enable or disable debug mode
pub fn init_debug_mode(debug: bool) {
    unsafe {
        DEBUG_MODE = debug;
    }
}

/// Initializes the silent mode for the application.
/// When enabled, no output will be printed to the console.
///
/// # Arguments
/// * `silent` - A boolean flag to enable or disable silent mode
pub fn init_silent_mode(silent: bool) {
    unsafe {
        SILENT_MODE = silent;
    }
}

/// Prints a debug message if debug mode is enabled.
///
/// # Arguments
/// * `message` - The debug message to print
pub fn debug(message: &str) {
    unsafe {
        if DEBUG_MODE {
            println!("DEBUG: {}", message);
        }
    }
}

/// Prints a log message if silent mode is not enabled.
///
/// # Arguments
/// * `message` - The message to log
pub fn log(message: &str) {
    unsafe {
        if !SILENT_MODE {
            println!("{}", message);
        }
    }
}

/// Parses a relative time period string and returns a DateTime object.
/// Supports the following formats:
/// - Y: Years (e.g., "2Y" for 2 years)
/// - M: Months (e.g., "6M" for 6 months)
/// - W: Weeks (e.g., "2W" for 2 weeks)
/// - D: Days (e.g., "5D" for 5 days)
/// - H: Hours (e.g., "12H" for 12 hours)
///
/// # Arguments
/// * `period` - A string in the format "number\[YMWDH\]"
///
/// # Returns
/// * `Option<DateTime<Utc>>` - The calculated DateTime if parsing succeeds, None otherwise
///
/// # Examples
/// ```
/// let six_months_ago = parse_period("6M");
/// let two_years_ago = parse_period("2Y");
/// let five_days_ago = parse_period("5D");
/// ```
pub fn parse_period(period: &str) -> Option<DateTime<Local>> {
    let re = Regex::new(r"^(\d+)([YMWDH])$").unwrap();

    if let Some(caps) = re.captures(period) {
        let amount: i64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str();

        let now = Local::now();

        match unit {
            "Y" => Some((now - Duration::days(amount * 365)).into()),
            "M" => Some((now - Duration::days(amount * 30)).into()),
            "W" => Some((now - Duration::days(amount * 7)).into()),
            "D" => Some((now - Duration::days(amount)).into()),
            "H" => Some((now - Duration::hours(amount)).into()),
            _ => None,
        }
    } else {
        None
    }
}

/// Gets the file extension from a file path.
///
/// # Arguments
/// * `file_path` - The path to the file
///
/// # Returns
/// * `String` - The file extension or "none" if no extension is found
pub fn get_file_extension(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('.').collect();
    if parts.len() > 1 {
        parts.last().unwrap().to_string()
    } else {
        "none".to_string()
    }
}

/// Counts the number of files and lines in a Git repository.
///
/// # Arguments
/// * `repo` - The path to the Git repository
///
/// # Returns
/// * `(usize, usize, HashMap<String, usize>)` - A tuple containing:
///   - Number of files
///   - Total number of lines
///   - Map of file extensions to their counts
pub fn count_files_and_lines(repo: &str) -> (usize, usize, HashMap<String, usize>) {
    // Get all files tracked by git
    debug(&format!("Counting files and lines in repo: {}", repo));

    let output = Command::new("git")
        .args(["-C", repo, "ls-files"])
        .output()
        .expect("Failed to run git ls-files");

    let files_output = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = files_output.lines().collect();
    let file_count = files.len();

    debug(&format!("Found {} tracked files in repo", file_count));

    // Count lines in all tracked files and track file types
    let mut total_lines = 0;
    let mut file_types = HashMap::new();
    let mut files_read = 0;
    let mut files_failed = 0;

    for file in files {
        let file_path = format!("{}/{}", repo, file);
        let extension = get_file_extension(file);
        *file_types.entry(extension).or_insert(0) += 1;

        if let Ok(content) = fs::read_to_string(&file_path) {
            let line_count = content.lines().count();
            total_lines += line_count;
            files_read += 1;
        } else {
            files_failed += 1;
        }
    }

    debug(&format!(
        "Successfully read {} files, failed to read {} files",
        files_read, files_failed
    ));
    debug(&format!("Total lines: {}", total_lines));

    (file_count, total_lines, file_types)
}

/// Gets the day with the maximum number of commits from a commit history.
///
/// # Arguments
/// * `commits_by_date` - A HashMap mapping dates to commit counts
///
/// # Returns
/// * `Option<(String, usize)>` - The date and count of the most active day, if any
pub fn get_max_commit_day(commits_by_date: &HashMap<String, usize>) -> Option<(String, usize)> {
    if commits_by_date.is_empty() {
        return None;
    }

    let mut max_date = String::new();
    let mut max_count = 0;

    for (date, count) in commits_by_date {
        if *count > max_count {
            max_count = *count;
            max_date = date.clone();
        }
    }

    Some((max_date, max_count))
}

/// Aggregates multiple RepoStats into a single RepoStats object.
///
/// # Arguments
/// * `stats_vec` - A slice of RepoStats to aggregate
///
/// # Returns
/// * `RepoStats` - The aggregated statistics
pub fn aggregate_stats(stats_vec: &[RepoStats]) -> RepoStats {
    let mut aggregated = RepoStats::default();

    for stats in stats_vec {
        aggregated.commit_count += stats.commit_count;
        aggregated.file_count += stats.file_count;
        aggregated.line_count += stats.line_count;

        // Merge commits by date
        for (date, count) in &stats.commits_by_date {
            *aggregated.commits_by_date.entry(date.clone()).or_insert(0) += count;
        }

        // Merge file types
        for (ext, count) in &stats.file_types {
            *aggregated.file_types.entry(ext.clone()).or_insert(0) += count;
        }
    }

    aggregated
}

/// Prints debug information about a Git command execution.
///
/// # Arguments
/// * `repo` - The repository path where the command was executed
/// * `cmd` - The Command object representing the Git command
/// * `output` - The output from the command execution
pub fn debug_git_command(repo: &str, cmd: &Command, output: &std::process::Output) {
    unsafe {
        if !DEBUG_MODE {
            return;
        }
    }

    println!("==== Git Command Debug ====");
    println!("Repository: {}", repo);
    println!("Command: {:?}", cmd);
    println!("Exit status: {}", output.status);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output lines: {}", stdout.lines().count());

        if stdout.lines().count() > 0 {
            println!("First few lines of output:");
            for line in stdout.lines().take(5) {
                println!("  > {}", line);
            }
        } else {
            println!("No output received");
        }
    } else {
        println!("Command failed");
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    println!("==========================");
}
