use crate::services::course::user_is_owner;

use super::language::Language;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::error::Error;
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

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct CourseOut {
    pub id: i32,
    #[validate(length(min = 1))]
    pub title: String,
    pub language: Language,
    #[serde(rename(serialize = "isOwner", deserialize = "isOwner"))]
    pub is_owner: bool,
}

impl CourseOut {
    pub async fn from_course(
        course: Course,
        user_id: i32,
        pool: &Pool<Postgres>,
    ) -> Result<CourseOut, Box<dyn Error>> {
        let is_owner = user_is_owner(user_id, course.id, pool)
            .await
            .unwrap_or(false);

        Ok(CourseOut {
            id: course.id,
            title: course.title.clone(),
            language: course.language,
            is_owner,
        })
    }
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
