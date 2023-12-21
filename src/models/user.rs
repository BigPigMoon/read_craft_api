use chrono::NaiveDateTime;

pub struct User {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub refresh_token_hash: Option<String>,
}

pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub refresh_token_hash: Option<String>,
}
