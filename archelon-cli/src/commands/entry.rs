use anyhow::{bail, Context, Result};
use archelon_core::{
    entry::{Entry, Frontmatter},
    parser::{read_entry, render_entry, write_entry},
};
use clap::Subcommand;
use std::{
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Subcommand)]
pub enum EntryCommand {
    /// List all entries in a vault directory
    List {
        /// Path to the vault (defaults to current directory)
        #[arg(default_value = ".")]
        vault: PathBuf,
    },
    /// Show the contents of an entry
    Show {
        /// Path to the entry file
        path: PathBuf,
    },
    /// Create a new entry.
    /// Without --body, opens $EDITOR (like `git commit` without -m).
    New {
        /// Output file name (e.g. "my-note" or "my-note.md")
        name: String,

        /// Title written into the frontmatter
        #[arg(long, short)]
        title: Option<String>,

        /// Tags written into the frontmatter (comma-separated)
        #[arg(long, short = 'T', value_delimiter = ',')]
        tags: Vec<String>,

        /// Inline body content — skips the editor (like git commit -m)
        #[arg(long, short)]
        body: Option<String>,
    },
    /// Open an entry in $EDITOR
    Edit {
        /// Path to the entry file
        path: PathBuf,
    },
    /// Update frontmatter fields without opening an editor (like git commit --amend -m)
    Set {
        /// Path to the entry file
        path: PathBuf,

        /// New title (omit to leave unchanged)
        #[arg(long, short)]
        title: Option<String>,

        /// Replace tags (omit to leave unchanged; pass with no value to clear)
        #[arg(long, short = 'T', num_args = 0.., value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
}

pub fn run(cmd: EntryCommand) -> Result<()> {
    match cmd {
        EntryCommand::List { vault } => list(&vault),
        EntryCommand::Show { path } => show(&path),
        EntryCommand::New { name, title, tags, body } => new(&name, title, tags, body),
        EntryCommand::Edit { path } => edit(&path),
        EntryCommand::Set { path, title, tags } => set(&path, title, tags),
    }
}

// ── list ──────────────────────────────────────────────────────────────────────

fn list(vault: &Path) -> Result<()> {
    let mut paths: Vec<_> = std::fs::read_dir(vault)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();

    paths.sort();

    for path in &paths {
        match read_entry(path) {
            Ok(entry) => println!("{}\t{}", path.display(), entry.title()),
            Err(e) => eprintln!("warn: {} — {e}", path.display()),
        }
    }

    Ok(())
}

// ── show ──────────────────────────────────────────────────────────────────────

fn show(path: &Path) -> Result<()> {
    let entry = read_entry(path)?;

    println!("# {}", entry.title());
    if let Some(date) = entry.frontmatter.date {
        println!("date: {date}");
    }
    if !entry.frontmatter.tags.is_empty() {
        println!("tags: {}", entry.frontmatter.tags.join(", "));
    }
    println!();
    print!("{}", entry.body);

    Ok(())
}

// ── new ───────────────────────────────────────────────────────────────────────

fn new(name: &str, title: Option<String>, tags: Vec<String>, body: Option<String>) -> Result<()> {
    let dest = resolve_dest(name);
    if dest.exists() {
        bail!("{} already exists", dest.display());
    }

    let body = match body {
        Some(b) => b,
        None => prompt_editor(title.as_deref(), &tags)?,
    };

    let entry = Entry {
        path: dest.clone(),
        frontmatter: Frontmatter { title, tags, ..Default::default() },
        body,
    };

    std::fs::write(&dest, render_entry(&entry))
        .with_context(|| format!("failed to write {}", dest.display()))?;

    println!("created: {}", dest.display());
    Ok(())
}

// ── edit ──────────────────────────────────────────────────────────────────────

fn edit(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("{} does not exist", path.display());
    }

    let editor = resolve_editor();
    let status = Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("failed to launch editor `{editor}`"))?;

    if !status.success() {
        bail!("editor exited with non-zero status");
    }

    Ok(())
}

// ── set ───────────────────────────────────────────────────────────────────────

fn set(path: &Path, title: Option<String>, tags: Option<Vec<String>>) -> Result<()> {
    if title.is_none() && tags.is_none() {
        bail!("nothing to update — specify at least --title or --tags");
    }

    let mut entry = read_entry(path)?;

    if let Some(t) = title {
        entry.frontmatter.title = Some(t);
    }
    if let Some(ts) = tags {
        entry.frontmatter.tags = ts;
    }

    write_entry(&entry)?;
    println!("updated: {}", path.display());
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Add `.md` extension if missing.
fn resolve_dest(name: &str) -> PathBuf {
    if name.ends_with(".md") {
        PathBuf::from(name)
    } else {
        PathBuf::from(format!("{name}.md"))
    }
}

fn resolve_editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into())
}

/// Open a temp file in $EDITOR and return the body the user typed.
fn prompt_editor(title: Option<&str>, tags: &[String]) -> Result<String> {
    let editor = resolve_editor();

    let mut template =
        "# archelon: Write your entry below. Lines starting with '# archelon:' are ignored.\n"
            .to_owned();
    if let Some(t) = title {
        template.push_str(&format!("# archelon: title = {t}\n"));
    }
    if !tags.is_empty() {
        template.push_str(&format!("# archelon: tags  = {}\n", tags.join(", ")));
    }
    template.push('\n');

    let mut tmp = tempfile::Builder::new()
        .prefix("archelon-")
        .suffix(".md")
        .tempfile()?;
    tmp.write_all(template.as_bytes())?;
    tmp.flush()?;

    let status = Command::new(&editor)
        .arg(tmp.path())
        .status()
        .with_context(|| format!("failed to launch editor `{editor}`"))?;

    if !status.success() {
        bail!("editor exited with non-zero status");
    }

    let content = std::fs::read_to_string(tmp.path())?;
    let body: String = content
        .lines()
        .filter(|l| !l.starts_with("# archelon:"))
        .collect::<Vec<_>>()
        .join("\n");
    let body = body.trim_start_matches('\n').to_owned();

    if body.is_empty() {
        bail!("aborting: empty entry");
    }

    Ok(body)
}
