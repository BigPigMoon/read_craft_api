use actix_web::http::header;
use actix_web::test::{self, TestRequest};
use actix_web::{http::StatusCode, App};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use dotenvy::dotenv;
use fake::{
    faker::internet::raw::{FreeEmail, Password, Username},
    locales::EN,
    Fake,
};

use rc_api::services::user::find_user_by_email;
use rc_api::{
    get_app_data, get_db_conn, get_key, models::auth::*, repos::auth::*, utils::jwt::JwtUtil,
};

fn signin_req(data: SignInData) -> TestRequest {
    test::TestRequest::post().uri("/signin").set_json(data)
}

fn logout_req(token: &str) -> TestRequest {
    test::TestRequest::post()
        .uri("/logout")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

fn refresh_req(token: &str) -> TestRequest {
    test::TestRequest::post()
        .uri("/refresh")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

fn signup_req(data: SignUpData) -> TestRequest {
    test::TestRequest::post().uri("/signup").set_json(data)
}

#[actix_web::test]
async fn test_signup() {
    let app = test::init_service(App::new().app_data(get_app_data().await).service(signup)).await;

    let email: String = FreeEmail(EN).fake();

    let signup_res = signup_req(SignUpData {
        email: email.clone(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    })
    .send_request(&app)
    .await;

    assert_eq!(signup_res.status(), StatusCode::CREATED);

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();

    assert!(user.refresh_token_hash.is_some());
}

#[actix_web::test]
async fn test_signup_tokens() {
    let app = test::init_service(App::new().app_data(get_app_data().await).service(signup)).await;
    let email: String = FreeEmail(EN).fake();

    let signup_res = signup_req(SignUpData {
        email: email.clone(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    })
    .send_request(&app)
    .await;

    assert!(signup_res.status().is_success());

    let tokens: Tokens = test::read_body_json(signup_res).await;

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();

    // check refresh token are hashed
    let mut hasher = Sha256::new();
    hasher.input_str(tokens.refresh.as_str());
    let refresh_hash = hasher.result_str();

    assert_eq!(user.refresh_token_hash.unwrap(), refresh_hash);

    // check JWT claims
    let jwt = JwtUtil { key: get_key() };
    let claims = jwt.decode_token(&tokens.access).unwrap();

    assert_eq!(user.id, claims.uid);
    assert_eq!(user.email, claims.email);
}

#[actix_web::test]
async fn test_signup_duplicated() {
    let app = test::init_service(App::new().app_data(get_app_data().await).service(signup)).await;

    let data = SignUpData {
        email: FreeEmail(EN).fake(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    };

    let signup_res = signup_req(data.clone()).send_request(&app).await;

    assert_eq!(signup_res.status(), StatusCode::CREATED);

    let signup_res = signup_req(data).send_request(&app).await;
    assert_eq!(signup_res.status(), StatusCode::CONFLICT);
}

#[actix_web::test]
async fn test_signup_fields_empty() {
    let app = test::init_service(App::new().app_data(get_app_data().await).service(signup)).await;

    let data = SignUpData {
        email: String::from(""),
        username: String::from(""),
        password: String::from(""),
    };

    let signup_res = signup_req(data).send_request(&app).await;
    assert_eq!(signup_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_signup_invalid_email() {
    dotenv().ok();

    let app = test::init_service(App::new().app_data(get_app_data().await).service(signup)).await;

    let data = SignUpData {
        email: String::from("test"),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    };

    let signup_res = signup_req(data).send_request(&app).await;
    assert_eq!(signup_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_signin() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(signup)
            .service(signin),
    )
    .await;

    let email: String = FreeEmail(EN).fake();
    let username: String = Username(EN).fake();
    let password: String = Password(EN, 6..12).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username,
        password: password.clone(),
    };

    let signin_data = SignInData {
        email: email.clone(),
        password,
    };

    let signup_res = signup_req(signup_data).send_request(&app).await;
    assert!(signup_res.status().is_success());

    let signin_res = signin_req(signin_data).send_request(&app).await;
    assert_eq!(signin_res.status(), StatusCode::OK);

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();

    assert!(user.refresh_token_hash.is_some());
}

#[actix_web::test]
async fn test_signin_signup_tokens_not_eq() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(signup)
            .service(signin),
    )
    .await;

    let email: String = FreeEmail(EN).fake();
    let username: String = Username(EN).fake();
    let password: String = Password(EN, 6..12).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username,
        password: password.clone(),
    };

    let signin_data = SignInData { email, password };

    let signup_res = signup_req(signup_data).send_request(&app).await;
    assert!(signup_res.status().is_success());

    let signin_res = signin_req(signin_data).send_request(&app).await;
    assert!(signin_res.status().is_success());

    let signup_res_data: Tokens = test::read_body_json(signup_res).await;
    let signin_res_data: Tokens = test::read_body_json(signin_res).await;

    assert_ne!(signup_res_data.access, signin_res_data.access);
    assert_ne!(signup_res_data.refresh, signin_res_data.refresh);
}

#[actix_web::test]
async fn test_logout() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(logout)
            .service(signup),
    )
    .await;

    let email: String = FreeEmail(EN).fake();
    let username: String = Username(EN).fake();
    let password: String = Password(EN, 6..12).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username,
        password: password.clone(),
    };

    let signup_res = signup_req(signup_data).to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_res).await;

    let logout_res = logout_req(&tokens.access).send_request(&app).await;
    assert_eq!(logout_res.status(), StatusCode::OK);

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();

    assert!(user.refresh_token_hash.is_none());
}

