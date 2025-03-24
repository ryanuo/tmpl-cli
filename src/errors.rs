use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Git clone error: {0}")]
    GitCloneError(String),
    #[error("Template directory not found: {0}")]
    InvalidTemplate(String),
    #[error("Target directory creation failed: {0}")]
    TargetError(String),
    #[error("Copying error: {0}")]
    CopyError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Template directory not found on remote")]
    TemplateNotFound,
    #[error("No repository URL provided and no cache found.")]
    MissingRepoUrl,
    #[error("Invalid target path")]
    InvalidTargetPath,
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error("Failed to create configuration directory: {0}")]
    ConfigDirCreationFailed(String),
}