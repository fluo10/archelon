use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(#[from] serde_yaml::Error),

    #[error("Invalid entry file: {0}")]
    InvalidEntry(String),
}

pub type Result<T> = std::result::Result<T, Error>;
