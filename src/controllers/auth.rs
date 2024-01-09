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
    cfg.service(
        web::scope("/auth")
            .service(signup)
            .service(signin)
            .service(logout)
            .service(refresh_token),
    );
}

const ACCESS_DURATION_MIN: u64 = 20;
const REFRESH_DURATION_DAY: u64 = 14;

/// Sign up request
///
/// Register user in system return pair of JWT
///
/// Path:
/// **/api/auth/signup**
#[post("/signup")]
pub async fn signup(data: web::Json<SignUpData>, app_data: web::Data<AppState>) -> impl Responder {
    let op = "signup";
    log::info!("{}: attempting to sign up user", op);

    let jwt_controller = &app_data.jwt;

    if data.validate().is_err() {
        log::error!("{}: data is not validated", op);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: String::from("invalid data"),
        });
    }

    let hashed_password = bcrypt::hash(data.password.clone(), bcrypt::DEFAULT_COST).unwrap();

    log::info!("{}: password was successfuly hashed", op);

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

    log::info!(
        "{}: user was successfuly created in database, new user id: {}",
        op,
        new_id
    );

    let access = jwt_controller
        .encode_token(
            JwtCred {
                uid: new_id,
                email: data.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(ACCESS_DURATION_MIN),
        )
        .unwrap();

    let refresh = jwt_controller
        .encode_token(
            JwtCred {
                uid: new_id,
                email: data.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(REFRESH_DURATION_DAY),
        )
        .unwrap();

    log::info!("{}: tokens was successfuly generated", op);

    let user = find_user_by_id(new_id, &app_data.pool).await.unwrap();
    // NOTE: mb it need to wrap in match
    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .expect("failed update user refresh hash");

    log::info!(
        "{}: refresh token was successfuly update and tokens are secccessfully sended",
        op
    );

    HttpResponse::Created().json(Tokens { access, refresh })
}

/// Sign in request
///
/// Login user in system return pair of JWT
///
/// Path:
/// **/api/auth/signin**
#[post("/signin")]
pub async fn signin(data: web::Json<SignInData>, app_data: web::Data<AppState>) -> impl Responder {
    let op = "sigin";
    log::info!("{}: attempting to login user", op);

    let jwt = &app_data.jwt;

    let user = find_user_by_email(&data.email, &app_data.pool).await;

    let user = match user {
        Ok(user) => user,
        Err(err) => {
            log::error!(
                "{}: user by email: {}, was not found, error: {}",
                op,
                data.email,
                err
            );

            return HttpResponse::NotFound().json(ErrorResponse {
                message: String::from("user not found"),
            });
        }
    };

    log::info!(
        "{}: user by email: {} was successfuly founded, user: {:?}",
        op,
        data.email,
        user
    );

    let valid = bcrypt::verify(data.password.clone(), &user.password_hash).unwrap();
    if !valid {
        log::error!("{}: user enter invalid password", op);

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
            Duration::from_mins(ACCESS_DURATION_MIN),
        )
        .unwrap();
    let refresh = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(REFRESH_DURATION_DAY),
        )
        .unwrap();

    log::info!("{}: tokens was successfuly generated", op);

    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .expect("failed update user refresh hash");

    log::info!(
        "{}: refresh token was successfuly update and tokens are secccessfully sended",
        op
    );

    HttpResponse::Ok().json(Tokens { access, refresh })
}

/// Logout request
///
/// Clear refresh token from database
///
/// Path:
/// **/api/auth/logout**
#[post("/logout")]
pub async fn logout(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "logout";
    log::info!("{}: attempting to logout user", op);

    let user = match find_user_by_id(creds.uid, &app_data.pool).await {
        Ok(user) => user,
        Err(err) => {
            log::error!(
                "{}: cannot found the user by id: {}, error: {}",
                op,
                creds.uid,
                err
            );

            return HttpResponse::NotFound();
        }
    };

    log::info!("{}: user was successfuly founded, user: {:?}", op, user);

    user.update_refresh_token(None, &app_data.pool)
        .await
        .unwrap();

    log::info!("{}: user refresh token was successfuly updated to None", op);

    HttpResponse::Ok()
}

/// Refresh token request
///
/// Get refresh token in header and generate new pairs of JWT
///
/// Path:
/// **/api/auth/refresh**
#[post("/refresh")]
pub async fn refresh_token(req: HttpRequest, app_data: web::Data<AppState>) -> impl Responder {
    let op = "refresh_token";
    log::info!("{}: attempting to refresh token", op);

    let jwt = &app_data.jwt;

    let token = get_token_from_req(req);

    let claims: JwtCred = match token {
        Ok(ref token) => match jwt.get_claims(&token.as_str(), scopes::REFRESH) {
            Some(claims) => claims,
            None => {
                log::error!(
                    "{}: header are not contain claims, or invalid token format, token: {}",
                    op,
                    token
                );

                return HttpResponse::BadRequest().json(ErrorResponse {
                    message: String::from("invalid token format"),
                });
            }
        },
        Err(err) => match err {
            AuthError::Unauthorized => {
                log::error!("{}: header are not contain claims", op);

                return HttpResponse::Unauthorized().json(ErrorResponse {
                    message: String::from("authorization header is missing"),
                });
            }
            AuthError::InvalidToken => {
                log::error!("{}: invalid token format", op);

                return HttpResponse::BadRequest().json(ErrorResponse {
                    message: String::from("invalid token format"),
                });
            }
        },
    };

    log::info!(
        "{}: claims was successfuly claimed, claims: {:?}",
        op,
        claims
    );

    let user = match find_user_by_id(claims.uid, &app_data.pool).await {
        Ok(user) => user,
        Err(err) => {
            log::error!(
                "{}: cannot find user by id: {}, error: {}",
                op,
                claims.uid,
                err
            );

            return HttpResponse::NotFound().json(ErrorResponse {
                message: String::from("user not found"),
            });
        }
    };

    log::info!("{}: user founded in database, user: {:?}", op, user);

    if user.refresh_token_hash.is_none() {
        log::error!("{}: user are logouted, user: {:?}", op, user);

        return HttpResponse::Unauthorized().json(ErrorResponse {
            message: String::from("user not unauthorized"),
        });
    }

    let mut hasher = Sha256::new();
    hasher.input_str(token.unwrap().as_str());
    let hashed_refresh = hasher.result_str();

    log::info!("{}: token from header are hashed", op);

    if user.refresh_token_hash.clone().unwrap().as_str() != hashed_refresh.as_str() {
        log::error!("{}: tokens in database and in header are not equal", op);

        return HttpResponse::Unauthorized().json(ErrorResponse {
            message: String::from("token are invalid"),
        });
    }

    log::info!("{}: tokens are equals", op);

    let access = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::ACCESS.to_string(),
            },
            Duration::from_mins(ACCESS_DURATION_MIN),
        )
        .unwrap();
    let refresh = jwt
        .encode_token(
            JwtCred {
                uid: user.id,
                email: user.email.clone(),
                scope: scopes::REFRESH.to_string(),
            },
            Duration::from_days(REFRESH_DURATION_DAY),
        )
        .unwrap();

    log::info!("{}: tokens was successfuly generated", op);

    user.update_refresh_token(Some(&refresh), &app_data.pool)
        .await
        .unwrap();

    log::info!(
        "{}: refresh token was successfuly update and tokens are secccessfully sended",
        op
    );

    HttpResponse::Ok().json(Tokens { access, refresh })
}
