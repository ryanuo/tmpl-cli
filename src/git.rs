use crate::errors::TemplateError;
use std::path::Path;
use std::process::Command as ProcessCommand;

pub fn clone_repo(repo_url: &str, branch: &str, target_path: &Path) -> Result<(), TemplateError> {
    let mut cmd = ProcessCommand::new("git");
    cmd.args([
        "clone",
        "--branch",
        branch,
        repo_url,
        target_path.to_str().expect("Invalid target path"),
    ]);

    let output = cmd.output().map_err(|e| TemplateError::IoError(e))?;

    if !output.status.success() {
        let error_str = String::from_utf8_lossy(&output.stderr);
        return Err(TemplateError::GitCloneError(error_str.into()));
    }

    Ok(())
}