# stale/hidden フィルタリング実装計画（Issue #288 対応・CLI/MCP側）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 明確な終わりのない長期タスク（stale）と明示的に隠したタスク（hidden）を、
list/tree のデフォルトの検索対象から除外する。オプトインのオプトアウト解除
（`--include-stale` / `--include-hidden`）でいつでも復元できる。

**Architecture:** stale は **計算フラグ**（手動フィールドにしない）。`labels::entry_flags()`
が未完了タスク（`closed_at` なし）かつ `updated_at` が閾値未満のタスクに `Stale` を、
フロントマター `hidden: true` のエントリーに `Hidden` を付与する。除外は
`EntryFilter::matches()` のエントリレベルゲートとして実装し、マッチ理由
（overdue/in_progress/created_at等）に関係なく適用する。閾値はジャーナル設定
`[journal] stale_after_days`（デフォルト30）。task_status のライフサイクル
（open→in_progress→done/cancelled/archived）は変更しない（stale/hidden は直交次元）。

**Tech Stack:** Rust 2021 / serde+serde_yaml / rusqlite / clap 4 / rmcp + schemars

**Spec:** GitHub Issue #288（スコープは GUI 側だが、本計画は同じ Issue の CLI/MCP 側。
設計判断は下の Global Constraints に反映済み）

## Global Constraints

- 全タスクで `cargo test --workspace` が緑であること。
  `cargo clippy --workspace --all-targets` も警告なし。
- コメントは日本語（このリポジトリの既存コードに合わせる）。
- **task_status の値域は変更しない**（pending 等の新状態を作らない）。
- stale の定義: `task.status` が open/in_progress（＝ `task.closed_at` が None）かつ
  `updated_at` が now より `stale_after_days` 日以上前。closed_at あり（done 等）は stale ではない。
  イベント・ノート（task なし）は stale ではない。
- hidden の定義: フロントマター最上位に `hidden: true`。欠落または false は通常表示。
- 除外のデフォルトON: `hidden: true` は常に除外。stale は `include_stale` なしは除外。
  除外は period セレクタのマッチ結果に**優先**する（stale な in_progress タスクは
  task_in_progress でヒットしてもデフォルトでは出さない）。`task_unstarted` の
  unstarted タスクは updated_at が古ければ stale として同様に除外される（これでよい）。
- stale 閾値のデフォルトは30日。上書きはジャーナル設定
  `<root>/.sapphire-journal/config.toml` の `[journal] stale_after_days = <u64>`。
- キャッシュスキーマは `SCHEMA_VERSION` 4 → 5 に上げ、`hidden` カラムを追加する。
  バージョン不一致時の既存動作（`Error::CacheSchemaTooNew` を返し list_entries は
  ディスクスキャンへフォールバック）は変更しない。マイグレーション書込不要
  （`cache rebuild` で再生成される）。
- 既存のセレクタ（task_overdue / task_in_progress / task_unstarted / event_span /
  created_at / updated_at）と `active: true` のセマンティクスは変更しない。

## File Structure

| ファイル | 役割 |
|---|---|
| `crates/sapphire-journal-core/src/entry.rs` | `Frontmatter.hidden`（Option<bool>、skip_serializing_if）と `FrontmatterView.hidden` を追加 |
| `crates/sapphire-journal-core/src/labels.rs` | `EntryFlag::Stale` / `EntryFlag::Hidden` 追加、`entry_flags()` に stale 判定（閾値引数） |
| `crates/sapphire-journal-core/src/journal.rs` | `JournalSection.stale_after_days: Option<u64>`（デフォルト None→30） |
| `crates/sapphire-journal-core/src/cache.rs` | SCHEMA_VERSION 5、`hidden` カラム、`upsert_entry` / `list_entries_from_cache` 対応 |
| `crates/sapphire-journal-core/src/ops.rs` | `EntryFilter` に stale/hidden 除外ゲート、`list_entries` が設定から閾値を読む |
| `crates/sapphire-journal-core/src/text_input/filter.rs` | `FilterInputs` に `include_stale` / `include_hidden` 追加 |
| `crates/sapphire-journal-core/src/ops.rs`（EntryFields）+ `cli/src/commands/entry.rs` + MCP new/modify | `hidden` の書き込み（new/modify）対応 |
| `cli/src/commands/entry.rs` | `--include-stale` / `--include-hidden` / （既存 flags 表示は変更なし） |
| `crates/sapphire-journal-mcp/src/server.rs` | `EntryListParams` に `include_stale` / `include_hidden`、docコメント更新 |
| `docs/config/journal-config.toml` | `stale_after_days` の例を追記 |

