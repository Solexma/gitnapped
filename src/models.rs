use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub author: Option<String>,
    #[serde(default)]
    pub repos: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone)]
pub struct RepoStats {
    pub commit_count: usize,
    pub file_count: usize,
    pub line_count: usize,
    pub commits_by_date: HashMap<String, usize>,
    pub file_types: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub path: String,
    pub group: Option<String>,
    pub vanity_name: String,
}

#[derive(Debug, Default)]
pub struct CategoryStats {
    pub name: String,
    pub repos: Vec<(String, RepoStats)>,
    pub total: RepoStats,
}

#[derive(Debug, Default)]
pub struct ProjectStats {
    pub name: String,
    pub group: Option<String>,
    pub repos: Vec<String>,
    pub stats: RepoStats,
}
