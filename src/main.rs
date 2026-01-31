use chrono::{Datelike, NaiveDate, Weekday};
use clap::Parser;
use ramhorns::{Content, Template};
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
    config: PathBuf,
}

#[derive(Deserialize)]
struct Config {
    month_names: [String; 12],
    day_names: [String; 7],
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
    is_last_day: bool,
    holiday_name: Option<String>,
}

#[derive(Content)]
struct TemplateData {
    year: i32,
    halves: Vec<HalfData>,
}

#[derive(Content)]
struct HalfData {
    months: Vec<MonthData>,
}

#[derive(Content)]
struct MonthData {
    name: String,
    days: Vec<DayData>,
}

#[derive(Content)]
struct DayData {
    number: u32,
    weekday: String,
    css_class: String,
    has_tooltip: bool,
    tooltip: String,
}

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
            let is_last_day = month == 12 && day == 31;
            DayEntry {
                day_number: day,
                weekday: wd,
                is_weekend,
                is_holiday,
                is_last_day,
                holiday_name,
            }
        })
        .collect()
}

fn build_template_data(year: i32, months: &[Vec<DayEntry>; 12], config: &Config) -> TemplateData {
    let day_entry_to_data = |entry: &DayEntry| -> DayData {
        let mut classes = Vec::new();
        if entry.is_weekend || entry.is_holiday {
            classes.push("red");
        }
        if entry.weekday == Weekday::Mon && entry.day_number != 1 {
            classes.push("week-start");
        }
        if entry.is_last_day {
            classes.push("last-day");
        }
        let has_tooltip = entry.holiday_name.is_some();
        let tooltip = entry.holiday_name.clone().unwrap_or_default();
        DayData {
            number: entry.day_number,
            weekday: config.day_names[entry.weekday.num_days_from_monday() as usize].clone(),
            css_class: classes.join(" "),
            has_tooltip,
            tooltip,
        }
    };

    let halves = vec![
        HalfData {
            months: (0..6)
                .map(|i| MonthData {
                    name: config.month_names[i].clone(),
                    days: months[i].iter().map(&day_entry_to_data).collect(),
                })
                .collect(),
        },
        HalfData {
            months: (6..12)
                .map(|i| MonthData {
                    name: config.month_names[i].clone(),
                    days: months[i].iter().map(&day_entry_to_data).collect(),
                })
                .collect(),
        },
    ];
    TemplateData { year, halves }
}

const TEMPLATE_SRC: &str = include_str!("../templates/calendar.mustache");

fn main() {
    let cli = Cli::parse();
    let year = cli.year;

    if year < 1 || year > 9999 {
        eprintln!("Error: year must be between 1 and 9999");
        process::exit(1);
    }

    let content = match fs::read_to_string(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading config file '{}': {}", cli.config.display(), e);
            process::exit(1);
        }
    };
    let config = match serde_json::from_str::<Config>(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config JSON: {}", e);
            process::exit(1);
        }
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

    let template = Template::new(TEMPLATE_SRC).expect("invalid calendar template");
    let data = build_template_data(year, &months, &config);
    let html = template.render(&data);
    print!("{}", html);
}
