use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use serde::{Deserialize, Serialize};
use ycal::{CalendarParams, SpecialDay};

#[derive(Parser)]
#[command(about = "ycal web server")]
struct Cli {
    /// Port to listen on
    #[arg(long, default_value = "3000")]
    port: u16,
}

const SHELL_HTML: &str = include_str!("../templates/shell.html");

enum Theme {
    Minimalist,
    Retro,
    Contemporary,
}

impl Theme {
    fn from_str(s: &str) -> Option<Theme> {
        match s {
            "minimalist" => Some(Theme::Minimalist),
            "retro" => Some(Theme::Retro),
            "contemporary" => Some(Theme::Contemporary),
            _ => None,
        }
    }

    fn css(&self) -> &'static str {
        match self {
            Theme::Minimalist => include_str!("../themes/minimalist.css"),
            Theme::Retro => include_str!("../themes/retro.css"),
            Theme::Contemporary => include_str!("../themes/contemporary.css"),
        }
    }
}

#[derive(Deserialize)]
struct CalendarQuery {
    #[serde(default = "default_year")]
    year: i32,
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default = "default_day_name_characters")]
    day_name_characters: usize,
    #[serde(default)]
    special_days: String,
}

#[derive(Deserialize)]
struct HolidaysQuery {
    year: i32,
    locale: String,
}

#[derive(Deserialize)]
struct NagerHoliday {
    date: NaiveDate,
    #[serde(rename = "localName")]
    local_name: String,
}

#[derive(Serialize)]
struct HolidayEntry {
    date: NaiveDate,
    name: String,
    is_holiday: bool,
}

fn default_year() -> i32 {
    Local::now().year()
}
fn default_locale() -> String {
    "en-GB".to_string()
}
fn default_theme() -> String {
    "minimalist".to_string()
}
fn default_day_name_characters() -> usize {
    1
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app = Router::new()
        .route("/", get(shell_handler))
        .route("/calendar", get(calendar_handler))
        .route("/holidays", get(holidays_handler));

    let addr = format!("0.0.0.0:{}", cli.port);
    eprintln!("Listening on http://localhost:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn shell_handler() -> Html<&'static str> {
    Html(SHELL_HTML)
}

async fn calendar_handler(Query(q): Query<CalendarQuery>) -> impl IntoResponse {
    let theme = match Theme::from_str(&q.theme) {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Error: unknown theme '{}'", q.theme),
            )
                .into_response();
        }
    };
    let theme_css = theme.css().to_string();

    let special_days: Vec<SpecialDay> = if q.special_days.is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&q.special_days).unwrap_or_default()
    };

    let params = CalendarParams {
        year: q.year,
        locale: q.locale,
        day_name_characters: q.day_name_characters,
        theme_css,
        special_days,
    };

    match ycal::generate_calendar(params) {
        Ok(html) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("Error: {}", e)).into_response(),
    }
}

async fn holidays_handler(Query(q): Query<HolidaysQuery>) -> impl IntoResponse {
    let country_code = match q.locale.split('-').nth(1) {
        Some(cc) => cc.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Error: locale must contain a country code (e.g. en-GB)".to_string(),
            )
                .into_response();
        }
    };

    let url = format!(
        "https://date.nager.at/api/v3/PublicHolidays/{}/{}",
        q.year, country_code
    );

    let response = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Error: failed to fetch holidays: {}", e),
            )
                .into_response();
        }
    };

    let nager_holidays: Vec<NagerHoliday> = match response.into_body().read_json() {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Error: failed to parse holidays: {}", e),
            )
                .into_response();
        }
    };

    let entries: Vec<HolidayEntry> = nager_holidays
        .into_iter()
        .map(|h| HolidayEntry {
            date: h.date,
            name: h.local_name,
            is_holiday: true,
        })
        .collect();

    Json(entries).into_response()
}
