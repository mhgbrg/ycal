# ycal

Tool for generating printable yearly calendars.

## Themes

ycal has three built-in themes:

- **minimalist** — Clean sans-serif design with minimal decoration
  ![minimalist theme](screenshots/minimalist.png)
- **retro** — Typewriter-style monospace font with a vintage feel
  ![retro theme](screenshots/retro.png)
- **contemporary** — Modern sans-serif with bolder visual accents
  ![contemporary theme](screenshots/contemporary.png)

## Web UI

The easiest way to use ycal is to use the web UI. To start the web server, run:

```bash
just serve
```

This opens a local server at `http://localhost:3000`.

## CLI

The CLI can be used to generate a self-contained HTML file that can be manually printed.

```
$ just gen --help

Generate a printable yearly calendar as HTML

Usage: cli [OPTIONS] --theme <THEME> <YEAR>

Arguments:
  <YEAR>  Year to generate calendar for (1-9999)

Options:
      --locale <LOCALE>
          Locale code (e.g. en-GB, sv-SE, de-DE) [default: en-GB]
      --day-name-characters <DAY_NAME_CHARACTERS>
          Number of characters to use for day names [default: 1]
      --special-days <SPECIAL_DAYS>
          Path to JSON special days file
      --theme <THEME>
          Path to CSS theme file
  -h, --help
          Print help
```

```bash
# Default settings
just gen 2026 > calendar.html

# Swedish calendar with retro theme, special days and using three characters for day names
just gen 2026 --locale sv-SE --day-name-characters 3 --special-days swedish_holidays_2026.json --theme themes/retro.css > calendar.html
```

### Special days

The `--special-days` option expects the file to have the following format:

```json
[
  { "date": "2026-07-07", "name": "Dad's birthday", "is_holiday": false },
  { "date": "2026-12-24", "name": "Christmas", "is_holiday": true }
]
```

Days with `"is_holiday": true` are styled in red like weekends, while `false` displays the name without any color change.

You can generate a special days file with public holidays using the bundled `holidays` script, which fetches the [Nager.Date API](https://date.nager.at/):

```bash
just holidays 2026 GB > holidays.json
just gen 2026 --locale en-GB --theme themes/minimalist.css --special-days holidays.json > calendar.html
```

You can also easily use Claude Code to generate a special days file with public holidays from an arbitrary website. For example, to create one with England's bank holidays:

```
claude -p "Fetch https://www.gov.uk/bank-holidays and extract the England bank holidays for 2026. Output a JSON array where each entry has \"date\" (YYYY-MM-DD), \"name\", and \"is_holiday\": true. Output only the JSON." > bank-holidays.json
```

## Development

```bash
cargo build
```

For live-reload during development:

```bash
# CLI. This starts a light-weight webserver that simply hosts the static files in the out/ folder.
just dev-cli

# Web UI. This starts the full web server.
just dev-server
```
