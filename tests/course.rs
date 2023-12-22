use actix_web::{test, App};
use rc_api::{get_app_data, main_config};

fn create_course_req() -> test::TestRequest {
    test::TestRequest::post().uri("/api/course/")
}

fn get_courses_req() -> test::TestRequest {
    test::TestRequest::get().uri("/api/course/")
}

fn get_course_req(id: i32, token: &str) -> test::TestRequest {
    test::TestRequest::get().uri(format!("/api/course/{id}").as_str())
}

#[actix_web::test]
async fn test_create_course_success() {
    let app = test::init_service(
        App::new()
            .app_data(get_app_data().await)
            .configure(main_config),
    )
    .await;
}
