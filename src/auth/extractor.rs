use axum::{extract::{FromRef, FromRequestParts}, http::request::Parts};
use jsonwebtoken::{DecodingKey, decode, Validation};

use crate::{errors::AppError, jwt::Claims, state::AppState};



pub struct AuthUser{
    pub user_id: i32, 
    
}

impl<S> FromRequestParts<S> for AuthUser
where 
    AppState: FromRef<S>,
    S: Send + Sync
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Missing Authorization header".to_string())
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Invalid authorization format".to_string())
            })?;
        
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        Ok(AuthUser { user_id:decoded.claims.sub})
    }
}
