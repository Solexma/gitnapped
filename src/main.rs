mod analyzer;
mod config;
mod display;
mod models;
mod parser;
mod utils;

use chrono::{Duration, Local};
use clap::{Arg, Command as ClapCommand};
use colored::*;
use std::collections::HashMap;

use analyzer::{analyze_all_categories, analyze_all_projects, create_repo_path_map};
use config::{load_config, parse_repos_from_config, push_to_empty_config};
use display::{print_category_summary, print_projects_summary, print_total_stats};
use models::RepoStats;
use utils::{aggregate_stats, debug, init_debug_mode, init_silent_mode, is_repo_active, log, parse_period};

fn main() {
    let matches = ClapCommand::new("gitnapped")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Find out why you didn't sleep — commit history across repos")
        .help_template(
            "{before-help}{name} {version} - by {author}\n{about}\n\n{usage-heading}\n  {usage}\n\n{all-args}{after-help}"
        )
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file, if not provided, the app will look for a 'gitnapped.yaml'"))
        .arg(Arg::new("dir")
            .short('d')
            .long("dir")
            .value_name("DIRECTORY")
            .help("Sets a custom directory to analyze, if not provided, the app will look for a 'gitnapped.yaml' in the current directory"))
        .arg(Arg::new("since")
            .short('s')
            .long("since")
            .help("Start date for analysis (YYYY-MM-DD)"))
        .arg(Arg::new("until")
            .short('u')
            .long("until")
            .help("End date for analysis (YYYY-MM-DD)"))
        .arg(Arg::new("period")
            .short('p')
            .long("period")
            .value_name("PERIOD")
            .help("Relative time period (e.g., 6M, 2Y, 5D, 12H)"))
        .arg(Arg::new("active-only")
            .long("active-only")
            .help("Show only repositories with commits in the period")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("sort-by")
            .long("sort-by")
            .value_name("FIELD")
            .help("Sort repositories by: commits, files, lines")
            .default_value("commits"))
        .arg(Arg::new("categories")
            .long("categories")
            .help("Show statistics by category as per: [Category][Vanity Name]")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("repo-details")
            .long("repo-details")
            .help("Show detailed information for each repository")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("filetypes")
            .long("filetypes")
            .help("Show file types used in the repositories")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("author")
            .short('a')
            .long("author")
            .value_name("AUTHOR")
            .help("Filter commits by specific author (overrides config file)"))
        .arg(Arg::new("all-authors")
            .long("all-authors")
            .help("Include commits from all authors (ignores author filter)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("projects")
            .long("projects")
            .help("Group repositories by vanity name as per: [Category][Vanity Name]")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("most-active-day")
            .long("most-active-day")
            .help("Show the most active day")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("silent")
            .long("silent")
            .help("Silent mode, no output")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("json")
            .long("json")
            .help("Output in JSON format")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("debug")
            .long("debug")
            .help("Enable debug messages")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let default_dir = String::from("");
    let dir = matches.get_one::<String>("dir").unwrap_or(&default_dir);
    let since: String;
    let until: String;
    let active_only = matches.get_flag("active-only");
    let default_sort = String::from("commits");
    let sort_by = matches
        .get_one::<String>("sort-by")
        .unwrap_or(&default_sort);
    let by_categories = matches.get_flag("categories");
    let by_projects = matches.get_flag("projects");
    let show_repo_details = matches.get_flag("repo-details");
    let show_filetypes = matches.get_flag("filetypes");
    let show_most_active_day = matches.get_flag("most-active-day");
    let debug_mode = matches.get_flag("debug");
    let silent_mode = matches.get_flag("silent");

    let mut mandatory_author = false; // An author is mandatory if a directory is provided
    let mut bypass_config = false; // Config is bypassed if a directory is provided

    init_debug_mode(debug_mode);
    init_silent_mode(silent_mode);

    // If a directory is provided, we need to
    if !dir.is_empty() {
        debug(&format!("Using directory: {}", dir));
        mandatory_author = true;
        bypass_config = true;
    }

    // Initialize config variable outside of if/else blocks
    let config = if !bypass_config {
        let default_config = String::from("gitnapped.yaml");
        let config_path = matches
            .get_one::<String>("config")
            .unwrap_or(&default_config);
        load_config(config_path)
    } else {
        debug(&format!("Loading empty config"));
        push_to_empty_config(&dir)
    };

    let config_author = config.author.clone();
    let cli_author = matches.get_one::<String>("author").cloned();
    let mut all_authors = matches.get_flag("all-authors");

    if mandatory_author {
        if cli_author.is_none() {
            log(&format!(
                "{}",
                "Warning: No author provided, assuming all-authors mode".bright_yellow()
            ));
            all_authors = true;
        }
    }

    // Priority: 1) all-authors flag, 2) author CLI arg, 3) config file author
    let author_filter = if all_authors {
        None // Don't filter by author, show commits from everyone
    } else if let Some(a) = cli_author {
        Some(a) // Use the author specified on the command line
    } else {
        config_author // Use the author from the config file (could be None)
    };

    // Display information about the author name being used as a filter
    if let Some(a) = &author_filter {
        log(&format!(
            "{}: {}",
            "Author filter".bright_yellow(),
            a.green()
        ));
    } else {
        log(&format!(
            "{}",
            "Showing commits from all authors".bright_yellow()
        ));
    }

    if let Some(period) = matches.get_one::<String>("period") {
        // Parse relative time period
        if let Some(start_date) = parse_period(period) {
            let now = Local::now();
            since = start_date.format("%Y-%m-%d %H:%M:%S").to_string();
            until = now.format("%Y-%m-%d %H:%M:%S").to_string();

            debug(&format!(
                "Using relative period '{}': from {} to {}",
                period, since, until
            ));
        } else {
            // If period format is invalid, fallback to defaults
            log(&format!(
                "{} '{}' - {}",
                "Warning: Invalid period format".bright_red(),
                period,
                "Expected format like 6M, 2Y, 5D, 12H".yellow()
            ));

            let now = Local::now();
            since = matches
                .get_one::<String>("since")
                .cloned()
                .unwrap_or_else(|| {
                    (now - Duration::days(1))
                        .format("%Y-%m-%d")
                        .to_string()
                });

            until = matches
                .get_one::<String>("until")
                .cloned()
                .unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
        }
    } else {
        // Standard behavior using since/until parameters
        let now = Local::now();
        since = matches
            .get_one::<String>("since")
            .cloned()
            .unwrap_or_else(|| {
                (now - Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string()
            });

        until = matches
            .get_one::<String>("until")
            .cloned()
            .unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
    }

    log(&format!(
        "{} {} {} {}",
        "Analyzing repos from".bright_yellow(),
        since.bright_cyan(),
        "to".bright_yellow(),
        until.bright_cyan()
    ));

    // Parse repository info to use for both categories and projects
    let repo_infos = parse_repos_from_config(&config);

    // Create a mapping between original strings and clean paths
    let repo_path_map = create_repo_path_map(&config);

    // Analyze all categories
    let (categories, all_repo_stats) = analyze_all_categories(
        &config,
        &repo_path_map,
        &author_filter,
        &since,
        &until,
        active_only,
        show_repo_details,
        show_filetypes,
    );

    // Create a map of repo path to its statistics for reuse
    let mut repo_stats_map: HashMap<String, RepoStats> = HashMap::new();
    for (path, stats) in &all_repo_stats {
        repo_stats_map.insert(path.clone(), stats.clone());
    }

    // Extract all repo stats into a vector for aggregation
    let repo_stats_only: Vec<RepoStats> = all_repo_stats
        .iter()
        .map(|(_, stats)| stats.clone())
        .collect();

    // Aggregate stats for all repositories
    let mut total_stats = aggregate_stats(&repo_stats_only);

    // Calculate the total number of active repositories
    let mut total_active_repos = all_repo_stats
        .iter()
        .filter(|(_, stats)| is_repo_active(stats))
        .count();

    // Handle projects if requested
    let projects = if by_projects {
        // Analyze projects using the repo_stats_map for efficiency
        let project_list = analyze_all_projects(
            &repo_infos,
            &repo_stats_map,
            &author_filter,
            &since,
            &until,
            active_only,
            show_repo_details,
            show_filetypes,
        );

        // Debug: Print all projects and their active status
        debug("\nProjects and their active status:");
        for project in &project_list {
            debug(&format!(
                "Project: {} - {} commits, {} active repos",
                project.name,
                project.stats.commit_count,
                project.repos.len()
            ));
        }

        Some(project_list)
    } else {
        None
    };

    // Print appropriate output based on flags
    if by_categories {
        print_category_summary(&categories, sort_by, show_filetypes);
    } else if let Some(project_list) = &projects {
        // Print project statistics
        print_projects_summary(project_list, sort_by, show_filetypes, show_repo_details);

        // Calculate overall stats for projects
        total_active_repos = project_list
            .iter()
            .flat_map(|project| project.repos.iter())
            .filter(|repo_path| {
                if let Some(stats) = repo_stats_map.get(*repo_path) {
                    is_repo_active(stats)
                } else {
                    false
                }
            })
            .count();

        // Extract project stats into a vector for aggregation
        let project_stats: Vec<RepoStats> = project_list
            .iter()
            .map(|project| project.stats.clone())
            .collect();

        // Aggregate stats for all projects
        total_stats = aggregate_stats(&project_stats);
    } else {
        // Otherwise sort and print overall top repos
        if !all_repo_stats.is_empty() {
            let mut sorted_repos = all_repo_stats.clone();
            match sort_by.as_str() {
                "commits" => sorted_repos.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count)),
                "files" => sorted_repos.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count)),
                "lines" => sorted_repos.sort_by(|a, b| b.1.line_count.cmp(&a.1.line_count)),
                _ => {}
            }
            if sorted_repos.len() > 1 {
                log(&format!(
                    "\n{} (sorted by {})",
                    "Most Active Repositories".bright_green(),
                    sort_by
                ));
                for (i, (repo, stats)) in sorted_repos.iter().enumerate().take(5) {
                    if is_repo_active(stats) || sort_by != "commits" {
                        log(&format!(
                            "{}. {} - {} commits, {} files, {} lines",
                            (i + 1).to_string().bright_yellow(),
                            repo.green(),
                            stats.commit_count.to_string().cyan(),
                            stats.file_count.to_string().blue(),
                            stats.line_count.to_string().magenta()
                        ));
                    }
                }
            }
        }
    }

    // Determine what type of items we're summarizing
    let item_type = if by_projects {
        "Projects"
    } else {
        "Repositories"
    };

    // Print totals once at the end
    print_total_stats(
        &total_stats,
        total_active_repos,
        item_type,
        show_filetypes,
        show_most_active_day || (!by_categories && !by_projects),
    );
}
