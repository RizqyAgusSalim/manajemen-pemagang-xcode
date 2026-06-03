use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Evaluation, CreateEvaluationRequest};
use sqlx::Row;
use chrono::{DateTime, Utc};

// ==================== GET EVALUATION ====================
pub async fn get_evaluations(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
) -> Result<Json<Vec<Evaluation>>, AppError> {
    tracing::info!("🔍 get_evaluations called for intern_id={}, by user_id={}", intern_id, claims.sub);

    if claims.role == "intern" {
        let intern_row = sqlx::query("SELECT user_id FROM interns WHERE id = ?")
            .bind(&intern_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to verify intern ownership for {}: {:?}", intern_id, e);
                AppError::Database(e)
            })?;

        let intern_row_unwrapped = intern_row
            .ok_or_else(|| {
                tracing::warn!("⚠️ Intern not found for evaluation lookup {}", intern_id);
                AppError::NotFound("Data pemagang tidak ditemukan".to_string())
            })?;
        
        let user_id: String = intern_row_unwrapped.get("user_id");
        if user_id != claims.sub {
            tracing::warn!("⚠️ Unauthorized access to evaluations for intern {} by user {}", intern_id, claims.sub);
            return Err(AppError::Unauthorized);
        }
    }

    let mut rows = sqlx::query(
        "SELECT id, intern_id, supervisor_id, discipline_score, performance_score, 
                attitude_score, final_score, feedback, created_at 
         FROM evaluations WHERE intern_id = ? ORDER BY created_at DESC"
    )
    .bind(&intern_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch evaluations for intern {}: {:?}", intern_id, e);
        AppError::Database(e)
    })?;

    if rows.is_empty() {
        tracing::debug!("🔎 No evaluations found for intern_id={}, trying fallback user_id search", intern_id);
        let fallback_intern_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM interns WHERE user_id = ?"
        )
        .bind(&intern_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to lookup internship fallback for {}: {:?}", intern_id, e);
            AppError::Database(e)
        })?;

        if let Some(fallback_id) = fallback_intern_id {
            tracing::debug!("🔁 Found fallback intern id {} for user_id {}", fallback_id, intern_id);
            rows = sqlx::query(
                "SELECT id, intern_id, supervisor_id, discipline_score, performance_score, 
                        attitude_score, final_score, feedback, created_at 
                 FROM evaluations WHERE intern_id = ? ORDER BY created_at DESC"
            )
            .bind(&fallback_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch evaluations for fallback intern {}: {:?}", fallback_id, e);
                AppError::Database(e)
            })?;
        }
    }

    let evaluations: Vec<Evaluation> = rows.into_iter().map(|row| Evaluation {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        supervisor_id: row.get::<String, _>("supervisor_id"),
        discipline_score: row.get::<i32, _>("discipline_score"),
        performance_score: row.get::<i32, _>("performance_score"),
        attitude_score: row.get::<i32, _>("attitude_score"),
        final_score: row.get::<i32, _>("final_score"),
        feedback: row.get::<Option<String>, _>("feedback"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }).collect();

    Ok(Json(evaluations))
}

