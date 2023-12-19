pub mod entities;
pub mod extractors;
pub mod models;
pub mod repos;
pub mod utils;

extern crate crypto;

use std::env;

use actix_web::web;
use dotenv::dotenv;
use jwt_simple::algorithms::HS256Key;
use sea_orm::{Database, DatabaseConnection};
use utils::jwt::JwtUtil;

#[derive(Debug)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub jwt: JwtUtil,
}

pub async fn get_db_conn() -> DatabaseConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    Database::connect(db_url).await.unwrap()
}

pub fn get_key() -> HS256Key {
    dotenv().ok();
    let key_srt = env::var("JWT_KEY").expect("JWT_KEY is not set in .env file");
    HS256Key::from_bytes(key_srt.as_bytes())
}

pub async fn get_app_data() -> web::Data<AppState> {
    let key = get_key();
    let conn = get_db_conn().await;

    web::Data::new(AppState {
        conn,
        jwt: JwtUtil { key },
    })
}
