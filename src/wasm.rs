use crate::{CalendarParams, SpecialDay};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct Params {
    year: i32,
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default = "default_day_name_characters")]
    day_name_characters: usize,
    #[serde(default = "default_day_font_size")]
    day_font_size: f32,
    #[serde(default = "default_month_font_size")]
    month_font_size: f32,
    #[serde(default = "default_notes_space")]
    notes_space: f32,
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default)]
    special_days: Vec<SpecialDay>,
}

fn default_locale() -> String {
    "en-GB".to_string()
}

fn default_day_name_characters() -> usize {
    1
}

fn default_day_font_size() -> f32 {
    7.0
}

fn default_month_font_size() -> f32 {
    7.0
}

fn default_notes_space() -> f32 {
    24.0
}

fn default_theme() -> String {
    "minimalist".to_string()
}

const MINIMALIST_CSS: &str = include_str!("../themes/minimalist.css");
const RETRO_CSS: &str = include_str!("../themes/retro.css");
const CONTEMPORARY_CSS: &str = include_str!("../themes/contemporary.css");

#[wasm_bindgen]
pub fn generate_calendar(params_json: &str) -> Result<String, JsValue> {
    let params: Params =
        serde_json::from_str(params_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let theme_css = resolve_theme(&params.theme)?;

    let calendar_params = CalendarParams {
        year: params.year,
        locale: params.locale,
        day_name_characters: params.day_name_characters,
        day_font_size_pt: params.day_font_size,
        month_font_size_pt: params.month_font_size,
        notes_space_mm: params.notes_space,
        theme_css: theme_css.to_string(),
        special_days: params.special_days,
    };

    crate::generate_calendar(calendar_params).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn resolve_theme(name: &str) -> Result<&'static str, JsValue> {
    match name {
        "minimalist" => Ok(MINIMALIST_CSS),
        "retro" => Ok(RETRO_CSS),
        "contemporary" => Ok(CONTEMPORARY_CSS),
        _ => Err(JsValue::from_str(&format!("unknown theme '{}'", name))),
    }
}
