use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::{Evaluation, CreateEvaluationRequest};
use sqlx::Row;
use chrono::{DateTime, Utc};

// ==================== GET EVALUATION ====================
pub async fn get_evaluation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
) -> Result<Json<Option<Evaluation>>, AppError> {
    tracing::info!("🔍 get_evaluation called for intern_id={}, by user_id={}", intern_id, claims.sub);

    let row = sqlx::query(
        "SELECT id, intern_id, supervisor_id, discipline_score, performance_score, 
                attitude_score, final_score, feedback, created_at 
         FROM evaluations WHERE intern_id = ?"
    )
    .bind(&intern_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch evaluation for intern {}: {:?}", intern_id, e);
        AppError::Database(e)
    })?;

    let eval = match row {
        Some(row) => {
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

            if claims.role == "intern" && evaluation.intern_id != claims.sub {
                tracing::warn!("⚠️ Unauthorized access to evaluation for intern {} by user {}", intern_id, claims.sub);
                return Err(AppError::Unauthorized);
            }

            Some(evaluation)
        },
        None => {
            tracing::debug!("📭 No evaluation found for intern {}", intern_id);
            None
        },
    };

    Ok(Json(eval))
}

// ==================== CREATE EVALUATION ====================
pub async fn create_evaluation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateEvaluationRequest>,
) -> Result<(StatusCode, Json<Evaluation>), AppError> {
    tracing::info!("📝 create_evaluation called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized attempt to create evaluation");
        return Err(AppError::Unauthorized);
    }

    let final_score = (payload.discipline_score + payload.performance_score + payload.attitude_score) / 3;
    let new_id = uuid::Uuid::new_v4().to_string();
    tracing::debug!("🆕 Creating evaluation with id={}, final_score={}", new_id, final_score);

    sqlx::query(
        "INSERT INTO evaluations (id, intern_id, supervisor_id, discipline_score, 
         performance_score, attitude_score, final_score, feedback) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&payload.intern_id)
    .bind(&claims.sub)
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