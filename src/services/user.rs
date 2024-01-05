use chrono::Utc;
use crypto::{digest::Digest, sha2::Sha256};
use sqlx::Postgres;
use std::error::Error;

use crate::models::user::{CreateUser, User};

/// create user function in database
pub async fn create_user(
    user: &CreateUser,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let id = sqlx::query!(
        "INSERT INTO users(email, username, password_hash, refresh_token_hash) VALUES ($1, $2, $3, $4) RETURNING id",
        user.email,
        user.username,
        user.password_hash,
        user.refresh_token_hash
    ).fetch_one(pool).await?.id;

    Ok(id)
}

/// find user by id
pub async fn find_user_by_id(id: i32, pool: &sqlx::Pool<Postgres>) -> Result<User, Box<dyn Error>> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id=$1", id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

/// find user by email
pub async fn find_user_by_email(
    email: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<User, Box<dyn Error>> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email=$1", email)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

impl User {
    /// update refresh token for user
    pub async fn update_refresh_token(
        &self,
        token: Option<&str>,
        pool: &sqlx::Pool<Postgres>,
    ) -> Result<(), Box<dyn Error>> {
        let token = match token {
            Some(token) => {
                let mut hasher = Sha256::new();
                hasher.input_str(token);
                Some(hasher.result_str())
            }
            None => None,
        };

        sqlx::query!(
            "UPDATE users SET refresh_token_hash=$2, updated_at = $3 WHERE id=$1",
            self.id,
            token,
            Utc::now().naive_utc(),
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