## Task 1: core — データモデル・フラグ計算・フィルタ

**Files:**
- Modify: `crates/sapphire-journal-core/src/entry.rs`
- Modify: `crates/sapphire-journal-core/src/labels.rs`
- Modify: `crates/sapphire-journal-core/src/journal.rs`
- Modify: `crates/sapphire-journal-core/src/cache.rs`
- Modify: `crates/sapphire-journal-core/src/ops.rs`
- Modify: `crates/sapphire-journal-core/src/text_input/filter.rs`

**Interfaces produced:**
- `Frontmatter.hidden: Option<bool>` / `FrontmatterView.hidden: Option<bool>`（serde: 欠落=null、`hidden: true` で Some(true)）
- `EntryFlag::Stale`（as_str = "stale"）、`EntryFlag::Hidden`（as_str = "hidden"）
- `entry_flags(task, event, created_at, updated_at, stale_after: chrono::Duration) -> Vec<EntryFlag>`（シグネチャ変更: 閾値引数を追加。呼出側2箇所: `entry.rs` From impl、`cache::list_entries_from_cache`）
- `JournalSection.stale_after_days: Option<u64>`（デフォルト None → 30 として解釈）
- `EntryFilter` に `include_stale: bool` / `include_hidden: bool`（Default=false）と stale 判定に必要な `stale_after_days: u32` 相当の情報、`text_input::filter::FilterInputs` に `include_stale` / `include_hidden`
- `list_entries` は `state.journal.config()` の `stale_after_days`（None→30）を stale 判定閾値として使用する
- `EntryFields.hidden: Option<bool>`（update 時 None=変更なし、Some(true)/Some(false) で書換、create 時 None=書かない）
- `ops::check_entry` / `fix_entry` が `hidden` を不正フィールドとして弾かないこと（serde flatten の extra 経由でなく一級フィールド化する為、check 側の許可リストが対象）
- cache: `SCHEMA_VERSION: i32 = 5`、entries テーブルに `hidden INTEGER`、`upsert_entry` が `fm.hidden` を書き、`list_entries_from_cache` が復元して `EntryFlag::Hidden` を付与

**Steps:**
- [ ] entry.rs / labels.rs / journal.rs / cache.rs / ops.rs / filter.rs を上記に従い修正
- [ ] 単体テスト追加（labels の stale 境界: 閾値きっかり/超過/未満、closed タスクは stale でない、task なしは stale でない、hidden: true で Hidden フラグ、include_* オプトインで除外されない）
- [ ] `cargo test -p sapphire-journal-core` 緑

## Task 2: CLI / MCP / docs の表面

**Files:**
- Modify: `cli/src/commands/entry.rs`（`--include-stale` / `--include-hidden` を EntryFilterArgs に、hidden は new/modify の EntryFields に）
- Modify: `crates/sapphire-journal-mcp/src/server.rs`（EntryListParams に `include_stale` / `include_hidden` の Option<bool>、EntryNewParams/EntryModifyParams に `hidden`、docコメントの説明文を更新し stale/hidden のデフォルト除外を明記）
- Modify: `docs/config/journal-config.toml`（stale_after_days のコメント例）

**Interfaces consumes:** Task 1 の `FilterInputs.include_stale/include_hidden`、`EntryFields.hidden`。

**Steps:**
- [ ] CLI・MCP・docs を上記に従い修正
- [ ] MCP の既存テスト（server.rs tests）が緑、可能なら stale/hidden フィルタのテスト追加
- [ ] `cargo test --workspace` 緑

## Global Constraints for review

上記 Global Constraints をそのままレビュー観点として渡すこと。特に
「task_status の値域を変えない」「除外はマッチ理由に優先する」の2点は仕様適合レビューで必ず確認する。 
