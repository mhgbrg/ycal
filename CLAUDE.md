# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ycal - Rust CLI tool that generates a self-contained HTML file for a printable yearly calendar (A4 portrait).

## Build & Run

To build:

```bash
cargo build
```

To run the CLI generator:

```bash
just cli 2026 --locale en-GB --theme themes/minimalist.css > out/en.html
```

To run the web server:

```bash
just server
```

When generating the html page, run the exact command above, do not include things like `2>&1 && echo "OK"`.

Only run the generator command you need to verify a change in the generated html. You can assume that the user is running `just dev-cli` or `just dev-server` in a separate terminal.

When refactoring code without modifying its behavior, run the generator before and after the refactor and verify that there were no changes to the output.

## Architecture

Two binaries (`src/cli.rs` for CLI generation, `src/server.rs` for the web server) + a shared library (`src/lib.rs`) + one template (`templates/calendar.mustache`).

Data flow: `main` → `build_template_data` (uses `date_to_day_data` closure to convert each `NaiveDate` into a `DayData` with semantic boolean fields) → ramhorns renders template → HTML to stdout.

Day styling uses semantic boolean fields on `DayData` (`is_weekend`, `is_holiday`, `is_week_start`, `is_month_start`, `is_last_day`). The template builds CSS class strings from these bools, and theme CSS files map the class names to visual styles.

Special days are provided via an optional `--special-days` JSON file with `[{ "date": "2026-01-01", "name": "Dad's birthday", "is_holiday": false }]`. These display their name but without the red holiday styling. The web server has a `/holidays` proxy endpoint that fetches public holidays from the Nager API so users can review and edit them before generating.

Output is a self-contained HTML file with embedded CSS, designed to print on one A4 portrait page.

## Style

- Rust code is formatted with `cargo fmt`.
- Mustache templates (`.mustache` files) use 2-space indentation.
- In Rust code, structs are grouped at the top (after `use` statements), followed by constants, then functions in step-down order: callers before callees (e.g. `main` first, then the functions it calls, then their helpers).
- Rust imports: import types/enums/traits directly so usages have one level of qualification. Import modules when you use their contents qualified. Never use fully-qualified paths inline (e.g. `use axum::http::StatusCode;` then `StatusCode::BAD_REQUEST`, not `axum::http::StatusCode::BAD_REQUEST`). Group related imports from the same crate with braces (e.g. `use axum::http::{header, StatusCode};`).
