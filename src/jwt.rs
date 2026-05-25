use crate::errors::AppError;
use argon2::password_hash::rand_core::OsRng;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

pub fn refresh_token_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::days(30)
}

pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_refresh_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}