#[actix_web::test]
async fn test_logout_and_signin() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(logout)
            .service(signin)
            .service(signup),
    )
    .await;

    let email: String = FreeEmail(EN).fake();
    let username: String = Username(EN).fake();
    let password: String = Password(EN, 6..12).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username,
        password: password.clone(),
    };

    let signin_data = SignInData {
        email: email.clone(),
        password: password.clone(),
    };

    let signup_res = signup_req(signup_data).to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_res).await;

    let logout_res = logout_req(&tokens.access).send_request(&app).await;
    assert_eq!(logout_res.status(), StatusCode::OK);

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();
    assert!(user.refresh_token_hash.is_none());

    let signin_res = signin_req(signin_data).send_request(&app).await;

    assert_eq!(signin_res.status(), StatusCode::OK);

    let user = find_user_by_email(&email, db).await.unwrap();
    assert!(user.refresh_token_hash.is_some());
}

#[actix_web::test]
async fn test_logout_with_refresh() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(logout)
            .service(signup),
    )
    .await;

    let email: String = FreeEmail(EN).fake();
    let username: String = Username(EN).fake();
    let password: String = Password(EN, 6..12).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username,
        password: password.clone(),
    };

    let signup_req = signup_req(signup_data).to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_req).await;

    let logout_res = logout_req(&tokens.refresh).send_request(&app).await;
    assert_eq!(logout_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_logout_not_authorized() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(logout)
            .service(signup),
    )
    .await;

    let signup_data = SignUpData {
        email: FreeEmail(EN).fake(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    };

    let signup_res = signup_req(signup_data).send_request(&app).await;
    assert!(signup_res.status().is_success());

    let logout_res = test::TestRequest::post()
        .uri("/logout")
        .send_request(&app)
        .await;
    assert_eq!(logout_res.status(), StatusCode::UNAUTHORIZED)
}

#[actix_web::test]
async fn test_refresh_token() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(signup)
            .service(refresh_token),
    )
    .await;

    let email: String = FreeEmail(EN).fake();

    let signup_data = SignUpData {
        email: email.clone(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    };

    let signup_req = signup_req(signup_data).to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_req).await;

    std::thread::sleep(std::time::Duration::from_secs(1));

    let refresh_res = refresh_req(&tokens.refresh).send_request(&app).await;
    assert_eq!(refresh_res.status(), StatusCode::OK);

    let new_tokens: Tokens = test::read_body_json(refresh_res).await;

    assert_ne!(tokens.refresh, new_tokens.refresh);

    let db = &get_db_conn().await;

    let user = find_user_by_email(&email, db).await.unwrap();

    let mut hasher = Sha256::new();
    hasher.input_str(tokens.refresh.as_str());
    let old_refresh_hash = hasher.result_str();

    let mut hasher = Sha256::new();
    hasher.input_str(new_tokens.refresh.as_str());
    let new_refresh_hash = hasher.result_str();

    assert!(user.refresh_token_hash.is_some());

    let user_refresh_token_hash = user.refresh_token_hash.unwrap();
    assert_ne!(user_refresh_token_hash, old_refresh_hash);
    assert_eq!(user_refresh_token_hash, new_refresh_hash);
}

#[actix_web::test]
async fn test_refresh_token_duplicated() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(signup)
            .service(refresh_token),
    )
    .await;

    let signup_data = SignUpData {
        email: FreeEmail(EN).fake(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    };

    let signup_req = signup_req(signup_data).to_request();
    let tokens: Tokens = test::call_and_read_body_json(&app, signup_req).await;

    std::thread::sleep(std::time::Duration::from_secs(1));

    let refresh_res = refresh_req(&tokens.refresh).send_request(&app).await;
    assert_eq!(refresh_res.status(), StatusCode::OK);

    std::thread::sleep(std::time::Duration::from_secs(1));

    let refresh_res = refresh_req(&tokens.refresh).send_request(&app).await;
    assert_eq!(refresh_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_refresh_tokens_with_access() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .service(signup)
            .service(refresh_token),
    )
    .await;

    let signup_req = signup_req(SignUpData {
        email: FreeEmail(EN).fake(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    })
    .to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_req).await;

    let refresh_res = refresh_req(&tokens.access).send_request(&app).await;
    assert_eq!(refresh_res.status(), StatusCode::BAD_REQUEST);
}
