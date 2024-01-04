use super::language::Language;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize, Validate)]
pub struct Course {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[validate(length(min = 1))]
    pub title: String,
    pub language: Language,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateCourse {
    #[validate(length(min = 1))]
    pub title: String,
    pub language: Language,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateCourse {
    pub id: i32,
    #[validate(length(min = 1))]
    pub title: String,
    pub language: Language,
}
