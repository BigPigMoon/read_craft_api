use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct Tokens {
    pub access: String,
    pub refresh: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct SignInData {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct SignUpData {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}
