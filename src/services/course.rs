use std::error::Error;

use chrono::Utc;
use sqlx::Postgres;

use crate::models::{
    course::{Course, CreateCourse, UpdateCourse},
    language::Language,
};

/// Create the course in db
pub async fn create_course_db(
    user_id: i32,
    course: &CreateCourse,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let new_course_id = sqlx::query!(
        "INSERT INTO courses (title, language) VALUES ($1, $2) RETURNING id",
        course.title,
        course.language as Language // NOTE: need to cast
    )
    .fetch_one(pool)
    .await?
    .id;

    // set owned for user
    sqlx::query!(
        "INSERT INTO course_user(owned, course_id, user_id) VALUES($1, $2, $3)",
        true,
        new_course_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(new_course_id)
}

/// Finding course by id
pub async fn find_course_by_id(
    id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Course, Box<dyn Error>> {
    let course = sqlx::query_as!(
        Course,
        r#"
            SELECT
            id, created_at, updated_at, title, language as "language!: Language"
            FROM courses
            WHERE id=$1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(course)
}

/// Update course in database
pub async fn update_course_db(
    new_course: UpdateCourse,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE courses SET title = $2, language = $3, updated_at = $4 WHERE id = $1",
        new_course.id,
        new_course.title,
        new_course.language as Language,
        Utc::now().naive_utc(),
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete course in database by id
pub async fn delete_course_db(
    course_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(r#"DELETE FROM courses WHERE id = $1"#, course_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// get all courses from database
pub async fn get_courses_db(pool: &sqlx::Pool<Postgres>) -> Result<Vec<Course>, Box<dyn Error>> {
    let courses = sqlx::query_as!(
        Course,
        r#"
            SELECT
            id, created_at, updated_at, title, language as "language!: Language"
            FROM courses
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(courses)
}

/// return true if user are owner of course
pub async fn user_is_owner(
    user_id: i32,
    course_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<bool, Box<dyn Error>> {
    let is_owner = sqlx::query!(
        r#"SELECT * FROM course_user WHERE course_id=$1 AND user_id=$2"#,
        course_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(is_owner.owned)
}

/// user are subscribe to course
pub async fn subscribe_to_course(
    user_id: i32,
    course_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO course_user(owned, course_id, user_id) VALUES($1, $2, $3)",
        false,
        course_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// user are unsubscribe to course
pub async fn unsubscribe_to_course(
    user_id: i32,
    course_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "DELETE FROM course_user WHERE course_id = $1 AND user_id = $2",
        course_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// get all courses in subscription
pub async fn get_subscribed(
    user_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<Course>, Box<dyn Error>> {
    let ids: Vec<i32> = sqlx::query!("SELECT id FROM course_user WHERE user_id=$1", user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|rec| rec.id)
        .collect();

    let mut courses: Vec<Course> = Vec::new();

    for id in ids.iter() {
        let course = find_course_by_id(*id, pool).await.unwrap();

        courses.push(course);
    }

    Ok(courses)
}
