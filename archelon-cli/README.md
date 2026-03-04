# archelon-cli

Command-line interface for [archelon](https://github.com/fluo10/archelon) — a Markdown-based task and note manager.

## Installation

```bash
cargo install --path .
```

## Usage

### Initialize a journal

```bash
# Create a journal in the current directory (or a given path)
archelon init [PATH]
```

This creates `.archelon/config.toml` with the detected local timezone.
A `.archelon/.gitignore` is also created to track `config.toml` while ignoring the cache directory.

### Entry commands

#### Create a new entry

```bash
# Opens $EDITOR (like `git commit` without -m)
archelon entry new <name>

# Inline body — skips the editor (like `git commit -m`)
archelon entry new <name> --body "body text"

# With frontmatter fields
archelon entry new <name> \
  [--title TITLE] [--slug SLUG] [--tags tag1,tag2] \
  [--task-due DATETIME] [--task-status STATUS] [--task-closed-at DATETIME] \
  [--event-start DATETIME] [--event-end DATETIME]
```

The filename is auto-generated as `{year}/{caretta-id}_{slug}.md`.

#### List entries

```bash
# List all entries
archelon entry list [PATH]

# Filter by date range
archelon entry list --date 2026-03-05          # single day
archelon entry list --date-start 2026-03-01 --date-end 2026-03-31
archelon entry list --today
archelon entry list --this-week                # week start from journal config (default: Monday)
archelon entry list --this-month
```

#### Show, edit, update an entry

Entries are identified by file path or CarettaId prefix.

```bash
archelon entry show <file-or-id>
archelon entry edit <file-or-id>

# Update frontmatter fields (same fields as `entry new`)
archelon entry set <file-or-id> --title "New title"
archelon entry set <file-or-id> --tags tag1,tag2
archelon entry set <file-or-id> --tags          # clear all tags
archelon entry set <file-or-id> --task-status done
```

When `--task-status` is set to `done`, `cancelled`, or `archived`, `closed_at` is set automatically.

### DATETIME format

`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`.
For start/open datetimes (`--task-due`, `--event-end`), date-only input is interpreted as `23:59:59`.
For end/close datetimes (`--event-start`, `--task-closed-at`), date-only input is interpreted as `00:00:00`.

### Journal configuration

`.archelon/config.toml`:

```toml
[journal]
timezone = "Asia/Tokyo"   # IANA timezone name
week_start = "monday"     # or "sunday" — used by --this-week
```

## License

MIT OR Apache-2.0
