use std::env;

use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
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

/// Create lesson from JSON
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

    if let Err(err) = find_course_by_id(lesson.course_id, &app_data.pool).await {
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

    let new_course_id = match create_lesson_db(
        &lesson.0,
        &format!("{}.md", lesson_filename),
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

/// Get all lessons in database
/// Or get lesson in course by query ?course={course_id}
#[get("/all")]
pub async fn get_lessons(
    _: JwtCred,
    filter: web::Query<GetAllLessonsFilter>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_lessons";

    log::info!("{}: attempting to get lessons, filter: {:?}", op, filter);

    match filter.course {
        Some(course_id) => match find_course_by_id(course_id, &app_data.pool).await {
            Ok(_) => match find_lessons_in_course(course_id, &app_data.pool).await {
                Ok(lessons) => {
                    log::info!("{}: courses are successfuly returned", op);

                    HttpResponse::Ok().json(lessons)
                }
                Err(err) => {
                    log::error!(
                        "{}: cannot get all lessons in course id: {}, error: {}",
                        op,
                        course_id,
                        err
                    );

                    HttpResponse::InternalServerError().json(ErrorResponse {
                        message: "".to_string(),
                    })
                }
            },
            Err(err) => {
                log::error!(
                    "{}: course by id: {} is not exist, error: {}",
                    op,
                    course_id,
                    err
                );

                HttpResponse::NotFound().json(ErrorResponse {
                    message: "course is not exist".to_string(),
                })
            }
        },

        None => match find_all_lessons(&app_data.pool).await {
            Ok(lessons) => {
                log::info!("{}: courses are successfuly returned", op);

                HttpResponse::Ok().json(lessons)
            }
            Err(err) => {
                log::error!("{}: cannot get all lessons, error: {}", op, err);

                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "cannot get all lessons".to_string(),
                })
            }
        },
    }
}

/// Get the lesson by id
#[get("/get/{id}")]
pub async fn get_lesson(
    _: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_lesson";
    let lesson_id = path.into_inner();

    log::info!("{}: attempting to get lesson by id: {}", op, lesson_id);

    let lesson = match find_lesson_by_id(lesson_id, &app_data.pool).await {
        Ok(lesson) => lesson,
        Err(err) => {
            log::error!(
                "{}: lesson by id: {} is not exist, error: {}",
                op,
                lesson_id,
                err,
            );

            return HttpResponse::NotFound().json(ErrorResponse {
                message: "lesson by id is not exist".to_string(),
            });
        }
    };

    log::info!("{}: lesson are getting, lesson: {:?}", op, lesson);

    HttpResponse::Ok().json(lesson)
}

/// Update lesson
/// Get json with new data
#[put("/update")]
pub async fn update_lesson(
    creds: JwtCred,
    new_lesson: web::Json<UpdateLesson>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "update_lesson";
    let user_id = creds.uid;
    let lesson_id = new_lesson.id;

    log::info!(
        "{}: attempting to update lesson, new_lesson: {:?}, user_id: {}",
        op,
        new_lesson,
        user_id
    );

    if new_lesson.validate().is_err() {
        log::error!("{}: data is not valid, data: {:?}", op, new_lesson);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "data is not valid".to_string(),
        });
    }

    let updating_lesson = match find_lesson_by_id(lesson_id, &app_data.pool).await {
        Ok(lesson) => lesson,
        Err(err) => {
            log::error!(
                "{}: lesson by id: {} is not exist, error: {}",
                op,
                lesson_id,
                err
            );

            return HttpResponse::NotFound().json(ErrorResponse {
                message: "lesson by id is not exist".to_string(),
            });
        }
    };

    if !user_is_owner(user_id, updating_lesson.course_id.unwrap(), &app_data.pool)
        .await
        .unwrap_or(false)
    {
        log::warn!(
            "{}: user by id: {}, is not owner of course id: {:?}",
            op,
            user_id,
            updating_lesson.course_id
        );

        return HttpResponse::Forbidden().json(ErrorResponse {
            message: "user is not owner of course".to_string(),
        });
    }

    if let Err(err) = update_lesson_db(&new_lesson, &app_data.pool).await {
        log::error!(
            "{}: connon update lesson by id: {} of user_id: {}, error: {}",
            op,
            lesson_id,
            user_id,
            err
        );

        return HttpResponse::InternalServerError().json(ErrorResponse {
            message: "cannot update lesson".to_string(),
        });
    };

    HttpResponse::Ok().json(lesson_id)
}

