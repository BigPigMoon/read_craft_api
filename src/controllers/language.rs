use actix_web::{get, web, Responder};
use strum::IntoEnumIterator;

use crate::models::language::Language;

/// Get all available languages
///
/// Path:
/// **/api/languages/available**
#[get("/languages/available")]
async fn get_languages() -> impl Responder {
    let languages: Vec<String> = Language::iter().map(|lang| lang.to_string()).collect();

    web::Json(languages)
}
