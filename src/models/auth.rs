use serde::{Deserialize, Serialize};
use validator::Validate;

/// JWT tokens model
#[derive(Serialize, Deserialize, Debug)]
pub struct Tokens {
    pub access: String,
    pub refresh: String,
}

/// JSON scheme for sign in
///
/// Email validating and password min length equal 1
#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct SignInData {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

/// JSON scheme for sign up
///
/// Email validating
///
/// Password and username min length equal 6
#[derive(Clone, Serialize, Deserialize, Validate, Debug)]
pub struct SignUpData {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}
