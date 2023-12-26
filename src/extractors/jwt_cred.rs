use actix_web::{dev::Payload, error, http::header, Error, FromRequest, HttpRequest};
use futures_util::future::ready;
use serde::{Deserialize, Serialize};

use crate::{
    get_key,
    utils::jwt::{scopes, JwtUtil},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtCred {
    pub uid: i32,
    pub email: String,
    pub scope: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    Unauthorized,
}

impl FromRequest for JwtCred {
    type Error = Error;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match get_token_from_req(req.clone()) {
            Ok(token) => {
                let jwt_controller = JwtUtil { key: get_key() };

                match jwt_controller.get_claims(&token.as_str(), scopes::ACCESS) {
                    Some(claims) => return ready(Ok(claims)),
                    None => {
                        return ready(Err(error::ErrorUnauthorized(
                            "authorization header is missing",
                        )))
                    }
                }
            }
            Err(err) => match err {
                AuthError::InvalidToken => {
                    ready(Err(error::ErrorBadRequest("invalid token format")))
                }
                AuthError::Unauthorized => ready(Err(error::ErrorUnauthorized(
                    "authorization header is missing",
                ))),
            },
        }
    }
}

pub fn get_token_from_req(req: HttpRequest) -> Result<String, AuthError> {
    match req.headers().get(header::AUTHORIZATION) {
        Some(beared_token) => {
            if let Ok(token_str) = beared_token.to_str() {
                if let Some(token) = token_str.split(' ').last() {
                    Ok(token.to_string())
                } else {
                    Err(AuthError::Unauthorized)
                }
            } else {
                Err(AuthError::Unauthorized)
            }
        }
        None => Err(AuthError::Unauthorized),
    }
}
