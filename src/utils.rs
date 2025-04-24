use crate::models::RepoStats;
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

static mut DEBUG_MODE: bool = false;
static mut SILENT_MODE: bool = false;

pub fn init_debug_mode(debug: bool) {
    unsafe {
        DEBUG_MODE = debug;
    }
}

pub fn init_silent_mode(silent: bool) {
    unsafe {
        SILENT_MODE = silent;
    }
}

pub fn debug(message: &str) {
    unsafe {
        if DEBUG_MODE {
            println!("DEBUG: {}", message);
        }
    }
}

pub fn log(message: &str) {
    unsafe {
        if !SILENT_MODE {
            println!("{}", message);
        }
    }
}
// Parses a relative time period string like "6M", "2Y", "5D", "12H" and returns a DateTime
pub fn parse_period(period: &str) -> Option<DateTime<Utc>> {
    let re = Regex::new(r"^(\d+)([YMWDH])$").unwrap();

    if let Some(caps) = re.captures(period) {
        let amount: i64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str();

        let now = Utc::now();

        match unit {
            "Y" => Some(now - Duration::days(amount * 365)),
            "M" => Some(now - Duration::days(amount * 30)),
            "W" => Some(now - Duration::days(amount * 7)),
            "D" => Some(now - Duration::days(amount)),
            "H" => Some(now - Duration::hours(amount)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn get_file_extension(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('.').collect();
    if parts.len() > 1 {
        parts.last().unwrap().to_string()
    } else {
        "none".to_string()
    }
}

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
