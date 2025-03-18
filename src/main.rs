mod cache;
mod cli;
mod errors;
mod git;
mod template;

use dialoguer::Select;
use errors::TemplateError;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = PathBuf::from(".template_cli_cache.json");

    let matches = cli::build_cli().get_matches();

    if matches.get_flag("check-cache") {
        if cache_path.exists() {
            let cache = cache::read_cache(&cache_path)?;
            println!("Cached configurations:");
            println!("Repo: {:?}", cache.repo);
            println!("Branch: {:?}", cache.branch);
            println!("Target Dir: {:?}", cache.target_dir);
            // println!("Template: {:?}", cache.template);
        } else {
            println!("No cache found.");
        }
        return Ok(());
    }

    if matches.get_flag("clear-cache") {
        cache::clear_cache(&cache_path)?;
        println!("Cache cleared.");
        return Ok(());
    }

    let mut cache = cache::read_cache(&cache_path).unwrap_or_default();

    let repo_url = match matches.get_one::<String>("repo") {
        Some(url) => {
            cache.repo = Some(url.clone());
            cache::write_cache(&cache_path, &cache).unwrap();
            url.clone()
        }
        None => {
            match cache.repo.clone() {
                Some(cached_url) => cached_url,
                None => return Err(Box::new(TemplateError::MissingRepoUrl)),
            }
        }
    };
    let branch = matches
        .get_one::<String>("branch")
        .cloned()
        .or_else(|| cache.branch.clone())
        .unwrap_or_else(|| "main".to_string());

    let target_dir_str = matches
        .get_one::<String>("target_dir")
        .cloned()
        .or_else(|| cache.target_dir.clone())
        .unwrap_or_else(|| "./".to_string());
    let target_dir = PathBuf::from(target_dir_str.clone());

    let selected_template = matches
        .get_one::<String>("template")
        .cloned()
        .or_else(|| cache.template.clone());

    cache.branch = Some(branch.clone());
    cache.target_dir = Some(target_dir_str.clone());
    // cache.template = selected_template.clone();

    cache::write_cache(&cache_path, &cache)?;

    let temp_dir = TempDir::new().map_err(|e| TemplateError::IoError(e))?;
    let clone_path = temp_dir.path().join("cloned_repo");

    git::clone_repo(&repo_url, &branch, &clone_path)?;

    let templates = template::get_template_list(&clone_path)?;

    let selected_template = match selected_template {
        Some(name) => {
            if !templates.contains(&name) {
                return Err(TemplateError::InvalidTemplate(name).into());
            }
            name
        }
        None => {
            let selection = Select::new()
                .with_prompt("Select Template:")
                .items(&templates)
                .default(0)
                .interact()?;
            templates[selection].clone()
        }
    };

    let source_dir = clone_path.join(&selected_template);
    let target_subdir = target_dir.join(&selected_template);
    let rename_option = matches.get_one::<String>("rename").map(|s| s.as_str());
    if let Err(e) = template::copy_template(&source_dir, &target_subdir, rename_option) {
        eprintln!("Copying failed: {:?}", e);
    }
    println!(
        "\nTemplate '{}' downloaded successfully!",
        selected_template
    );

    Ok(())
}
