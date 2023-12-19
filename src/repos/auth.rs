use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use crypto::{digest::Digest, sha2::Sha256};
use jwt_simple::prelude::Duration;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use validator::Validate;

use crate::{
    entities::{prelude::*, user},
    extractors::jwt_cred::{get_token_from_req, AuthError, JwtCred},
    models::{
        auth::{SignInData, SignUpData, Tokens},
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

#[post("/signin")]
pub async fn signin(data: web::Json<SignInData>, app_data: web::Data<AppState>) -> impl Responder {
    let db = &app_data.conn;
    let jwt = &app_data.jwt;

    let user = User::find()
        .filter(user::Column::Email.eq(data.email.clone()))
        .one(db)
        .await
        .unwrap();

    let user = match user {
        Some(user) => user,
        None => {
            return HttpResponse::NotFound().json(ErrorResponse {
                message: String::from("user not found"),
            })
        }
    };

    let valid = bcrypt::verify(data.password.clone(), &user.password_hash).unwrap();
    if !valid {
        return HttpResponse::Forbidden().json(ErrorResponse {
            message: String::from("invalid password"),
        });
    }

    let access = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(20),
        )
        .unwrap();
    let refresh = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(14),
        )
        .unwrap();

    update_user_refresh(user.id, db, Some(refresh.clone())).await;

    HttpResponse::Ok().json(Tokens { access, refresh })
}

#[post("/logout")]
pub async fn logout(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let db = &app_data.conn;

    if let None = User::find_by_id(creds.uid).one(db).await.unwrap() {
        return HttpResponse::NotFound();
    }

    update_user_refresh(creds.uid, db, None).await;

    HttpResponse::Ok()
}

#[post("/refresh")]
pub async fn refresh_token(req: HttpRequest, app_data: web::Data<AppState>) -> impl Responder {
    let db = &app_data.conn;
    let jwt = &app_data.jwt;

    let token = get_token_from_req(req);

    let claims: JwtCred = match token {
        Ok(ref token) => match jwt.get_claims(&token.as_str(), scopes::REFRESH) {
            Some(claims) => claims,
            None => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    message: String::from("invalid token format"),
                })
            }
        },
        Err(err) => match err {
            AuthError::Unauthorized => {
                return HttpResponse::Unauthorized().json(ErrorResponse {
                    message: String::from("authorization header is missing"),
                })
            }
            AuthError::InvalidToken => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    message: String::from("invalid token format"),
                })
            }
        },
    };

    let user = User::find_by_id(claims.uid).one(db).await.unwrap();
    let user = match user {
        Some(user) => user,
        None => {
            return HttpResponse::NotFound().json(ErrorResponse {
                message: String::from("user not found"),
            })
        }
    };

    let mut hasher = Sha256::new();
    hasher.input_str(token.unwrap().as_str());
    let hashed_refresh = hasher.result_str();

    if let None = user.refresh_token_hash {
        return HttpResponse::Unauthorized().json(ErrorResponse {
            message: String::from("user not unauthorized"),
        });
    }

    if user.refresh_token_hash.unwrap().as_str() != hashed_refresh.as_str() {
        return HttpResponse::Unauthorized().json(ErrorResponse {
            message: String::from("token are invalid"),
        });
    }

    let access = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(20),
        )
        .unwrap();
    let refresh = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(14),
        )
        .unwrap();

    update_user_refresh(user.id, db, Some(refresh.clone())).await;

    HttpResponse::Ok().json(Tokens { access, refresh })
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
