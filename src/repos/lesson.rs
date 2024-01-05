use std::{
    env,
    fs::{self, File},
};

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use dotenvy::dotenv;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::jwt_cred::JwtCred,
    models::{
        common::ErrorResponse,
        lesson::{CreateLesson, UpdateLesson},
    },
    services::{
        course::{find_course_by_id, user_is_owner},
        lesson::*,
    },
    AppState,
};

pub fn lesson_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/lesson")
            .service(create_lesson)
            .service(get_lessons)
            .service(get_lesson)
            .service(update_lesson)
            .service(upload_lesson_text)
            .service(delete_lesson),
    );
}

#[post("/create")]
pub async fn create_lesson(
    creds: JwtCred,
    lesson: web::Json<CreateLesson>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "create_lesson";

    let user_id = creds.uid;

    log::info!("{}: attempting to create lesson, lesson: {:?}", op, lesson);

    if lesson.validate().is_err() {
        log::error!("{}: data is not validated, data: {:?}", op, lesson);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("title field is empty"),
        });
    }

    if let Err(err) = find_course_by_id(&lesson.course_id, &app_data.pool).await {
        log::warn!(
            "{}: course by id: {} not found, error: {}",
            op,
            lesson.course_id,
            err
        );

        return HttpResponse::NotFound().json(ErrorResponse {
            message: "course not found".to_string(),
        });
    }

    if !user_is_owner(user_id, lesson.course_id, &app_data.pool)
        .await
        .unwrap_or(false)
    {
        log::warn!(
            "{}: user by id: {} is not owner of course id: {}",
            op,
            user_id,
            lesson.course_id,
        );

        return HttpResponse::Forbidden().json(ErrorResponse {
            message: "course not found".to_string(),
        });
    }

    let lesson_filename = Uuid::new_v4().to_string();

    dotenv().ok();

    let lessons_dir = env::var("LESSONS_DIR").unwrap_or("/lessons".to_string());

    let new_course_id = match create_lesson_db(
        &lesson.0,
        &format!("{}/{}.md", lessons_dir, lesson_filename),
        &app_data.pool,
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            log::error!(
                "{}: cannot create new lesson, error: {}, lesson: {:?}",
                op,
                err,
                lesson
            );

            return HttpResponse::BadRequest().json(ErrorResponse {
                message: String::from("invalid data"),
            });
        }
    };

    log::info!(
        "{}: lesson are successfuly created, course id: {}",
        op,
        new_course_id
    );

    HttpResponse::Created().json(new_course_id)
}

#[derive(Debug, Deserialize)]
pub struct GetAllLessonsFilter {
    course: Option<i32>,
}

#[get("/all")]
pub async fn get_lessons(
    creds: JwtCred,
    filter: web::Query<GetAllLessonsFilter>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    todo!();
    HttpResponse::Ok()
}

#[get("/get/{id}")]
pub async fn get_lesson(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    todo!();
    HttpResponse::Ok()
}

#[put("/update")]
pub async fn update_lesson(
    creds: JwtCred,
    new_lesson: web::Json<UpdateLesson>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    todo!();
    HttpResponse::Ok()
}

#[delete("/delete/{id}")]
pub async fn delete_lesson(
    creds: JwtCred,
    new_lesson: web::Json<UpdateLesson>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    todo!();
    HttpResponse::Ok()
}

#[post("/upload/{id}")]
pub async fn upload_lesson_text(
    creds: JwtCred,
    new_lesson: web::Json<UpdateLesson>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    todo!();
    HttpResponse::Ok()
}
