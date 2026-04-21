use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Ubah ke String
    pub role: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, crate::error::AppError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| crate::error::AppError::Internal)
        .map(|h| h.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, crate::error::AppError> {
    let parsed = PasswordHash::new(hash).map_err(|_| crate::error::AppError::Internal)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}

pub fn generate_token(user_id: String, role: &str, secret: &str) -> Result<String, crate::error::AppError> {
    // Parse String ke Uuid untuk JWT claims (atau bisa langsung pakai String)
    let _uuid = Uuid::parse_str(&user_id).map_err(|_| crate::error::AppError::Internal)?;
    
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 86400;

    let claims = Claims {
        sub: user_id,  // Tetap String
        role: role.to_string(),
        exp,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|_| crate::error::AppError::Internal)
}