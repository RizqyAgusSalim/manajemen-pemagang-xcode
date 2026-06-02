use axum::{extract::{State, Path, Multipart}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Task, CreateTaskRequest, UpdateTaskRequest};
use sqlx::Row;
use chrono::{NaiveDate, DateTime, Utc};

// ==================== LIST TASKS ====================
pub async fn list_tasks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Task>>, AppError> {
    tracing::info!("🔍 list_tasks called by user_id={}, role={}", claims.sub, claims.role);

    let rows = if claims.role == "admin" || claims.role == "supervisor" || claims.role == "superadmin" {
        tracing::debug!("📦 Fetching all tasks");
        sqlx::query(
            "SELECT id, intern_id, supervisor_id, title, description, status, deadline, submission_file, feedback, created_at FROM tasks"
        )
        .fetch_all(&state.pool)
        .await
    } else {
        tracing::debug!("📦 Fetching tasks for intern user_id={}", claims.sub);
        sqlx::query(
            "SELECT t.id, t.intern_id, t.supervisor_id, t.title, t.description, t.status, t.deadline, t.submission_file, t.feedback, t.created_at 
             FROM tasks t 
             LEFT JOIN interns i ON t.intern_id = i.id 
             WHERE t.intern_id = ? OR i.user_id = ?"
        )
        .bind(&claims.sub)
        .bind(&claims.sub)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch tasks: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Found {} tasks", rows.len());

    let tasks: Vec<Task> = rows.iter().map(|row| Task {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        supervisor_id: row.get::<String, _>("supervisor_id"),
        title: row.get::<String, _>("title"),
        description: row.get::<Option<String>, _>("description"),
        status: row.get::<String, _>("status"),
        deadline: row.get::<Option<NaiveDate>, _>("deadline"),
        submission_file: row.try_get::<Option<String>, _>("submission_file").ok().flatten(),
        feedback: row.try_get::<Option<String>, _>("feedback").ok().flatten(),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }).collect();

    Ok(Json(tasks))
}

// ==================== SUBMIT TASK (Intern) ====================
pub async fn submit_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    tracing::info!("📤 submit_task called for task_id={} by user_id={}", id, claims.sub);

    // Cek kepemilikan tugas
    let task = sqlx::query("SELECT t.id FROM tasks t JOIN interns i ON t.intern_id = i.id WHERE t.id = ? AND i.user_id = ?")
        .bind(&id)
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let mut file_path = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::Internal)? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("submission.pdf").to_string();
            let data = field.bytes().await.map_err(|_| AppError::Internal)?;
            
            // ✅ SECURITY: Batas ukuran file (10MB)
            const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;
            if data.len() > MAX_UPLOAD_SIZE {
                return Err(AppError::BadRequest("File terlalu besar (max 10MB)".into()));
            }
            
            // ✅ SECURITY: Validasi tipe file
            let allowed_extensions = ["pdf", "doc", "docx", "zip", "rar", "png", "jpg", "jpeg"];
            let ext = std::path::Path::new(&filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                return Err(AppError::BadRequest(
                    format!("Tipe file '{}' tidak diizinkan. Gunakan: {}", ext, allowed_extensions.join(", "))
                ));
            }
            
            let upload_dir = "uploads/tasks";
            tokio::fs::create_dir_all(upload_dir).await.map_err(|_| AppError::Internal)?;
            
            // ✅ SECURITY: Sanitasi filename — hapus path separator
            let sanitized_name = filename.replace('/', "").replace('\\', "").replace("..", "").replace('\0', "");
            let safe_filename = format!("{}-{}", uuid::Uuid::new_v4(), sanitized_name);
            let full_path = format!("{}/{}", upload_dir, safe_filename);
            
            tokio::fs::write(&full_path, data).await.map_err(|_| AppError::Internal)?;
            file_path = full_path;
        }
    }

    if file_path.is_empty() {
        return Err(AppError::BadRequest("File tidak ditemukan dalam request".into()));
    }

    sqlx::query("UPDATE tasks SET submission_file = ?, status = 'under_review' WHERE id = ?")
        .bind(&file_path)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::OK)
}

// ==================== REVIEW TASK (Supervisor/Admin) ====================
#[derive(serde::Deserialize)]
pub struct ReviewTaskRequest {
    pub status: String, // 'completed', 'rejected', 'revision'
    pub feedback: Option<String>,
}

pub async fn review_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ReviewTaskRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("⚖️ review_task called for id={} by role={}", id, claims.role);

    if !["supervisor", "admin", "superadmin"].contains(&claims.role.as_str()) {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("UPDATE tasks SET status = ?, feedback = ? WHERE id = ?")
        .bind(&payload.status)
        .bind(&payload.feedback)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::OK)
}

