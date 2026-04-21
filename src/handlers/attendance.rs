use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use uuid::Uuid;
use sqlx::Row;
use chrono::NaiveTime;
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Attendance, CreateAttendanceRequest, UpdateAttendanceStatusRequest, UpdateEndTimeRequest};

// ==================== LIST ATTENDANCES ====================
pub async fn list_attendances(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Attendance>>, AppError> {
    tracing::info!("📋 list_attendances called by user_id={}, role={}", claims.sub, claims.role);

    let attendances = match claims.role.as_str() {
        "superadmin" | "admin" => {
            sqlx::query_as::<_, Attendance>(
                "SELECT id, intern_id, date, attendance_time, start_time, end_time, description, status, confirmed_by, confirmed_at, created_at 
                 FROM attendances ORDER BY date DESC"
            )
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch attendances: {:?}", e);
                AppError::Database(e)
            })?
        },
        "supervisor" => {
            sqlx::query_as::<_, Attendance>(
                "SELECT a.* FROM attendances a
                 INNER JOIN interns i ON a.intern_id = i.id
                 INNER JOIN tasks t ON t.intern_id = i.id
                 WHERE t.supervisor_id = ?
                 ORDER BY a.date DESC"
            )
            .bind(&claims.sub)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch supervisor attendances: {:?}", e);
                AppError::Database(e)
            })?
        },
        "intern" => {
            sqlx::query_as::<_, Attendance>(
                "SELECT id, intern_id, date, attendance_time, start_time, end_time, description, status, confirmed_by, confirmed_at, created_at 
                 FROM attendances 
                 WHERE intern_id = (SELECT id FROM interns WHERE user_id = ?)
                 ORDER BY date DESC"
            )
            .bind(&claims.sub)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch intern attendances: {:?}", e);
                AppError::Database(e)
            })?
        },
        _ => return Err(AppError::Unauthorized),
    };

    Ok(Json(attendances))
}

// ==================== CREATE ATTENDANCE (Check-In) ====================
pub async fn create_attendance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateAttendanceRequest>,
) -> Result<(StatusCode, Json<Attendance>), AppError> {
    tracing::info!("📝 create_attendance called by user_id={}", claims.sub);

    if claims.role != "intern" {
        tracing::warn!("⚠️ Non-intern tried to create attendance");
        return Err(AppError::Unauthorized);
    }

    // Validasi status kehadiran
    if !["Hadir", "Izin", "Alfa", "Sakit"].contains(&payload.status.as_str()) {
        tracing::warn!("⚠️ Invalid attendance status: {}", payload.status);
        return Err(AppError::Internal);
    }

    let intern_exists = sqlx::query(
        "SELECT id FROM interns WHERE id = ? AND user_id = ?"
    )
    .bind(&payload.intern_id)
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !intern_exists {
        tracing::warn!("⚠️ Intern {} not owned by user {}", payload.intern_id, claims.sub);
        return Err(AppError::Unauthorized);
    }

    let new_id = Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating attendance with id={}", new_id);

    // ✅ INSERT dengan start_time dan end_time
    sqlx::query(
        "INSERT INTO attendances (id, intern_id, date, attendance_time, start_time, end_time, description, status) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&payload.intern_id)
    .bind(&payload.date)
    .bind(&payload.attendance_time)  // Waktu check-in
    .bind(&payload.start_time)       // Jam mulai kerja
    .bind(&payload.end_time)         // Jam selesai kerja (bisa NULL jika belum selesai)
    .bind(&payload.description)
    .bind(&payload.status)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create attendance: {:?}", e);
        AppError::Database(e)
    })?;

    // Fetch created attendance
    let attendance = sqlx::query_as::<_, Attendance>(
        "SELECT id, intern_id, date, attendance_time, start_time, end_time, description, status, confirmed_by, confirmed_at, created_at 
         FROM attendances WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created attendance: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Attendance created successfully: {}", new_id);
    Ok((StatusCode::CREATED, Json(attendance)))
}

