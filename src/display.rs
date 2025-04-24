use std::collections::HashMap;
use colored::*;
use crate::models::CategoryStats;

pub fn get_max_commit_day(commits_by_date: &HashMap<String, usize>) -> Option<(String, usize)> {
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

pub fn print_category_summary(categories: &[CategoryStats], sort_by: &str, show_filetypes: bool) {
    println!("\n{}", "📊 Category Statistics:".bright_green());
    
    for category in categories {
        if category.repos.is_empty() {
            continue;
        }
        
        println!("\n{} {}", "Category:".bright_yellow(), category.name.bright_cyan());
        
        // Count active repositories in the category
        let active_repos_count = category.repos.iter()
            .filter(|(_, stats)| stats.commit_count > 0)
            .count();
        println!("{}: {}", "Active repositories".yellow(), active_repos_count.to_string().cyan());
        
        println!("{}: {}", "Total commits".yellow(), category.total.commit_count.to_string().cyan());
        println!("{}: {}", "Total files".yellow(), category.total.file_count.to_string().cyan());
        println!("{}: {}", "Total lines of code".yellow(), category.total.line_count.to_string().cyan());
        
        // Show the most active day for this category
        if let Some((max_date, max_count)) = get_max_commit_day(&category.total.commits_by_date) {
            println!("{} {} ({} {})", 
                "📅 Most active day:".bright_magenta(), 
                max_date.bright_cyan(), 
                max_count.to_string(), 
                "commits".green());
        }
        
        // Show file types for this category if requested
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