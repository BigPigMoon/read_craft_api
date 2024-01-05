use std::error::Error;

use sqlx::Postgres;

use crate::models::lesson::CreateLesson;

pub async fn create_lesson_db(
    lesson: &CreateLesson,
    content_path: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let new_lesson_id = sqlx::query!("INSERT INTO lessons (title, cover_path, subject, content_path, course_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    lesson.title,
    lesson.cover_path,
    lesson.subject,
    content_path,
    lesson.course_id,
)
        .fetch_one(pool)
        .await?
        .id;

    Ok(new_lesson_id)
}

pub async fn find_lesson_by_id() -> Result<(), Box<dyn Error>> {
    todo!();
}

pub async fn find_all_lessons() -> Result<(), Box<dyn Error>> {
    todo!();
}

pub async fn find_lessons_in_course() -> Result<(), Box<dyn Error>> {
    todo!();
}

pub async fn delete_lesson() -> Result<(), Box<dyn Error>> {
    todo!();
}

pub async fn update_lesson() -> Result<(), Box<dyn Error>> {
    todo!();
}
