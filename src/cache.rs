use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub target_dir: Option<String>,
    pub template: Option<String>,
}

pub fn read_cache(cache_path: &PathBuf) -> Result<Cache, std::io::Error> {
    let data = fs::read_to_string(cache_path)?;
    let cache: Cache = serde_json::from_str(&data)?;
    Ok(cache)
}

pub fn write_cache(cache_path: &PathBuf, cache: &Cache) -> Result<(), std::io::Error> {
    let data = serde_json::to_string(cache)?;
    fs::write(cache_path, data)?;
    Ok(())
}

pub fn clear_cache(cache_path: &PathBuf) -> Result<(), std::io::Error> {
    fs::remove_file(cache_path)?;
    Ok(())
}

pub fn check_cache(cache_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if cache_path.exists() {
        let cache = read_cache(cache_path)?;
        println!("Cached configurations:");
        println!("Repo: {:?}", cache.repo);
        println!("Branch: {:?}", cache.branch);
        println!("Target Dir: {:?}", cache.target_dir);
    } else {
        println!("No cache found.");
    }
    Ok(())
}