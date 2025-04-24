use crate::models::CategoryStats;
use crate::models::ProjectStats;
use crate::models::RepoStats;
use crate::utils::get_max_commit_day;
use crate::utils::log;
use colored::*;
use std::collections::HashMap;

pub fn print_category_summary(categories: &[CategoryStats], sort_by: &str, show_filetypes: bool) {
    log(&format!("\n{}", "Category Statistics:".bright_green()));

    for category in categories {
        if category.repos.is_empty() {
            continue;
        }

        log(&format!(
            "\n{} {}",
            "Category:".bright_yellow(),
            category.name.bright_cyan()
        ));

        // Count active repositories in the category
        let active_repos_count = category
            .repos
            .iter()
            .filter(|(_, stats)| stats.commit_count > 0)
            .count();
        log(&format!(
            "{}: {}",
            "Active repositories".yellow(),
            active_repos_count.to_string().cyan()
        ));

        log(&format!(
            "{}: {}",
            "Total commits".yellow(),
            category.total.commit_count.to_string().cyan()
        ));
        log(&format!(
            "{}: {}",
            "Total files".yellow(),
            category.total.file_count.to_string().cyan()
        ));
        log(&format!(
            "{}: {}",
            "Total lines of code".yellow(),
            category.total.line_count.to_string().cyan()
        ));

        // Show file types for this category if requested
        if show_filetypes && !category.total.file_types.is_empty() {
            log(&format!("  {}", "File types:".bright_magenta()));
            let mut types: Vec<(String, usize)> = category
                .total
                .file_types
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();

            // Sort by count (descending)
            types.sort_by(|a, b| b.1.cmp(&a.1));

            // Show top 5 file types per category
            for (ext, count) in types.iter().take(5) {
                log(&format!(
                    "    {} - {} {}",
                    ext.bright_yellow(),
                    count,
                    "files".green()
                ));
            }
        }

        // Sort repos by criterion
        let mut sorted_repos = category.repos.clone();
        match sort_by {
            "commits" => sorted_repos.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count)),
            "files" => sorted_repos.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count)),
            "lines" => sorted_repos.sort_by(|a, b| b.1.line_count.cmp(&a.1.line_count)),
            _ => {}
        }

        // Show top repos in this category
        if !sorted_repos.is_empty() {
            log(&format!(
                "  {} (sorted by {})",
                "Top repositories:".bright_blue(),
                sort_by
            ));
            for (i, (repo, stats)) in sorted_repos.iter().enumerate().take(3) {
                if stats.commit_count > 0 || sort_by != "commits" {
                    log(&format!(
                        "   {}. {} - {} commits, {} files, {} lines",
                        (i + 1).to_string().bright_yellow(),
                        repo.split('/').last().unwrap_or(repo).green(),
                        stats.commit_count.to_string().cyan(),
                        stats.file_count.to_string().blue(),
                        stats.line_count.to_string().magenta()
                    ));
                }
            }
        }
    }
}

pub fn print_projects_summary(
    projects: &[ProjectStats],
    sort_by: &str,
    show_filetypes: bool,
    show_repo_details: bool,
) {
    log(&format!("\n{}", "Projects Statistics:".bright_green()));

    // Group by group if available
    let mut by_group: HashMap<Option<String>, Vec<&ProjectStats>> = HashMap::new();
    for project in projects {
        by_group
            .entry(project.group.clone())
            .or_insert_with(Vec::new)
            .push(project);
    }

    // For each group (or no group)
    for (group, group_projects) in by_group {
        if let Some(group_name) = &group {
            log(&format!(
                "\n{} {}",
                "Group:".bright_yellow(),
                group_name.bright_cyan()
            ));
        } else {
            log(&format!("\n{}", "Ungrouped Projects:".bright_yellow()));
        }

        // Sort projects in group based on criterion
        let mut sorted_projects = group_projects.clone();
        match sort_by {
            "commits" => {
                sorted_projects.sort_by(|a, b| b.stats.commit_count.cmp(&a.stats.commit_count))
            }
            "files" => sorted_projects.sort_by(|a, b| b.stats.file_count.cmp(&a.stats.file_count)),
            "lines" => sorted_projects.sort_by(|a, b| b.stats.line_count.cmp(&a.stats.line_count)),
            _ => {}
        }

        // Print statistics for each project
        for (i, project) in sorted_projects.iter().enumerate() {
            log(&format!(
                "{}. {} - {} commits, {} files, {} lines (from {} repos)",
                (i + 1).to_string().bright_yellow(),
                project.name.green(),
                project.stats.commit_count.to_string().cyan(),
                project.stats.file_count.to_string().blue(),
                project.stats.line_count.to_string().magenta(),
                project.repos.len().to_string().yellow()
            ));

            // If requested, show the repositories included in this project
            if show_repo_details {
                for repo_path in &project.repos {
                    log(&format!("   • {}", repo_path));
                }
            }

            // If requested, show the file types
            if show_filetypes && !project.stats.file_types.is_empty() {
                log(&format!("   {}", "File types:".bright_magenta()));
                let mut types: Vec<(String, usize)> = project
                    .stats
                    .file_types
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();

                // Sort by count (descending)
                types.sort_by(|a, b| b.1.cmp(&a.1));

                // Show top 5 file types per project
                for (ext, count) in types.iter().take(5) {
                    log(&format!(
                        "     {} - {} {}",
                        ext.bright_yellow(),
                        count,
                        "files".green()
                    ));
                }
            }
        }
    }
}

pub fn print_most_active_day(commits_by_date: &HashMap<String, usize>) {
    if let Some((max_date, max_count)) = get_max_commit_day(commits_by_date) {
        log(&format!(
            "\n{} {} ({} {})",
            "Most active day:".bright_magenta(),
            max_date.bright_cyan(),
            max_count.to_string(),
            "commits".green()
        ));
    }
}

pub fn print_total_stats(
    stats: &RepoStats,
    active_count: usize,
    entity_name: &str,
    show_filetypes: bool,
    show_most_active: bool,
) {
    log(&format!(
        "\n{}",
        format!("Total Stats Across All {}:", entity_name).bright_green()
    ));
    log(&format!(
        "{}: {}",
        format!("Active {}", entity_name).yellow(),
        active_count.to_string().cyan()
    ));
    log(&format!(
        "{}: {}",
        "Total commits".yellow(),
        stats.commit_count.to_string().cyan()
    ));
    log(&format!(
        "{}: {}",
        "Total files".yellow(),
        stats.file_count.to_string().cyan()
    ));
    log(&format!(
        "{}: {}",
        "Total lines of code".yellow(),
        stats.line_count.to_string().cyan()
    ));

    if show_most_active {
        print_most_active_day(&stats.commits_by_date);
    }

    // Show total file types if requested
    if show_filetypes && !stats.file_types.is_empty() {
        log(&format!(
            "\n{}",
            format!("File types across all {}:", entity_name).bright_magenta()
        ));
        let mut types: Vec<(String, usize)> = stats
            .file_types
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Sort by count (descending)
        types.sort_by(|a, b| b.1.cmp(&a.1));

        // Show top 10 file types
        for (ext, count) in types.iter().take(10) {
            log(&format!(
                "  {} - {} {}",
                ext.bright_yellow(),
                count,
                "files".green()
            ));
        }
    }
}
