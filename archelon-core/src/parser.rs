use std::path::Path;

use crate::{
    entry::{Entry, Frontmatter},
    error::{Error, Result},
};

const FENCE: &str = "---";

/// Parse a Markdown file into an [`Entry`].
///
/// Frontmatter is optional. If the file starts with `---`, everything until
/// the closing `---` is parsed as YAML. The rest is the body.
pub fn parse_entry(path: &Path, source: &str) -> Result<Entry> {
    let (frontmatter, body) = split_frontmatter(source)?;
    Ok(Entry {
        path: path.to_path_buf(),
        frontmatter,
        body: body.to_owned(),
    })
}

/// Read a file from disk and parse it.
pub fn read_entry(path: &Path) -> Result<Entry> {
    let source = std::fs::read_to_string(path)?;
    parse_entry(path, &source)
}

fn split_frontmatter(source: &str) -> Result<(Frontmatter, &str)> {
    let Some(rest) = source.strip_prefix(FENCE) else {
        // No frontmatter — treat whole file as body.
        return Ok((Frontmatter::default(), source));
    };

    // The opening `---` must be followed by a newline.
    let Some(rest) = rest.strip_prefix('\n') else {
        return Ok((Frontmatter::default(), source));
    };

    let Some(end) = rest.find(&format!("\n{FENCE}")) else {
        return Err(Error::InvalidEntry(
            "frontmatter block is not closed".into(),
        ));
    };

    let yaml = &rest[..end];
    let body = &rest[end + 1 + FENCE.len()..]; // skip `\n---`
    let body = body.strip_prefix('\n').unwrap_or(body);

    let frontmatter: Frontmatter = serde_yaml::from_str(yaml)?;
    Ok((frontmatter, body))
}

/// Serialize an [`Entry`] back to Markdown source.
pub fn render_entry(entry: &Entry) -> String {
    let fm = &entry.frontmatter;
    let has_fm = fm.title.is_some() || fm.date.is_some() || !fm.tags.is_empty();
    let mut out = String::new();

    if has_fm {
        out.push_str("---\n");
        if let Some(ref t) = fm.title {
            out.push_str(&format!("title: {t}\n"));
        }
        if let Some(ref d) = fm.date {
            out.push_str(&format!("date: {d}\n"));
        }
        if !fm.tags.is_empty() {
            out.push_str(&format!("tags: [{}]\n", fm.tags.join(", ")));
        }
        out.push_str("---\n");
        if !entry.body.is_empty() {
            out.push('\n');
        }
    }

    out.push_str(&entry.body);
    out
}

/// Write an [`Entry`] back to its source file.
pub fn write_entry(entry: &Entry) -> Result<()> {
    std::fs::write(&entry.path, render_entry(entry))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_path() -> PathBuf {
        PathBuf::from("test.md")
    }

    #[test]
    fn parses_entry_with_frontmatter() {
        let src = "---\ntitle: Hello\ntags: [rust, cli]\n---\nsome body\n";
        let entry = parse_entry(&dummy_path(), src).unwrap();
        assert_eq!(entry.frontmatter.title.as_deref(), Some("Hello"));
        assert_eq!(entry.frontmatter.tags, vec!["rust", "cli"]);
        assert_eq!(entry.body, "some body\n");
    }

    #[test]
    fn parses_entry_without_frontmatter() {
        let src = "just a body\n";
        let entry = parse_entry(&dummy_path(), src).unwrap();
        assert!(entry.frontmatter.title.is_none());
        assert_eq!(entry.body, "just a body\n");
    }

    #[test]
    fn title_falls_back_to_file_stem() {
        let src = "body\n";
        let entry = parse_entry(&PathBuf::from("my-note.md"), src).unwrap();
        assert_eq!(entry.title(), "my-note");
    }
}
