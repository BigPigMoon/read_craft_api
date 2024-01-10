use actix_web::web;

pub fn book_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/book"));
}
