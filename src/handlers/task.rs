use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
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
            "SELECT id, intern_id, supervisor_id, title, description, status, deadline, created_at FROM tasks"
        )
        .fetch_all(&state.pool)
        .await
    } else {
        tracing::debug!("📦 Fetching tasks for intern_id={}", claims.sub);
        sqlx::query(
            "SELECT id, intern_id, supervisor_id, title, description, status, deadline, created_at FROM tasks WHERE intern_id = ?"
        )
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
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }).collect();

    Ok(Json(tasks))
}

// ==================== CREATE TASK ====================
pub async fn create_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    tracing::info!("📝 create_task called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to create task");
        return Err(AppError::Unauthorized);
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating task with id={}", new_id);

    sqlx::query(
        "INSERT INTO tasks (id, intern_id, supervisor_id, title, description, status, deadline) 
         VALUES (?, ?, ?, ?, ?, 'pending', ?)"
    )
    .bind(&new_id)
    .bind(&payload.intern_id)
    .bind(&claims.sub)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.deadline)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create task: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Task created successfully: {}", new_id);

    let row = sqlx::query(
        "SELECT id, intern_id, supervisor_id, title, description, status, deadline, created_at 
         FROM tasks WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created task: {:?}", e);
        AppError::Database(e)
    })?;

    let task = Task {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        supervisor_id: row.get::<String, _>("supervisor_id"),
        title: row.get::<String, _>("title"),
        description: row.get::<Option<String>, _>("description"),
        status: row.get::<String, _>("status"),
        deadline: row.get::<Option<NaiveDate>, _>("deadline"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
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