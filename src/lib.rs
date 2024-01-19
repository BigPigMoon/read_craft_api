pub mod controllers;
pub mod extractors;
pub mod models;
pub mod services;
pub mod utils;

extern crate crypto;

use std::{env, sync::Mutex};

use actix_web::web;
use controllers::{
    auth::auth_config, book::book_config, card::card_config, course::course_config,
    group::group_config, image::image_config, language::get_languages, lesson::lesson_config,
    translator::trasnlator_config,
};
use dotenvy::dotenv;
use jwt_simple::algorithms::HS256Key;
use sqlx::{Pool, Postgres};
use utils::jwt::JwtUtil;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt: JwtUtil,
    pub redis: Mutex<redis::Connection>,
}

pub async fn get_db_conn() -> Pool<Postgres> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::postgres::PgPool::connect(&db_url)
        .await
        .expect("couldn't connected to database")
}

pub fn get_key() -> HS256Key {
    dotenv().ok();
    let key_srt = env::var("JWT_KEY").expect("JWT_KEY is not set in .env file");
    HS256Key::from_bytes(key_srt.as_bytes())
}

pub fn get_redis_conn() -> redis::Connection {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

    let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");

    let conn = client
        .get_connection()
        .expect("Failed to get redis connection");

    conn
}

pub async fn get_app_data() -> web::Data<AppState> {
    web::Data::new(AppState {
        pool: get_db_conn().await,
        jwt: JwtUtil { key: get_key() },
        redis: Mutex::new(get_redis_conn()),
    })
}

pub fn main_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth_config)
            .configure(course_config)
            .configure(lesson_config)
            .configure(book_config)
            .configure(card_config)
            .configure(group_config)
            .configure(trasnlator_config)
            .configure(image_config)
            .service(get_languages),
    );
}
