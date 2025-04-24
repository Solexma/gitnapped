use crate::models::RepoInfo;
use crate::utils::debug;
use std::collections::HashMap;

/// Parse a repo string in the format: "path [label1][label2]" or "path [label]"
pub fn parse_repo_string(input: &str) -> RepoInfo {
    debug(&format!("Parsing repo string: '{}'", input));

    let parts: Vec<&str> = input.split('[').collect();
    if parts.is_empty() {
        debug("Empty input string");
        return RepoInfo {
            path: input.trim().to_string(),
            group: None,
            vanity_name: input.trim().to_string(),
        };
    }

    let path = parts[0].trim().to_string();

    let mut labels = Vec::new();
    for part in parts.iter().skip(1) {
        if let Some(label) = part.split(']').next() {
            labels.push(label.trim().to_string());
        }
    }

    debug(&format!("Extracted path: '{}', labels: {:?}", path, labels));

    match labels.len() {
        2 => RepoInfo {
            path,
            group: Some(labels[0].clone()),
            vanity_name: labels[1].clone(),
        },
        1 => RepoInfo {
            path,
            group: None,
            vanity_name: labels[0].clone(),
        },
        _ => RepoInfo {
            path: path.clone(),
            group: None,
            vanity_name: path,
        },
    }
}

/// Utility function to group repositories by vanity name
pub fn group_repos_by_vanity(repos: &[RepoInfo]) -> HashMap<String, Vec<RepoInfo>> {
    let mut grouped = HashMap::new();

    for repo in repos {
        grouped
            .entry(repo.vanity_name.clone())
            .or_insert_with(Vec::new)
            .push(repo.clone());
    }

    grouped
}
