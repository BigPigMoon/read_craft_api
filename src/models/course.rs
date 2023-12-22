use super::language::Language;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Course {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub language: Language,
}

#[derive(Debug)]
pub struct CreateCourse {
    pub title: String,
    pub language: Language,
}
