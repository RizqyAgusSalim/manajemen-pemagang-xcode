use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

pub use crate::services::auth::Claims;

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

    let token = extract_token(&req).ok_or_else(|| {
        tracing::warn!("⚠️ [AUTH] No token for path: {}", path);
        (StatusCode::UNAUTHORIZED, "Missing token".to_string())
    })?;

    tracing::debug!("🔑 [AUTH] Validating token...");

    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set");
    
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|e| {
        tracing::error!("❌ [AUTH] JWT decode failed: {:?}", e);
        (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
    })?;

    tracing::info!("✅ [AUTH] OK: user_id={}, role={}", 
        token_data.claims.sub, token_data.claims.role);

    let mut req = req;
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}