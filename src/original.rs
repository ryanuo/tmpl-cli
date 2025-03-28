use colored::*;
use dialoguer::Select;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::{cli, utils};
use crate::errors::TemplateError;

pub fn fetch_project_from_json(json_source: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = fetch_json_data(json_source)?;
    let json: Value = serde_json::from_str(&json_data)
        .map_err(|e| TemplateError::InvalidJsonFormat(e.to_string()))?;

    let main_categories: Vec<&String> = json.as_object().unwrap().keys().collect();
    let selected_category = select_category(&main_categories)?;

    let items = json[selected_category]
        .as_array()
        .ok_or("Failed to retrieve items in the selected category")?;
    if items.is_empty() {
        println!("There are no available projects in the current category.");
        return Ok(());
    }

    let formatted_choices = format_choices(items);
    let selected_item = select_item(items, &formatted_choices, selected_category)?;

    process_selected_item(selected_item)?;

    Ok(())
}

fn fetch_json_data(json_source: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(source) = json_source {
        if source.starts_with("http://") || source.starts_with("https://") {
            Ok(reqwest::blocking::get(source)?.text()?)
        } else {
            Err(Box::new(TemplateError::InvalidSourceUrl("The provided source URL is invalid.".to_string())))
        }
    } else {
        let cache_path = utils::read_config_json("data.json");
        Ok(fs::read_to_string(cache_path)?)
    }
}

fn select_category<'a>(categories: &'a [&'a String]) -> Result<&'a String, Box<dyn std::error::Error>> {
    let main_selection = Select::new()
        .with_prompt("Please select a category")
        .items(categories)
        .default(0)
        .interact()?;
    Ok(categories[main_selection])
}

fn format_choices(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            let name = item["name"].as_str().unwrap().to_string();
            let desc = item["desc"].as_str().unwrap().to_string();
            format!("{} {}", name.green(), desc)
        })
        .collect()
}

fn select_item<'a>(
    items: &'a [Value],
    formatted_choices: &[String],
    category: &String,
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let item_selection = Select::new()
        .with_prompt(format!("Please select a project under {}", category))
        .items(formatted_choices)
        .default(0)
        .interact()?;
    Ok(&items[item_selection])
}

fn process_selected_item(selected_item: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let link = selected_item["link"].as_str().unwrap();
    let project_name = selected_item["name"].as_str().unwrap();
    let matches = cli::build_cli().get_matches();
    let order_info = utils::resolve_order_info(&matches);
    let dir_path = PathBuf::from(&order_info.target_dir);
    let target_path = utils::get_target_path(&dir_path, project_name)?;

    utils::clone_repository(link, &target_path)?;

    println!(
        "You selected the project: {}",
        selected_item["name"].as_str().unwrap().green()
    );
    println!("Link: {}", link.blue());
    println!("Desc: {}", selected_item["desc"].as_str().unwrap().bright_yellow());

    Ok(())
}