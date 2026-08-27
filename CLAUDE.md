# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust CLI/TUI tool that interactively collects exam-application data (course, professor, schedule, student roster) and renders it into a Brazilian Portuguese "ata de aplicação de prova" (exam proctoring record) as a LaTeX document, then compiles it to PDF in-process via `tectonic` (no external LaTeX install required) and opens it with `xdg-open`.

## Commands

- Build: `cargo build`
- Run (interactive prompts via `inquire`): `cargo run`
- Test: `cargo test`
- Run a single test: `cargo test test_render_tex` (tests currently live only in `src/tex_tmpl_rs/lib.rs`, which is dead code — see below)

The binary is interactive (menu-driven via `inquire`), not scriptable with flags.

## Architecture

- `src/main.rs` — entry point. Presents a top-level menu (Create ata / Open existing ata / Exit), orchestrates saving the collected data to JSON and generating the PDF. `generate_pdf_from_template` is the join point: builds an `AtaTemplate`, renders it with Askama, writes `ata.tex`, then calls `tectonic::latex_to_pdf` to produce `ata.pdf` and opens it.
- `src/tui.rs` — all interactive data collection (`inquire` prompts) for `ExamApplication`, including the student sub-menu (import from CSV, add/remove/edit/list) and the CSV import shape (`CsvStudent`, expects columns `ID`, `nome`, `turma`, `email`).
- `src/template.rs` — the data model (`ExamApplication`, `Schedule`, `Student`), all `Serialize`/`Deserialize` so an application can be saved to and reloaded from a JSON file (see "Open existing ata" flow). `AtaTemplate` is the Askama `Template` wrapping `&ExamApplication`, bound to `templates/ata.tex` with `escape = "tex"` (configured in `askama.toml`).
- `src/latex.rs` — `Tex`, a custom Askama `Escaper` that escapes LaTeX special characters (`\ { } _ ^ % & # $ ~`) in interpolated values. This is what makes the `"tex"` escaper in `askama.toml` safe to use for user-entered strings (names, observations, etc.) inside the template.
- `templates/ata.tex` — the Askama template (Jinja-like `{{ }}` / `{% %}` syntax) defining the actual LaTeX document layout: header, identification table, attendance list (looped over `exam.students`), schedule tracking, temporary-exit log (looped `exam.exit_rows` times), incident section, closing tally, and signature block.
- `src/tex_tmpl_rs/` — a **separate, unused Handlebars-based** TeX-rendering module (`prepare_tex`/`render_pdf`/`render_tex` using the `handlebars` crate). It is not declared as a module from `main.rs` and is not compiled into the binary; treat it as legacy/dead code, not part of the live rendering path. The live path is Askama (`src/template.rs` + `templates/ata.tex`), not this module.

## Data flow

1. `tui::create_exam_application` prompts for exam metadata and builds/edits the student roster (optionally importing from a CSV export).
2. The resulting `ExamApplication` can be persisted as JSON (e.g. `application.json`) and reloaded later via "Open existing ata" instead of re-entering everything.
3. `main::generate_pdf_from_template` renders `templates/ata.tex` through Askama (escaping via `latex::Tex`) into `ata.tex`, then compiles that to `ata.pdf` via `tectonic`.

Several `*.json` files at the repo root (e.g. `application.json`, `REC_A.json`, `dfg.json`, `ex_final_f_26_1.json`) are example/real saved `ExamApplication` payloads from prior runs — useful as reference for the JSON shape, not part of the build.
