use chrono::{Datelike, Days, NaiveDate, Weekday};
use clap::Parser;
use ramhorns::{Content, Template};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{array, fs, process};

#[derive(Parser)]
#[command(about = "Generate a printable yearly calendar as HTML")]
struct Cli {
    /// Year to generate calendar for (1-9999)
    year: i32,
    /// Path to JSON locale file
    #[arg(short, long)]
    locale: PathBuf,
    /// Path to JSON holidays file
    #[arg(long)]
    holidays: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Locale {
    month_names: [String; 12],
    day_names: [String; 7],
}

#[derive(Deserialize)]
struct Holiday {
    date: NaiveDate,
    name: String,
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
    day_number: u32,
    weekday: String,
    week_number: u32,
    is_week_start: bool,
    is_weekend: bool,
    is_month_start: bool,
    is_last_day: bool,
    holiday_name: String,
    css_class: String,
}

fn read_json<T: DeserializeOwned>(path: &Path) -> T {
    let content = match fs::read_to_string(path) {
        Ok(string) => string,
        Err(e) => {
            eprintln!("Error: unable to read file '{}': {}", path.display(), e);
            process::exit(1);
        }
    };
    match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "Error: unable to parse file content as JSON '{}': {}",
                path.display(),
                e
            );
            process::exit(1);
        }
    }
}

fn build_template_data(
    year: i32,
    months: &[Vec<NaiveDate>; 12],
    locale: &Locale,
    holidays: &[Holiday],
) -> TemplateData {
    let holiday_map: HashMap<NaiveDate, String> = holidays
        .iter()
        .filter(|h| h.date.year() == year)
        .map(|h| (h.date, h.name.clone()))
        .collect();

    let date_to_day_data = |date: &NaiveDate| -> DayData {
        let day = date.day();
        let wd = date.weekday();
        let is_weekend = wd == Weekday::Sat || wd == Weekday::Sun;
        let holiday_name = holiday_map.get(date);
        let is_holiday = holiday_name.is_some();
        let is_last_day = date.month() == 12 && day == 31;

        let mut classes = Vec::new();
        if is_weekend || is_holiday {
            classes.push("red");
        }
        if wd == Weekday::Mon && day != 1 {
            classes.push("week-start");
        }
        if is_holiday {
            classes.push("has-holiday");
        }
        if is_last_day {
            classes.push("last-day");
        }

        DayData {
            day_number: day,
            weekday: locale.day_names[wd.num_days_from_monday() as usize].clone(),
            week_number: date.iso_week().week(),
            is_week_start: wd == Weekday::Mon,
            is_weekend,
            is_month_start: day == 1,
            is_last_day,
            holiday_name: holiday_name.cloned().unwrap_or_default(),
            css_class: classes.join(" "),
        }
    };

    let halves = vec![
        HalfData {
            months: (0..6)
                .map(|i| MonthData {
                    name: locale.month_names[i].clone(),
                    days: months[i].iter().map(&date_to_day_data).collect(),
                })
                .collect(),
        },
        HalfData {
            months: (6..12)
                .map(|i| MonthData {
                    name: locale.month_names[i].clone(),
                    days: months[i].iter().map(&date_to_day_data).collect(),
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

    let locale: Locale = read_json(&cli.locale);
    let holidays: Vec<Holiday> = cli
        .holidays
        .map(|path| read_json(path.as_ref()))
        .unwrap_or_default();

    let months: [Vec<NaiveDate>; 12] = array::from_fn(|i| {
        let month = (i + 1) as u32;
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let num_days = first.num_days_in_month() as u32;
        (0..num_days).map(|d| first + Days::new(d as u64)).collect()
    });

    let template = Template::new(TEMPLATE_SRC).expect("invalid calendar template");
    let template_data = build_template_data(year, &months, &locale, &holidays);
    let html = template.render(&template_data);
    print!("{}", html);
}
