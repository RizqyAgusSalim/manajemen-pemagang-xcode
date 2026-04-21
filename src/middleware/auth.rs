use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, EncodingKey};
use serde::{Serialize, Deserialize};
use crate::error::AppError;

// ✅ Claims harus: Serialize + Deserialize + Clone + Send + Sync
#[derive(Debug, Serialize, Deserialize, Clone)]  // ✅ TAMBAH Clone
pub struct Claims {
    pub sub: String,      // user_id
    pub role: String,     // role
    pub exp: usize,       // expiry timestamp
}

fn extract_token(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!("🔐 [MIDDLEWARE] {} {}", req.method(), req.uri().path());

    let path = req.uri().path();
    if path.starts_with("/auth/") || path == "/health" {
        tracing::debug!("⏭️ Skipping auth for: {}", path);
        return Ok(next.run(req).await);
    }

    let token = extract_token(&req).ok_or_else(|| {
        tracing::warn!("⚠️ [AUTH] No token for path: {}", path);
        (StatusCode::UNAUTHORIZED, "Missing token".to_string())
    })?;

    tracing::debug!("🔑 [AUTH] Validating token...");

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "fallback_secret_key_for_dev".to_string());
    
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| {
        tracing::error!("❌ [AUTH] JWT decode failed: {:?}", e);
        (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
    })?;

    tracing::info!("✅ [AUTH] OK: user_id={}, role={}", 
        token_data.claims.sub, token_data.claims.role);

    let mut req = req;
    req.extensions_mut().insert(token_data.claims);  // ✅ Sekarang Claims: Clone

    Ok(next.run(req).await)
}

// ✅ Helper: Generate JWT token
pub fn generate_token(user_id: String, role: &str, secret: &str) -> Result<String, AppError> {
    use chrono::Utc;
    
    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp: (Utc::now().timestamp() as usize) + 86400, // 24 jam
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| {
        tracing::error!("❌ Failed to encode JWT: {:?}", e);
        AppError::Internal
    })?;

    Ok(token)
}

// Helper functions
pub fn has_role(claims: &Claims, role: &str) -> bool {
    claims.role == role
}

pub fn is_owner(claims: &Claims, resource_user_id: &str) -> bool {
    claims.sub == resource_user_id
}