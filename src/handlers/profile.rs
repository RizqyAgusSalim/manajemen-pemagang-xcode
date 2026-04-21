use axum::{extract::State, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::User;
use sqlx::Row;
use chrono::{DateTime, Utc};

// Request struct untuk update profil
#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
}

// ✅ GET /api/profile - Ambil profil user yang login
pub async fn get_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    tracing::info!("👤 get_profile called by user_id={}", claims.sub);

    let row = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users WHERE id = ?"
    )
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch profile: {:?}", e);
        AppError::Database(e)
    })?;

    let user = User {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    Ok(Json(user))
}

// ✅ PUT /api/profile - Update profil user yang login (hanya diri sendiri)
pub async fn update_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<User>, AppError> {
    tracing::info!("✏️ update_profile called by user_id={}", claims.sub);

    // ✅ Hanya update user yang login (claims.sub) - tidak bisa edit orang lain
    sqlx::query(
        "UPDATE users SET 
         full_name = COALESCE(?, full_name),
         email = COALESCE(?, email)
         WHERE id = ?"
    )
    .bind(&payload.full_name)
    .bind(&payload.email)
    .bind(&claims.sub)  // ✅ Hanya update diri sendiri
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update profile: {:?}", e);
        AppError::Database(e)
    })?;

    // Return updated user
    let row = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users WHERE id = ?"
    )
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch updated profile: {:?}", e);
        AppError::Database(e)
    })?;

    let user = User {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    tracing::info!("✅ Profile updated successfully for user {}", claims.sub);
    Ok(Json(user))
}