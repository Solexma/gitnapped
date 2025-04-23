use std::process::Command;
use std::fs;
use clap::{Arg, Command as ClapCommand};
use serde::Deserialize;
use chrono::{Utc, Duration, NaiveDate};
use std::collections::HashMap;
use colored::*;

#[derive(Debug, Deserialize)]
struct Config {
    author: Option<String>,
    #[serde(default)]
    repos: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone)]
struct RepoStats {
    commit_count: usize,
    file_count: usize,
    line_count: usize,
    commits_by_date: HashMap<String, usize>,
    file_types: HashMap<String, usize>,
}

#[derive(Debug, Default)]
struct CategoryStats {
    name: String,
    repos: Vec<(String, RepoStats)>,
    total: RepoStats,
}

fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path).expect("Unable to read config file");
    serde_yaml::from_str(&contents).expect("Invalid config format")
}

fn get_file_extension(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('.').collect();
    if parts.len() > 1 {
        parts.last().unwrap().to_string()
    } else {
        "none".to_string()
    }
}

fn count_files_and_lines(repo: &str) -> (usize, usize, HashMap<String, usize>) {
    // Get all files tracked by git
    let output = Command::new("git")
        .args(["-C", repo, "ls-files"])
        .output()
        .expect("Failed to run git ls-files");
    
    let files_output = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = files_output.lines().collect();
    let file_count = files.len();
    
    // Count lines in all tracked files and track file types
    let mut total_lines = 0;
    let mut file_types = HashMap::new();
    
    for file in files {
        let file_path = format!("{}/{}", repo, file);
        let extension = get_file_extension(file);
        *file_types.entry(extension).or_insert(0) += 1;
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            total_lines += content.lines().count();
        }
    }
    
    (file_count, total_lines, file_types)
}

fn analyze_repo(repo: &str, author: &Option<String>, since: &str, until: &str, show_details: bool, show_filetypes: bool) -> RepoStats {
    let mut stats = RepoStats::default();
    
    // Get commit history
    let mut cmd = Command::new("git");
    cmd.args(["-C", repo, "log", "--pretty=format:%h %ad %s", "--date=short"]);

    if let Some(a) = author {
        cmd.arg(format!("--author={}", a));
    }

    cmd.arg(format!("--since={}", since));
    cmd.arg(format!("--until={}", until));

    let output = cmd.output().expect("Failed to run git log");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<&str> = stdout.lines().collect();
    
    stats.commit_count = commits.len();
    
    // Parse commits by date
    for commit in &commits {
        if let Some(date_part) = commit.split_whitespace().nth(1) {
            *stats.commits_by_date.entry(date_part.to_string()).or_insert(0) += 1;
        }
    }
    
    // Count files and lines
    let (file_count, line_count, file_types) = count_files_and_lines(repo);
    stats.file_count = file_count;
    stats.line_count = line_count;
    stats.file_types = file_types;

    if show_details {
        // Print repo stats with colors
        println!("\n{} {}", "📁 Repo:".bright_blue(), repo.green());
        println!("{}: {}", "Commits".yellow(), stats.commit_count.to_string().cyan());
        println!("{}: {}", "Files".yellow(), stats.file_count.to_string().cyan());
        println!("{}: {}", "Lines of code".yellow(), stats.line_count.to_string().cyan());
        
        // Show commit history
        if !commits.is_empty() {
            println!("\n{}", "Commit history:".bright_magenta());
            for commit in commits {
                println!("{}", commit);
            }
            
            // Show commits by date (sorted)
            println!("\n{}", "Commits by date:".bright_magenta());
            let mut dates: Vec<(String, usize)> = stats.commits_by_date.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            // Sort by date (descending)
            dates.sort_by(|a, b| {
                NaiveDate::parse_from_str(&b.0, "%Y-%m-%d")
                    .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                    .cmp(&NaiveDate::parse_from_str(&a.0, "%Y-%m-%d")
                        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()))
            });
            
            for (date, count) in dates {
                println!("  {} - {} {}", date.bright_cyan(), count, "commits".green());
            }
            
            // Show file types
            if show_filetypes {
                if !stats.file_types.is_empty() {
                    println!("\n{}", "File types:".bright_magenta());
                    let mut types: Vec<(String, usize)> = stats.file_types.iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                
                // Sort by count (descending)
                types.sort_by(|a, b| b.1.cmp(&a.1));
                
                for (ext, count) in types {
                        println!("  {} - {} {}", ext.bright_yellow(), count, "files".green());
                    }
                }
            }
        }
    }
    
    stats
}