/// Delete lesson by id from path
#[delete("/delete/{id}")]
pub async fn delete_lesson(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "delete_lesson";

    let lesson_id = path.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attempting to delete lesson, lesson id: {}, user_id: {}",
        op,
        lesson_id,
        user_id
    );

    let deleting_lesson = match find_lesson_by_id(lesson_id, &app_data.pool).await {
        Ok(lesson) => lesson,
        Err(err) => {
            log::error!(
                "{}: lesson by id: {} was not found, error: {}",
                op,
                lesson_id,
                err,
            );

            return HttpResponse::NotFound();
        }
    };

    let course_id = deleting_lesson.course_id.unwrap();
    if !user_is_owner(user_id, course_id, &app_data.pool)
        .await
        .unwrap_or(false)
    {
        log::warn!(
            "{}: user by id: {}, is not owner of course id: {}",
            op,
            user_id,
            course_id
        );

        return HttpResponse::Forbidden();
    }

    if let Err(err) = delete_lesson_db(lesson_id, &app_data.pool).await {
        log::error!("{}: cannot delete lesson, error: {}", op, err);

        return HttpResponse::InternalServerError();
    }

    HttpResponse::Ok()
}

/// Upload lesson text to server
#[post("/upload/{id}")]
pub async fn upload_lesson_text(
    creds: JwtCred,
    path: web::Path<i32>,
    lesson_text: String,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "upload_lesson_text";

    let lesson_id = path.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attempting to upload lesson text to lesson: {}, user id: {}",
        op,
        lesson_id,
        user_id
    );

    let uploading_lesson = match find_lesson_by_id(lesson_id, &app_data.pool).await {
        Ok(lesson) => lesson,
        Err(err) => {
            log::error!(
                "{}: lesson by id: {} was not found, error: {}",
                op,
                lesson_id,
                err,
            );

            return HttpResponse::NotFound();
        }
    };

    if !user_is_owner(user_id, uploading_lesson.course_id.unwrap(), &app_data.pool)
        .await
        .unwrap_or(false)
    {
        log::warn!(
            "{}: user by id: {}, is not owner of course id: {:?}",
            op,
            user_id,
            uploading_lesson.course_id
        );

        return HttpResponse::Forbidden();
    }

    // create directory first
    // FIXME: add it into app_data!
    dotenv().ok();
    let file_path = env::var("LESSONS_DIR").unwrap_or("./lessons".to_string());

    create_dir_all(&file_path).await.unwrap();

    // write text into file
    let mut lesson_file =
        match File::create(format!("{}/{}", &file_path, uploading_lesson.content_path)).await {
            Ok(file) => file,
            Err(err) => {
                log::error!(
                    "{}: cannot create file, file path: {}/{}, error: {}",
                    op,
                    file_path,
                    uploading_lesson.content_path,
                    err
                );

                return HttpResponse::InternalServerError();
            }
        };

    if let Err(err) = lesson_file.write_all(lesson_text.as_bytes()).await {
        log::error!(
            "{}: cannot write lesson into file, file path: {}, error: {}",
            op,
            uploading_lesson.content_path,
            err
        );

        return HttpResponse::InternalServerError();
    };

    log::info!("{}: lessons are writed into file successfuly", op);

    HttpResponse::Ok()
}
