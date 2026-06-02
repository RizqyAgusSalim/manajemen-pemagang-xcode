use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use uuid::Uuid;
use sqlx::Row;
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{
    Attendance, 
    AttendanceWithIntern,  // ✅ Import struct baru
    CreateAttendanceRequest, 
    UpdateAttendanceStatusRequest, 
    UpdateEndTimeRequest
};

// ==================== LIST ATTENDANCES ====================
pub async fn list_attendances(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AttendanceWithIntern>>, AppError> {
    tracing::info!("📋 list_attendances called by user_id={}, role={}", claims.sub, claims.role);

    let attendances = match claims.role.as_str() {
        "superadmin" | "admin" | "supervisor" => {
            // ✅ JOIN dengan interns dan users untuk mendapatkan nama lengkap
            sqlx::query_as::<_, AttendanceWithIntern>(
                "SELECT 
                    a.id, 
                    a.intern_id, 
                    a.date, 
                    a.attendance_time, 
                    a.start_time, 
                    a.end_time, 
                    a.description, 
                    a.status, 
                    a.confirmed_by, 
                    a.confirmed_at, 
                    a.created_at,
                    i.nama_lengkap as intern_name,
                    i.nim as intern_nim,
                    u.email as intern_email
                 FROM attendances a
                 LEFT JOIN interns i ON a.intern_id = i.id
                 LEFT JOIN users u ON i.user_id = u.id
                 ORDER BY i.nama_lengkap ASC, a.date ASC, a.created_at ASC"
            )
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch attendances: {:?}", e);
                AppError::Database(e)
            })?
        },
        "intern" => {
            // ✅ Untuk intern juga JOIN agar konsisten
            sqlx::query_as::<_, AttendanceWithIntern>(
                "SELECT 
                    a.id, 
                    a.intern_id, 
                    a.date, 
                    a.attendance_time, 
                    a.start_time, 
                    a.end_time, 
                    a.description, 
                    a.status, 
                    a.confirmed_by, 
                    a.confirmed_at, 
                    a.created_at,
                    i.nama_lengkap as intern_name,
                    i.nim as intern_nim,
                    u.email as intern_email
                 FROM attendances a
                 LEFT JOIN interns i ON a.intern_id = i.id
                 LEFT JOIN users u ON i.user_id = u.id
                 WHERE i.user_id = ?
                 ORDER BY a.date ASC, a.created_at ASC"
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

    tracing::info!("✅ Found {} attendances for role={}", attendances.len(), claims.role);
    Ok(Json(attendances))
}

// ==================== CREATE ATTENDANCE ====================
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

    // ✅ Izinkan semua status termasuk "pending"
    let valid_statuses = ["Hadir", "Izin", "Alfa", "Sakit", "pending"];
    if !valid_statuses.contains(&payload.status.as_str()) {
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

    sqlx::query(
        "INSERT INTO attendances (id, intern_id, date, attendance_time, start_time, end_time, description, status) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&payload.intern_id)
    .bind(&payload.date)
    .bind(&payload.attendance_time)
    .bind(&payload.start_time)
    .bind(&payload.end_time)
    .bind(&payload.description)
    .bind(&payload.status)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create attendance: {:?}", e);
        AppError::Database(e)
    })?;

    let attendance = sqlx::query_as::<_, Attendance>(
        "SELECT id, intern_id, date, attendance_time, start_time, end_time, 
                description, status, confirmed_by, confirmed_at, created_at 
         FROM attendances WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created attendance: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Attendance created: {}", new_id);
    Ok((StatusCode::CREATED, Json(attendance)))
}

// ==================== UPDATE END TIME ====================
pub async fn update_end_time(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEndTimeRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("⏰ update_end_time called for id={}", id);

    if claims.role != "intern" {
        return Err(AppError::Unauthorized);
    }

    let is_owner = sqlx::query(
        "SELECT 1 FROM attendances a
         INNER JOIN interns i ON a.intern_id = i.id
         WHERE a.id = ? AND i.user_id = ?"
    )
    .bind(&id)
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_owner {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("UPDATE attendances SET end_time = ? WHERE id = ?")
    .bind(&payload.end_time)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    tracing::info!("✅ End time updated for attendance {}", id);
    Ok(StatusCode::OK)
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateAttendanceRequest {
    pub date: Option<chrono::NaiveDate>,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub status: Option<String>,
    pub description: Option<String>,
}

// ==================== UPDATE ATTENDANCE (Admin/Supervisor & Intern owner) ====================
pub async fn update_attendance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAttendanceRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✏️ update_attendance called for id={}, by user_id={}, role={}", id, claims.sub, claims.role);

    // 1. Ambil data absen yang sudah ada
    let existing = sqlx::query_as::<_, Attendance>(
        "SELECT id, intern_id, date, attendance_time, start_time, end_time, 
                description, status, confirmed_by, confirmed_at, created_at 
         FROM attendances WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("Data absen tidak ditemukan".to_string()))?;

    // 2. Terapkan validasi otorisasi dan bisnis berdasarkan role
    let (final_date, final_status) = if claims.role == "intern" {
        // Cek kepemilikan data absen
        let is_owner = sqlx::query(
            "SELECT 1 FROM interns WHERE id = ? AND user_id = ?"
        )
        .bind(&existing.intern_id)
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .is_some();

        if !is_owner {
            tracing::warn!("⚠️ Intern {} tried to edit attendance owned by someone else", claims.sub);
            return Err(AppError::Unauthorized);
        }

        // Intern tidak boleh mengubah absensi hari kemarin
        let today = chrono::Local::now().date_naive();
        if existing.date != today {
            tracing::warn!("⚠️ Intern tried to edit past attendance for date={}", existing.date);
            return Err(AppError::BadRequest("Hanya bisa mengedit absensi hari ini".to_string()));
        }

        // Intern tidak boleh mengubah absensi yang sudah diverifikasi (approved) atau ditolak (rejected)
        if existing.status == "approved" || existing.status == "Hadir" || existing.status == "rejected" || existing.status == "Ditolak" {
            tracing::warn!("⚠️ Intern tried to edit locked attendance with status={}", existing.status);
            return Err(AppError::BadRequest("Absensi yang sudah diverifikasi atau ditolak tidak dapat diubah".to_string()));
        }

        // Intern tidak boleh mengubah tanggal dan status secara manual
        (existing.date, existing.status)
    } else if claims.role == "supervisor" || claims.role == "admin" || claims.role == "superadmin" {
        // Admin/Supervisor bebas mengubah tanggal dan status
        (payload.date.unwrap_or(existing.date), payload.status.unwrap_or(existing.status))
    } else {
        return Err(AppError::Unauthorized);
    };

    let final_start = payload.start_time.or(existing.start_time);
    let final_end = payload.end_time.or(existing.end_time);
    let final_desc = payload.description.or(existing.description);

    // 3. Lakukan update langsung
    sqlx::query(
        "UPDATE attendances SET 
         date = ?,
         start_time = ?,
         end_time = ?,
         status = ?,
         description = ?
         WHERE id = ?"
    )
    .bind(final_date)
    .bind(final_start)
    .bind(final_end)
    .bind(final_status)
    .bind(final_desc)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update attendance: {:?}", e);
        AppError::Database(e)
    })?;

    Ok(StatusCode::OK)
}

// ==================== UPDATE ATTENDANCE STATUS ====================
pub async fn update_attendance_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAttendanceStatusRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✅ update_attendance_status id={}, by role={}", id, claims.role);

    // ✅ Hanya supervisor, admin, superadmin yang bisa konfirmasi
    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Role {} tidak bisa konfirmasi absen", claims.role);
        return Err(AppError::Unauthorized);
    }

    if payload.status != "approved" && payload.status != "rejected" && payload.status != "Hadir" && payload.status != "Izin" && payload.status != "Sakit" && payload.status != "Alfa" {
        tracing::warn!("⚠️ Invalid status: {}", payload.status);
        return Err(AppError::Internal);
    }

    // Cek attendance ada
    let attendance_exists = sqlx::query("SELECT 1 FROM attendances WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .is_some();

    if !attendance_exists {
        tracing::error!("❌ Attendance {} not found", id);
        return Err(AppError::NotFound("Data absen tidak ditemukan".to_string()));
    }

    // ✅ Update status + confirmed_by + confirmed_at
    sqlx::query(
        "UPDATE attendances SET 
         status = ?,
         confirmed_by = ?, 
         confirmed_at = NOW()
         WHERE id = ?"
    )
    .bind(&payload.status)
    .bind(&claims.sub)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update attendance status: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Attendance {} -> {} by {}", id, payload.status, claims.sub);
    Ok(StatusCode::OK)
}

// ==================== GET ATTENDANCE BY ID ====================
pub async fn get_attendance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<AttendanceWithIntern>, AppError> {  // ✅ Ganti ke AttendanceWithIntern
    tracing::info!("🔍 get_attendance id={}", id);

    let attendance = sqlx::query_as::<_, AttendanceWithIntern>(
        "SELECT 
            a.id, 
            a.intern_id, 
            a.date, 
            a.attendance_time, 
            a.start_time, 
            a.end_time, 
            a.description, 
            a.status, 
            a.confirmed_by, 
            a.confirmed_at, 
            a.created_at,
            i.nama_lengkap as intern_name,
            i.nim as intern_nim,
            u.email as intern_email
         FROM attendances a
         LEFT JOIN interns i ON a.intern_id = i.id
         LEFT JOIN users u ON i.user_id = u.id
         WHERE a.id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::Internal)?;

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
            return Err(AppError::Unauthorized);
        }
    }

    Ok(Json(attendance))
}