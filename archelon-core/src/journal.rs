use std::path::{Path, PathBuf};

use caretta_id::CarettaId;
use chrono::Datelike as _;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const ARCHELON_DIR: &str = ".archelon";

/// A located journal — a directory tree that contains a `.archelon` directory.
#[derive(Debug, Clone)]
pub struct Journal {
    /// The directory that directly contains `.archelon/`.
    pub root: PathBuf,
}

impl Journal {
    /// Create a `Journal` from an explicit root path.
    ///
    /// Returns `Err(Error::JournalNotFound)` if `root` does not contain a `.archelon` directory.
    pub fn from_root(root: PathBuf) -> Result<Self> {
        if root.join(ARCHELON_DIR).is_dir() {
            Ok(Journal { root })
        } else {
            Err(Error::JournalNotFound)
        }
    }

    /// Walk up from `start` until a directory containing `.archelon/` is found.
    ///
    /// Returns `Err(Error::JournalNotFound)` if no such directory exists.
    pub fn find_from(start: &Path) -> Result<Self> {
        let mut current = start.to_path_buf();
        loop {
            if current.join(ARCHELON_DIR).is_dir() {
                return Ok(Journal { root: current });
            }
            if !current.pop() {
                return Err(Error::JournalNotFound);
            }
        }
    }

    /// Walk up from the current working directory.
    pub fn find() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::find_from(&cwd)
    }

    /// Path to the `.archelon` directory itself.
    pub fn archelon_dir(&self) -> PathBuf {
        self.root.join(ARCHELON_DIR)
    }

    /// Read the journal config from `.archelon/config.toml`.
    /// Returns the default config if the file does not exist.
    pub fn config(&self) -> Result<JournalConfig> {
        let path = self.archelon_dir().join("config.toml");
        if !path.exists() {
            return Ok(JournalConfig::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        toml::from_str(&contents).map_err(|e| Error::InvalidConfig(e.to_string()))
    }

    /// Find a single `.md` entry file whose stem starts with `id_prefix`.
    ///
    /// Scans `self.root` and all direct year subdirectories.
    /// Returns `Err(EntryNotFound)` if nothing matches, or `Err(AmbiguousId)`
    /// if more than one file matches.
    pub fn find_entry_by_id(&self, id_prefix: &str) -> Result<PathBuf> {
        let mut matches = Vec::new();
        for dir in std::iter::once(self.root.clone()).chain(self.year_subdirs()?) {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("md")
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|stem| stem.starts_with(id_prefix))
                {
                    matches.push(p);
                }
            }
        }

        match matches.len() {
            0 => Err(Error::EntryNotFound(id_prefix.to_owned())),
            1 => Ok(matches.remove(0)),
            n => Err(Error::AmbiguousId(id_prefix.to_owned(), n)),
        }
    }

    /// Collect all `.md` entry files in the journal: root + year subdirectories.
    pub fn collect_entries(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        collect_md_in(&self.root, &mut paths)?;
        for subdir in self.year_subdirs()? {
            collect_md_in(&subdir, &mut paths)?;
        }
        paths.sort();
        Ok(paths)
    }

    fn year_subdirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        for entry in std::fs::read_dir(&self.root)?.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') && name.chars().all(|c| c.is_ascii_digit()) {
                        dirs.push(p);
                    }
                }
            }
        }
        Ok(dirs)
    }
}

fn collect_md_in(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Ok(()) };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
    Ok(())
}

// ── config ────────────────────────────────────────────────────────────────────

/// Contents of `.archelon/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalConfig {
    #[serde(default)]
    pub journal: JournalSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalSection {
    /// IANA timezone name. Used when frontmatter timestamps have no timezone.
    /// Defaults to `"UTC"`.
    pub timezone: String,

    /// First day of the week, used by `--this-week`. Defaults to `monday`.
    #[serde(default)]
    pub week_start: WeekStart,
}

impl Default for JournalSection {
    fn default() -> Self {
        JournalSection { timezone: "UTC".to_owned(), week_start: WeekStart::Monday }
    }
}

/// First day of the week for `--this-week` calculations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

// ── filename helpers ──────────────────────────────────────────────────────────

/// Return `true` if `path` follows the archelon-managed filename convention:
/// `{7-char-CarettaId}_{slug}.md` or `{7-char-CarettaId}.md`.
pub fn is_managed_filename(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let Some(id_str) = stem.get(..7) else {
        return false;
    };
    if id_str.parse::<CarettaId>().is_err() {
        return false;
    }
    let rest = &stem[7..];
    rest.is_empty() || rest.starts_with('_')
}

/// Convert a title to a filename-safe slug.
///
/// Lowercases the string, replaces whitespace with `_`, and strips any
/// character that is not ASCII alphanumeric or `_`.
///
/// ```
/// # use archelon_core::journal::slugify;
/// assert_eq!(slugify("My Example Entry!"), "my_example_entry");
/// ```
pub fn slugify(title: &str) -> String {
    title
        .chars()
        .map(|c| if c.is_whitespace() { '_' } else { c.to_ascii_lowercase() })
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
        .trim_matches('_')
        .to_owned()
}

/// Build the canonical entry filename: `{id}_{slug}.md`.
///
/// If the slug is empty the filename is just `{id}.md`.
pub fn entry_filename(id: CarettaId, title: &str) -> String {
    let slug = slugify(title);
    if slug.is_empty() {
        format!("{id}.md")
    } else {
        format!("{id}_{slug}.md")
    }
}

/// Generate a relative path for a new entry: `{year}/{id}_{slug}.md`.
///
/// The ID is based on the current Unix time (`CarettaId::now_unix()`), so
/// filenames sort chronologically within a year directory.
pub fn new_entry_path(title: &str) -> PathBuf {
    let id = CarettaId::now_unix();
    let year = chrono::Local::now().year();
    PathBuf::from(year.to_string()).join(entry_filename(id, title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Example Entry"), "my_example_entry");
    }

    #[test]
    fn slugify_strips_special_chars() {
        assert_eq!(slugify("Hello, World!"), "hello_world");
    }

    #[test]
    fn slugify_trims_underscores() {
        assert_eq!(slugify("  leading"), "leading");
    }
}
