use jwt_simple::prelude::{Duration, VerificationOptions, *};

use crate::extractors::jwt_cred::JwtCred;

pub mod scopes {
    pub const ACCESS: &str = "access";
    pub const REFRESH: &str = "refresh";
}

#[derive(Clone, Debug)]
pub struct JwtUtil {
    pub key: HS256Key,
}

impl JwtUtil {
    pub fn get_claims(&self, token: &str, scope_check: &str) -> Option<JwtCred> {
        match self.decode_token(token) {
            Ok(claims) => {
                if claims.scope.eq(scope_check) {
                    Some(claims)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn encode_token(
        &self,
        claims: JwtCred,
        duration: Duration,
    ) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(claims, duration);

        self.key.authenticate(claims)
    }

    pub fn decode_token(&self, token: &str) -> Result<JwtCred, jwt_simple::Error> {
        match self
            .key
            .verify_token::<JwtCred>(token, Some(self.get_options()))
        {
            Ok(data) => Ok(data.custom),
            Err(err) => Err(err),
        }
    }

    fn get_options(&self) -> VerificationOptions {
        VerificationOptions {
            accept_future: false,
            time_tolerance: Some(Duration::from_secs(0)),
            ..Default::default()
        }
    }
}
