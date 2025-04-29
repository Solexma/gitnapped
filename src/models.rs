use serde::Deserialize;
use std::collections::HashMap;

/// Configuration structure for the application.
/// This structure represents the contents of the gitnapped.yaml configuration file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Optional author name to filter commits
    pub author: Option<String>,
    /// Map of category names to lists of repository paths
    #[serde(default)]
    pub repos: HashMap<String, Vec<String>>,
}

/// Statistics for a single repository or aggregated repositories.
#[derive(Debug, Default, Clone)]
pub struct RepoStats {
    /// Total number of commits
    pub commit_count: usize,
    /// Number of commits made outside working hours
    pub out_of_hours_commits: usize,
    /// Total number of files
    pub file_count: usize,
    /// Total number of lines of code
    pub line_count: usize,
    /// Map of dates to number of commits on that date
    pub commits_by_date: HashMap<String, usize>,
    /// Map of file extensions to number of files with that extension
    pub file_types: HashMap<String, usize>,
}

/// Information about a repository, including its path and categorization.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    /// Path to the repository
    pub path: String,
    /// Optional group/category name
    pub group: Option<String>,
    /// Display name for the repository
    pub vanity_name: String,
}

/// Statistics for a category of repositories.
#[derive(Debug, Default)]
pub struct CategoryStats {
    /// Name of the category
    pub name: String,
    /// List of repositories in this category with their stats
    pub repos: Vec<(String, RepoStats)>,
    /// Aggregated stats for all repositories in this category
    pub total: RepoStats,
}

/// Statistics for a project (group of related repositories).
#[derive(Debug, Default)]
pub struct ProjectStats {
    /// Name of the project
    pub name: String,
    /// Optional group/category this project belongs to
    pub group: Option<String>,
    /// List of repository paths in this project
    pub repos: Vec<String>,
    /// Aggregated stats for all repositories in this project
    pub stats: RepoStats,
}