fn analyze_category(name: &str, repos: Vec<String>, author: &Option<String>, since: &str, until: &str, active_only: bool, show_repo_details: bool, show_filetypes: bool) -> CategoryStats {
    let mut category_stats = CategoryStats {
        name: name.to_string(),
        repos: Vec::new(),
        total: RepoStats::default(),
    };
    
    for repo in repos {
        let repo_stats = analyze_repo(&repo, author, since, until, show_repo_details, show_filetypes);
        
        // Skip inactive repos if active-only flag is set
        if active_only && repo_stats.commit_count == 0 {
            continue;
        }
        
        category_stats.repos.push((repo, repo_stats.clone()));
        
        // Update category totals
        category_stats.total.commit_count += repo_stats.commit_count;
        category_stats.total.file_count += repo_stats.file_count;
        category_stats.total.line_count += repo_stats.line_count;
        
        // Merge commits by date
        for (date, count) in repo_stats.commits_by_date {
            *category_stats.total.commits_by_date.entry(date).or_insert(0) += count;
        }
        
        // Merge file types
        for (ext, count) in repo_stats.file_types {
            *category_stats.total.file_types.entry(ext).or_insert(0) += count;
        }
    }
    
    category_stats
}

fn print_category_summary(categories: &[CategoryStats], sort_by: &str, show_filetypes: bool) {
    println!("\n{}", "📊 Category Statistics:".bright_green());
    
    for category in categories {
        if category.repos.is_empty() {
            continue;
        }
        
        println!("\n{} {}", "Category:".bright_yellow(), category.name.bright_cyan());
        
        // Conto i repository attivi nella categoria
        let active_repos_count = category.repos.iter()
            .filter(|(_, stats)| stats.commit_count > 0)
            .count();
        println!("{}: {}", "Active repositories".yellow(), active_repos_count.to_string().cyan());
        
        println!("{}: {}", "Total commits".yellow(), category.total.commit_count.to_string().cyan());
        println!("{}: {}", "Total files".yellow(), category.total.file_count.to_string().cyan());
        println!("{}: {}", "Total lines of code".yellow(), category.total.line_count.to_string().cyan());
        
        // Mostra il giorno più attivo per questa categoria
        if let Some((max_date, max_count)) = get_max_commit_day(&category.total.commits_by_date) {
            println!("{} {} ({} {})", 
                "📅 Most active day:".bright_magenta(), 
                max_date.bright_cyan(), 
                max_count.to_string(), 
                "commits".green());
        }
        
        // Mostra i tipi di file per questa categoria se richiesto
        if show_filetypes && !category.total.file_types.is_empty() {
            println!("  {}", "File types:".bright_magenta());
            let mut types: Vec<(String, usize)> = category.total.file_types.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            // Sort by count (descending)
            types.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Show top 5 file types per category
            for (ext, count) in types.iter().take(5) {
                println!("    {} - {} {}", ext.bright_yellow(), count, "files".green());
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
            println!("  {} (sorted by {})", "Top repositories:".bright_blue(), sort_by);
            for (i, (repo, stats)) in sorted_repos.iter().enumerate().take(3) {
                if stats.commit_count > 0 || sort_by != "commits" {
                    println!("   {}. {} - {} commits, {} files, {} lines", 
                        (i + 1).to_string().bright_yellow(), 
                        repo.split('/').last().unwrap_or(repo).green(), 
                        stats.commit_count.to_string().cyan(),
                        stats.file_count.to_string().blue(),
                        stats.line_count.to_string().magenta());
                }
            }
        }
    }
}

// Aggiungo una funzione per trovare il giorno con più commit e la relativa formattazione
fn get_max_commit_day(commits_by_date: &HashMap<String, usize>) -> Option<(String, usize)> {
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
    
    // Determina quale autore usare per filtrare i commit
    let config_author = config.author.clone();
    let cli_author = matches.get_one::<String>("author").cloned();
    let all_authors = matches.get_flag("all-authors");
    
    // Priorità: 1) all-authors flag, 2) author CLI arg, 3) config file author
    let author_filter = if all_authors {
        None // Non filtrare per autore, mostra i commit di tutti
    } else if let Some(a) = cli_author {
        Some(a) // Usa l'autore specificato da linea di comando
    } else {
        config_author // Usa l'autore dal file di configurazione (può essere None)
    };
    
    // Visualizza le informazioni sull'autore utilizzato
    if let Some(a) = &author_filter {
        println!("{}: {}", "Author filter".bright_yellow(), a.green());
    } else {
        println!("{}", "Showing commits from all authors".bright_yellow());
    }

    // Process all repos by category
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
        // Contiamo il numero totale di repository attivi in tutte le categorie
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
        
        // Show total file types solo se show_filetypes è true
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
        
        // Conto i repository con commit nel periodo indicato
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
        
        // Show total file types solo se show_filetypes è true
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
