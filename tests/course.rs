use actix_web::{
    http::{header, StatusCode},
    test, App,
};
use fake::{
    faker::{
        internet::raw::{FreeEmail, Password, Username},
        name::raw::Title,
    },
    locales::EN,
    Fake,
};
use lazy_static::lazy_static;
use rc_api::{
    get_app_data, get_db_conn, get_key, main_config,
    models::{
        auth::{SignUpData, Tokens},
        course::{Course, CreateCourse},
        language::Language,
    },
    utils::jwt::{scopes, JwtUtil},
};
use tokio::task;

fn create_course_req(course: CreateCourse, token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/course/")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(course)
}

fn get_courses_req(token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri("/api/course/")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

fn get_course_req(course_id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri(format!("/api/course/{course_id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

fn get_subs_course_req(token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri("/api/course/subscribe")
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

/// Send reqeust to **/api/course/subs/{id}**
fn subscribe_to_course_req(course_id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri(format!("/api/course/subscribe/{course_id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

/// Send reqeust to **/api/course/owner/{id}**
fn user_is_owner_req(course_id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri(format!("/api/course/owner/{course_id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {}", token)))
}

fn signup_req(data: SignUpData) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/auth/signup")
        .set_json(data)
}

// lazy_static! {
//     static ref IVAN: (i32, String) = task::block_in_place(|| tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(async { init_jwt().await }));
//     static ref ANTON: (i32, String) = task::block_in_place(|| tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(async { init_jwt().await }));
// }

async fn init_user() -> (i32, String) {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let signup_req = signup_req(SignUpData {
        email: FreeEmail(EN).fake(),
        username: Username(EN).fake(),
        password: Password(EN, 6..12).fake(),
    })
    .to_request();

    let tokens: Tokens = test::call_and_read_body_json(&app, signup_req).await;

    let jwt = JwtUtil { key: get_key() };

    let id = jwt.get_claims(&tokens.access, scopes::ACCESS).unwrap().uid;

    (id, tokens.access)
}

#[actix_web::test]
async fn test_create_course_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let title: String = Title(EN).fake();
    let lang = Language::En;

    let user = init_user().await;

    let create_course_res = create_course_req(
        CreateCourse {
            title,
            language: lang,
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert_eq!(create_course_res.status(), StatusCode::CREATED);

    let _: i32 = test::read_body_json(create_course_res).await;
}

#[actix_web::test]
async fn test_create_course_badrequest() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let create_course_res = create_course_req(
        CreateCourse {
            title: "".to_string(),
            language: Language::De,
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert_eq!(create_course_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_create_course_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let create_course_res = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        "wron data",
    )
    .send_request(&app)
    .await;

    assert_eq!(create_course_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_course_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    // init vars
    let lang = Language::En;
    let title: String = Title(EN).fake();

    let user = init_user().await;

    // create the course
    let create_course_res = create_course_req(
        CreateCourse {
            title: title.clone(),
            language: lang.clone(),
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    // course is created
    assert!(create_course_res.status().is_success());

    // get course id
    let id: i32 = test::read_body_json(create_course_res).await;

    // send get course response
    let get_course_res = get_course_req(id, user.1.as_str()).send_request(&app).await;

    // assertion
    assert_eq!(get_course_res.status(), StatusCode::OK);

    let course: Course = test::read_body_json(get_course_res).await;

    assert_eq!(course.title, title);
    assert_eq!(course.language, lang);
}

#[actix_web::test]
async fn test_get_course_notfound() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let id = i32::MAX - 2;

    let get_course_res = get_course_req(id, user.1.as_str()).send_request(&app).await;

    assert_eq!(get_course_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_get_course_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let get_course_res = get_course_req(1, "wron data").send_request(&app).await;

    assert_eq!(get_course_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_all_courses() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let get_courses_res = get_courses_req(user.1.as_str()).send_request(&app).await;

    assert_eq!(get_courses_res.status(), StatusCode::OK);

    let courses: Vec<Course> = test::read_body_json(get_courses_res).await;

    assert_ne!(courses.len(), 0);
}

#[actix_web::test]
async fn test_get_all_courses_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let get_courses_res = get_courses_req("wrong data").send_request(&app).await;

    assert_eq!(get_courses_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_user_is_owner_yes() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let new_course = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert!(new_course.status().is_success());

    let new_course_id = test::read_body_json(new_course).await;

    let user_is_owner_res = user_is_owner_req(new_course_id, user.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(user_is_owner_res.status(), StatusCode::OK);

    let is_owner: bool = test::read_body_json(user_is_owner_res).await;

    assert!(is_owner);
}

#[actix_web::test]
async fn test_user_is_owner_no() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user1 = init_user().await;
    let user2 = init_user().await;

    let new_course = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        user1.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert!(new_course.status().is_success());

    let new_course_id = test::read_body_json(new_course).await;

    let user_is_owner_res = user_is_owner_req(new_course_id, user2.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(user_is_owner_res.status(), StatusCode::OK);

    let is_owner: bool = test::read_body_json(user_is_owner_res).await;

    assert!(!is_owner);
}

#[actix_web::test]
async fn test_user_is_owner_notfound() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let course_id = i32::MAX;

    let user_is_owner_res = user_is_owner_req(course_id, user.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(user_is_owner_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_user_is_owner_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let new_course = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert!(new_course.status().is_success());

    let new_course_id = test::read_body_json(new_course).await;

    let user_is_owner_res = user_is_owner_req(new_course_id, "wrong data")
        .send_request(&app)
        .await;

    assert_eq!(user_is_owner_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_subscribe() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user1 = init_user().await;
    let user2 = init_user().await;

    let new_course = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        user1.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert!(new_course.status().is_success());

    let new_course_id = test::read_body_json(new_course).await;

    let subscribe_res = subscribe_to_course_req(new_course_id, user2.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(subscribe_res.status(), StatusCode::OK);

    let pool = &get_db_conn().await;

    let select = sqlx::query!(
        "SELECT * FROM course_user WHERE course_id=$1 AND user_id=$2;",
        new_course_id,
        user2.0
    )
    .fetch_optional(pool)
    .await
    .unwrap();

    assert!(select.is_some());
}

#[actix_web::test]
async fn test_subscribe_notfound() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let course_id = i32::MAX;

    let subscribe_res = subscribe_to_course_req(course_id, user.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(subscribe_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_subscribe_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let new_course = create_course_req(
        CreateCourse {
            title: Title(EN).fake(),
            language: Language::En,
        },
        user.1.as_str(),
    )
    .send_request(&app)
    .await;

    assert!(new_course.status().is_success());

    let new_course_id = test::read_body_json(new_course).await;

    let subscribe_res = subscribe_to_course_req(new_course_id, "wrong data")
        .send_request(&app)
        .await;

    assert_eq!(subscribe_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_subscribed() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let subs_res = get_subs_course_req(user.1.as_str())
        .send_request(&app)
        .await;

    assert_eq!(subs_res.status(), StatusCode::OK);

    let _: Vec<i32> = test::read_body_json(subs_res).await;
}

// new user has empty subs list
// #[actix_web::test]
// async fn test_get_subscribed_empty() {
//     let app = test::init_service(
//         App::new()
//             .app_data(get_app_data().await)
//             .configure(main_config),
//     )
//     .await;
// }

#[actix_web::test]
async fn test_get_subscribed_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let subs_res = get_subs_course_req("wrong data").send_request(&app).await;

    assert_eq!(subs_res.status(), StatusCode::UNAUTHORIZED);
}
