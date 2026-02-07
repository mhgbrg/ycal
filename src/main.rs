use chrono::{Datelike, Days, Locale, NaiveDate, Weekday};
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
    /// Locale code (e.g. en-GB, sv-SE, de-DE)
    #[arg(short, long, default_value = "en-GB")]
    locale: String,
    /// Number of characters to use for day names
    #[arg(short = 'd', long, default_value = "1")]
    day_name_characters: usize,
    /// Path to JSON special days file
    #[arg(long)]
    special_days: Option<PathBuf>,
    /// Path to CSS theme file
    #[arg(long, default_value = "config/themes/minimalist.css")]
    theme: PathBuf,
}

#[derive(Deserialize)]
struct SpecialDay {
    date: NaiveDate,
    name: String,
}

#[derive(Deserialize)]
struct NagerHoliday {
    date: NaiveDate,
    #[serde(rename = "localName")]
    local_name: String,
}

#[derive(Content)]
struct TemplateData {
    year: i32,
    halves: Vec<HalfData>,
    theme_css: String,
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

const TEMPLATE_SRC: &str = include_str!("../templates/calendar.mustache");

fn main() {
    let cli = Cli::parse();

    let year = cli.year;
    if year < 1 || year > 9999 {
        eprintln!("Error: year must be between 1 and 9999");
        process::exit(1);
    }

    let locale_str = cli.locale.replace('-', "_");
    let locale: Locale = match locale_str.parse() {
        Ok(l) => l,
        Err(_) => {
            eprintln!(
                "Error: unknown locale '{}'. Use a locale code like en-GB, sv-SE, de-DE.",
                cli.locale
            );
            process::exit(1);
        }
    };

    let holidays: Vec<SpecialDay> = cli
        .locale
        .split('-')
        .nth(1)
        .map(|country_code| fetch_holidays(country_code, year))
        .unwrap_or_default();

    let special_days: Vec<SpecialDay> = cli
        .special_days
        .as_ref()
        .map(|path| read_json(path.as_ref()))
        .unwrap_or_default();

    let theme_css = read_string(&cli.theme);

    let months: [Vec<NaiveDate>; 12] = array::from_fn(|i| {
        let month = (i + 1) as u32;
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let num_days = first.num_days_in_month() as u32;
        (0..num_days).map(|d| first + Days::new(d as u64)).collect()
    });

    let template = Template::new(TEMPLATE_SRC).expect("invalid calendar template");
    let template_data = build_template_data(
        year,
        &months,
        locale,
        cli.day_name_characters,
        &holidays,
        &special_days,
        theme_css,
    );
    let html = template.render(&template_data);
    print!("{}", html);
}

fn build_template_data(
    year: i32,
    months: &[Vec<NaiveDate>; 12],
    locale: Locale,
    day_name_chars: usize,
    holidays: &[SpecialDay],
    special_days: &[SpecialDay],
    theme_css: String,
) -> TemplateData {
    let mut holiday_map: HashMap<NaiveDate, Vec<String>> = HashMap::new();
    for h in holidays.iter().filter(|h| h.date.year() == year) {
        holiday_map.entry(h.date).or_default().push(h.name.clone());
    }

    let mut special_day_map: HashMap<NaiveDate, Vec<String>> = HashMap::new();
    for s in special_days.iter().filter(|s| s.date.year() == year) {
        special_day_map.entry(s.date).or_default().push(s.name.clone());
    }

    let date_to_day_data = |date: &NaiveDate| -> DayData {
        let day = date.day();
        let wd = date.weekday();
        let is_weekend = wd == Weekday::Sat || wd == Weekday::Sun;
        let holiday_names = holiday_map.get(date);
        let special_day_names = special_day_map.get(date);
        let is_holiday = holiday_names.is_some();
        let is_special = special_day_names.is_some();
        let is_last_day = date.month() == 12 && day == 31;

        let display_name: String = holiday_names
            .into_iter()
            .chain(special_day_names)
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

        let mut classes = Vec::new();
        if is_weekend || is_holiday {
            classes.push("holiday");
        }
        if wd == Weekday::Mon && day != 1 {
            classes.push("week-start");
        }
        if is_holiday || is_special {
            classes.push("has-holiday");
        }
        if is_last_day {
            classes.push("last-day");
        }

        let weekday_name = date.format_localized("%A", locale).to_string();
        let weekday_abbr: String = weekday_name.chars().take(day_name_chars).collect();

        DayData {
            day_number: day,
            weekday: capitalize_first(&weekday_abbr),
            week_number: date.iso_week().week(),
            is_week_start: wd == Weekday::Mon,
            is_weekend,
            is_month_start: day == 1,
            is_last_day,
            holiday_name: display_name,
            css_class: classes.join(" "),
        }
    };

    let halves = vec![
        HalfData {
            months: (0..6)
                .map(|i| {
                    let month_date = NaiveDate::from_ymd_opt(year, (i + 1) as u32, 1).unwrap();
                    let month_name = month_date.format_localized("%B", locale).to_string();
                    MonthData {
                        name: capitalize_first(&month_name),
                        days: months[i].iter().map(&date_to_day_data).collect(),
                    }
                })
                .collect(),
        },
        HalfData {
            months: (6..12)
                .map(|i| {
                    let month_date = NaiveDate::from_ymd_opt(year, (i + 1) as u32, 1).unwrap();
                    let month_name = month_date.format_localized("%B", locale).to_string();
                    MonthData {
                        name: capitalize_first(&month_name),
                        days: months[i].iter().map(&date_to_day_data).collect(),
                    }
                })
                .collect(),
        },
    ];

    TemplateData {
        year,
        halves,
        theme_css,
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn fetch_holidays(country_code: &str, year: i32) -> Vec<SpecialDay> {
    let url = format!(
        "https://date.nager.at/api/v3/PublicHolidays/{}/{}",
        year, country_code
    );
    let mut response = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: failed to fetch holidays from API: {}", e);
            process::exit(1);
        }
    };
    let nager_holidays: Vec<NagerHoliday> = match response.body_mut().read_json() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: failed to parse holidays from API: {}", e);
            process::exit(1);
        }
    };
    nager_holidays
        .into_iter()
        .map(|h| SpecialDay {
            date: h.date,
            name: h.local_name,
        })
        .collect()
}

fn read_json<T: DeserializeOwned>(path: &Path) -> T {
    let content = read_string(path);
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

fn read_string(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(string) => string,
        Err(e) => {
            eprintln!("Error: unable to read file '{}': {}", path.display(), e);
            process::exit(1);
        }
    }
}

