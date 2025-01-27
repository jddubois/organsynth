use super::Config;
use std::fs;

pub fn load(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
