use crate::models::{Config, RepoInfo};
use crate::parser::parse_repo_string;
use colored::*;
use std::fs;
use std::path::Path;
use std::process;
use std::collections::HashMap;

pub fn push_to_empty_config(dir: &str) -> Config {
    let mut repos = HashMap::new();
    let dir_to_string = format!("{} [Uncategorized][Unnamed]", dir);
    repos.insert("Uncategorized".to_string(), vec![dir_to_string]);

    Config {
        author: None,
        repos: repos,
    }
}

pub fn load_config(path: &str) -> Config {
    if !Path::new(path).exists() {
        eprintln!(
            "{} '{}' {}",
            "Error:".bright_red(),
            path.yellow(),
            "file not found. Please create a configuration file or specify a valid path."
                .bright_red()
        );
        process::exit(1);
    }

    let contents = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!(
                "{} '{}': {}",
                "Error reading config file".bright_red(),
                path.yellow(),
                err.to_string().bright_red()
            );
            process::exit(1);
        }
    };

    match serde_yaml::from_str(&contents) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "{} '{}': {}",
                "Invalid YAML format in config file".bright_red(),
                path.yellow(),
                err.to_string().bright_red()
            );
            process::exit(1);
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
