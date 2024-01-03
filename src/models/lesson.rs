use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Lesson {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub content_path: String,
    pub cover_path: Option<String>,
    pub subject: Option<String>,
    pub course_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateLesson {
    pub title: String,
    pub cover_path: Option<String>,
    pub subject: Option<String>,
    pub course_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateLesson {
    pub id: i32,
    pub title: String,
    pub cover_path: Option<String>,
    pub subject: Option<String>,
}
