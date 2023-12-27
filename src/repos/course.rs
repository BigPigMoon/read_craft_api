use actix_web::{get, post, web, HttpResponse, Responder};

use crate::extractors::jwt_cred::JwtCred;

use crate::models::common::ErrorResponse;
use crate::models::course::CreateCourse;
use crate::services::course::{
    create_course_db, find_course_by_id, get_courses_db, get_subscribed, subscribe_to_course,
    user_is_owner,
};
use crate::AppState;

pub fn course_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/course")
            .service(create_course)
            .service(get_course)
            .service(get_courses)
            .service(get_subs)
            .service(subscribe)
            .service(is_owner),
    );
}

/// create course with CreateCourse model
#[post("/")]
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

    if course.title.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("title field is empty"),
        });
    }

    let new_course_id = match create_course_db(creds.uid, &course, &app_data.pool).await {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                message: String::from("invalid data"),
            })
        }
    };

    log::info!(
        "{}: course are successfuly created, course id: {}",
        op,
        new_course_id
    );

    HttpResponse::Created().json(new_course_id)
}

/// get all courses
#[get("/")]
pub async fn get_courses(_: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "get_courses";

    log::info!("{}: attempting to get all courses", op);

    let courses = match get_courses_db(&app_data.pool).await {
        Ok(courses) => courses,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "can't get all courses".to_string(),
            })
        }
    };

    log::info!(
        "{}: all courses are returting successful, course count: {}",
        op,
        courses.len()
    );

    HttpResponse::Ok().json(courses)
}

/// get course by id
#[get("/{id}")]
pub async fn get_course(
    _: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let course_id = path.into_inner();
    let op = "get_course";

    log::info!(
        "{}: attempting to get course by id: course id: {}",
        op,
        course_id
    );

    let course = match find_course_by_id(&course_id, &app_data.pool).await {
        Ok(course) => course,
        Err(_) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                message: "course by id is not exist".to_string(),
            })
        }
    };

    log::info!(
        "{}: course are getting, id: {}, title: {}, language: {:?}",
        op,
        course.id,
        course.title,
        course.language
    );

    HttpResponse::Ok().json(course)
}

/// get all courses to which the user is subscribed
#[get("/subscribe/")]
pub async fn get_subs(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "get_subs";

    log::info!("{}: attempting to get all subscription course", op);

    let courses = match get_subscribed(creds.uid, &app_data.pool).await {
        Ok(courses) => courses,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: "can't get subscribed courses".to_string(),
            })
        }
    };

    log::info!(
        "{}: all courses are returning successfuly, count of course: {}",
        op,
        courses.len()
    );

    HttpResponse::Ok().json(courses)
}

/// subscribes the user to the course
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

    match subscribe_to_course(creds.uid, course_id, &app_data.pool).await {
        Ok(_) => {}
        Err(_) => return HttpResponse::NotFound(),
    };

    log::info!(
        "{}: user are successfuly subscribed, user_id: {}, course_id: {}",
        op,
        creds.uid,
        course_id
    );

    HttpResponse::Ok()
}

/// return true if user are owner of course
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
        Err(_) => {
            log::info!("{}: course by id: {} is not exist", op, course_id);
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
