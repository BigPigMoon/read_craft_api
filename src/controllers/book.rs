use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use epub::doc::EpubDoc;
use futures_util::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sqlx::Postgres;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    extractors::jwt_cred::JwtCred,
    models::{
        book::{Book, CreateBook, UpdateBook},
        common::ErrorResponse,
    },
    services::book::{
        all_user_book, create_book_db, delete_book_db, find_book_by_id, get_book_chunk,
        set_book_chunk, update_book_db, user_is_onwer_book,
    },
    AppState,
};

pub fn book_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/book")
            .service(create_book)
            .service(get_book)
            .service(get_books)
            .service(delete_book)
            .service(update_book)
            .service(get_chunk)
            .service(change_page)
            .service(download_book)
            .service(upload_book),
    );
}

/// Create the book in database
///
/// Path:
/// POST: **/api/book/create**
#[post("/create")]
async fn create_book(
    creds: JwtCred,
    book: web::Json<CreateBook>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "create_book";

    let user_id = creds.uid;

    log::info!("{}: attempting to craete book, book: {:?}", op, book);

    if book.validate().is_err() {
        log::error!("{}: data is not valid, data: {:?}", op, book);

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "invalid data".to_string(),
        });
    }

    let new_book_id = match create_book_db(&book, user_id, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!("{}: error: {}", op, err);

            return HttpResponse::BadRequest().json(ErrorResponse {
                message: "invalid data".to_string(),
            });
        }
    };

    log::info!(
        "{}: book is successfuly created, book id: {}",
        op,
        new_book_id
    );

    HttpResponse::Created().json(new_book_id)
}

/// Get book info by id
///
/// Path:
/// GET: **/api/book/get/{id}**
#[get("/get/{id}")]
async fn get_book(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_book";

    let book_id = path.into_inner();
    let user_id = creds.uid;

    log::info!("{}: attemting to get book by id: {}", op, book_id);

    let book = match check_available(book_id, user_id, &app_data.pool, op).await {
        Ok(book) => book,
        Err(err) => match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        },
    };

    log::info!("{}: book are successfuly returned, book: {:?}", op, book);

    HttpResponse::Ok().json(book)
}

/// Get all books of user
///
/// Path:
/// GET: **/api/book/all**
#[get("/all")]
async fn get_books(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "get_books";

    let user_id = creds.uid;

    match all_user_book(user_id, &app_data.pool).await {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(err) => {
            log::error!(
                "{}: failed to get all user books, user_id: {}, error: {}",
                op,
                user_id,
                err
            );

            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Delete the book by id
///
/// Path:
/// DELETE: **/api/book/delete/{id}**
#[delete("/delete/{id}")]
async fn delete_book(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "delete_book";

    let book_id = path.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attempting to delete the book id: {} by user: {}",
        op,
        book_id,
        user_id
    );

    if let Err(err) = check_available(book_id, user_id, &app_data.pool, op).await {
        match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        }
    };

    if let Err(err) = delete_book_db(book_id, &app_data.pool).await {
        log::error!(
            "{}: can not delete the book by id: {}, error: {}",
            op,
            book_id,
            err
        );

        return HttpResponse::InternalServerError().finish();
    }

    log::info!("{}: book are successfuly deleted", op);

    HttpResponse::Ok().finish()
}

/// Update the book from getting JSON
///
/// Path:
/// PUT: **/api/book/update**
#[put("/update")]
async fn update_book(
    creds: JwtCred,
    book: web::Json<UpdateBook>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "update_book";

    let user_id = creds.uid;
    let book_id = book.id;

    log::info!(
        "{}: attempting to update the book id: {} by user: {}",
        op,
        book.id,
        user_id
    );

    if let Err(err) = check_available(book_id, user_id, &app_data.pool, op).await {
        match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        }
    };

    if let Err(err) = update_book_db(&book, &app_data.pool).await {
        log::error!(
            "{}: can not update the book: {}, error: {}",
            op,
            book_id,
            err
        );

        return HttpResponse::InternalServerError().finish();
    }

    log::info!("{}: book: {} are successfully updated", op, book_id);

    HttpResponse::Ok().finish()
}

