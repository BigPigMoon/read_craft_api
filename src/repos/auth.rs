use actix_web::{post, web, HttpResponse, Responder};
use crypto::{digest::Digest, sha2::Sha256};
use jwt_simple::prelude::{Duration, *};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use validator::Validate;

use crate::{
    entities::{prelude::*, user},
    extractors::jwt_cred::JwtCred,
    models::{
        auth::{SignUpData, Tokens},
        common::ErrorResponse,
    },
    utils::jwt::scopes,
    AppState,
};

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(signup));
}

#[post("/signup")]
pub async fn signup(data: web::Json<SignUpData>, app_data: web::Data<AppState>) -> impl Responder {
    let db = &app_data.conn;
    let jwt_controller = &app_data.jwt;

    if data.validate().is_err() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("invalid data"),
        });
    }

    let hashed_password = bcrypt::hash(data.password.clone(), bcrypt::DEFAULT_COST).unwrap();

    let new_user = user::ActiveModel {
        email: ActiveValue::Set(data.email.clone()),
        username: ActiveValue::Set(data.username.clone()),
        password_hash: ActiveValue::Set(hashed_password),
        ..Default::default()
    };

    let res = match User::insert(new_user).exec(db).await {
        Ok(res) => res,
        Err(_) => {
            return HttpResponse::Conflict().json(ErrorResponse {
                message: String::from("user already exist"),
            })
        }
    };

    let access = jwt_controller
        .encode_token(
            JwtCred {
                uid: res.last_insert_id,
                email: data.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(20),
        )
        .unwrap();

    let refresh = jwt_controller
        .encode_token(
            JwtCred {
                uid: res.last_insert_id,
                email: data.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(14),
        )
        .unwrap();

    update_user_refresh(res.last_insert_id, db, Some(refresh.clone())).await;

    HttpResponse::Created().json(Tokens { access, refresh })
}

async fn update_user_refresh(id: i32, db: &DatabaseConnection, token: Option<String>) {
    let mut user: user::ActiveModel = User::find_by_id(id).one(db).await.unwrap().unwrap().into();

    if token.is_some() {
        let mut hasher = Sha256::new();
        hasher.input_str(&token.unwrap());
        let refresh_hash = hasher.result_str();
        user.refresh_token_hash = ActiveValue::Set(Some(refresh_hash));
    } else {
        user.refresh_token_hash = ActiveValue::Set(None);
    }

    user.update(db).await.unwrap();
}