// ==================== UPDATE END TIME (Check-Out) ====================
pub async fn update_end_time(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEndTimeRequest>,  // Import struct baru di models
) -> Result<StatusCode, AppError> {
    tracing::info!("⏰ update_end_time called for id={}, by user_id={}", id, claims.sub);

    if claims.role != "intern" {
        tracing::warn!("⚠️ Non-intern tried to update end time");
        return Err(AppError::Unauthorized);
    }

    // Cek apakah attendance milik user ini
    let is_owner = sqlx::query(
        "SELECT 1 FROM attendances a
         INNER JOIN interns i ON a.intern_id = i.id
         WHERE a.id = ? AND i.user_id = ? AND a.end_time IS NULL"
    )
    .bind(&id)
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_owner {
        tracing::warn!("⚠️ Attendance {} not found or already has end_time", id);
        return Err(AppError::Unauthorized);
    }

    // Update end_time
    sqlx::query(
        "UPDATE attendances SET end_time = ? WHERE id = ?"
    )
    .bind(&payload.end_time)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update end_time: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ End time updated for attendance {}", id);
    Ok(StatusCode::OK)
}

// ==================== UPDATE ATTENDANCE STATUS (Konfirmasi Supervisor) ====================
pub async fn update_attendance_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAttendanceStatusRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✅ update_attendance_status called for id={}, by user_id={}, role={}", 
        id, claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to update attendance status");
        return Err(AppError::Unauthorized);
    }

    if payload.status != "approved" && payload.status != "rejected" {
        tracing::warn!("⚠️ Invalid confirmation status: {}", payload.status);
        return Err(AppError::Internal);
    }

    let attendance = sqlx::query(
        "SELECT a.*, i.user_id as intern_user_id FROM attendances a
         INNER JOIN interns i ON a.intern_id = i.id
         WHERE a.id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?;

    match attendance {
        Some(row) => {
            let _intern_user_id: String = row.get("intern_user_id");
            let current_status: String = row.get("status");
            
            // Supervisor hanya bisa konfirmasi absen dari intern yang dibimbing
            if claims.role == "supervisor" {
                let is_supervisor = sqlx::query(
                    "SELECT 1 FROM tasks WHERE intern_id = ? AND supervisor_id = ?"
                )
                .bind(&row.get::<String, _>("intern_id"))
                .bind(&claims.sub)
                .fetch_optional(&state.pool)
                .await?
                .is_some();
                
                if !is_supervisor {
                    tracing::warn!("⚠️ Supervisor {} not assigned to intern", claims.sub);
                    return Err(AppError::Unauthorized);
                }
            }
            
            // Tidak bisa konfirmasi jika sudah confirmed
            if current_status == "approved" || current_status == "rejected" {
                tracing::warn!("⚠️ Attendance {} already confirmed (status: {})", id, current_status);
                return Err(AppError::Internal);
            }
        },
        None => {
            tracing::error!("❌ Attendance {} not found", id);
            return Err(AppError::Internal);
        }
    }

    // Update status konfirmasi + confirmed_by + confirmed_at
    sqlx::query(
        "UPDATE attendances SET 
         confirmed_by = ?, 
         confirmed_at = NOW()
         WHERE id = ?"
    )
    .bind(&claims.sub)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update attendance confirmation: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Attendance {} confirmed by user {}", id, claims.sub);
    Ok(StatusCode::OK)
}

// ==================== GET ATTENDANCE BY ID ====================
pub async fn get_attendance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Attendance>, AppError> {
    tracing::info!("🔍 get_attendance called for id={}, by user_id={}", id, claims.sub);

    let attendance = sqlx::query_as::<_, Attendance>(
        "SELECT id, intern_id, date, attendance_time, start_time, end_time, description, status, confirmed_by, confirmed_at, created_at 
         FROM attendances WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch attendance {}: {:?}", id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::error!("❌ Attendance {} not found", id);
        AppError::Internal
    })?;

    // RBAC: Intern hanya bisa lihat absen sendiri
    if claims.role == "intern" {
        let is_owner = sqlx::query(
            "SELECT 1 FROM interns WHERE id = ? AND user_id = ?"
        )
        .bind(&attendance.intern_id)
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .is_some();
        
        if !is_owner {
            tracing::warn!("⚠️ Intern {} tried to access attendance {} not owned", claims.sub, id);
            return Err(AppError::Unauthorized);
        }
    }

    Ok(Json(attendance))
}