/// Get text chunk of book by id
///
/// Path
/// GET: **/api/book/chunk/{book_id}
#[get("/chunk/{book_id}")]
async fn get_chunk(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_chunk";

    let book_id = path.into_inner();
    let user_id = creds.uid;

    log::info!("{}: attempting to get chunk of book: {}", op, book_id);

    let book = match check_available(book_id, user_id, &app_data.pool, op).await {
        Ok(book) => book,
        Err(err) => match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        },
    };

    let chunk = match get_book_chunk(user_id, book_id, &app_data.pool).await {
        Ok(chunk) => chunk,
        Err(err) => {
            log::error!(
                "{}: can not get the chunk of book: {}, error: {}",
                op,
                book_id,
                err
            );

            return HttpResponse::InternalServerError().finish();
        }
    };

    // FIXME: directory must be a const!
    let filepath = format!("./uploads/books/{}", &book.filename);

    let mut book_text = EpubDoc::new(&filepath).unwrap();

    book_text.set_current_page(chunk as usize);
    let content = book_text.get_current_str().unwrap().0;

    log::info!("{}: chunk are successfuly returnted", op);

    HttpResponse::Ok().json((content, chunk))
}

#[derive(Debug, Deserialize)]
pub struct ChangePageOptions {
    page: i32,
}

/// Change page of book to number in options
///
/// Path:
/// POST: **/api/book/page/{book_id}?page={page}
#[post("/page/{book_id}")]
async fn change_page(
    creds: JwtCred,
    path: web::Path<i32>,
    options: web::Query<ChangePageOptions>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_courses";

    let book_id = path.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attemting to change page of book: {}, page: {}",
        op,
        book_id,
        options.page
    );

    let book = match check_available(book_id, user_id, &app_data.pool, op).await {
        Ok(book) => book,
        Err(err) => match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        },
    };

    // FIXME: directory must be a const!
    let filepath = format!("./uploads/books/{}", &book.filename);

    let num_pages = EpubDoc::new(&filepath).unwrap().get_num_pages() as i32;

    if options.page < 0 || options.page >= num_pages {
        let new_progress = 100;

        if let Err(err) = update_book_db(
            &UpdateBook {
                id: book.id,
                title: book.title,
                language: book.language,
                cover_path: book.cover_path,
                author: book.author,
                subject: book.subject,
                progress: new_progress,
            },
            &app_data.pool,
        )
        .await
        {
            log::error!(
                "{}: can not update the book progress: {}, error: {}",
                op,
                book_id,
                err
            );
        }
        return HttpResponse::BadRequest().finish();
    }

    if let Err(err) = set_book_chunk(user_id, book_id, options.page, &app_data.pool).await {
        log::error!(
            "{}: can not set page: {} for book: {}, error: {}",
            op,
            options.page,
            book_id,
            err
        );

        return HttpResponse::InternalServerError().finish();
    }

    let new_progress = options.page as f32 / num_pages as f32 * 100.0;

    if let Err(err) = update_book_db(
        &UpdateBook {
            id: book.id,
            title: book.title,
            language: book.language,
            cover_path: book.cover_path,
            author: book.author,
            subject: book.subject,
            progress: new_progress.floor() as i32,
        },
        &app_data.pool,
    )
    .await
    {
        log::error!(
            "{}: can not update the book progress: {}, error: {}",
            op,
            book_id,
            err
        );
    }

    return HttpResponse::Ok().finish();
}

