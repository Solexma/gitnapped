use crate::models::{Config, RepoInfo};
use crate::parser::parse_repo_string;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

fn is_git_repository(dir: &str) -> bool {
    let output = Command::new("git")
        .args(["-C", dir, "rev-parse", "--is-inside-work-tree"])
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn push_to_empty_config(dir: &str) -> Result<Config, String> {
    if !is_git_repository(dir) {
        return Err(format!(
            "'{}' {}",
            dir.yellow(),
            "is not a Git repository. Please provide a valid Git repository path.".bright_red()
        ));
    }

    let mut repos = HashMap::new();
    let dir_to_string = format!("{} [Uncategorized][Unnamed]", dir);
    repos.insert("Uncategorized".to_string(), vec![dir_to_string]);

    Ok(Config {
        author: None,
        repos: repos,
    })
}

pub fn load_config(path: &str) -> Result<Config, String> {
    if !Path::new(path).exists() {
        return Err(format!("Config file '{}' not found", path));
    }

    let contents = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            return Err(format!("Error reading config file '{}': {}", path, err));
        }
    };

    match serde_yaml::from_str(&contents) {
        Ok(config) => Ok(config),
        Err(err) => {
            Err(format!("Invalid YAML format in config file '{}': {}", path, err))
        }
    }
}

pub fn parse_repos_from_config(config: &Config) -> Vec<RepoInfo> {
    // On an empty config, we return an empty vector
    if config.repos.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();

    for (_, repos) in &config.repos {
        for repo_str in repos {
            let repo_info = parse_repo_string(repo_str);
            result.push(repo_info);
        }
    }

    result
}
