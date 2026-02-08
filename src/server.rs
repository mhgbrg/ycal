use axum::extract::Query;
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use chrono::Datelike;
use clap::Parser;
use serde::Deserialize;
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
    public_holidays: bool,
    #[serde(default)]
    special_days: String,
}

fn default_year() -> i32 {
    chrono::Local::now().year()
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
        .route("/calendar", get(calendar_handler));

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
                axum::http::StatusCode::BAD_REQUEST,
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
        public_holidays: q.public_holidays,
    };

    match ycal::generate_calendar(params) {
        Ok(html) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, format!("Error: {}", e)).into_response(),
    }
}
