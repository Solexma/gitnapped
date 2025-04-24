use std::process::Command;
use colored::*;
use chrono::NaiveDate;
use crate::models::{RepoStats, CategoryStats};
use crate::utils::count_files_and_lines;

pub fn analyze_repo(repo: &str, author: &Option<String>, since: &str, until: &str, show_details: bool, show_filetypes: bool) -> RepoStats {
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

pub fn analyze_category(name: &str, repos: Vec<String>, author: &Option<String>, since: &str, until: &str, active_only: bool, show_repo_details: bool, show_filetypes: bool) -> CategoryStats {
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