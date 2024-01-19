use std::env;

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::{StreamExt, TryStreamExt};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::{extractors::jwt_cred::JwtCred, models::common::ErrorResponse};

pub fn image_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/image")
            .service(get_image)
            .service(upload_image),
    );
}

/// Upload image of lesson to server
///
/// Path:
/// GET: /api/lesson/image/upload
#[post("/upload")]
async fn upload_image(_: JwtCred, mut payload: Multipart) -> impl Responder {
    let op = "upload_image";
    let mut filename = String::new();

    log::info!("{}: attempting to upload image", op);

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(err) => {
                log::error!("{}: lost file in request, error: {}", op, err);

                return HttpResponse::BadRequest().json(ErrorResponse {
                    message: "Failed to process the request".to_string(),
                });
            }
        };

        filename = Uuid::new_v4().to_string();

        let lessons_dir = env::var("IMAGE_DIR").unwrap_or("./uploads/images".to_string());

        let filepath = format!("{}/{}", lessons_dir, &filename);

        let mut file = match File::create(&filepath).await {
            Ok(file) => file,
            Err(err) => {
                log::error!(
                    "{}: failed to save the image, filepath: {}, error: {}",
                    op,
                    filepath,
                    err
                );

                return HttpResponse::InternalServerError().finish();
            }
        };

        while let Some(chunk) = field.try_next().await.unwrap() {
            if let Err(err) = file.write_all(&chunk).await {
                log::error!("{}: failed to write the chunk, error: {}", op, err);

                return HttpResponse::InternalServerError().finish();
            };
        }
    }

    log::info!(
        "{}: image is successfuly uploaded, filename: {}",
        op,
        filename
    );

    HttpResponse::Ok().json(filename)
}

/// Get the image from filename
///
/// Path:
/// GET: /api/lesson/image/{filename}
#[get("/{filename}")]
async fn get_image(path: web::Path<String>) -> impl Responder {
    let op = "get_image";

    let filename = path.into_inner();

    log::info!("{}: attempting to get image: {}", op, filename);

    let lessons_dir = env::var("IMAGE_DIR").unwrap_or("./uploads/images".to_string());
    let filepath = format!("{}/{}", lessons_dir, &filename);

    match File::open(&filepath).await {
        Ok(mut file) => {
            let mut buffer = Vec::new();

            if let Err(err) = file.read_to_end(&mut buffer).await {
                log::error!(
                    "{}: error reading image: {}, error: {:?}",
                    op,
                    filepath,
                    err
                );

                return HttpResponse::InternalServerError().finish();
            }

            log::info!("{}: image succesfuly returned", op);

            HttpResponse::Ok().content_type("image/jpeg").body(buffer)
        }
        Err(err) => {
            log::error!(
                "{}: can not open the image from path: {}, error: {}",
                op,
                filepath,
                err
            );

            HttpResponse::NotFound().finish()
        }
    }
}
