# archelon

Markdown-based task and note manager that keeps your data alive as plain text — timeless like fossils.

## Concept

- **Markdown as source of truth** — all data lives in plain `.md` files you can read and edit with any tool
- **SQLite as cache** — fast querying and indexing on top of the Markdown files (planned)
- **Bullet-journal style** — notes and tasks coexist freely in one file, no forced separation
- **Obsidian-compatible layout** — flat directory of `.md` files with YAML frontmatter and `[[wikilinks]]`

## Data model

Each file is an **Entry** — the primary unit of data. An entry can contain free-form notes, task checkboxes (`- [ ]`), or both.

```markdown
---
title: My Note
date: 2026-03-04
tags: [work, project]
---

- [ ] Task A
- [x] Task B (done)

Some free-form notes here.
```

## CLI usage

```bash
# Create a new entry (opens $EDITOR if --body is omitted)
archelon entry new <name> [--title TITLE] [--tags tag1,tag2] [--body BODY]

# List all entries in a directory
archelon entry list [PATH]

# Show an entry
archelon entry show <file>

# Open an entry in $EDITOR
archelon entry edit <file>

# Update frontmatter fields inline (no editor)
archelon entry set <file> [--title TITLE] [--tags tag1,tag2]
archelon entry set <file> --tags          # clear all tags
```

## Project structure

```
archelon/
├── archelon-core/   # Data model, Markdown parser/serializer, (future) SQLite cache
└── archelon-cli/    # CLI binary built with clap
```

## Status

Early development — CLI is functional for basic entry management.
SQLite caching and `[[wikilink]]` support are planned.

## License

MIT OR Apache-2.0
