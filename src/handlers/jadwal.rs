use axum::{extract::{State, Path}, Json, Extension, http::StatusCode};
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use sqlx::Row;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// ==================== STRUCTS ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JadwalMagang {
    pub id: String,
    pub hari: String,
    pub kelompok: String,
    pub shift: String,
    pub keterangan: Option<String>,
    pub jumlah_anggota: i64,
    pub anggota: Vec<AnggotaJadwal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnggotaJadwal {
    pub nama_lengkap: Option<String>,
    pub university: Option<String>,
    pub nim: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJadwalRequest {
    pub hari: String,
    pub shift: String,
    pub kelompok: String,
    pub keterangan: Option<String>,
    pub intern_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJadwalInternRequest {
    pub jadwal_id: String,
    pub intern_id: String,
    pub action: String,  // "add" or "remove"
}

// ==================== LIST JADWAL ====================
pub async fn list_jadwal(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<JadwalMagang>>, AppError> {
    tracing::info!("📅 list_jadwal called");

    let rows = sqlx::query(
        "SELECT id, hari, kelompok, shift, keterangan FROM jadwal_magang 
         ORDER BY 
           CASE hari 
             WHEN 'Senin' THEN 1 WHEN 'Selasa' THEN 2 WHEN 'Rabu' THEN 3 
             WHEN 'Kamis' THEN 4 WHEN 'Jumat' THEN 5 ELSE 6 END,
           CASE shift WHEN 'Pagi' THEN 1 WHEN 'Siang' THEN 2 ELSE 3 END"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let mut jadwals: Vec<JadwalMagang> = Vec::new();

    for row in rows.iter() {
        let jadwal_id: String = row.get("id");

        let anggota_rows = sqlx::query(
            "SELECT i.nama_lengkap, i.university, i.nim 
             FROM interns i
             INNER JOIN intern_jadwal ij ON i.id = ij.intern_id
             WHERE ij.jadwal_id = ?"
        )
        .bind(&jadwal_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let anggota: Vec<AnggotaJadwal> = anggota_rows.iter().map(|r| AnggotaJadwal {
            nama_lengkap: r.get("nama_lengkap"),
            university: r.get("university"),
            nim: r.get("nim"),
        }).collect();

        jadwals.push(JadwalMagang {
            id: jadwal_id,
            hari: row.get("hari"),
            kelompok: row.get("kelompok"),
            shift: row.get("shift"),
            keterangan: row.get("keterangan"),
            jumlah_anggota: anggota.len() as i64,
            anggota,
        });
    }

    tracing::info!("✅ Returning {} jadwals", jadwals.len());
    Ok(Json(jadwals))
}

// ==================== GET JADWAL PER INTERN ====================
pub async fn get_jadwal_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
) -> Result<Json<Vec<String>>, AppError> {
    tracing::info!("📅 get_jadwal_intern called for intern={}", intern_id);

    // ✅ SECURITY: Intern hanya bisa lihat jadwal miliknya sendiri
    if claims.role == "intern" {
        let user_intern_id: Option<String> = sqlx::query_scalar("SELECT id FROM interns WHERE user_id = ?")
            .bind(&claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?;
        if user_intern_id != Some(intern_id.clone()) {
            return Err(AppError::Unauthorized);
        }
    }

    let rows = sqlx::query(
        "SELECT jadwal_id FROM intern_jadwal WHERE intern_id = ?"
    )
    .bind(&intern_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let jadwal_ids: Vec<String> = rows.iter().map(|r| r.get("jadwal_id")).collect();
    tracing::info!("✅ Found {} jadwal_ids", jadwal_ids.len());
    Ok(Json(jadwal_ids))
}

// ==================== CREATE JADWAL ====================
pub async fn create_jadwal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateJadwalRequest>,
) -> Result<(StatusCode, Json<JadwalMagang>), AppError> {
    if claims.role != "superadmin" && claims.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    let jadwal_id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO jadwal_magang (id, hari, shift, kelompok, keterangan) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&jadwal_id)
    .bind(&payload.hari)
    .bind(&payload.shift)
    .bind(&payload.kelompok)
    .bind(&payload.keterangan)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    for intern_id in &payload.intern_ids {
        // ✅ Cek dulu apakah sudah assigned
        let already: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM intern_jadwal WHERE jadwal_id = ? AND intern_id = ?")
            .bind(&jadwal_id)
            .bind(intern_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        if already == 0 {
            // ✅ Generate UUID untuk PRIMARY KEY 'id'
            let pivot_id = Uuid::new_v4().to_string();
            
            // ✅ Insert dengan menyertakan id
            let result = sqlx::query("INSERT INTO intern_jadwal (id, jadwal_id, intern_id) VALUES (?, ?, ?)")
                .bind(&pivot_id)
                .bind(&jadwal_id)
                .bind(intern_id)
                .execute(&state.pool)
                .await
                .map_err(|e| AppError::Database(e))?;
            
            tracing::debug!("📌 Insert intern_jadwal: id={}, rows_affected={}", pivot_id, result.rows_affected());
        } else {
            tracing::debug!("⚠️ Intern {} sudah assigned ke jadwal {}", intern_id, jadwal_id);
        }
    }

    let anggota_rows = sqlx::query(
        "SELECT i.nama_lengkap, i.university, i.nim 
         FROM interns i
         INNER JOIN intern_jadwal ij ON i.id = ij.intern_id
         WHERE ij.jadwal_id = ?
         ORDER BY i.nama_lengkap"
    )
    .bind(&jadwal_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Query anggota gagal untuk jadwal_id {}: {:?}", jadwal_id, e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Jadwal {}: found {} anggota", jadwal_id, anggota_rows.len());

    let anggota: Vec<AnggotaJadwal> = anggota_rows.iter().map(|r| AnggotaJadwal {
        nama_lengkap: r.get("nama_lengkap"),
        university: r.get("university"),
        nim: r.get("nim"),
    }).collect();

    let result = JadwalMagang {
        id: jadwal_id.clone(),
        hari: payload.hari,
        kelompok: payload.kelompok,
        shift: payload.shift,
        keterangan: payload.keterangan,
        jumlah_anggota: anggota.len() as i64,
        anggota,
    };

    Ok((StatusCode::CREATED, Json(result)))
}

// ==================== DELETE JADWAL ====================
pub async fn delete_jadwal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if claims.role != "superadmin" && claims.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("DELETE FROM intern_jadwal WHERE jadwal_id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    sqlx::query("DELETE FROM jadwal_magang WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== ✅ UPDATE JADWAL INTERN - FIXED ====================
pub async fn update_jadwal_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateJadwalInternRequest>,
) -> Result<StatusCode, AppError> {
    if claims.role != "superadmin" && claims.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    tracing::info!("📅 update_jadwal_intern: jadwal_id={}, intern_id={}, action={}", 
                   payload.jadwal_id, payload.intern_id, payload.action);

    match payload.action.as_str() {
        "add" => {
            // ✅ 1. Validasi: cek apakah intern_id valid
            let intern_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM interns WHERE id = ?")
                .bind(&payload.intern_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| AppError::Database(e))?;

            if intern_exists == 0 {
                tracing::warn!("⚠️ Intern tidak ditemukan: {}", payload.intern_id);
                return Err(AppError::BadRequest("Intern tidak ditemukan".into()));
            }

            // ✅ 2. Validasi: cek apakah jadwal_id valid
            let jadwal_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jadwal_magang WHERE id = ?")
                .bind(&payload.jadwal_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| AppError::Database(e))?;

            if jadwal_exists == 0 {
                tracing::warn!("⚠️ Jadwal tidak ditemukan: {}", payload.jadwal_id);
                return Err(AppError::BadRequest("Jadwal tidak ditemukan".into()));
            }

            // ✅ 3. Cek apakah sudah assigned (hindari duplikat)
            let already_assigned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM intern_jadwal WHERE jadwal_id = ? AND intern_id = ?")
                .bind(&payload.jadwal_id)
                .bind(&payload.intern_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| AppError::Database(e))?;

            if already_assigned > 0 {
                tracing::warn!("⚠️ Intern sudah assigned ke jadwal ini: intern={}, jadwal={}", 
                              payload.intern_id, payload.jadwal_id);
                // Sudah ada, anggap sukses (idempotent)
                return Ok(StatusCode::OK);
            }

            // ✅ 4. Generate UUID untuk PRIMARY KEY 'id'
            let pivot_id = Uuid::new_v4().to_string();
            
            // ✅ 5. Insert dengan menyertakan id
            let result = sqlx::query("INSERT INTO intern_jadwal (id, jadwal_id, intern_id) VALUES (?, ?, ?)")
                .bind(&pivot_id)
                .bind(&payload.jadwal_id)
                .bind(&payload.intern_id)
                .execute(&state.pool)
                .await
                .map_err(|e| {
                    tracing::error!("❌ Failed to insert intern_jadwal: {:?}", e);
                    AppError::Database(e)
                })?;
            
            if result.rows_affected() == 0 {
                tracing::error!("❌ INSERT berhasil tapi rows_affected=0 - kemungkinan constraint violation");
                return Err(AppError::BadRequest("Gagal assign: data sudah ada atau constraint violation".into()));
            }
            
            tracing::info!("✅ Inserted: id={}, intern={} ke jadwal={}, rows_affected={}", 
                          pivot_id, payload.intern_id, payload.jadwal_id, result.rows_affected());
        }
        "remove" => {
            let result = sqlx::query("DELETE FROM intern_jadwal WHERE jadwal_id = ? AND intern_id = ?")
                .bind(&payload.jadwal_id)
                .bind(&payload.intern_id)
                .execute(&state.pool)
                .await
                .map_err(|e| {
                    tracing::error!("❌ Failed to delete from intern_jadwal: {:?}", e);
                    AppError::Database(e)
                })?;
            
            tracing::info!("✅ Deleted: intern={} dari jadwal={}, rows_affected={}", 
                          payload.intern_id, payload.jadwal_id, result.rows_affected());
        }
        _ => return Err(AppError::BadRequest("Action must be 'add' or 'remove'".into())),
    }

    Ok(StatusCode::OK)
}

// ==================== LEGACY: set_jadwal_intern (opsional, bisa dihapus) ====================
#[allow(dead_code)]
pub async fn set_jadwal_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
    Json(payload): Json<UpdateJadwalInternRequest>,
) -> Result<StatusCode, AppError> {
    if claims.role != "superadmin" && claims.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    tracing::warn!("⚠️ set_jadwal_intern is deprecated, use update_jadwal_intern instead");
    
    sqlx::query("UPDATE interns SET jadwal_id = ? WHERE id = ?")
        .bind(&payload.jadwal_id)
        .bind(&intern_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    Ok(StatusCode::OK)
}