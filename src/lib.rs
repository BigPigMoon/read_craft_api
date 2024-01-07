pub mod extractors;
pub mod models;
pub mod controllers;
pub mod services;
pub mod utils;

extern crate crypto;

use std::env;

use actix_web::web;
use dotenvy::dotenv;
use jwt_simple::algorithms::HS256Key;
use controllers::{auth::auth_config, course::course_config, lesson::lesson_config};
use sqlx::{Pool, Postgres};
use utils::jwt::JwtUtil;

#[derive(Debug)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt: JwtUtil,
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

pub async fn get_app_data() -> web::Data<AppState> {
    let key = get_key();
    let pool = get_db_conn().await;

    web::Data::new(AppState {
        pool,
        jwt: JwtUtil { key },
    })
}

pub fn main_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth_config)
            .configure(course_config)
            .configure(lesson_config),
    );
}
