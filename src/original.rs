use colored::*;
use dialoguer::Select;
use serde_json::Value;
use std::fs;

use crate::utils;
use crate::errors::TemplateError;

pub fn select_project_from_json(json_source: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = if let Some(source) = json_source {
        if source.starts_with("http://") || source.starts_with("https://") {
            reqwest::blocking::get(source)?.text()?
        } else {
            return Err(Box::new(TemplateError::InvalidSourceUrl("The provided source URL is invalid.".to_string())));
        }
    } else {
        let cache_path = utils::read_config_json("data.json");
        fs::read_to_string(cache_path)?
    };

    let json: Value = serde_json::from_str(&json_data).map_err(|e| TemplateError::InvalidJsonFormat(e.to_string()))?;

    let main_categories: Vec<_> = json.as_object().unwrap().keys().collect();

    let main_selection = Select::new()
        .with_prompt("Please select a category")
        .items(&main_categories)
        .default(0)
        .interact()?;

    let selected_category = &main_categories[main_selection];
    let items = json[selected_category].as_array().unwrap();

    if items.is_empty() {
        println!("There are no available projects in the current category.");
        return Ok(());
    }

    let formatted_choices: Vec<String> = items
        .iter()
        .map(|item| {
            let name = item["name"].as_str().unwrap().to_string();
            let desc = item["desc"].as_str().unwrap().to_string();
            format!("{} {}", name.green(), desc)
        })
        .collect();

    let item_selection = Select::new()
        .with_prompt(format!(
            "Please select a project under {}",
            selected_category
        ))
        .items(&formatted_choices)
        .default(0)
        .interact()?;

    let selected_item = &items[item_selection];
    let link = selected_item["link"].as_str().unwrap();
    let project_name = selected_item["name"].as_str().unwrap();

    let target_path = utils::get_target_path(project_name)?;

    // Call the separate function to clone the project
    utils::clone_repository(link, &target_path)?;

    println!(
        "You selected the project: {}",
        formatted_choices[item_selection]
    );
    println!("Link: {}", link);
    println!("Description: {}", selected_item["desc"].as_str().unwrap());

    Ok(())
}