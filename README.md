# archelon

Markdown-based task and note manager that keeps your data alive as plain text — timeless like fossils.

## Concept

- **Markdown as source of truth** — all data lives in plain `.md` files you can read and edit with any tool
- **SQLite as cache** — fast querying and indexing on top of the Markdown files (planned)
- **Bullet-journal style** — notes and tasks coexist freely in one file, no forced separation
- **Obsidian-compatible layout** — flat directory of `.md` files with YAML frontmatter and `[[wikilinks]]`
- **Human–AI collaborative editing** — designed to work alongside AI agents (Claude, etc.) that can read, create, and edit entries in the same journal via git or Syncthing sync

## Design decisions

### Entry IDs: caretta-id instead of sequential numbers

Each entry filename is prefixed with a [caretta-id](https://github.com/fluo10/caretta-id) — a 7-character BASE32 identifier with decisecond precision (e.g. `123abcd_my_note.md`).

Sequential IDs would collide when a human and an AI agent add entries at the same time in a shared journal synced via git or Syncthing.
caretta-id uses the current Unix time in deciseconds as its value, so two entries created more than 0.1 seconds apart are guaranteed to have different IDs — a collision-free guarantee without any central coordinator.

### File layout: `{year}/{id}_{slug}.md`

Entries are grouped into year directories (e.g. `2026/`) to prevent the journal root from filling up over time, while keeping the hierarchy shallow enough to stay navigable.
The slug derived from the entry title keeps filenames readable even without opening archelon.

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

See [archelon-cli/README.md](archelon-cli/README.md) for the full command reference.

A **journal** is any directory tree that contains a `.archelon/` directory.
`archelon` locates it by walking up from the current directory, the same way `git` finds `.git/`.
Use `archelon init` to create one.

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
