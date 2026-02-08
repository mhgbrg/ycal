use chrono::NaiveDate;
use clap::Parser;
use serde::Deserialize;
use std::{fmt, process};
use ycal::SpecialDay;

#[derive(Parser)]
#[command(about = "Fetch public holidays from Nager.Date and output as special days JSON")]
struct Cli {
    /// Year to fetch holidays for
    year: i32,
    /// Country code (e.g. GB, SE, DE)
    country_code: String,
}

#[derive(Deserialize)]
struct NagerHoliday {
    date: NaiveDate,
    #[serde(rename = "localName")]
    local_name: String,
}

#[derive(Debug)]
enum HolidayError {
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

fn main() {
    let cli = Cli::parse();

    let special_days = fetch_holidays(cli.year, &cli.country_code).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        process::exit(1);
    });

    let json = serde_json::to_string_pretty(&special_days).unwrap();
    println!("{}", json);
}

fn fetch_holidays(year: i32, country_code: &str) -> Result<Vec<SpecialDay>, HolidayError> {
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
