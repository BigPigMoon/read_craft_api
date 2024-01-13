use std::error::Error;

use sqlx::Postgres;

use crate::models::{
    book::{Book, CreateBook, UpdateBook},
    language::Language,
};

/// Create new book in database
pub async fn create_book_db(
    book: &CreateBook,
    user_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let new_book_id = sqlx::query!(
        "INSERT INTO books (title, language, filename, cover_path, author, subject) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        book.title,
        book.language as Language, // NOTE: need to cast
        book.filename,
        book.cover_path,
        book.author,
        book.subject,
    )
    .fetch_one(pool)
    .await?
    .id;

    sqlx::query!(
        "INSERT INTO book_user (book_id, user_id, chunk) VALUES ($1, $2, 0)",
        new_book_id,
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(new_book_id)
}

pub async fn find_book_by_id(id: i32, pool: &sqlx::Pool<Postgres>) -> Result<Book, Box<dyn Error>> {
    let book = sqlx::query_as!(
        Book,
        r#"
        SELECT
            id, created_at, updated_at,
            title, language as "language!: Language", filename,
            cover_path, author, subject, progress 
        FROM
            books
        WHERE
            id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(book)
}

/// return true if user are owner of book
pub async fn user_is_onwer_book(
    user_id: i32,
    book_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<bool, Box<dyn Error>> {
    let is_owner = sqlx::query!(
        r#"SELECT * FROM book_user WHERE book_id=$1 AND user_id=$2"#,
        book_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(is_owner.is_some())
}

pub async fn all_user_book(
    user_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<Book>, Box<dyn Error>> {
    let ids: Vec<Option<i32>> =
        sqlx::query!("SELECT book_id FROM book_user WHERE user_id = $1", user_id,)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|rec| rec.book_id)
            .collect();

    let mut books: Vec<Book> = Vec::new();

    for id in ids.iter() {
        if let Some(id) = id {
            let book = find_book_by_id(*id, pool).await.unwrap();

            books.push(book);
        }
    }

    Ok(books)
}

pub async fn update_book_db(
    book: &UpdateBook,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE books SET title = $2, language = $3, author = $4, subject = $5, cover_path = $6 WHERE id = $1",
        book.id,
        book.title,
        book.language as Language,
        book.author,
        book.subject,
        book.cover_path,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_book_db(
    book_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!("DELETE FROM books WHERE id = $1", book_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_book_chunk(
    user_id: i32,
    book_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let chunk = sqlx::query!(
        "SELECT * from book_user WHERE user_id = $1 AND book_id = $2",
        user_id,
        book_id
    )
    .fetch_one(pool)
    .await?
    .chunk;

    Ok(chunk)
}

pub async fn set_book_chunk(
    user_id: i32,
    book_id: i32,
    page: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE book_user SET chunk = $3 WHERE user_id = $1 AND book_id = $2",
        user_id,
        book_id,
        page
    )
    .execute(pool)
    .await?;

    Ok(())
}

/*

async fn query(pool: &sqlx::Pool<Postgres>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

*/
