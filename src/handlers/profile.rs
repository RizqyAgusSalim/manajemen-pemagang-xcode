use axum::{extract::State, Json, Extension};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use sqlx::Row;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub nim: Option<String>,
    pub university: Option<String>,
    pub major: Option<String>,
    pub division: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub password: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub nim: Option<String>,
    pub university: Option<String>,
    pub major: Option<String>,
    pub division: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
}

// ✅ GET /api/profile - Ambil profil user yang login
pub async fn get_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> Result<Json<ProfileResponse>, AppError> {
    tracing::info!("👤 get_profile called by user_id={}", claims.sub);

    let row = sqlx::query(
        "SELECT u.id, u.email, u.role, u.full_name, u.created_at,
            i.nim AS intern_nim,
            i.university,
            i.major,
            i.division,
            i.start_date,
            i.end_date,
            i.status
     FROM users u
     LEFT JOIN interns i ON i.user_id = u.id
     WHERE u.id = ?"
    )
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch profile: {:?}", e);
        AppError::Database(e)
    })?;

    let profile = ProfileResponse {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        nim: row.try_get::<Option<String>, _>("intern_nim").ok().flatten(),
        university: row.try_get::<Option<String>, _>("university").ok().flatten(),
        major: row.try_get::<Option<String>, _>("major").ok().flatten(),
        division: row.try_get::<Option<String>, _>("division").ok().flatten(),
        start_date: row.try_get::<Option<chrono::NaiveDate>, _>("start_date").ok().flatten().map(|d| d.format("%Y-%m-%d").to_string()),
        end_date: row.try_get::<Option<chrono::NaiveDate>, _>("end_date").ok().flatten().map(|d| d.format("%Y-%m-%d").to_string()),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
    };

    Ok(Json(profile))
}

// ✅ PUT /api/profile - Update profil user yang login (hanya diri sendiri)
pub async fn update_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    tracing::info!("✏️ update_profile called by user_id={}", claims.sub);

    sqlx::query(
        "UPDATE users SET 
         full_name = COALESCE(?, full_name),
         email = COALESCE(?, email)
         WHERE id = ?"
    )
    .bind(&payload.full_name)
    .bind(&payload.email)
    .bind(&claims.sub)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to update profile: {:?}", e);
        AppError::Database(e)
    })?;

    if let Some(password) = payload.password.clone() {
        let password = password.trim();
        if !password.is_empty() {
            if password.len() < 8 {
                return Err(AppError::BadRequest("Password harus minimal 8 karakter".into()));
            }
            let hash = crate::services::auth::hash_password(password)?;
            sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
                .bind(&hash)
                .bind(&claims.sub)
                .execute(&state.pool)
                .await
                .map_err(|e| {
                    tracing::error!("❌ Failed to update password: {:?}", e);
                    AppError::Database(e)
                })?;
        }
    }

    if claims.role == "intern" {
        // Fetch current start_date and end_date to prevent editing them if they already exist
        let current_intern: Option<(Option<chrono::NaiveDate>, Option<chrono::NaiveDate>)> = sqlx::query(
            "SELECT start_date, end_date FROM interns WHERE user_id = ?"
        )
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map(|opt_row| {
            opt_row.map(|r| {
                let sd: Option<chrono::NaiveDate> = r.try_get("start_date").ok().flatten();
                let ed: Option<chrono::NaiveDate> = r.try_get("end_date").ok().flatten();
                (sd, ed)
            })
        })
        .unwrap_or(None);

        let (db_start, db_end) = current_intern.unwrap_or((None, None));
        let db_start_str = db_start.map(|d| d.format("%Y-%m-%d").to_string());
        let db_end_str = db_end.map(|d| d.format("%Y-%m-%d").to_string());

        // If database already has start_date/end_date, keep them. Otherwise, use payload's value (or keep existing NULL).
        let final_start_date = if db_start_str.is_some() && !db_start_str.as_ref().unwrap().trim().is_empty() {
            db_start_str
        } else {
            payload.start_date.clone().filter(|s| !s.trim().is_empty())
        };

        let final_end_date = if db_end_str.is_some() && !db_end_str.as_ref().unwrap().trim().is_empty() {
            db_end_str
        } else {
            payload.end_date.clone().filter(|s| !s.trim().is_empty())
        };

        // Division is stored as a string directly
        let division = payload.division.clone();

        let result = sqlx::query(
            "UPDATE interns SET 
               nama_lengkap = COALESCE(?, nama_lengkap),
               nim = COALESCE(?, nim),
               university = COALESCE(?, university),
               major = COALESCE(?, major),
               division = COALESCE(?, division),
               start_date = COALESCE(?, start_date),
               end_date = COALESCE(?, end_date)
               WHERE user_id = ?"
        )
        .bind(&payload.full_name)
        .bind(&payload.nim)
        .bind(&payload.university)
        .bind(&payload.major)
        .bind(&division)
        .bind(&final_start_date)
        .bind(&final_end_date)
        .bind(&claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to update intern profile: {:?}", e);
            AppError::Database(e)
        })?;

        if result.rows_affected() == 0 {
            let user_row = sqlx::query(
                "SELECT full_name, email FROM users WHERE id = ?"
            )
            .bind(&claims.sub)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to fetch user for intern creation: {:?}", e);
                AppError::Database(e)
            })?;

            let name = user_row.get::<String, _>("full_name");
            let email = user_row.get::<String, _>("email");
            let start_date = payload.start_date.clone().unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
            let end_date = payload.end_date.clone().unwrap_or_else(|| (chrono::Local::now() + chrono::Duration::days(180)).format("%Y-%m-%d").to_string());
            let intern_id = uuid::Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO interns (id, user_id, university, major, start_date, end_date, status, nama_lengkap, nim, division)
                 VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?, ?)"
            )
            .bind(&intern_id)
            .bind(&claims.sub)
            .bind(&payload.university.clone().unwrap_or_default())
            .bind(&payload.major.clone().unwrap_or_default())
            .bind(&start_date)
            .bind(&end_date)
            .bind(&name)
            .bind(&payload.nim.clone().unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_string()))
            .bind(&division)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to create missing intern record: {:?}", e);
                AppError::Database(e)
            })?;
        }
    }

    let row = sqlx::query(
        "SELECT u.id, u.email, u.role, u.full_name, u.created_at,
                i.nim AS intern_nim,
                i.university,
                i.major,
                i.division,
                i.start_date,
                i.end_date,
                i.status
         FROM users u
         LEFT JOIN interns i ON i.user_id = u.id
         WHERE u.id = ?"
    )
    .bind(&claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch updated profile: {:?}", e);
        AppError::Database(e)
    })?;

    let profile = ProfileResponse {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        nim: row.try_get::<Option<String>, _>("intern_nim").ok().flatten(),
        university: row.try_get::<Option<String>, _>("university").ok().flatten(),
        major: row.try_get::<Option<String>, _>("major").ok().flatten(),
        division: row.try_get::<Option<String>, _>("division").ok().flatten(),
        start_date: row.try_get::<Option<chrono::NaiveDate>, _>("start_date").ok().flatten().map(|d| d.format("%Y-%m-%d").to_string()),
        end_date: row.try_get::<Option<chrono::NaiveDate>, _>("end_date").ok().flatten().map(|d| d.format("%Y-%m-%d").to_string()),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
    };

    tracing::info!("✅ Profile updated successfully for user {}", claims.sub);
    Ok(Json(profile))
}
