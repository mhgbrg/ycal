# ycal

CLI tool that generates printable yearly calendars.

## Features

- Print-optimized single-page A4 portrait layout
- Full theming support, with three built-in themes: minimalist, retro, and contemporary
- Month and day name localization
- Custom special days via a JSON file
- Experimental support for automatic inclusion of public holidays via the [Nager.Date API](https://date.nager.at/)

## Usage

```
cargo run -- <YEAR> [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--locale` | `en-GB` | Locale code (e.g. `sv-SE`, `de-DE`) |
| `--theme` | `config/themes/minimalist.css` | Path to CSS theme file |
| `--day-name-characters` | `1` | Number of characters for weekday abbreviation |
| `--special-days` | — | Path to JSON file with special days |
| `--public-holidays` | `false` | Fetch public holidays from the Nager.Date API (experimental) |

### Examples

```bash
# Swedish calendar with retro theme
cargo run -- 2026 --locale sv-SE --theme config/themes/retro.css > calendar.html

# German calendar with default theme
cargo run -- 2026 --locale de-DE > calendar.html
```

## Special days

Provide a JSON file with custom days to highlight on the calendar:

```json
[
  { "date": "2026-06-15", "name": "Dad's birthday", "is_holiday": false },
  { "date": "2026-12-25", "name": "Christmas", "is_holiday": true }
]
```

Pass it with `--special-days`:

```bash
cargo run -- 2026 --special-days my-days.json > calendar.html
```

Public holidays can be automatically included with the `--public-holidays` flag (experimental). They are fetched from the Nager.Date API based on the country code in the locale (e.g. `GB` from `en-GB`).

You can also easily use Claude Code to generate a special days file with public holidays from a website. For example, to create one with England's bank holidays:

```
claude -p "Fetch https://www.gov.uk/bank-holidays and extract the England bank holidays for 2026. Output a JSON array where each entry has \"date\" (YYYY-MM-DD), \"name\", and \"is_holiday\": true. Output only the JSON." > bank-holidays.json
```

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
