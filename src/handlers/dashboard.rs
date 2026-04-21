use axum::{extract::State, Json, Extension, http::StatusCode};
use serde::Serialize;
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use sqlx::Row;

// ==================== RESPONSE STRUCTS ====================
#[derive(Serialize)]
pub struct AdminStats {
    pub total_interns: i64,
    pub active_interns: i64,
    pub completed_interns: i64,
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub pending_logbooks: i64,
    pub total_evaluations: i64,
}

#[derive(Serialize)]
pub struct SupervisorStats {
    pub assigned_interns: i64,
    pub my_tasks: i64,
    pub pending_approvals: i64,
    pub completed_evaluations: i64,
}

#[derive(Serialize)]
pub struct InternStats {
    pub my_tasks: i64,
    pub completed_tasks: i64,
    pub submitted_logbooks: i64,
    pub approved_logbooks: i64,
    pub evaluation_score: Option<i32>,
}

// ==================== ADMIN DASHBOARD ====================
pub async fn admin_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<AdminStats>, AppError> {
    tracing::info!("📊 admin_stats called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized access to admin_stats by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Executing admin stats query");

    // ✅ FIX: Query terpisah untuk tiap tabel + COALESCE untuk handle NULL
    let total_interns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM interns"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let active_interns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM interns WHERE status = 'active'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let completed_interns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM interns WHERE status = 'completed'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let total_tasks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let pending_tasks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE status = 'pending'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let pending_logbooks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM logbooks WHERE status = 'submitted'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let total_evaluations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM evaluations"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    tracing::info!("✅ Admin stats fetched successfully");

    Ok(Json(AdminStats {
        total_interns,
        active_interns,
        completed_interns,
        total_tasks,
        pending_tasks,
        pending_logbooks,
        total_evaluations,
    }))
}

// ==================== SUPERVISOR DASHBOARD ====================
pub async fn supervisor_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SupervisorStats>, AppError> {
    tracing::info!("📊 supervisor_stats called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized access to supervisor_stats by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Executing supervisor stats query");

    // ✅ FIX: Query sederhana dengan proper JOIN
    let stats = sqlx::query(
        r#"SELECT 
            COUNT(DISTINCT i.id) as assigned_interns,
            COUNT(DISTINCT t.id) as my_tasks,
            COUNT(DISTINCT CASE WHEN l.status = 'submitted' THEN l.id END) as pending_approvals,
            COUNT(DISTINCT e.id) as completed_evaluations
        FROM interns i
        LEFT JOIN tasks t ON t.intern_id = i.id AND t.supervisor_id = ?
        LEFT JOIN logbooks l ON l.intern_id = i.id
        LEFT JOIN evaluations e ON e.intern_id = i.id AND e.supervisor_id = ?
        WHERE t.supervisor_id = ? OR e.supervisor_id = ?
        "#
    )
    .bind(&claims.sub)
    .bind(&claims.sub)
    .bind(&claims.sub)
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch supervisor stats: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Supervisor stats fetched successfully");

    Ok(Json(SupervisorStats {
        assigned_interns: stats.get::<Option<i64>, _>("assigned_interns").unwrap_or(0),
        my_tasks: stats.get::<Option<i64>, _>("my_tasks").unwrap_or(0),
        pending_approvals: stats.get::<Option<i64>, _>("pending_approvals").unwrap_or(0),
        completed_evaluations: stats.get::<Option<i64>, _>("completed_evaluations").unwrap_or(0),
    }))
}

// ==================== INTERN DASHBOARD ====================
pub async fn intern_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<InternStats>, AppError> {
    tracing::info!("📊 intern_stats called by user_id={}, role={}", claims.sub, claims.role);

    if claims.role != "intern" {
        tracing::warn!("⚠️ Unauthorized access to intern_stats by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Executing intern stats query for user_id={}", claims.sub);

    let stats = sqlx::query(
        r#"SELECT 
            COUNT(DISTINCT t.id) as my_tasks,
            COUNT(DISTINCT CASE WHEN t.status = 'completed' THEN t.id END) as completed_tasks,
            COUNT(DISTINCT CASE WHEN l.status IN ('submitted', 'approved') THEN l.id END) as submitted_logbooks,
            COUNT(DISTINCT CASE WHEN l.status = 'approved' THEN l.id END) as approved_logbooks,
            e.final_score as evaluation_score
        FROM interns i
        LEFT JOIN tasks t ON t.intern_id = i.id
        LEFT JOIN logbooks l ON l.intern_id = i.id
        LEFT JOIN evaluations e ON e.intern_id = i.id
        WHERE i.user_id = ?
        GROUP BY i.id, e.final_score"#
    )
    .bind(&claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch intern stats: {:?}", e);
        AppError::Database(e)
    })?;

    let result = match stats {
        Some(row) => InternStats {
            my_tasks: row.get::<Option<i64>, _>("my_tasks").unwrap_or(0),
            completed_tasks: row.get::<Option<i64>, _>("completed_tasks").unwrap_or(0),
            submitted_logbooks: row.get::<Option<i64>, _>("submitted_logbooks").unwrap_or(0),
            approved_logbooks: row.get::<Option<i64>, _>("approved_logbooks").unwrap_or(0),
            evaluation_score: row.get::<Option<i32>, _>("evaluation_score"),
        },
        None => {
            tracing::debug!("📭 No stats found for intern {}", claims.sub);
            InternStats {
                my_tasks: 0,
                completed_tasks: 0,
                submitted_logbooks: 0,
                approved_logbooks: 0,
                evaluation_score: None,
            }
        },
    };

    tracing::info!("✅ Intern stats fetched successfully");

    Ok(Json(result))
}