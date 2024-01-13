use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::language::Language;

#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Book {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub language: Language,
    pub filename: String,
    pub cover_path: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub progress: i32,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct CreateBook {
    pub title: String,
    pub language: Language,
    pub filename: String,
    pub cover_path: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct UpdateBook {
    pub id: i32,
    pub title: String,
    pub language: Language,
    pub cover_path: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
}
