use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::future::try_join_all;
use serde::Deserialize;
use validator::Validate;

use crate::extractors::jwt_cred::JwtCred;

use crate::models::common::ErrorResponse;
use crate::models::course::{CourseOut, CreateCourse, UpdateCourse};
use crate::services::course::*;
use crate::AppState;

pub fn course_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/course")
            .service(create_course)
            .service(get_course)
            .service(get_courses)
            .service(delete_course)
            .service(update_course)
            .service(subscribe)
            .service(unsubscribe)
            .service(is_owner),
    );
}

/// Create course request
///
/// Create course with CreateCourse model
///
/// Path:
/// **/api/course/create**
#[post("/create")]
pub async fn create_course(
    creds: JwtCred,
    course: web::Json<CreateCourse>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "create_course";
    log::info!(
        "{}: attempting to create course, title: {}",
        op,
        course.title
    );

    if course.validate().is_err() {
        log::error!("{}: data is not validated, data: {:?}", op, course);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("title field is empty"),
        });
    }

    let new_course_id = match create_course_db(creds.uid, &course, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!(
                "{}: can't create new course, error: {}, title: {}, language: {:?}",
                op,
                err,
                course.title,
                course.language
            );
            return HttpResponse::BadRequest().json(ErrorResponse {
                message: String::from("invalid data"),
            });
        }
    };

    log::info!(
        "{}: course are successfuly created, course id: {}",
        op,
        new_course_id
    );

    HttpResponse::Created().json(new_course_id)
}

#[derive(Debug, Deserialize)]
pub struct GetAllCoursesFilter {
    subscriptions: Option<bool>,
}

/// Get all courses request
///
/// Path:
/// **/api/course/all**
/// or if need only subscriptions course
/// **/api/course/all?subscriptions=true**
#[get("/all")]
pub async fn get_courses(
    creds: JwtCred,
    filter: web::Query<GetAllCoursesFilter>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_courses";

    let user_id = creds.uid;

    log::info!(
        "{}: attempting to get all courses, filter: {:?}",
        op,
        filter
    );

    let courses;

    if filter.subscriptions.unwrap_or(false) {
        courses = get_subscribed(creds.uid, &app_data.pool).await;
    } else {
        courses = get_courses_db(&app_data.pool).await;
    }

    let courses = match courses {
        Ok(courses) => courses,
        Err(err) => {
            log::error!("{}: error getting courses, error: {}", op, err);

            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "can't get courses".to_string(),
            });
        }
    };

    let courses = try_join_all(
        courses
            .into_iter()
            .map(|course| CourseOut::from_course(course, user_id, &app_data.pool)),
    )
    .await
    .unwrap();

    log::info!(
        "{}: all courses are returting successful, course count: {}",
        op,
        courses.len()
    );

    HttpResponse::Ok().json(courses)
}

/// Get course by id request
///
/// Path:
/// **/api/course/get/*{id}***
#[get("/get/{id}")]
pub async fn get_course(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_course";

    let course_id = path.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attempting to get course by id: course id: {}",
        op,
        course_id
    );

    let course = match find_course_by_id(course_id, &app_data.pool).await {
        Ok(course) => CourseOut::from_course(course, user_id, &app_data.pool)
            .await
            .unwrap(),
        Err(err) => {
            log::error!(
                "{}: course by id: {} is not exist, error: {}",
                op,
                course_id,
                err
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                message: "course by id is not exist".to_string(),
            });
        }
    };

    log::info!("{}: course are getting, course: {:?}", op, course);

    HttpResponse::Ok().json(course)
}

/// Update course, get JSON with new data
///
/// Path:
/// **/api/course/update**
#[put("/update")]
pub async fn update_course(
    creds: JwtCred,
    new_course: web::Json<UpdateCourse>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "update_course";

    let user_id = creds.uid;
    let course_id = new_course.id;

    if new_course.validate().is_err() {
        log::error!("{}: data is not validated, data: {:?}", op, new_course);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "title field is empty".to_string(),
        });
    }

    if let Err(err) = find_course_by_id(course_id, &app_data.pool).await {
        log::warn!(
            "{}: course by id: {} was not found, error: {}",
            op,
            course_id,
            err
        );

        return HttpResponse::NotFound().json(ErrorResponse {
            message: "course by id not founded".to_string(),
        });
    }

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

        return HttpResponse::Forbidden().json(ErrorResponse {
            message: "user is not owner of course".to_string(),
        });
    }

    if let Err(err) = update_course_db(new_course.0, &app_data.pool).await {
        log::error!(
            "{}: connon update course by id: {} of user_id: {}, error: {}",
            op,
            course_id,
            user_id,
            err
        );

        return HttpResponse::InternalServerError().json(ErrorResponse {
            message: "cannot update course".to_string(),
        });
    };

    HttpResponse::Ok().json(course_id)
}

