use std::collections::HashMap;
use std::fs;
use std::process::Command;

pub fn get_file_extension(file_path: &str) -> String {
    let parts: Vec<&str> = file_path.split('.').collect();
    if parts.len() > 1 {
        parts.last().unwrap().to_string()
    } else {
        "none".to_string()
    }
}

pub fn count_files_and_lines(repo: &str) -> (usize, usize, HashMap<String, usize>) {
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