use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::errors::ApiError;
use crate::models::user::Claims;
use crate::config::AppConfig;

pub fn create_token(user_id: &str, email: &str, role: &str, config: &AppConfig) -> Result<String, ApiError> {
    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now.timestamp()) + config.jwt.expiration;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| ApiError::JwtError(e.to_string()))
}

pub fn verify_token(token: &str, config: &AppConfig) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| ApiError::JwtError(e.to_string()))
}
