use std::env;

use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use rc_api::get_app_data;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("It's actix read craft api")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("HOST is not set in .env file");
    let addrs = format!("{}:{}", host, port);

    let state = get_app_data().await;

    log::info!("starting HTTP server at http://{}", addrs);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%r, status: %s, time taken: %D"))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(state.clone())
            .service(web::scope("/api").service(index))
    })
    .bind(addrs)?
    .run()
    .await
}
