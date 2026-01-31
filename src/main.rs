use chrono::{Datelike, NaiveDate, Weekday};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, process};

#[derive(Parser)]
#[command(about = "Generate a printable yearly calendar as HTML")]
struct Cli {
    /// Year to generate calendar for (1-9999)
    year: i32,
    /// Path to JSON configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
struct Config {
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    holidays: Vec<Holiday>,
}

#[derive(Deserialize)]
struct Holiday {
    date: String,
    name: String,
}

struct DayEntry {
    day_number: u32,
    weekday: Weekday,
    is_weekend: bool,
    is_holiday: bool,
    holiday_name: Option<String>,
}

fn weekday_abbrev(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "Mo",
        Weekday::Tue => "Tu",
        Weekday::Wed => "We",
        Weekday::Thu => "Th",
        Weekday::Fri => "Fr",
        Weekday::Sat => "Sa",
        Weekday::Sun => "Su",
    }
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

fn build_month(
    year: i32,
    month: u32,
    holiday_map: &HashMap<NaiveDate, String>,
) -> Vec<DayEntry> {
    let num_days = days_in_month(year, month);
    (1..=num_days)
        .map(|day| {
            let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let wd = date.weekday();
            let is_weekend = wd == Weekday::Sat || wd == Weekday::Sun;
            let holiday_name = holiday_map.get(&date).cloned();
            let is_holiday = holiday_name.is_some();
            DayEntry {
                day_number: day,
                weekday: wd,
                is_weekend,
                is_holiday,
                holiday_name,
            }
        })
        .collect()
}

const PASTEL_COLORS: &[&str] = &[
    "#b3d9ff", "#ffd9b3", "#d9b3ff", "#b3ffb3", "#ffb3d9", "#ffffb3", "#b3ffff", "#ffb3b3",
];

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_html(year: i32, months: &[Vec<DayEntry>; 12], config: &Config) -> String {
    let mut html = String::with_capacity(32_000);

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>");
    html.push_str(&year.to_string());
    html.push_str(" Calendar</title>\n<style>\n");
    html.push_str(CSS);
    html.push_str("</style>\n</head>\n<body>\n<div class=\"page\">\n");

    // Header
    html.push_str("<div class=\"header\">");
    html.push_str(&year.to_string());
    html.push_str("</div>\n");

    // Two halves: Jan-Jun, Jul-Dec
    for half in 0..2 {
        html.push_str("<div class=\"half\">\n");
        for col in 0..6 {
            let month_idx = half * 6 + col;
            let month_data = &months[month_idx];
            html.push_str("<div class=\"month\">\n");
            html.push_str("<div class=\"month-name\">");
            html.push_str(MONTH_NAMES[month_idx]);
            html.push_str("</div>\n");
            for entry in month_data {
                let is_red = entry.is_weekend || entry.is_holiday;
                let class = if is_red { " class=\"red\"" } else { "" };
                let tooltip = match &entry.holiday_name {
                    Some(name) => format!(" title=\"{}\"", html_escape(name)),
                    None => String::new(),
                };
                html.push_str("<div");
                html.push_str(class);
                html.push_str(&tooltip);
                html.push_str("><span class=\"day-label\">");
                html.push_str(&entry.day_number.to_string());
                html.push_str(weekday_abbrev(entry.weekday));
                html.push_str("</span></div>\n");
            }
            html.push_str("</div>\n");
        }
        html.push_str("</div>\n");
    }

    // Footer with category legend
    if !config.categories.is_empty() {
        html.push_str("<div class=\"footer\">\n");
        for (i, cat) in config.categories.iter().enumerate() {
            let color = PASTEL_COLORS[i % PASTEL_COLORS.len()];
            html.push_str("<span class=\"cat\" style=\"background:");
            html.push_str(color);
            html.push_str("\">");
            html.push_str(&html_escape(cat));
            html.push_str("</span>\n");
        }
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n</body>\n</html>\n");
    html
}

const CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }

@page {
    size: A4 portrait;
    margin: 8mm;
}

body {
    font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 8.5pt;
    line-height: 1.0;
    color: #222;
}

.page {
    width: 194mm;
    height: 281mm;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.header {
    height: 8mm;
    font-size: 16pt;
    font-weight: 700;
    display: flex;
    align-items: center;
    padding-left: 1mm;
}

.half {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    border-top: 1px solid #444;
    border-left: 1px solid #444;
    min-height: 0;
}

.month {
    border-right: 1px solid #444;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.month-name {
    font-weight: 700;
    font-size: 7.5pt;
    text-align: center;
    padding: 0.5mm 0;
    border-bottom: 1px solid #444;
    background: #f5f5f5;
    flex-shrink: 0;
}

.month > div:not(.month-name) {
    flex: 1;
    display: flex;
    align-items: center;
    padding: 0 0.8mm;
    border-bottom: 1px solid #ddd;
    min-height: 0;
    font-variant-numeric: tabular-nums;
}

.day-label {
    white-space: nowrap;
    flex-shrink: 0;
}

.red {
    color: #cc0000;
}

.footer {
    height: 10mm;
    display: flex;
    align-items: center;
    gap: 3mm;
    padding: 0 1mm;
    border-top: 1px solid #444;
    flex-shrink: 0;
}

.cat {
    padding: 1mm 2.5mm;
    border-radius: 1mm;
    font-size: 7.5pt;
    font-weight: 500;
    border: 0.5px solid #bbb;
}

@media screen {
    body {
        display: flex;
        justify-content: center;
        padding: 10mm;
        background: #e0e0e0;
    }
    .page {
        background: white;
        box-shadow: 0 1mm 4mm rgba(0,0,0,0.2);
        padding: 8mm;
        width: 210mm;
        height: 297mm;
    }
}

@media print {
    body {
        background: none;
    }
}
"#;

fn main() {
    let cli = Cli::parse();
    let year = cli.year;

    if year < 1 || year > 9999 {
        eprintln!("Error: year must be between 1 and 9999");
        process::exit(1);
    }

    let config = match cli.config {
        Some(path) => {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading config file '{}': {}", path.display(), e);
                    process::exit(1);
                }
            };
            match serde_json::from_str::<Config>(&content) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing config JSON: {}", e);
                    process::exit(1);
                }
            }
        }
        None => Config::default(),
    };

    // Build holiday lookup map
    let mut holiday_map: HashMap<NaiveDate, String> = HashMap::new();
    for h in &config.holidays {
        match NaiveDate::parse_from_str(&h.date, "%Y-%m-%d") {
            Ok(date) => {
                if date.year() == year {
                    holiday_map.insert(date, h.name.clone());
                }
            }
            Err(e) => {
                eprintln!("Warning: skipping invalid holiday date '{}': {}", h.date, e);
            }
        }
    }

    // Build month data
    let months: [Vec<DayEntry>; 12] = std::array::from_fn(|i| {
        build_month(year, (i + 1) as u32, &holiday_map)
    });

    let html = render_html(year, &months, &config);
    print!("{}", html);
}
