use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Intern, CreateInternRequest, UpdateInternRequest};
use sqlx::Row;

// ==================== LIST INTERNS ====================
pub async fn list_interns(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Intern>>, AppError> {
    // ✅ LOG: Catat akses
    tracing::info!("🔍 list_interns called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized access to list_interns by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Executing SQL query to fetch interns...");
    
    let rows = sqlx::query(
        "SELECT id, user_id, university, major, start_date, end_date, status, created_at FROM interns"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch interns: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Found {} interns", rows.len());

    let interns: Vec<Intern> = rows.iter().map(|row| Intern {
        id: row.get("id"),
        user_id: row.get("user_id"),
        university: row.get("university"),
        major: row.get("major"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(Json(interns))
}

// ==================== GET INTERN BY ID ====================
pub async fn get_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Intern>, AppError> {
    tracing::info!("🔍 get_intern called for id={}, by user_id={}", id, claims.sub);

    let row = sqlx::query(
        "SELECT id, user_id, university, major, start_date, end_date, status, created_at FROM interns WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch intern {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    let intern = Intern {
        id: row.get("id"),
        user_id: row.get("user_id"),
        university: row.get("university"),
        major: row.get("major"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    };

    if claims.role != "admin" && claims.role != "superadmin" && intern.user_id != claims.sub {
        tracing::warn!("⚠️ Unauthorized access to intern {} by user {}", id, claims.sub);
        return Err(AppError::Unauthorized);
    }

    Ok(Json(intern))
}

// ==================== CREATE INTERN ====================
pub async fn create_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateInternRequest>,
) -> Result<(StatusCode, Json<Intern>), AppError> {
    tracing::info!("📝 create_intern called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to create intern");
        return Err(AppError::Unauthorized);
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating intern with id={}", new_id);

    sqlx::query(
        "INSERT INTO interns (id, user_id, university, major, start_date, end_date, status) VALUES (?, ?, ?, ?, ?, ?, 'active')"
    )
    .bind(&new_id)
    .bind(&payload.user_id)
    .bind(&payload.university)
    .bind(&payload.major)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create intern: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Intern created successfully: {}", new_id);

    let row = sqlx::query(
        "SELECT id, user_id, university, major, start_date, end_date, status, created_at FROM interns WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created intern: {:?}", e);
        AppError::Database(e)
    })?;

    let intern = Intern {
        id: row.get("id"),
        user_id: row.get("user_id"),
        university: row.get("university"),
        major: row.get("major"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        status: row.get("status"),
        created_at: row.get("created_at"),
    };

    Ok((StatusCode::CREATED, Json(intern)))
}

// ==================== UPDATE INTERN ====================
pub async fn update_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateInternRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✏️ update_intern called for id={}, by user_id={}", id, claims.sub);

    if claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to update intern {}", id);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Updating intern {} with payload: {:?}", id, payload);

    sqlx::query(
        "UPDATE interns SET university = COALESCE(?, university), major = COALESCE(?, major), end_date = COALESCE(?, end_date), status = COALESCE(?, status) WHERE id = ?"
    )
    .bind(&payload.university)
    .bind(&payload.major)
    .bind(&payload.end_date)
    .bind(&payload.status)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update intern {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Intern {} updated successfully", id);

    Ok(StatusCode::OK)
}