// ==================== UPDATE EVALUATION ====================
pub async fn update_evaluation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(evaluation_id): Path<String>,
    Json(payload): Json<crate::models::UpdateEvaluationRequest>,
) -> Result<Json<Evaluation>, AppError> {
    tracing::info!("✏️ update_evaluation called for eval_id={}, by user_id={}, role={}", evaluation_id, claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized update evaluation by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    // ✅ SECURITY: Validasi range score (0-100)
    if !(0..=100).contains(&payload.discipline_score) ||
       !(0..=100).contains(&payload.performance_score) ||
       !(0..=100).contains(&payload.attitude_score) {
        return Err(AppError::BadRequest("Score harus dalam range 0-100".into()));
    }

    // ✅ SECURITY: Supervisor hanya bisa edit evaluasi miliknya
    if claims.role == "supervisor" {
        let eval_row = sqlx::query("SELECT supervisor_id FROM evaluations WHERE id = ?")
            .bind(&evaluation_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Evaluasi tidak ditemukan".to_string()))?;
        let eval_supervisor: String = eval_row.get("supervisor_id");
        if eval_supervisor != claims.sub {
            return Err(AppError::Unauthorized);
        }
    }

    // Memastikan evaluasi ada sebelum mencoba update
    sqlx::query("SELECT id FROM evaluations WHERE id = ?")
        .bind(&evaluation_id)
        .fetch_optional(&state.pool).await?.ok_or_else(|| {
            AppError::NotFound("Evaluasi tidak ditemukan".to_string())
        })?
        ;

    let final_score = (payload.discipline_score + payload.performance_score + payload.attitude_score) / 3;

    sqlx::query(
        "UPDATE evaluations SET discipline_score = ?, performance_score = ?, attitude_score = ?, final_score = ?, feedback = ? WHERE id = ?"
    )
    .bind(payload.discipline_score)
    .bind(payload.performance_score)
    .bind(payload.attitude_score)
    .bind(final_score)
    .bind(&payload.feedback)
    .bind(&evaluation_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update evaluation {}: {:?}", evaluation_id, e);
        AppError::Database(e)
    })?;

    let row = sqlx::query(
        "SELECT id, intern_id, supervisor_id, discipline_score, performance_score, 
                attitude_score, final_score, feedback, created_at 
         FROM evaluations WHERE id = ?"
    )
    .bind(&evaluation_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch updated evaluation {}: {:?}", evaluation_id, e);
        AppError::Database(e)
    })?;

    let evaluation = Evaluation {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        supervisor_id: row.get::<String, _>("supervisor_id"),
        discipline_score: row.get::<i32, _>("discipline_score"),
        performance_score: row.get::<i32, _>("performance_score"),
        attitude_score: row.get::<i32, _>("attitude_score"),
        final_score: row.get::<i32, _>("final_score"),
        feedback: row.get::<Option<String>, _>("feedback"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    Ok(Json(evaluation))
}

// ==================== CREATE EVALUATION ====================
pub async fn create_evaluation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateEvaluationRequest>,
) -> Result<(StatusCode, Json<Evaluation>), AppError> {
    tracing::info!("📝 create_evaluation called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "superadmin" && claims.role != "admin" {
        tracing::warn!("⚠️ Unauthorized create evaluation by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    // ✅ SECURITY: Validasi range score (0-100)
    if !(0..=100).contains(&payload.discipline_score) ||
       !(0..=100).contains(&payload.performance_score) ||
       !(0..=100).contains(&payload.attitude_score) {
        return Err(AppError::BadRequest("Score harus dalam range 0-100".into()));
    }

    let final_score = (payload.discipline_score + payload.performance_score + payload.attitude_score) / 3;
    let new_id = uuid::Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating evaluation with id={}, final_score={}", new_id, final_score);

    let supervisor_id = if claims.sub == "superadmin_id" {
        None
    } else {
        Some(claims.sub.clone())
    };

    sqlx::query(
        "INSERT INTO evaluations (id, intern_id, supervisor_id, discipline_score, 
         performance_score, attitude_score, final_score, feedback) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&payload.intern_id)
    .bind(&supervisor_id)
    .bind(payload.discipline_score)
    .bind(payload.performance_score)
    .bind(payload.attitude_score)
    .bind(final_score)
    .bind(&payload.feedback)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create evaluation: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Evaluation created successfully: {}", new_id);

    let row = sqlx::query(
        "SELECT id, intern_id, supervisor_id, discipline_score, performance_score, 
                attitude_score, final_score, feedback, created_at 
         FROM evaluations WHERE id = ?"
    )
    .bind(&new_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch created evaluation: {:?}", e);
        AppError::Database(e)
    })?;

    let evaluation = Evaluation {
        id: row.get::<String, _>("id"),
        intern_id: row.get::<String, _>("intern_id"),
        supervisor_id: row.get::<String, _>("supervisor_id"),
        discipline_score: row.get::<i32, _>("discipline_score"),
        performance_score: row.get::<i32, _>("performance_score"),
        attitude_score: row.get::<i32, _>("attitude_score"),
        final_score: row.get::<i32, _>("final_score"),
        feedback: row.get::<Option<String>, _>("feedback"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    };

    Ok((StatusCode::CREATED, Json(evaluation)))
}