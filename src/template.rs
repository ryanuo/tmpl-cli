use crate::errors::TemplateError;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub fn get_template_list(target_dir: &Path) -> Result<Vec<String>, TemplateError> {
    const EXCLUDED_TEMPLATES: [&str; 1] = [".git"];
    let templates: Vec<String> = fs::read_dir(target_dir)?
        .filter_map(|entry_result| {
            entry_result.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    path.file_name()
                        .and_then(OsStr::to_str)
                        .filter(|name| !EXCLUDED_TEMPLATES.contains(&name))
                        .map(|name| name.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    if templates.is_empty() {
        Err(TemplateError::TemplateNotFound)
    } else {
        Ok(templates)
    }
}

pub fn copy_template(source: &Path, dest: &Path) -> Result<(), TemplateError> {
    if !source.exists() {
        return Err(TemplateError::InvalidTemplate(source.display().to_string()));
    }

    fs::create_dir_all(dest).map_err(|e| {
        TemplateError::TargetError(format!("Failed to create {}: {}", dest.display(), e))
    })?;

    for entry in fs::read_dir(source)? {
        let entry = entry.map_err(|e| TemplateError::IoError(e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_template(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| {
                TemplateError::CopyError(format!(
                    "Failed to copy file '{}' : {}",
                    src_path.display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}