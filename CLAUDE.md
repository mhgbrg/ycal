# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ycal - Rust CLI tool that generates a self-contained HTML file for a printable yearly calendar (A4 portrait).

## Build & Run

To build:

```bash
cargo build
```

To run the generator:

```bash
cargo run -- 2026 --locale config/locale/en.json --holidays config/holidays/england_and_wales_2026.json > out/en.html
```

When generating the html page, run the exact command above, do not include things like `2>&1 && echo "OK"`.

Only run the generator command you need to verify a change in the generated html. You can assume that the user is running `./dev.sh` in a separate terminal.

When refactoring code without modifying its behavior, run the generator before and after the refactor and verify that there were no changes to the output.

## Architecture

Single-file Rust application (`src/main.rs`) + one template (`templates/calendar.mustache`).

Data flow: `main` → `build_template_data` (uses `date_to_day_data` closure to convert each `NaiveDate` into a `DayData` with combined `css_class` string) → ramhorns renders template → HTML to stdout.

Day styling uses a single `css_class: String` field that accumulates space-separated classes (e.g. `"red week-start"`). Ramhorns treats empty strings as falsy, so `{{#css_class}}` conditionals work without a separate boolean.

Holidays are provided via an optional `--holidays` JSON file with `[{ "date": "2026-01-01", "name": "New Year" }]`.

Output is a self-contained HTML file with embedded CSS, designed to print on one A4 portrait page.

## Style

- Rust code is formatted with `cargo fmt`.
- Mustache templates (`.mustache` files) use 2-space indentation.
