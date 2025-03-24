mod cache;
mod cli;
mod errors;
mod git;
mod original;
mod template;
mod utils;

use errors::TemplateError;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli::build_cli().get_matches();

    if matches.contains_id("original") {
        handle_original(&matches)?;
        return Ok(());
    }

    let cache_path = utils::read_config_json(".template_cli_cache.json");

    if matches.get_flag("check-cache") {
        cache::check_cache(&cache_path)?;
        return Ok(());
    }

    if matches.get_flag("clear-cache") {
        cache::clear_cache(&cache_path)?;
        println!("Cache cleared.");
        return Ok(());
    }

    handle_template_workflow(&matches, &cache_path)?;
    Ok(())
}

fn handle_original(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(original_repo) = matches.get_one::<String>("original") {
        let project_name = original_repo.split('/').last().unwrap_or("");
        let target_path = utils::get_target_path(project_name)?;

        utils::clone_repository(original_repo, &target_path)?;
    } else {
        let _ = original::select_project_from_json();
    }
    Ok(())
}

fn handle_template_workflow(
    matches: &clap::ArgMatches,
    cache_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = cache::read_cache(&cache_path).unwrap_or_default();
    let repo_url = get_repo_url(matches, &mut cache, cache_path)?;
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
    let target_dir = PathBuf::from(&target_dir_str);

    cache.branch = Some(branch.clone());
    cache.target_dir = Some(target_dir_str.clone());
    cache::write_cache(cache_path, &cache)?;

    let temp_dir = TempDir::new().map_err(TemplateError::IoError)?;
    let clone_path = temp_dir.path().join("cloned_repo");
    git::clone_repo(&repo_url, &branch, &clone_path)?;

    process_templates(matches, &clone_path, &target_dir)?;
    Ok(())
}

fn process_templates(
    matches: &clap::ArgMatches,
    clone_path: &PathBuf,
    target_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = template::get_template_list(clone_path)?;
    let selected_template =
        template::select_template(matches.get_one::<String>("template"), &templates)?;

    let source_dir = clone_path.join(&selected_template);
    let target_subdir = target_dir.join(&selected_template);
    let target_path = utils::get_target_path(&selected_template)?;
    let rename_option = target_path.as_path().to_str();

    if let Err(e) = template::copy_template(&source_dir, &target_subdir, rename_option) {
        eprintln!("Copying failed: {:?}", e);
    }

    println!("Template '{}' downloaded successfully!", selected_template);
    Ok(())
}

fn get_repo_url(
    matches: &clap::ArgMatches,
    cache: &mut cache::Cache,
    cache_path: &PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(url) = matches.get_one::<String>("repo") {
        cache.repo = Some(url.clone());
        cache::write_cache(cache_path, cache)?;
        Ok(url.clone())
    } else {
        cache
            .repo
            .clone()
            .ok_or(Box::new(TemplateError::MissingRepoUrl))
    }
}