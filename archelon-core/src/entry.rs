use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Frontmatter metadata stored at the top of each .md file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A single entry — one Markdown file in the vault.
/// Tasks and notes coexist freely in the body (bullet-journal style).
#[derive(Debug, Clone)]
pub struct Entry {
    /// Absolute path to the source .md file.
    pub path: PathBuf,

    /// Parsed frontmatter. Defaults to empty if the file has none.
    pub frontmatter: Frontmatter,

    /// Raw Markdown body (everything after the frontmatter block).
    pub body: String,
}

impl Entry {
    /// Returns the title: frontmatter title → file stem → "(untitled)".
    pub fn title(&self) -> &str {
        if let Some(ref t) = self.frontmatter.title {
            return t.as_str();
        }
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("(untitled)")
    }
}
