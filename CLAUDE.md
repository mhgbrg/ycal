# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ycal - Rust CLI tool that generates a self-contained HTML file for a printable yearly calendar (A4 portrait).

## Build & Run

```bash
cargo build
cargo run -- 2026 > out/ycal.html
```

When generating the html page, run exactly `cargo run -- 2026 > out/ycal.html`, do not run `cargo run -- 2026 > out/ycal.html 2>&1 && echo "OK"`

## Architecture

Single-file Rust application (`src/main.rs`) + one template (`templates/calendar.mustache`).

Data flow: `main` → `build_month` (creates `DayEntry` per day) → `build_template_data` → `day_entry_to_data` (converts to `DayData` with combined `css_class` string) → ramhorns renders template → HTML to stdout.

Day styling uses a single `css_class: String` field that accumulates space-separated classes (e.g. `"red week-start"`). Ramhorns treats empty strings as falsy, so `{{#css_class}}` conditionals work without a separate boolean.

Holidays are provided via optional JSON config (`--config`) with `{ "holidays": [{ "date": "2026-01-01", "name": "New Year" }] }`.

Output is a self-contained HTML file with embedded CSS, designed to print on one A4 portrait page.

## Style

- Mustache templates (`.mustache` files) use 2-space indentation.
