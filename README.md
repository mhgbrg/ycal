# ycal

CLI tool that generates printable yearly calendars.

## Features

- Print-optimized single-page A4 portrait layout
- Full theming support, with three built-in themes: minimalist, retro, and contemporary
- Month and day name localization
- Automatic public holidays fetched from the [Nager.Date API](https://date.nager.at/)
- Custom special days via a JSON file

## Usage

```
cargo run -- <YEAR> [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--locale, -l` | `en-GB` | Locale code (e.g. `sv-SE`, `de-DE`) |
| `--theme` | `config/themes/minimalist.css` | Path to CSS theme file |
| `--day-name-characters, -d` | `1` | Number of characters for weekday abbreviation |
| `--special-days` | — | Path to JSON file with custom days |

### Examples

```bash
# Swedish calendar with retro theme
cargo run -- 2026 --locale sv-SE --theme config/themes/retro.css > calendar.html

# German calendar, open in browser
cargo run -- 2026 -l de-DE > calendar.html && open calendar.html
```

## Special days

Provide a JSON file with custom days to highlight on the calendar:

```json
[
  { "date": "2026-06-15", "name": "Dad's birthday" }
]
```

Pass it with `--special-days`:

```bash
cargo run -- 2026 --special-days my-days.json > calendar.html
```

Public holidays are fetched automatically based on the country code in the locale (e.g. `GB` from `en-GB`).

## Themes

Three themes are bundled in `config/themes/`:

- **minimalist** — Clean sans-serif design with minimal decoration (default)
- **retro** — Typewriter-style monospace font with a vintage feel
- **contemporary** — Modern sans-serif with bolder visual accents

Use any CSS file as a custom theme:

```bash
cargo run -- 2026 --theme path/to/my-theme.css > calendar.html
```

## Development

```bash
cargo build
```

For a live-reload workflow during development, run `./dev.sh` in a separate terminal.
