# Contributing

Thank you for helping with sapphire-journal. A couple of conventions keep the
repos across this family consistent.

## Language

- **Code, comments, commit messages, and specs are written in English.** These
  projects are open source, and English keeps the entry point low for outside
  contributors.
- **Existing Japanese comments and specs are grandfathered.** You may leave a
  Japanese comment as-is, but please convert it to English when you touch that
  file for another reason (the "boy-scout rule"). A dedicated translation-only
  commit is not required. **New** comments must be in English.
- **User-facing docs are English-first**, with a Japanese version alongside
  (`README.ja.md` next to `README.md`). The two link to each other at the top.

## Repository layout

- **Binary crates** live in a directory named after the short name
  (`cli/`, `desktop/`, `server/`) directly under the repo root.
- **Library crates** live under `crates/`, in a directory named after the
  full crate name (e.g. `crates/sapphire-journal-core/`).
- Non-crate directories (the `sapphire-journal-vscode/` extension, `docs/`)
  stay at the repo root.

---

# 開発者向け（日本語）

## 言語規約

- **コード・コメント・コミットメッセージ・仕様書は英語で書きます。**
  オープンソースとして間口を広げるためです。
- **すでに日本語で書かれたコメントや仕様はそのままでも構いません。**
  ただし別目的でそのファイルを触るときに英語へ書き換えてください
  （ボーイスカウト・ルール）。翻訳専用のコミットは不要です。
  **新規**のコメントは英語で書いてください。
- **ユーザー向けドキュメントは英語版を基本**とし、同階層に日本語版
  （`README.md` と同じ階層の `README.ja.md`）を用意します。両者は冒頭で
  相互リンクします。

## リポジトリ構成

- **バイナリ用クレート**は、短い名前（`cli/`, `desktop/`, `server/`）の
  ディレクトリをリポジトリ直下に置きます。
- **ライブラリ用クレート**は `crates/` 直下に、正式なクレート名の
  ディレクトリで置きます（例: `crates/sapphire-journal-core/`）。
- クレートではないディレクトリ（`sapphire-journal-vscode/` 拡張機能や
  `docs/`）はリポジトリ直下のままにします。
