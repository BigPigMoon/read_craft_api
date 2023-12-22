use actix_web::{get, post, web, HttpResponse, Responder};

pub fn course_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/course")
            .service(create_course)
            .service(get_course)
            .service(get_courses),
    );
}

#[post("/")]
pub async fn create_course() -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/")]
pub async fn get_courses() -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/{id}")]
pub async fn get_course() -> impl Responder {
    HttpResponse::NotImplemented()
}
