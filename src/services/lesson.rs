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
    content_path,
    lesson.subject,
    lesson.course_id,
)
        .fetch_one(pool)
        .await?
        .id;

    Ok(new_lesson_id)
}
