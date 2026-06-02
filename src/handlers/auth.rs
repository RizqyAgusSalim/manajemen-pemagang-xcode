use axum::{extract::State, Json, http::StatusCode};
use crate::state::AppState;
use crate::models::{RegisterRequest, LoginRequest, User, ForgotPasswordRequest, ResetPasswordRequest};
use crate::services::auth::{hash_password, verify_password, generate_token};
use crate::error::AppError;
use sqlx::Row;
use rand::Rng;

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

// ==================== REGISTER ====================
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    tracing::info!("📝 register called for email={}", payload.email);

    // ✅ Validasi input
    if payload.password.len() < 8 {
        return Err(AppError::BadRequest("Password harus minimal 8 karakter".into()));
    }
    if !payload.email.contains('@') || !payload.email.contains('.') {
        return Err(AppError::BadRequest("Format email tidak valid".into()));
    }
    if payload.full_name.trim().is_empty() {
        return Err(AppError::BadRequest("Nama lengkap tidak boleh kosong".into()));
    }

    let hash = hash_password(&payload.password)?;
    let user_id = uuid::Uuid::new_v4().to_string();
    
    tracing::debug!("🆕 Creating user with id={}", user_id);
    
    // 1️⃣ INSERT ke tabel users
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, role, full_name) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&hash)
    .bind("intern") // ✅ SECURITY: Hardcode role
    .bind(&payload.full_name)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to register user: {:?}", e);
        AppError::Database(e)
    })?;

    // 2️⃣ FETCH user yang baru dibuat
    let row = sqlx::query(
        "SELECT id, email, role, full_name, created_at FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch registered user: {:?}", e);
        AppError::Database(e)
    })?;

    let user = User {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
    };

    // 3️⃣ AUTO-CREATE INTERN RECORD (role = intern)
    let intern_id = uuid::Uuid::new_v4().to_string();
    
    let university = payload.university.clone().unwrap_or_default();
    let major = payload.major.clone().unwrap_or_default();
    let start_date = payload.start_date.filter(|s| !s.is_empty()).unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });
    let end_date = payload.end_date.filter(|s| !s.is_empty()).unwrap_or_else(|| {
        (chrono::Local::now() + chrono::Duration::days(180)).format("%Y-%m-%d").to_string()
    });
    let nama_lengkap = payload.full_name.clone();
    let nim = payload.email.split('@').next().unwrap_or("").to_string();
    let division = payload.division.clone();

    sqlx::query(
        "INSERT INTO interns (id, user_id, university, major, start_date, end_date, status, nama_lengkap, nim, division) 
         VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?, ?)"
    )
    .bind(&intern_id)
    .bind(&user_id)
    .bind(&university)
    .bind(&major)
    .bind(&start_date)
    .bind(&end_date)
    .bind(&nama_lengkap)
    .bind(&nim)
    .bind(&division)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to create intern record: {:?}", e);
        AppError::Database(e)
    })?;
    
    tracing::info!("✅ Intern record created: id={}, user_id={}, division={:?}", intern_id, user_id, division);

    // 4️⃣ Generate token & return response
    let token = generate_token(user.id.clone(), &user.role, &state.config.jwt_secret)?;
    
    tracing::info!("✅ User registered successfully: {}", user_id);

    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

// ==================== LOGIN ====================
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::info!("🔐 login attempt for email={}", payload.email);

    let row = sqlx::query(
        "SELECT id, email, password_hash, role, full_name, created_at FROM users WHERE email = ?"
    )
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch user for login: {:?}", e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::warn!("⚠️ Login failed: user not found for email {}", payload.email);
        AppError::Unauthorized
    })?;

    let password_hash: String = row.get("password_hash");
    if !verify_password(&payload.password, &password_hash)? {
        tracing::warn!("⚠️ Login failed: invalid password for email {}", payload.email);
        return Err(AppError::Unauthorized);
    }

    let user = User {
        id: row.get("id"),
        email: row.get("email"),
        role: row.get("role"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
    };
    
    let token = generate_token(user.id.clone(), &user.role, &state.config.jwt_secret)?;
    
    tracing::info!("✅ Login successful for user_id={}, role={}", user.id, user.role);

    Ok(Json(AuthResponse { token, user }))
}

// ==================== FORGOT PASSWORD ====================
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("🔑 forgot_password requested for email={}", req.email);

    let user = sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to check user for forgot password: {:?}", e);
            AppError::Database(e)
        })?;

    if user.is_none() {
        tracing::debug!("📭 User not found for email {}, returning OK for security", req.email);
        return Ok(StatusCode::OK);
    }

    // Gunakan 8 karakter alfanumerik untuk reset code yang lebih aman
    let code: String = (0..8).map(|_| {
        let idx = rand::thread_rng().gen_range(0..36);
        if idx < 10 { (b'0' + idx) as char } else { (b'A' + idx - 10) as char }
    }).collect();
    let reset_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO password_resets (id, email, code, expires_at)
        VALUES (?, ?, ?, NOW() + INTERVAL 15 MINUTE)"
    )
    .bind(&reset_id)
    .bind(&req.email)
    .bind(&code)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to save password reset code: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("📧 Kode reset telah dikirim untuk email={}", req.email);

    Ok(StatusCode::OK)
}

// ==================== RESET PASSWORD ====================
pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    tracing::info!("🔄 reset_password requested for email={}", req.email);

    let valid = sqlx::query(
        "SELECT id FROM password_resets
        WHERE email = ? AND code = ? AND expires_at > NOW()
        ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&req.email)
    .bind(&req.code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to verify reset code: {:?}", e);
        AppError::Database(e)
    })?
    .is_some();

    if !valid {
        tracing::warn!("⚠️ Reset password failed: invalid or expired code for email {}", req.email);
        return Err(AppError::Unauthorized);
    }

    // ✅ SECURITY: Invalidate code before updating password
    sqlx::query("DELETE FROM password_resets WHERE email = ?")
        .bind(&req.email)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to delete used reset code: {:?}", e);
            AppError::Database(e)
        })?;

    // ✅ SECURITY: Validasi kompleksitas password
    if req.new_password.len() < 8 {
        return Err(AppError::BadRequest("Password baru harus minimal 8 karakter".into()));
    }

    let hash = hash_password(&req.new_password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE email = ?")
        .bind(&hash)
        .bind(&req.email)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("❌ Failed to update password: {:?}", e);
            AppError::Database(e)
        })?;

    tracing::info!("✅ Password reset successful for email={}", req.email);

    Ok(StatusCode::OK)
}