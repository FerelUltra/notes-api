use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

pub fn generate_access_token(user_id: i32, jwt_secret: &str) -> Result<String, AppError> {
    let expires_at = Utc::now()
        .checked_add_signed(Duration::minutes(60))
        .ok_or_else(|| {
            AppError::InternalServerError("Failed to calculate token expiration".to_string())
        })?;

    let claims = Claims {
        sub: user_id,
        exp: expires_at.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError("Failed to generate token".to_string()))?;

    Ok(token)
}
