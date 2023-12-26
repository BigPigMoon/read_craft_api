use actix_web::{get, post, web, HttpResponse, Responder};

use crate::extractors::jwt_cred::JwtCred;

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

#[post("/")]
pub async fn create_course(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/")]
pub async fn get_courses(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/{id}")]
pub async fn get_course(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/subscribe")]
pub async fn get_subs(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[post("/subscribe/{id}")]
pub async fn subscribe(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[get("/owner/{id}")]
pub async fn is_owner(creds: JwtCred) -> impl Responder {
    HttpResponse::NotImplemented()
}
