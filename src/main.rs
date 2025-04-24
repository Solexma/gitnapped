mod config;
mod models;
mod analyzer;
mod display;
mod utils;

use clap::{Arg, Command as ClapCommand};
use chrono::{Utc, Duration};
use colored::*;

use config::load_config;
use models::RepoStats;
use analyzer::analyze_category;
use display::{print_category_summary, get_max_commit_day};

fn main() {
    let matches = ClapCommand::new("gitnapped")
        .version("0.1")
        .author("Marco Orlandin")
        .about("Find out why you didn't sleep — commit history across repos")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file"))
        .arg(Arg::new("since")
            .long("since")
            .help("Start date for analysis"))
        .arg(Arg::new("until")
            .long("until")
            .help("End date for analysis"))
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
            .help("Show statistics by category")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("repo-details")
            .long("repo-details")
            .help("Show detailed information for each repository")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("filetypes")
            .long("filetypes")
            .help("Show file types")
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
        .get_matches();

    let default_config = String::from("gitnapped.yaml");
    let config_path = matches.get_one::<String>("config").unwrap_or(&default_config);
    let config = load_config(config_path);

    let since = matches.get_one::<String>("since")
        .cloned()
        .unwrap_or_else(|| (Utc::now() - Duration::days(1)).format("%Y-%m-%d").to_string());

    let until = matches.get_one::<String>("until")
        .cloned()
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    
    let active_only = matches.get_flag("active-only");
    let default_sort = String::from("commits");
    let sort_by = matches.get_one::<String>("sort-by").unwrap_or(&default_sort);
    let by_categories = matches.get_flag("categories");
    let show_repo_details = matches.get_flag("repo-details");
    let show_filetypes = matches.get_flag("filetypes");
    println!("{} {} {} {} {}", 
        "Analyzing repos from".bright_yellow(),
        since.bright_cyan(),
        "to".bright_yellow(),
        until.bright_cyan(),
        if active_only { "(active repos only)".bright_red() } else { "".normal() });

    let mut total_stats = RepoStats::default();
    let mut all_repo_stats = Vec::new();
    let mut categories = Vec::new();
    
    // Determine which author to use for filtering commits
    let config_author = config.author.clone();
    let cli_author = matches.get_one::<String>("author").cloned();
    let all_authors = matches.get_flag("all-authors");
    
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
        println!("{}: {}", "Author filter".bright_yellow(), a.green());
    } else {
        println!("{}", "Showing commits from all authors".bright_yellow());
    }

    // By design we process all repos by category as per the config file
    for (category_name, repos) in &config.repos {
        let stats = analyze_category(category_name, repos.clone(), &author_filter, &since, &until, active_only, show_repo_details, show_filetypes);
        
        // Add all repos to the flat list for overall stats
        if !by_categories {
            for (repo, repo_stats) in &stats.repos {
                all_repo_stats.push((repo.clone(), repo_stats.clone()));
            }
        }
        
        // Update overall totals
        total_stats.commit_count += stats.total.commit_count;
        total_stats.file_count += stats.total.file_count;
        total_stats.line_count += stats.total.line_count;
        
        for (date, count) in &stats.total.commits_by_date {
            *total_stats.commits_by_date.entry(date.clone()).or_insert(0) += count;
        }
        
        for (ext, count) in &stats.total.file_types {
            *total_stats.file_types.entry(ext.clone()).or_insert(0) += count;
        }
        
        categories.push(stats);
    }
    
    // Print category summary if requested
    if by_categories {
        // Count the total number of active repositories across all categories
        let total_active_repos = categories.iter()
            .map(|cat| cat.repos.iter().filter(|(_, stats)| stats.commit_count > 0).count())
            .sum::<usize>();
        
        print_category_summary(&categories, sort_by, show_filetypes);
        
        // Print totals across all repos
        println!("\n{}", "📊 Total Stats Across All Repositories:".bright_green());
        println!("{}: {}", "Active repositories".yellow(), total_active_repos.to_string().cyan());
        println!("{}: {}", "Total commits".yellow(), total_stats.commit_count.to_string().cyan());
        println!("{}: {}", "Total files".yellow(), total_stats.file_count.to_string().cyan());
        println!("{}: {}", "Total lines of code".yellow(), total_stats.line_count.to_string().cyan());
        
        // Find and show the most active day
        if let Some((max_date, max_count)) = get_max_commit_day(&total_stats.commits_by_date) {
            println!("\n{} {} ({} {})", 
                "📅 Most active day:".bright_magenta(), 
                max_date.bright_cyan(), 
                max_count.to_string(), 
                "commits".green());
        }
        
        // Show total file types only if show_filetypes is true
        if show_filetypes && !total_stats.file_types.is_empty() {
            println!("\n{}", "File types across all repos:".bright_magenta());
            let mut types: Vec<(String, usize)> = total_stats.file_types.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            // Sort by count (descending)
            types.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Show top 10 file types
            for (ext, count) in types.iter().take(10) {
                println!("  {} - {} {}", ext.bright_yellow(), count, "files".green());
            }
        }
    } else {
        // Otherwise sort and print overall top repos
        if !all_repo_stats.is_empty() {
            match sort_by.as_str() {
                "commits" => all_repo_stats.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count)),
                "files" => all_repo_stats.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count)),
                "lines" => all_repo_stats.sort_by(|a, b| b.1.line_count.cmp(&a.1.line_count)),
                _ => {}
            }
            
            println!("\n{} (sorted by {})", "📊 Most Active Repositories".bright_green(), sort_by);
            for (i, (repo, stats)) in all_repo_stats.iter().enumerate().take(5) {
                if stats.commit_count > 0 || sort_by != "commits" {
                    println!("{}. {} - {} commits, {} files, {} lines", 
                        (i + 1).to_string().bright_yellow(), 
                        repo.green(), 
                        stats.commit_count.to_string().cyan(),
                        stats.file_count.to_string().blue(),
                        stats.line_count.to_string().magenta());
                }
            }
        }
        
        // Print totals across all repos
        println!("\n{}", "📊 Total Stats Across All Repositories:".bright_green());
        
        // Count repositories with commits in the specified period
        let active_repos_count = all_repo_stats.iter()
            .filter(|(_, stats)| stats.commit_count > 0)
            .count();
        println!("{}: {}", "Active repositories".yellow(), active_repos_count.to_string().cyan());
        
        println!("{}: {}", "Total commits".yellow(), total_stats.commit_count.to_string().cyan());
        println!("{}: {}", "Total files".yellow(), total_stats.file_count.to_string().cyan());
        println!("{}: {}", "Total lines of code".yellow(), total_stats.line_count.to_string().cyan());
        
        // Find and show the most active day
        if let Some((max_date, max_count)) = get_max_commit_day(&total_stats.commits_by_date) {
            println!("\n{} {} ({} {})", 
                "📅 Most active day:".bright_magenta(), 
                max_date.bright_cyan(), 
                max_count.to_string(), 
                "commits".green());
        }
        
        // Show total file types only if show_filetypes is true
        if show_filetypes && !total_stats.file_types.is_empty() {
            println!("\n{}", "File types across all repos:".bright_magenta());
            let mut types: Vec<(String, usize)> = total_stats.file_types.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            // Sort by count (descending)
            types.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Show top 10 file types
            for (ext, count) in types.iter().take(10) {
                println!("  {} - {} {}", ext.bright_yellow(), count, "files".green());
            }
        }
    }
}
