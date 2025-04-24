use crate::models::{CategoryStats, Config, ProjectStats, RepoInfo, RepoStats};
use crate::parser::{group_repos_by_vanity, parse_repo_string};
use crate::utils::{aggregate_stats, count_files_and_lines, debug, debug_git_command, log};
use chrono::NaiveDate;
use colored::*;
use std::collections::HashMap;
use std::process::Command;

/// Analizza un singolo repository e restituisce le statistiche
pub fn analyze_repo(
    repo: &str,
    author: &Option<String>,
    since: &str,
    until: &str,
    show_details: bool,
    show_filetypes: bool,
) -> RepoStats {
    let mut stats = RepoStats::default();

    // Get commit history
    let mut cmd = Command::new("git");
    cmd.args([
        "-C",
        repo,
        "log",
        "--pretty=format:%h %ad %s",
        "--date=short",
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
    let commits: Vec<&str> = stdout.lines().collect();

    stats.commit_count = commits.len();

    // Mostra informazioni sui commit trovati
    debug(&format!(
        "Found {} commits in repository {}",
        commits.len(),
        repo
    ));

    // Parse commits by date
    for commit in &commits {
        if let Some(date_part) = commit.split_whitespace().nth(1) {
            *stats
                .commits_by_date
                .entry(date_part.to_string())
                .or_insert(0) += 1;
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

/// Crea una mappa di corrispondenza tra percorsi originali del config file e percorsi puliti
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

/// Analizza tutte le categorie dai dati di configurazione
pub fn analyze_all_categories(
    config: &Config,
    repo_path_map: &HashMap<String, String>,
    author_filter: &Option<String>,
    since: &str,
    until: &str,
    active_only: bool,
    show_repo_details: bool,
    show_filetypes: bool,
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
            // Ottieni il percorso analizzato per questo repository
            let repo_path = repo_path_map.get(repo_str).unwrap_or(repo_str);

            // Controlla se abbiamo già analizzato questo repo
            let repo_stats = analyze_repo(
                repo_path,
                author_filter,
                since,
                until,
                show_repo_details,
                show_filetypes,
            );

            // Salta i repository inattivi se è impostato il flag active-only
            if active_only && repo_stats.commit_count == 0 {
                continue;
            }

            category_stats
                .repos
                .push((repo_path.clone(), repo_stats.clone()));
            category_repo_stats.push(repo_stats.clone());
            all_repo_stats.push((repo_path.clone(), repo_stats));
        }

        // Aggrega le statistiche per questa categoria
        category_stats.total = aggregate_stats(&category_repo_stats);
        categories.push(category_stats);
    }

    // Filtra solo i repository attivi se necessario
    if active_only {
        all_repo_stats.retain(|(_, stats)| stats.commit_count > 0);
    }

    (categories, all_repo_stats)
}

/// Analizza progetti raggruppando i repository per nome vanity
pub fn analyze_all_projects(
    repo_infos: &[RepoInfo],
    repo_stats_map: &HashMap<String, RepoStats>,
    author_filter: &Option<String>,
    since: &str,
    until: &str,
    active_only: bool,
    show_repo_details: bool,
    show_filetypes: bool,
) -> Vec<ProjectStats> {
    let grouped_repos = group_repos_by_vanity(repo_infos);
    let mut project_list = Vec::new();

    for (vanity_name, repo_group) in grouped_repos {
        let mut project_stats = ProjectStats {
            name: vanity_name,
            group: repo_group.first().and_then(|r| r.group.clone()),
            repos: Vec::new(),
            stats: RepoStats::default(),
        };

        let mut project_repo_stats = Vec::new();

        for repo_info in repo_group {
            let repo_path = &repo_info.path;
            project_stats.repos.push(repo_path.clone());

            // Usa le statistiche già calcolate per questo repo o analizzalo
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
                )
            };

            // Salta i repository inattivi se è impostato il flag active-only
            if active_only && repo_stats.commit_count == 0 {
                continue;
            }

            project_repo_stats.push(repo_stats);

            // Debug per il percorso del repository
            debug(&format!(
                "Repository path: '{}', path per git: '{}'",
                repo_path, repo_path
            ));
        }

        // Aggrega le statistiche per questo progetto
        project_stats.stats = aggregate_stats(&project_repo_stats);
        project_list.push(project_stats);
    }

    project_list
}
