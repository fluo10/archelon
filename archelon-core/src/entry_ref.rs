use std::path::PathBuf;

/// A reference to a journal entry — a filesystem path, a CarettaId, or a title.
///
/// This is the canonical input type for commands that operate on a single entry
/// (show, fix, remove, etc.).  Parse raw CLI user input with [`EntryRef::parse`],
/// then resolve it to a concrete [`PathBuf`] via [`ops::resolve_entry`].
///
/// # Syntax (CLI)
///
/// | Input form              | Resolved as     |
/// |-------------------------|-----------------|
/// | `@abc1234`              | `Id("abc1234")` |
/// | `path/to/file.md`       | `Path(...)`     |
/// | `./relative.md`         | `Path(...)`     |
/// | `~/absolute.md`         | `Path(...)`     |
/// | `anything_else`         | `Title(...)`    |
///
/// The `@` prefix is required for IDs to avoid ambiguity with titles that
/// happen to be 7 alphanumeric characters.
#[derive(Debug, Clone)]
pub enum EntryRef {
    /// A filesystem path to the entry file.
    Path(PathBuf),
    /// A CarettaId (the `@` prefix has been stripped).
    Id(String),
    /// An exact entry title (case-sensitive).
    Title(String),
}

impl EntryRef {
    /// Classify a raw CLI string as a path, an ID, or a title.
    ///
    /// - Starts with `@` → [`EntryRef::Id`] (prefix stripped).
    /// - Contains `/` or `\`, starts with `.` or `~`, or ends with `.md`
    ///   → [`EntryRef::Path`].
    /// - Anything else → [`EntryRef::Title`].
    pub fn parse(s: &str) -> Self {
        if let Some(id) = s.strip_prefix('@') {
            return EntryRef::Id(id.to_owned());
        }
        if s.contains('/')
            || s.contains(std::path::MAIN_SEPARATOR)
            || s.starts_with('.')
            || s.starts_with('~')
            || s.ends_with(".md")
        {
            EntryRef::Path(PathBuf::from(s))
        } else {
            EntryRef::Title(s.to_owned())
        }
    }
}

impl From<&str> for EntryRef {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for EntryRef {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}
