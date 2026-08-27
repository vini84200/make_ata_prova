# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust CLI/TUI tool that interactively collects exam-application data (course, professor, schedule, student roster) and renders it into a Brazilian Portuguese "ata de aplicação de prova" (exam proctoring record) as a LaTeX document, then compiles it to PDF in-process via `tectonic` (no external LaTeX install required) and opens it with the platform's PDF viewer.

## Commands

- Build: `cargo build`
- Run (interactive prompts via `inquire`): `cargo run`
- Test: `cargo test`
- Run a single test: `cargo test <test_name>` (e.g. `cargo test escapes_each_special_character`)
- Run the end-to-end PDF compile test (slow, exercises the real `tectonic` engine, skipped by default): `cargo test -- --ignored`

The binary is interactive (menu-driven via `inquire`), not scriptable with flags.

## Architecture

- `src/main.rs` — entry point. Presents a top-level menu (Create ata / Open existing ata / Exit), orchestrates saving the collected data to JSON and generating the PDF. `generate_pdf_from_template` is the join point: builds an `AtaTemplate`, renders it with Askama, writes `ata.tex`, then calls `tectonic::latex_to_pdf` to produce `ata.pdf` and opens it via the OS-appropriate command (`open_pdf`).
- `src/tui.rs` — all interactive data collection (`inquire` prompts) for `ExamApplication`, including the student sub-menu (import from CSV, add/remove/edit/list) and the CSV import shape (`CsvStudent`, expects columns `ID`, `nome`, `turma`, `email`). `edit_exam_application` lets an already-loaded application (from "Open existing ata") be revised — roster and exit-row count — before regenerating the PDF. Re-importing a CSV merges by `id` and keeps any `observations` already recorded, rather than discarding the current roster.
- `src/template.rs` — the data model (`ExamApplication`, `Schedule`, `Student`), all `Serialize`/`Deserialize` so an application can be saved to and reloaded from a JSON file (see "Open existing ata" flow). `AtaTemplate` is the Askama `Template` wrapping `&ExamApplication`, bound to `templates/ata.tex` with `escape = "tex"` (configured in `askama.toml`). `ExamApplication::date_br` and `Schedule::scheduled_{start,end}_br` format dates/times as `dd/mm/yyyy`/`HH:MM` for the template, instead of relying on `chrono`'s default (ISO) `Display`.
- `src/latex.rs` — `Tex`, a custom Askama `Escaper` that escapes LaTeX special characters (`\ { } _ ^ % & # $ ~`) in interpolated values. This is what makes the `"tex"` escaper in `askama.toml` safe to use for user-entered strings (names, observations, etc.) inside the template — it's the only thing standing between free-text input and a valid `.tex` file, so it has direct unit test coverage.
- `templates/ata.tex` — the Askama template (Jinja-like `{{ }}` / `{% %}` syntax) defining the actual LaTeX document layout: header, identification table, attendance list (looped over `exam.students`), schedule tracking, temporary-exit log (looped `exam.exit_rows` times), incident section, closing tally, and signature block.
- `src/ontime.rs` — optionally builds a 3-entry rundown (room opening, the timed exam with warning/danger timer thresholds, closing/collection) for [OnTime](https://ontime.vini84200.cc/mcp), a rundown/timer tool with its own MCP server, from `build_rundown_entries`. Two export formats read from that single source of truth:
  - `write_rundown_spreadsheet` writes an `.xlsx` with the column headers OnTime's spreadsheet importer recognizes (Rundown > Import > Import from spreadsheet) — this is the one that's actually importable through the OnTime UI.
  - `rundown_entries_json` matches the JSON shape OnTime's `ontime_batch_create_entries` MCP tool expects for its `entries` argument — for calling that tool directly (e.g. from an agent with the OnTime MCP server connected), not for importing as a project. OnTime's own project-file format (with a versioned `settings` block, custom fields, multiple rundowns, etc.) is internal and undocumented; do not hand-rederive it — feeding this JSON to OnTime's *project* import fails with "unable to parse settings, missing or incorrect version".

## Data flow

1. `tui::create_exam_application` prompts for exam metadata and builds/edits the student roster (optionally importing from a CSV export).
2. The resulting `ExamApplication` can be persisted as JSON (e.g. `application.json`) and reloaded later via "Open existing ata" — optionally through `tui::edit_exam_application` — instead of re-entering everything.
3. `main::generate_pdf_from_template` renders `templates/ata.tex` through Askama (escaping via `latex::Tex`) into `ata.tex`, then compiles that to `ata.pdf` via `tectonic`.

Generated `ata.tex`/`ata.pdf` and any application JSON files are gitignored — they're per-run output and carry real student data (name, email, ID), not source.

## Releases

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`, …) — `feat`/`fix` drive the version bump and `CHANGELOG.md` entries.

Releasing (tag, changelog, commit) goes through [release-it](https://github.com/release-it/release-it), run via `scripts/release.sh` (a thin `npx` wrapper — no `package.json`/lockfile in this repo). Configuration is in `.release-it.json`:

- `@release-it/conventional-changelog` reads commit history since the last `v*` tag to pick the next version and update `CHANGELOG.md`.
- `@release-it/bumper` reads/writes the version straight from `Cargo.toml` (`[package].version`) — there is no `package.json`; Cargo.toml is the single source of truth for the version.
- The `after:bump` hook runs `cargo check` so `Cargo.lock` picks up the new version before the release commit.
- `npm` publishing and GitHub releases are both disabled (`"npm": false`, `"github.release": false`) — this only manages git tags/commits/changelog locally. Enable `github.release` (and set `GITHUB_TOKEN`) once the repo has a GitHub remote, if desired.

Preview a release without changing anything: `scripts/release.sh --dry-run`.
