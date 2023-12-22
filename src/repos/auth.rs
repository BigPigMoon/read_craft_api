use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use crypto::{digest::Digest, sha2::Sha256};
use jwt_simple::prelude::Duration;
use validator::Validate;

use crate::{
    extractors::jwt_cred::{get_token_from_req, AuthError, JwtCred},
    models::{
        auth::{SignInData, SignUpData, Tokens},
        common::ErrorResponse,
        user::CreateUser,
    },
    services::user::{create_user, find_user_by_email, find_user_by_id},
    utils::jwt::scopes,
    AppState,
};

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(signup));
}

#[post("/signup")]
pub async fn signup(data: web::Json<SignUpData>, app_data: web::Data<AppState>) -> impl Responder {
    let jwt_controller = &app_data.jwt;

    if data.validate().is_err() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("invalid data"),
        });
    }

    let hashed_password = bcrypt::hash(data.password.clone(), bcrypt::DEFAULT_COST).unwrap();

    let user = CreateUser {
        email: data.email.clone(),
        username: data.username.clone(),
        password_hash: hashed_password,
        refresh_token_hash: None,
    };

    let new_id = match create_user(&user, &app_data.pool).await {
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
                uid: new_id,
                email: data.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(20),
        )
        .unwrap();

    let refresh = jwt_controller
        .encode_token(
            JwtCred {
                uid: new_id,
                email: data.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(14),
        )
        .unwrap();

    let user = find_user_by_id(new_id, &app_data.pool).await.unwrap();
    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .expect("failed update user refresh hash");

    HttpResponse::Created().json(Tokens { access, refresh })
}

#[post("/signin")]
pub async fn signin(data: web::Json<SignInData>, app_data: web::Data<AppState>) -> impl Responder {
    let jwt = &app_data.jwt;

    let user = find_user_by_email(&data.email, &app_data.pool).await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
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

    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .expect("failed update user refresh hash");

    HttpResponse::Ok().json(Tokens { access, refresh })
}

#[post("/logout")]
pub async fn logout(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let user = match find_user_by_id(creds.uid, &app_data.pool).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound(),
    };

    user.update_refresh_token(None, &app_data.pool)
        .await
        .unwrap();

    HttpResponse::Ok()
}

#[post("/refresh")]
pub async fn refresh_token(req: HttpRequest, app_data: web::Data<AppState>) -> impl Responder {
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

    let user = match find_user_by_id(claims.uid, &app_data.pool).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                message: String::from("user not found"),
            })
        }
    };

    let mut hasher = Sha256::new();
    hasher.input_str(token.unwrap().as_str());
    let hashed_refresh = hasher.result_str();

    if user.refresh_token_hash.is_none() {
        return HttpResponse::Unauthorized().json(ErrorResponse {
            message: String::from("user not unauthorized"),
        });
    }

    if user.refresh_token_hash.clone().unwrap().as_str() != hashed_refresh.as_str() {
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

    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(Tokens { access, refresh })
}
