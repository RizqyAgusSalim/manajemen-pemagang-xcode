use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Logbook, CreateLogbookRequest, UpdateLogbookRequest, ApproveLogbookRequest};
use sqlx::Row;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;  // ✅ TAMBAH INI

// ==================== LIST LOGBOOKS ====================
pub async fn list_logbooks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Logbook>>, AppError> {
    tracing::info!("🔍 list_logbooks called by user_id={}, role={}", claims.sub, claims.role);

    let rows = if claims.role == "admin" || claims.role == "supervisor" || claims.role == "superadmin" {
        tracing::debug!("📦 Fetching all logbooks");
        sqlx::query(
            "SELECT id, intern_id, date, activity, description, status, supervisor_notes, created_at 
             FROM logbooks ORDER BY date DESC"
        )
        .fetch_all(&state.pool)
        .await
    } else {
        tracing::debug!("📦 Fetching logbooks for intern_id={}", claims.sub);
        sqlx::query(
            "SELECT id, intern_id, date, activity, description, status, supervisor_notes, created_at 
             FROM logbooks WHERE intern_id = ? ORDER BY date DESC"
        )
        .bind(&claims.sub)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch logbooks: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Found {} logbooks", rows.len());

    let logbooks: Vec<Logbook> = rows.iter().map(|row| Logbook {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        date: row.get::<NaiveDate, _>("date"),
        activity: row.get::<String, _>("activity"),
        description: row.get::<Option<String>, _>("description"),
        status: row.get::<String, _>("status"),
        supervisor_notes: row.get::<Option<String>, _>("supervisor_notes"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }).collect();

    Ok(Json(logbooks))
}

// ==================== GET LOGBOOK ====================
pub async fn get_logbook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Logbook>, AppError> {
    tracing::info!("🔍 get_logbook called for id={}, by user_id={}", id, claims.sub);

    let row = sqlx::query(
        "SELECT id, intern_id, date, activity, description, status, supervisor_notes, created_at 
         FROM logbooks WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)  // ✅ UBAH: fetch_one -> fetch_optional
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch logbook {}: {:?}", id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {  // ✅ TAMBAH: ok_or_else untuk handle Option
        tracing::error!("❌ Logbook {} not found in database", id);
        AppError::Internal
    })?;

    let logbook = Logbook {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        date: row.get::<NaiveDate, _>("date"),
        activity: row.get::<String, _>("activity"),
        description: row.get::<Option<String>, _>("description"),
        status: row.get::<String, _>("status"),
        supervisor_notes: row.get::<Option<String>, _>("supervisor_notes"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    if claims.role == "intern" && logbook.intern_id != claims.sub {
        tracing::warn!("⚠️ Unauthorized access to logbook {} by user {}", id, claims.sub);
        return Err(AppError::Unauthorized);
    }

    Ok(Json(logbook))
}

// ==================== CREATE LOGBOOK ====================
pub async fn create_logbook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateLogbookRequest>,
) -> Result<(StatusCode, Json<Logbook>), AppError> {
    tracing::info!("📝 create_logbook called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "intern" {
        tracing::warn!("⚠️ Unauthorized attempt to create logbook by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    let new_id = Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating logbook with id={}", new_id);

    sqlx::query(
        "INSERT INTO logbooks (id, intern_id, date, activity, description, status) 
         VALUES (?, ?, ?, ?, ?, 'draft')"
    )
    .bind(&new_id)
    .bind(&claims.sub)
    .bind(&payload.date)
    .bind(&payload.activity)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create logbook: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Logbook created successfully: {}", new_id);

    let row = sqlx::query(
        "SELECT id, intern_id, date, activity, description, status, supervisor_notes, created_at 
         FROM logbooks WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created logbook: {:?}", e);
        AppError::Database(e)
    })?;

    let logbook = Logbook {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        date: row.get::<NaiveDate, _>("date"),
        activity: row.get::<String, _>("activity"),
        description: row.get::<Option<String>, _>("description"),
        status: row.get::<String, _>("status"),
        supervisor_notes: row.get::<Option<String>, _>("supervisor_notes"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    Ok((StatusCode::CREATED, Json(logbook)))
}

// ==================== UPDATE LOGBOOK ====================
pub async fn update_logbook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLogbookRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✏️ update_logbook called for id={}, by user_id={}", id, claims.sub);

    let check_row = sqlx::query(
        "SELECT intern_id FROM logbooks WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to check logbook ownership: {:?}", e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::error!("❌ Logbook {} not found for update", id);
        AppError::Internal
    })?;

    let intern_id: String = check_row.get("intern_id");

    if claims.role == "intern" && intern_id != claims.sub {
        tracing::warn!("⚠️ Unauthorized update attempt for logbook {} by user {}", id, claims.sub);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Updating logbook {}", id);  // ✅ HAPUS payload dari debug (tidak implement Debug)

    sqlx::query(
        "UPDATE logbooks SET activity = COALESCE(?, activity), description = COALESCE(?, description) 
         WHERE id = ?"
    )
    .bind(&payload.activity)
    .bind(&payload.description)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update logbook {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Logbook {} updated successfully", id);

    Ok(StatusCode::OK)
}

// ==================== APPROVE LOGBOOK ====================
pub async fn approve_logbook(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveLogbookRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✅ approve_logbook called for id={}, by user_id={}, role={}", id, claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to approve logbook by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }
    
    if payload.status != "approved" && payload.status != "rejected" {
        tracing::warn!("⚠️ Invalid status value '{}' for logbook {}", payload.status, id);
        return Err(AppError::Internal);
    }

    tracing::debug!("📦 Approving logbook {} with status={}", id, payload.status);

    sqlx::query(
        "UPDATE logbooks SET status = ?, supervisor_notes = COALESCE(?, supervisor_notes) 
         WHERE id = ?"
    )
    .bind(&payload.status)
    .bind(&payload.notes)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to approve logbook {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Logbook {} approved/rejected successfully", id);

    Ok(StatusCode::OK)
}