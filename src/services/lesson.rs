use std::error::Error;

use ::chrono::Utc;
use sqlx::Postgres;

use crate::models::lesson::{CreateLesson, Lesson, UpdateLesson};

/// Create the lesson in database
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

/// Find the lesson by id in database
pub async fn find_lesson_by_id(
    id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Lesson, Box<dyn Error>> {
    let lesson = sqlx::query_as!(
        Lesson,
        r#"
        SELECT
            id, created_at, updated_at, title, content_path, cover_path, subject, course_id
        FROM
            lessons
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(lesson)
}

/// Find all lessons in database
pub async fn find_all_lessons(pool: &sqlx::Pool<Postgres>) -> Result<Vec<Lesson>, Box<dyn Error>> {
    let lessons = sqlx::query_as!(
        Lesson,
        r#"
        SELECT
            id, created_at, updated_at, title, content_path, cover_path, subject, course_id
        FROM
            lessons
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(lessons)
}

/// Get all lessons in course from database
pub async fn find_lessons_in_course(
    course_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<Lesson>, Box<dyn Error>> {
    let lessons = sqlx::query_as!(
        Lesson,
        r#"
        SELECT
            id, created_at, updated_at, title, content_path, cover_path, subject, course_id
        FROM
            lessons
        WHERE
            course_id = $1
        ORDER BY created_at ASC
        "#,
        course_id
    )
    .fetch_all(pool)
    .await?;

    Ok(lessons)
}

/// Delete lesson by id from database
pub async fn delete_lesson_db(id: i32, pool: &sqlx::Pool<Postgres>) -> Result<(), Box<dyn Error>> {
    sqlx::query!("DELETE FROM lessons WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update lesson in database
pub async fn update_lesson_db(
    new_lesson: &UpdateLesson,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE lessons SET title = $2, subject = $3, cover_path = $4, updated_at = $5 WHERE id = $1",
        new_lesson.id,
        new_lesson.title,
        new_lesson.subject,
        new_lesson.cover_path,
        Utc::now().naive_utc(),
    )
    .execute(pool)
    .await?;

    Ok(())
}
