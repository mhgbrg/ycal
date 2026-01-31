# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ycal - Rust CLI tool that generates a self-contained HTML file for a printable yearly calendar (A4 portrait).

## Build & Run

```bash
cargo build
cargo run -- 2026 > calendar.html
cargo run -- 2026 --config example_config.json > calendar.html
```

## Architecture

Single-file Rust application (`src/main.rs`). Dependencies: clap (CLI), chrono (dates), serde/serde_json (config parsing).

Output is a self-contained HTML file with embedded CSS, designed to print on one A4 portrait page.
