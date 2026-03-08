# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-03-09

### Added

- `task.started_at` timestamp field: auto-set when task status transitions to `in_progress`; supports manual override via `--task-started-at DATETIME` in `entry new` / `entry set`
- `--task-started` filter for `entry list` (CLI and MCP): matches in-progress tasks (with optional `--period` overlap check)
- Preserve unknown frontmatter fields across round-trips — unknown YAML keys in `Frontmatter`, `TaskMeta`, and `EventMeta` are now retained on read/write, preventing data loss when entries were created by a newer version of archelon

## [0.2.1] - 2026-03-08

### Fixed

- `entry fix`: repeated calls no longer accumulate blank lines between the frontmatter fence and the body ([#30](https://github.com/fluo10/archelon/pull/30))

## [0.2.0] - 2026-03-08

### Added

- Install scripts for Linux/macOS (`install.sh`) and Windows (`install.ps1`) — installs pre-built binaries to `~/.local/bin`
- `cargo-binstall` support for `archelon-cli` and `archelon-mcp`
- VS Code extension (`archelon-vscode`) with auto-fix on save, New Entry, Open Entry, Remove Entry, and List Entries commands; CLI binary bundled in platform-specific VSIX
- `--version` flag for `archelon-cli` and `archelon-mcp`
- `--journal-dir` global option for `archelon-mcp`
- `entry fix --touch` flag to optionally refresh `updated_at` when fixing an entry
- `entry fix` now syncs `closed_at` based on task status when fixing
- Crate-level documentation for `archelon-core`

### Changed

- `entry list`: per-field `--FIELD PERIOD` arguments replaced with `--FIELD` boolean selectors that apply a shared `--period` value — simplifies the interface and avoids redundant argument pairs
- `entry list`: `--event-start` / `--event-end` filters replaced with `--event-span`, which matches entries whose event `[start, end]` interval overlaps the given period (in-progress events are included)

### Fixed

- `period`: `overlaps_event` now correctly returns `false` when an entry has no event instead of always matching

## [0.1.1] - 2026-03-07

### Fixed

- `TaskMeta.due` and `TaskMeta.closed_at`: add `#[serde(default)]` to prevent "missing field" errors when these optional fields are absent from the YAML frontmatter

## [0.1.0] - 2026-03-07

### Added

- Initial project structure with workspace crates: `archelon-core`, `archelon-cli`, `archelon-mcp`
- Entry management: create, list, edit, remove commands
- Journal initialization (`archelon init`) with `.archelon/` directory discovery (walks up from current directory)
- Entry types: note, task (with status/due), event (with start/end)
- YAML frontmatter with required fields: `id`, `title`, `created_at`
- `id` field using [caretta-id](https://github.com/fluo10/caretta-id) — decisecond-precision BASE32 identifiers for collision-free concurrent editing
- File layout: `{year}/{id}_{slug}.md` for shallow, readable hierarchy
- Auto-rename entry file on create/update to keep filename in sync with title slug
- Entry list filtering: by period (per-field), tags, overdue status
- Entry list sorting
- Entry check/fix/remove commands with shared `EntryRef` type
- `--journal-dir` global option to override journal root
- `--new` flag for `edit` command to create entries in editor
- MCP server (`archelon-mcp`) for AI agent integration via stdio transport
- CI workflow for pull requests
- Release workflow

### Changed

- Unified title parameter and made frontmatter fields required
- Made `TaskMeta.status` and `EventMeta.start`/`end` required fields
- Minute-precision timestamp serialization
- Moved shared entry operations into `archelon-core::ops`
- Moved `body` into `EntryFields`
- Unmanaged files excluded from entry list output
