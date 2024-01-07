use actix_http::body::MessageBody;
use actix_web::{
    http::{header, StatusCode},
    test, App,
};
use fake::{
    faker::{
        internet::raw::{FreeEmail, Password, Username},
        lorem::raw::Words,
    },
    locales::EN,
    Fake,
};

use rc_api::{
    get_app_data, main_config,
    models::{
        auth::{SignUpData, Tokens},
        course::CreateCourse,
        language::Language,
        lesson::{CreateLesson, Lesson, UpdateLesson},
    },
};

/// Send request to **/api/lesson/create**
fn create_lesson_req(lesson: CreateLesson, token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/lesson/create")
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_json(lesson)
}

/// Send request to **/api/lesson/all**
fn get_lessons_req(token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri("/api/lesson/all")
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
}

/// Send request to **/api/lesson/all?course={course_id}**
fn get_lessons_in_course_req(course_id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri(format!("/api/lesson/all?course={course_id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
}

/// Send request to **/api/lesson/get**
fn get_lesson_req(id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri(format!("/api/lesson/get/{id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
}

/// Send request to **/api/lesson/update**
fn update_lesson_req(new_lesson: UpdateLesson, token: &str) -> test::TestRequest {
    test::TestRequest::put()
        .uri("/api/lesson/update")
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_json(new_lesson)
}

/// Send request to **/api/lesson/delete/{id}**
fn delete_lesson_req(id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::delete()
        .uri(format!("/api/lesson/delete/{id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
}

/// Send request to **/api/lesson/upload/{id}**
fn upload_lesson_req(id: i32, text: String, token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri(format!("/api/lesson/upload/{id}").as_str())
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_payload(text)
}

/// send request to **/api/auth/signup**
fn signup_req(data: SignUpData) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/auth/signup")
        .set_json(data)
}

/// send request to **/api/course/create**
fn create_course_req(course: CreateCourse, token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/course/create")
        .append_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_json(course)
}

async fn init_user() -> String {
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

    tokens.access
}

async fn init_course(user: &str) -> i32 {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let title: Vec<String> = Words(EN, 5..12).fake();
    let title: String = title.join(" ");
    let lang = Language::En;

    let create_course_res = create_course_req(
        CreateCourse {
            title,
            language: lang,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(create_course_res.status(), StatusCode::CREATED);

    test::read_body_json(create_course_res).await
}

async fn init_lesson(course_id: i32, user: &str) -> i32 {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let title: Vec<String> = Words(EN, 5..12).fake();
    let title = title.join(" ");

    let subject: Vec<String> = Words(EN, 20..50).fake();
    let subject = Some(subject.join(" "));

    let create_lesson_res = create_lesson_req(
        CreateLesson {
            title,
            subject,
            cover_path: None,
            course_id,
        },
        user,
    )
    .send_request(&app)
    .await;

    assert_eq!(create_lesson_res.status(), StatusCode::CREATED);

    test::read_body_json(create_lesson_res).await
}

#[actix_web::test]
async fn test_create_lesson_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    let title: Vec<String> = Words(EN, 5..12).fake();
    let title = title.join(" ");

    let subject: Vec<String> = Words(EN, 20..50).fake();
    let subject = Some(subject.join(" "));

    let create_lesson_res = create_lesson_req(
        CreateLesson {
            title,
            subject,
            cover_path: None,
            course_id,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(create_lesson_res.status(), StatusCode::CREATED);

    let _: i32 = test::read_body_json(create_lesson_res).await;
}

#[actix_web::test]
async fn test_create_lesson_fordibben() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    let title: Vec<String> = Words(EN, 5..12).fake();
    let title = title.join(" ");

    let subject: Vec<String> = Words(EN, 20..50).fake();
    let subject = Some(subject.join(" "));

    let user = init_user().await;
    let create_lesson_res = create_lesson_req(
        CreateLesson {
            title,
            subject,
            cover_path: None,
            course_id,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(create_lesson_res.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_create_lesson_bad_request() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    let create_lesson_res = create_lesson_req(
        CreateLesson {
            title: "".to_string(),
            subject: None,
            cover_path: None,
            course_id,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(create_lesson_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_create_lesson_is_private() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    let title: Vec<String> = Words(EN, 5..12).fake();
    let title = title.join(" ");

    let subject: Vec<String> = Words(EN, 20..50).fake();
    let subject = Some(subject.join(" "));

    let create_lesson_res = create_lesson_req(
        CreateLesson {
            title,
            subject,
            cover_path: None,
            course_id,
        },
        "wrong data",
    )
    .send_request(&app)
    .await;

    assert_eq!(create_lesson_res.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_lessons_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let get_lessons_res = get_lessons_req(&user).send_request(&app).await;

    assert_eq!(get_lessons_res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_lessons_in_course_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    init_lesson(course_id, &user).await;

    let get_lessons_in_course_res = get_lessons_in_course_req(course_id, &user)
        .send_request(&app)
        .await;

    assert_eq!(get_lessons_in_course_res.status(), StatusCode::OK);

    let lessons: Vec<Lesson> = test::read_body_json(get_lessons_in_course_res).await;

    assert_eq!(lessons.len(), 1);
}

#[actix_web::test]
async fn test_get_lessons_in_course_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = i32::MAX;

    let get_lessons_in_course_res = get_lessons_in_course_req(course_id, &user)
        .send_request(&app)
        .await;

    assert_eq!(get_lessons_in_course_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_get_lesson_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let get_lesson_res = get_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(get_lesson_res.status(), StatusCode::OK);

    let _: Lesson = test::read_body_json(get_lesson_res).await;
}

#[actix_web::test]
async fn test_get_lesson_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let lesson_id = i32::MAX;

    let get_lesson_res = get_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(get_lesson_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_update_lesson_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let new_title: Vec<String> = Words(EN, 5..12).fake();
    let new_title = new_title.join(" ");

    let new_subject: Vec<String> = Words(EN, 20..50).fake();
    let new_subject = Some(new_subject.join(" "));

    let update_lesson_res = update_lesson_req(
        UpdateLesson {
            id: lesson_id,
            title: new_title.to_string(),
            cover_path: None,
            subject: new_subject.clone(),
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(update_lesson_res.status(), StatusCode::OK);

    let updated_lesson_id: i32 = test::read_body_json(update_lesson_res).await;

    assert_eq!(updated_lesson_id, lesson_id);

    let get_lesson_res = get_lesson_req(updated_lesson_id, &user)
        .send_request(&app)
        .await;

    assert!(get_lesson_res.status().is_success());

    let updated_lesson: Lesson = test::read_body_json(get_lesson_res).await;

    assert_eq!(updated_lesson.title, new_title);
    assert_eq!(updated_lesson.subject, new_subject);
}

#[actix_web::test]
async fn test_update_lesson_forbidden() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let user = init_user().await;

    let new_title: Vec<String> = Words(EN, 5..12).fake();
    let new_title = new_title.join(" ");

    let new_subject: Vec<String> = Words(EN, 20..50).fake();
    let new_subject = Some(new_subject.join(" "));

    let update_lesson_res = update_lesson_req(
        UpdateLesson {
            id: lesson_id,
            title: new_title.to_string(),
            cover_path: None,
            subject: new_subject.clone(),
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(update_lesson_res.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_update_lesson_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let lesson_id: i32 = i32::MAX;

    let new_title: Vec<String> = Words(EN, 5..12).fake();
    let new_title = new_title.join(" ");

    let new_subject: Vec<String> = Words(EN, 20..50).fake();
    let new_subject = Some(new_subject.join(" "));

    let update_lesson_res = update_lesson_req(
        UpdateLesson {
            id: lesson_id,
            title: new_title,
            cover_path: None,
            subject: new_subject,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(update_lesson_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_update_lesson_bad_request() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;

    let lesson_id = init_lesson(course_id, &user).await;

    let new_title = "".to_string();
    let new_subject = None;

    let update_lesson_res = update_lesson_req(
        UpdateLesson {
            id: lesson_id,
            title: new_title,
            cover_path: None,
            subject: new_subject,
        },
        &user,
    )
    .send_request(&app)
    .await;

    assert_eq!(update_lesson_res.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_delete_lesson_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;
    let delete_lesson_res = delete_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(delete_lesson_res.status(), StatusCode::OK);

    let get_lesson_res = get_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(get_lesson_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_delete_lesson_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let lesson_id: i32 = i32::MAX;

    let delete_lesson_res = delete_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(delete_lesson_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_delete_lesson_forbidden() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;

    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let user = init_user().await;
    let delete_lesson_res = delete_lesson_req(lesson_id, &user).send_request(&app).await;

    assert_eq!(delete_lesson_res.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_upload_lesson_text_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let lesson_text: Vec<String> = Words(EN, 80..100).fake();
    let lesson_text = lesson_text.join(" ");

    let upload_lesson_res = upload_lesson_req(lesson_id, lesson_text, &user)
        .send_request(&app)
        .await;

    assert_eq!(upload_lesson_res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_upload_lesson_text_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let lesson_id = i32::MAX;

    let lesson_text: Vec<String> = Words(EN, 80..100).fake();
    let lesson_text = lesson_text.join(" ");

    let upload_lesson_res = upload_lesson_req(lesson_id, lesson_text, &user)
        .send_request(&app)
        .await;

    assert_eq!(upload_lesson_res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_upload_lesson_text_forbidden() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;

    let user = init_user().await;
    let course_id = init_course(&user).await;
    let lesson_id = init_lesson(course_id, &user).await;

    let lesson_text: Vec<String> = Words(EN, 80..100).fake();
    let lesson_text = lesson_text.join(" ");

    let user = init_user().await;
    let upload_lesson_res = upload_lesson_req(lesson_id, lesson_text, &user)
        .send_request(&app)
        .await;

    assert_eq!(upload_lesson_res.status(), StatusCode::FORBIDDEN);
}
