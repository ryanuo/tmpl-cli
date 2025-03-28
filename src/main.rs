mod cache;
mod cli;
mod errors;
mod git;
mod original;
mod template;
mod utils;

use colored::Colorize;
use errors::TemplateError;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e.to_string().red());
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli::build_cli().get_matches();
    let (cache_path, _) = utils::get_cache_info();

    if matches.contains_id("original") {
        handle_original(&matches)?;
        return Ok(());
    }

    if matches.get_flag("check-cache") {
        cache::check_cache(&cache_path)?;
        return Ok(());
    }

    if matches.get_flag("clear-cache") {
        cache::clear_cache(&cache_path)?;
        println!("Cache cleared.");
        return Ok(());
    }

    handle_template_workflow(&matches)?;
    Ok(())
}

// Handle the original workflow
fn handle_original(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let json_source = matches.get_one::<String>("original").map(String::as_str);
    original::fetch_project_from_json(json_source)?;
    Ok(())
}

// Handle the template workflow
fn handle_template_workflow(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let repo_url = get_repo_url(matches)?;
    let order_info = utils::resolve_order_info(matches);
    let target_dir = PathBuf::from(&order_info.target_dir);

    let temp_dir = TempDir::new().map_err(TemplateError::IoError)?;
    let clone_path = temp_dir.path().join("cloned_repo");
    git::clone_repo(&repo_url, &order_info.branch, &clone_path)?;

    process_templates(matches, &clone_path, &target_dir)?;
    Ok(())
}

/**
 * Process the templates
 * This function processes the templates by selecting a template from the list
 */
fn process_templates(
    matches: &clap::ArgMatches,
    clone_path: &PathBuf,
    target_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = template::get_template_list(clone_path)?;
    let selected_template =
        template::select_template(matches.get_one::<String>("template"), &templates)?;
    let source_dir = clone_path.join(&selected_template);
    let target_path = utils::get_target_path(&target_dir, &selected_template)?;
    let rename_option = target_path.as_path().to_str();

    if let Err(e) = template::copy_template(&source_dir, &target_dir, rename_option) {
        eprintln!("Copying failed: {:?}", e);
    }

    println!("Template '{}' downloaded successfully!", selected_template);
    Ok(())
}

fn get_repo_url(matches: &clap::ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    let (cache_path, mut cache) = utils::get_cache_info();
    if let Some(url) = matches.get_one::<String>("repo") {
        cache.repo = Some(url.clone());
        cache::write_cache(&cache_path, &cache)?;
        Ok(url.clone())
    } else {
        cache
            .repo
            .clone()
            .ok_or(Box::new(TemplateError::MissingRepoUrl))
    }
}