// ==================== CREATE TASK ====================
// ==================== CREATE TASK ====================
pub async fn create_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    tracing::info!("📝 create_task called by user_id={}, role={}", claims.sub, claims.role);

    if !["supervisor", "admin", "superadmin"].contains(&claims.role.as_str()) {
        return Err(AppError::Unauthorized);
    }

    // ✅ Validasi: pastikan intern target ada dan aktif
    let mut intern_id = payload.intern_id.clone();
    let mut intern_exists = sqlx::query("SELECT id, status FROM interns WHERE id = ?")
        .bind(&intern_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    if intern_exists.is_none() {
        // Fallback: if payload.intern_id was actually a user_id, map to intern.id
        tracing::debug!("🔁 Fallback checking intern by user_id={} for task creation", intern_id);
        if let Some(row) = sqlx::query("SELECT id, status FROM interns WHERE user_id = ?")
            .bind(&intern_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?
        {
            intern_id = row.get("id");
            intern_exists = Some(row);
        }
    }

    match intern_exists {
        Some(row) => {
            let status: String = row.get("status");
            if status != "active" {
                return Err(AppError::BadRequest("Pemagang tidak aktif".into()));
            }

            // Note: division-based validation skipped because 'divisi' column may not exist in DB.
        }
        None => return Err(AppError::NotFound("Pemagang tidak ditemukan".into())),
    }

    let new_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO tasks (id, intern_id, supervisor_id, title, description, status, deadline) 
         VALUES (?, ?, ?, ?, ?, 'pending', ?)"
    )
    .bind(&new_id)
    .bind(&intern_id)
    .bind(&claims.sub)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.deadline)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let row = sqlx::query(
        "SELECT id, intern_id, supervisor_id, title, description, status, deadline, submission_file, feedback, created_at 
         FROM tasks WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let task = Task {
        id: row.get("id"),
        intern_id: row.get("intern_id"),
        supervisor_id: row.get("supervisor_id"),
        title: row.get("title"),
        description: row.get("description"),
        status: row.get("status"),
        deadline: row.get("deadline"),
        submission_file: row.try_get::<Option<String>, _>("submission_file").ok().flatten(),
        feedback: row.try_get::<Option<String>, _>("feedback").ok().flatten(),
        created_at: row.get("created_at"),
    };

    Ok((StatusCode::CREATED, Json(task)))
}

// ==================== UPDATE TASK ====================
pub async fn update_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("✏️ update_task called for id={}, by user_id={}", id, claims.sub);

    // ✅ SECURITY: Hanya supervisor/admin/superadmin yang boleh update task
    if !["supervisor", "admin", "superadmin"].contains(&claims.role.as_str()) {
        tracing::warn!("⚠️ Unauthorized update attempt by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    if claims.role == "supervisor" {
        let exists = sqlx::query("SELECT id FROM tasks WHERE id = ? AND supervisor_id = ?")
            .bind(&id)
            .bind(&claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to check task ownership: {:?}", e);
                AppError::Database(e)
            })?;
            
        if exists.is_none() {
            tracing::warn!("⚠️ Supervisor {} tried to update task {} they don't own", claims.sub, id);
            return Err(AppError::Unauthorized);
        }
    }

    tracing::debug!("📦 Updating task {} with payload: {:?}", id, payload);

    sqlx::query(
        "UPDATE tasks SET status = COALESCE(?, status), description = COALESCE(?, description) WHERE id = ?"
    )
    .bind(&payload.status)
    .bind(&payload.description)
    .bind(&id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update task {}: {:?}", id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Task {} updated successfully", id);

    Ok(StatusCode::OK)
}

// ==================== DELETE TASK ====================
pub async fn delete_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("🗑️ delete_task called for id={} by user_id={} role={}", id, claims.sub, claims.role);

    if claims.role == "supervisor" {
        let exists = sqlx::query("SELECT id FROM tasks WHERE id = ? AND supervisor_id = ?")
            .bind(&id)
            .bind(&claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to verify task ownership: {:?}", e);
                AppError::Database(e)
            })?;

        if exists.is_none() {
            tracing::warn!("⚠️ Supervisor {} tried to delete task {} they don't own", claims.sub, id);
            return Err(AppError::Unauthorized);
        }
    } else    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized review attempt by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to delete task {}: {:?}", id, e);
            AppError::Database(e)
        })?;

    tracing::info!("✅ Task {} deleted successfully", id);
    Ok(StatusCode::OK)
}

// ==================== GET INTERNS BY SUPERVISOR/DIVISION ====================
pub async fn get_assignable_interns(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    tracing::info!("👥 get_assignable_interns called by user_id={}, role={}", claims.sub, claims.role);

    let interns = if claims.role == "superadmin" {
        // Superadmin bisa lihat semua pemagang aktif
        sqlx::query(
            "SELECT i.id, i.nama_lengkap, i.nim, i.university, i.major, i.status, i.division 
             FROM interns i
             WHERE i.status = 'active' 
             ORDER BY i.nama_lengkap ASC"
        )
        .fetch_all(&state.pool)
        .await
    } else if claims.role == "supervisor" || claims.role == "admin" {
        // Supervisor hanya lihat pemagang di divisi yang sama atau yang belum punya supervisor
        sqlx::query(
            "SELECT i.id, i.nama_lengkap, i.nim, i.university, i.major, i.status, i.division 
             FROM interns i
             WHERE i.status = 'active' 
             AND (i.supervisor_id IS NULL OR i.supervisor_id = ?)
             ORDER BY i.nama_lengkap ASC"
        )
        .bind(&claims.sub)
        .fetch_all(&state.pool)
        .await
    } else {
        // Intern tidak boleh akses endpoint ini
        return Err(AppError::Unauthorized);
    }
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch assignable interns: {:?}", e);
        AppError::Database(e)
    })?;

    let result: Vec<serde_json::Value> = interns.iter().map(|row| {
        let division_name = row.try_get::<Option<String>, _>("division").ok().flatten();
        serde_json::json!({
            "id": row.get::<String, _>("id"),
            "nama_lengkap": row.get::<String, _>("nama_lengkap"),
            "nim": row.get::<String, _>("nim"),
            "university": row.get::<Option<String>, _>("university"),
            "major": row.get::<Option<String>, _>("major"),
            "divisi": division_name.clone(),
            "division": division_name,
            "status": row.get::<String, _>("status"),
        })
    }).collect();

    Ok(Json(result))
}