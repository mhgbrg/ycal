use clap::Parser;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::{fs, process};
use ycal::{CalendarParams, SpecialDay};

#[derive(Parser)]
#[command(about = "Generate a printable yearly calendar as HTML")]
struct Cli {
    /// Year to generate calendar for (1-9999)
    year: i32,
    /// Locale code (e.g. en-GB, sv-SE, de-DE)
    #[arg(long, default_value = "en-GB")]
    locale: String,
    /// Number of characters to use for day names
    #[arg(long, default_value = "1")]
    day_name_characters: usize,
    /// Path to JSON special days file
    #[arg(long)]
    special_days: Option<PathBuf>,
    /// Day font size in pt
    #[arg(long, default_value = "10")]
    day_font_size: f32,
    /// Month name font size in pt
    #[arg(long, default_value = "10")]
    month_font_size: f32,
    /// Week number font size in pt
    #[arg(long, default_value = "6")]
    week_number_font_size: f32,
    /// Special day name font size in pt
    #[arg(long, default_value = "6")]
    special_day_font_size: f32,
    /// Space for notes below month names in mm
    #[arg(long, default_value = "40")]
    notes_space: f32,
    /// Highlight weekends and holidays with a background color
    #[arg(long, default_value = "true")]
    highlight_holidays: bool,
    /// Treat Saturdays as weekend days
    #[arg(long, default_value = "false")]
    saturday_is_weekend: bool,
    /// Path to CSS theme file
    #[arg(long)]
    theme: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    generate(cli);
}

fn generate(cli: Cli) {
    let user_special_days: Vec<SpecialDay> = cli
        .special_days
        .as_ref()
        .map(|path| read_json(path.as_ref()))
        .unwrap_or_default();

    let theme_css = read_string(&cli.theme);

    let params = CalendarParams {
        year: cli.year,
        locale: cli.locale,
        day_name_characters: cli.day_name_characters,
        day_font_size_pt: cli.day_font_size,
        month_font_size_pt: cli.month_font_size,
        week_number_font_size_pt: cli.week_number_font_size,
        special_day_font_size_pt: cli.special_day_font_size,
        notes_space_mm: cli.notes_space,
        theme_css,
        special_days: user_special_days,
        highlight_holidays: cli.highlight_holidays,
        saturday_is_weekend: cli.saturday_is_weekend,
    };

    match ycal::generate_calendar(params) {
        Ok(html) => print!("{}", html),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
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
