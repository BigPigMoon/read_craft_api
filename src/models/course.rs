use super::language::Language;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Course {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub language: Language,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCourse {
    pub title: String,
    pub language: Language,
}
