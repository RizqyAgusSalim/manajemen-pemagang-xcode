use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use uuid::Uuid;
use sqlx::Row;
use chrono::{DateTime, Utc};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::User;
use crate::services::auth::hash_password;

// ==================== REQUEST STRUCTS ====================
#[derive(Debug, serde::Deserialize)]  // ✅ TAMBAH Debug
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Debug, serde::Deserialize)]  // ✅ TAMBAH Debug
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

// ==================== HELPER: MAP ROW TO USER ====================
fn map_user(row: &sqlx::mysql::MySqlRow) -> User {
    User {
        id: row.get::<String, _>("id"),
        email: row.get::<String, _>("email"),
        role: row.get::<String, _>("role"),
        full_name: row.get::<String, _>("full_name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }
}

// ==================== LIST USERS ====================
pub async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<User>>, AppError> {
    tracing::info!("🔍 list_users called by user_id={}, role={}", claims.sub, claims.role);
    tracing::info!("🚪 [ENTRY] list_users handler called");

    if claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized access attempt to /users by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Executing SQL query to fetch users...");
    
    let rows = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ SQL query failed to fetch users: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Found {} users", rows.len());

    Ok(Json(rows.iter().map(|r| map_user(r)).collect()))
}

// ==================== GET USER BY ID ====================
pub async fn get_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    tracing::info!("🔍 get_user called for id={}, by user_id={}", id, claims.sub);

    if claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized access to get_user by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    // ✅ PERBAIKAN: Gunakan fetch_optional + ok_or_else
    let row = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)  // ✅ UBAH: fetch_one -> fetch_optional
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch user {}: {:?}", id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {  // ✅ TAMBAH: Handle Option<Row>
        tracing::error!("❌ User {} not found in database", id);
        AppError::Internal
    })?;

    Ok(Json(map_user(&row)))
}

// ==================== CREATE USER ====================
pub async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    tracing::info!("📝 create_user called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized attempt to create user");
        return Err(AppError::Unauthorized);
    }

    if claims.role == "admin" && payload.role == "superadmin" {
        tracing::warn!("⚠️ Admin tried to create superadmin - denied");
        return Err(AppError::Unauthorized);
    }

    let user_id = Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating user with id={}", user_id);

    let password_hash = hash_password(&payload.password).map_err(|e| {
        tracing::error!("❌ Failed to hash password: {:?}", e);
        AppError::Internal
    })?;

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, role, full_name) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.role)
    .bind(&payload.full_name)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to insert user {}: {:?}", user_id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ User created successfully: {}", user_id);

    let row = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.pool)  // ✅ fetch_one OK di sini karena kita baru insert
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created user {}: {:?}", user_id, e);
        AppError::Database(e)
    })?;

    Ok((StatusCode::CREATED, Json(map_user(&row))))
}

// ==================== UPDATE USER ====================
pub async fn update_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✏️ update_user called for id={}, by user_id={}", id, claims.sub);

    if claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized attempt to update user {}", id);
        return Err(AppError::Unauthorized);
    }

    if let Some(ref new_role) = payload.role {
        if new_role == "superadmin" && claims.role != "superadmin" {
            tracing::warn!("⚠️ Non-superadmin tried to set role to superadmin");
            return Err(AppError::Unauthorized);
        }
    }

    let password_hash = if let Some(ref pwd) = payload.password {
        match hash_password(pwd) {
            Ok(h) => Some(h),
            Err(e) => {
                tracing::error!("❌ Failed to hash new password: {:?}", e);
                return Err(AppError::Internal);
            }
        }
    } else {
        None
    };

    // ✅ PERBAIKAN: Hapus {:?} untuk payload karena tidak semua field implement Debug dengan baik
    tracing::debug!("📦 Updating user {} (email={:?}, role={:?})", 
        id, payload.email, payload.role);

    sqlx::query(
        "UPDATE users SET 
         email = COALESCE(?, email),
         full_name = COALESCE(?, full_name),
         role = COALESCE(?, role),
         password_hash = COALESCE(?, password_hash)
         WHERE id = ?"
    )
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .bind(&password_hash)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update user {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ User {} updated successfully", id);

    Ok(StatusCode::OK)
}

// ==================== DELETE USER ====================
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("🗑️ delete_user called for id={}, by user_id={}", id, claims.sub);

    if claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized attempt to delete user {}", id);
        return Err(AppError::Unauthorized);
    }

    let target = sqlx::query("SELECT role FROM users WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to check user {} for deletion: {:?}", id, e);
            AppError::Database(e)
        })?;

    match target {
        Some(row) => {
            let role: String = row.get("role");
            
            if role == "superadmin" && claims.role != "superadmin" {
                tracing::warn!("⚠️ Non-superadmin tried to delete superadmin {}", id);
                return Err(AppError::Unauthorized);
            }
            
            if id == claims.sub {
                tracing::error!("❌ User {} tried to delete themselves", claims.sub);
                return Err(AppError::Internal);
            }
            
            tracing::debug!("✅ Authorization passed for deleting user {}", id);
        },
        None => {
            tracing::error!("❌ User {} not found for deletion", id);
            return Err(AppError::Internal);
        }
    }

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to delete user {}: {:?}", id, e);
            AppError::Database(e)
        })?;

    tracing::info!("✅ User {} deleted successfully", id);

    Ok(StatusCode::NO_CONTENT)
}