/// Delete course by id
///
/// Path:
/// **/api/course/delete/*{id}***
#[delete("/delete/{id}")]
pub async fn delete_course(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "delete_course";

    let course_id = path.into_inner();
    let user_id = creds.uid;

    if let Err(err) = find_course_by_id(course_id, &app_data.pool).await {
        log::warn!(
            "{}: course by id: {} was not found, error: {}",
            op,
            course_id,
            err
        );

        return HttpResponse::NotFound();
    }

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

    if let Err(err) = delete_course_db(course_id, &app_data.pool).await {
        log::error!("{}: cannot delete course, error: {}", op, err);

        return HttpResponse::InternalServerError();
    }

    HttpResponse::Ok()
}

/// Subscribes the user to the course request
///
/// Path:
/// **/api/course/subscribe/*{id}***
#[post("/subscribe/{id}")]
pub async fn subscribe(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let course_id = path.into_inner();

    let op = "subscribe";

    log::info!(
        "{}: attempting to subscribe user on course, user_id: {}, course_id: {}",
        op,
        creds.uid,
        course_id
    );

    if let Err(err) = find_course_by_id(course_id, &app_data.pool).await {
        log::error!(
            "{}: course not found, error: {}, course_id: {}",
            op,
            err,
            course_id
        );

        return HttpResponse::NotFound();
    }

    if let Err(err) = subscribe_to_course(creds.uid, course_id, &app_data.pool).await {
        log::error!(
            "{}: error with subscribe to course, error: {}, course_id: {}, user_id: {}",
            op,
            err,
            course_id,
            creds.uid
        );

        return HttpResponse::InternalServerError();
    };

    log::info!(
        "{}: user are successfuly subscribed, user_id: {}, course_id: {}",
        op,
        creds.uid,
        course_id
    );

    HttpResponse::Ok()
}

/// Unsubscribe from the course by id
///
/// Path:
/// **/api/course/unsubscribe/*{id}***
#[post("/unsubscribe/{id}")]
pub async fn unsubscribe(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let course_id = path.into_inner();

    let op = "unsubscribe";

    log::info!(
        "{}: attempting to unsubscribe user on course, course_id: {}, user_id: {}",
        op,
        course_id,
        creds.uid
    );

    if let Err(err) = find_course_by_id(course_id, &app_data.pool).await {
        log::error!(
            "{}: course not found, error: {}, course_id: {}",
            op,
            err,
            course_id
        );

        return HttpResponse::NotFound();
    }

    if let Err(err) = unsubscribe_to_course(creds.uid, course_id, &app_data.pool).await {
        log::error!(
            "{}: error with unsubscribe to course, error: {}, course_id: {}, user_id: {}",
            op,
            err,
            course_id,
            creds.uid
        );

        return HttpResponse::InternalServerError();
    };

    log::info!(
        "{}: user are successfuly unsubscribed, user_id: {}, course_id: {}",
        op,
        creds.uid,
        course_id
    );

    HttpResponse::Ok()
}

/// Return true if user are owner of course request
///
/// Path:
/// **/api/course/owner/{id}**
#[get("/owner/{id}")]
pub async fn is_owner(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let course_id = path.into_inner();

    let op = "is_owner";

    log::info!(
        "{}: attempting to get owner of course, user_id: {}, course_id: {}",
        op,
        creds.uid,
        course_id
    );

    let ownered = match user_is_owner(creds.uid, course_id, &app_data.pool).await {
        Ok(ownered) => ownered,
        Err(err) => {
            log::error!(
                "{}: course by id: {} is not exist, error: {}",
                op,
                course_id,
                err
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                message: "course by id is not exist".to_string(),
            });
        }
    };

    log::info!(
        "{}: user id: {} is owner: {} by course id: {}",
        op,
        creds.uid,
        ownered,
        course_id
    );

    HttpResponse::Ok().json(ownered)
}
