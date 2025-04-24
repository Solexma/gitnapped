use std::fs;
use crate::models::Config;

pub fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path).expect("Unable to read config file");
    serde_yaml::from_str(&contents).expect("Invalid config format")
} 