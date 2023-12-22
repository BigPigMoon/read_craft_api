use actix_web::{get, post, web, Responder};

pub fn course_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/course")
            .service(create_course)
            .service(get_course)
            .service(get_courses),
    );
}

#[post("/")]
async fn create_course() -> impl Responder {}

#[get("/")]
async fn get_courses() -> impl Responder {}

#[get("/{id}")]
async fn get_course() -> impl Responder {}
