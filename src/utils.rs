use colored::*;
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::fs;
use crate::cache;
use crate::errors::TemplateError;

pub fn read_config_json(config_file: &str) -> PathBuf {
    let home_dir = dirs::home_dir().ok_or(TemplateError::HomeDirNotFound).unwrap();
    let config_dir = home_dir.join(".tmpl-cli");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| TemplateError::ConfigDirCreationFailed(e.to_string()))
            .unwrap();
    }

    config_dir.join(config_file)
}

pub fn clone_repository(
    github_url: &str,
    target_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up loading indicator
    let pb = ProgressBar::new_spinner(); // 使用 Spinner 类型的进度条
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["-", "\\", "|", "/"]),
    );

    pb.set_message("Initializing...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Start cloning the repository
    pb.set_message("Cloning the repository...");
    let output = Command::new("git")
        .args(&["clone", github_url, target_path.to_str().unwrap()])
        .output()?;

    if output.status.success() {
        pb.set_message("Finalizing...");
        std::thread::sleep(Duration::from_millis(500)); // 模拟一些后续操作

        pb.finish_with_message(
            format!("Successfully cloned the project to: {:?}", target_path)
                .green()
                .to_string(),
        );
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        pb.finish_with_message(
            format!("Failed to clone the project: {}", error_message)
                .red()
                .to_string(),
        );
    }

    Ok(())
}

/// Prompts the user for the target folder name to clone into and returns the path.
pub fn get_target_path(default_path: &Path, default_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let input: String = Input::new()
        .with_prompt("Enter the folder name to clone into (leave empty to use default name)")
        .default(default_name.to_string())
        .interact_text()?;

    let target_path = if input.is_empty() {
        default_path.join(default_name)
    } else {
        default_path.join(input)
    };

    Ok(target_path)
}

pub fn get_cache_info() -> (PathBuf, cache::Cache) {
    let cache_path = read_config_json(".template_cli_cache.json");
    let cache = cache::read_cache(&cache_path).unwrap_or_default();
    (cache_path, cache)
}

pub struct OrderInfo {
    pub branch: String,
    pub target_dir: String,
}
pub fn resolve_order_info(matches: &clap::ArgMatches) -> OrderInfo {
    let (cache_path, mut cache) = get_cache_info();
    let branch = matches
        .get_one::<String>("branch")
        .cloned()
        .or_else(|| cache.branch.clone())
        .unwrap_or_else(|| "main".to_string());
    let target_dir = matches
        .get_one::<String>("target_dir")
        .cloned()
        .or_else(|| cache.target_dir.clone())
        .unwrap_or_else(|| "./".to_string());

    cache.branch = Some(branch.clone());
    cache.target_dir = Some(target_dir.clone());
    cache::write_cache(&cache_path, &cache).expect("Failed to write cache");

    OrderInfo {
        branch,
        target_dir,
    }
}
