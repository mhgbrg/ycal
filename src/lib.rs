use chrono::{Datelike, Days, Locale, NaiveDate, Weekday};
use ramhorns::{Content, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{array, fmt};

pub struct CalendarParams {
    pub year: i32,
    pub locale: String,
    pub day_name_characters: usize,
    pub theme_css: String,
    pub special_days: Vec<SpecialDay>,
}

#[derive(Deserialize, Serialize)]
pub struct SpecialDay {
    pub date: NaiveDate,
    pub name: String,
    pub is_holiday: bool,
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
    is_holiday: bool,
    holiday_name: String,
}

#[derive(Deserialize)]
struct NagerHoliday {
    date: NaiveDate,
    #[serde(rename = "localName")]
    local_name: String,
}

#[derive(Debug)]
pub enum CalendarError {
    InvalidYear(i32),
    InvalidLocale(String),
    Template(String),
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::InvalidYear(y) => {
                write!(f, "year must be between 1 and 9999, got {}", y)
            }
            CalendarError::InvalidLocale(l) => {
                write!(
                    f,
                    "unknown locale '{}'. Use a locale code like en-GB, sv-SE, de-DE.",
                    l
                )
            }
            CalendarError::Template(e) => write!(f, "invalid template: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum HolidayError {
    Fetch(ureq::Error),
    Parse(ureq::Error),
}

impl fmt::Display for HolidayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HolidayError::Fetch(e) => write!(f, "failed to fetch holidays: {}", e),
            HolidayError::Parse(e) => write!(f, "failed to parse holidays: {}", e),
        }
    }
}

pub const TEMPLATE_SRC: &str = include_str!("../templates/calendar.mustache");

pub fn generate_calendar(params: CalendarParams) -> Result<String, CalendarError> {
    let year = params.year;
    if year < 1 || year > 9999 {
        return Err(CalendarError::InvalidYear(year));
    }

    let locale_str = params.locale.replace('-', "_");
    let locale: Locale = locale_str
        .parse()
        .map_err(|_| CalendarError::InvalidLocale(params.locale.clone()))?;

    let months: [Vec<NaiveDate>; 12] = array::from_fn(|i| {
        let month = (i + 1) as u32;
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let num_days = first.num_days_in_month() as u32;
        (0..num_days).map(|d| first + Days::new(d as u64)).collect()
    });

    let template =
        Template::new(TEMPLATE_SRC).map_err(|e| CalendarError::Template(e.to_string()))?;
    let template_data = build_template_data(
        year,
        &months,
        locale,
        params.day_name_characters,
        &params.special_days,
        params.theme_css,
    );
    Ok(template.render(&template_data))
}

fn build_template_data(
    year: i32,
    months: &[Vec<NaiveDate>; 12],
    locale: Locale,
    day_name_chars: usize,
    special_days: &[SpecialDay],
    theme_css: String,
) -> TemplateData {
    let mut day_map: HashMap<NaiveDate, Vec<&SpecialDay>> = HashMap::new();
    for d in special_days {
        day_map.entry(d.date).or_default().push(d);
    }

    let date_to_day_data = |date: &NaiveDate| -> DayData {
        let day = date.day();
        let wd = date.weekday();
        let is_weekend = wd == Weekday::Sat || wd == Weekday::Sun;
        let entries = day_map.get(date);
        let is_holiday = entries
            .map(|e| e.iter().any(|d| d.is_holiday))
            .unwrap_or(false);
        let is_last_day = date.month() == 12 && day == 31;

        let display_name: String = entries
            .into_iter()
            .flatten()
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

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
            is_holiday,
            holiday_name: display_name,
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

pub fn fetch_holidays(year: i32, country_code: &str) -> Result<Vec<SpecialDay>, HolidayError> {
    let url = format!(
        "https://date.nager.at/api/v3/PublicHolidays/{}/{}",
        year, country_code
    );

    let response = ureq::get(&url).call().map_err(HolidayError::Fetch)?;

    let nager_holidays: Vec<NagerHoliday> = response
        .into_body()
        .read_json()
        .map_err(HolidayError::Parse)?;

    Ok(nager_holidays
        .into_iter()
        .map(|h| SpecialDay {
            date: h.date,
            name: h.local_name,
            is_holiday: true,
        })
        .collect())
}