/// Download the file by path in directory uploads/books/{filename}
///
/// Path:
/// GET: **/api/book/download/{book_id}**
#[get("/download/{book_id}")]
async fn download_book(
    creds: JwtCred,
    info: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "download_book";

    let book_id = info.into_inner();
    let user_id = creds.uid;

    log::info!(
        "{}: attempting to download book file by book id: {}",
        op,
        book_id
    );

    let book = match check_available(book_id, user_id, &app_data.pool, op).await {
        Ok(book) => book,
        Err(err) => match err {
            AvailableError::NotFound => {
                return HttpResponse::NotFound().json(ErrorResponse {
                    message: "book is not exist".to_string(),
                });
            }
            AvailableError::Forbidden => {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    message: "user is not owned of book".to_string(),
                });
            }
        },
    };

    let filepath = format!("uploads/books/{}", book.filename);

    // Открываем файл
    let mut file = match File::open(&filepath).await {
        Ok(file) => file,
        Err(err) => {
            log::error!(
                "{}: file by path: {} not found, error: {}",
                op,
                filepath,
                err
            );

            return HttpResponse::NotFound().json(ErrorResponse {
                message: "file not found".to_string(),
            });
        }
    };

    // Читаем содержимое файла в буфер
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.unwrap();

    // Устанавливаем заголовок Content-Disposition
    let content_disposition = format!(
        "attachment; filename=\"{}\"",
        format!("{}.epub", book.title)
    );

    log::info!("{}: book is successfuly downloaded", op);

    // Строим HTTP-ответ с буфером и заголовком Content-Disposition
    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header(("Content-Disposition", content_disposition))
        .body(buffer)
}

/// Upload the file for book
///
/// Path:
/// POST: **/api/book/upload**
#[post("/upload")]
async fn upload_book(_: JwtCred, mut payload: Multipart) -> impl Responder {
    let op = "upload_book";
    let mut filename = String::new();

    log::info!("{}: attempting to upload book", op);

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

        let content_type = field.content_disposition();
        let orig_filename = content_type.get_filename().unwrap_or("unknown.txt");

        // Проверяем, что файл имеет формат EPUB
        if !orig_filename.to_lowercase().ends_with(".epub") {
            log::error!(
                "{}: file doen't have epub extension, filename: {}",
                op,
                orig_filename
            );

            return HttpResponse::BadRequest().json(ErrorResponse {
                message: "invalid file format".to_string(),
            });
        }

        filename = Uuid::new_v4().to_string();

        // FIXME: directory must be a const!
        let filepath = format!("./uploads/books/{}", &filename);

        let mut file = match File::create(&filepath).await {
            Ok(file) => file,
            Err(err) => {
                log::error!(
                    "{}: failed to save the file, file path: {}, error: {}",
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

        // FIXME: need to wait then file are saving
        // let doc = EpubDoc::new(&filepath);

        // if doc.is_err() {
        //     log::error!(
        //         "{}: file is not correct epub, error: {}",
        //         op,
        //         doc.err().unwrap()
        //     );

        //     fs::remove_file(filepath).await.unwrap();

        //     return HttpResponse::BadRequest().json(ErrorResponse {
        //         message: "invalid file format".to_string(),
        //     });
        // }
    }

    log::info!(
        "{}: book is successfuly uploaded, filename: {}",
        op,
        filename
    );

    HttpResponse::Ok().json(filename)
}

enum AvailableError {
    NotFound,
    Forbidden,
}

async fn check_available(
    book_id: i32,
    user_id: i32,
    pool: &sqlx::Pool<Postgres>,
    op: &str,
) -> Result<Book, AvailableError> {
    let book = match find_book_by_id(book_id, pool).await {
        Ok(book) => book,
        Err(err) => {
            log::error!("{}: book id: {} is not exist, error: {}", op, book_id, err);

            return Err(AvailableError::NotFound);
        }
    };

    if !user_is_onwer_book(user_id, book_id, pool)
        .await
        .unwrap_or(false)
    {
        log::error!(
            "{}: user id: {} is not owned by book: {}",
            op,
            user_id,
            book_id
        );

        return Err(AvailableError::Forbidden);
    }

    Ok(book)
}
