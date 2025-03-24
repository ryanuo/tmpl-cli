use colored::*;
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::fs;
use crate::errors::TemplateError;

pub fn read_config_json(config_file: &str) -> PathBuf {
    let home_dir = dirs::home_dir().ok_or(TemplateError::HomeDirNotFound).unwrap();

    println!("Home directory: {:?}", home_dir); // 打印 home_dir

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
pub fn get_target_path(default_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let default_path = std::env::current_dir()?.join(default_name);

    let input: String = Input::new()
        .with_prompt("Enter the folder name to clone into (leave empty to use default name)")
        .default(default_name.to_string())
        .interact_text()?;

    let target_path = if input.is_empty() {
        default_path
    } else {
        std::env::current_dir()?.join(input)
    };

    Ok(target_path)
}
