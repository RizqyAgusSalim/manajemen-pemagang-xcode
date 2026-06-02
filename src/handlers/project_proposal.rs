use axum::{
    extract::{State, Path},
    Json, Extension, http::StatusCode,
};
use crate::{
    state::AppState,
    error::AppError,
    middleware::auth::Claims,
    models::{ProjectProposal, ProjectProposalWithIntern, CreateProjectProposalRequest, UpdateProjectProposalRequest, ReviewProjectProposalRequest},
};
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;

// ==================== LIST ALL PROJECT PROPOSALS ====================
pub async fn list_project_proposals(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<ProjectProposalWithIntern>>, AppError> {
    tracing::info!("📝 list_project_proposals called by user_id={}", claims.sub);

    // Supervisor/Admin bisa lihat semua, Intern hanya miliknya sendiri
    let query = if claims.role == "superadmin" || claims.role == "admin" || claims.role == "supervisor" {
        "SELECT 
            pp.id, pp.intern_id, pp.judul_project, pp.deskripsi_project, 
            pp.catatan_mahasiswa, pp.status, pp.tanggal_pengajuan, 
            pp.catatan_reviewer, pp.reviewed_by, pp.reviewed_at, 
            pp.created_at, pp.updated_at,
            u.full_name as intern_name, i.nim as intern_nim, 
            i.university as intern_university, u.email as intern_email
         FROM project_proposals pp
         LEFT JOIN interns i ON pp.intern_id = i.id
         LEFT JOIN users u ON i.user_id = u.id
         ORDER BY pp.tanggal_pengajuan DESC"
    } else {
        // Untuk intern, ambil dulu intern_id mereka
        let intern_row = sqlx::query("SELECT id FROM interns WHERE user_id = ?")
            .bind(&claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        if let Some(_) = intern_row {
            "SELECT 
                pp.id, pp.intern_id, pp.judul_project, pp.deskripsi_project, 
                pp.catatan_mahasiswa, pp.status, pp.tanggal_pengajuan, 
                pp.catatan_reviewer, pp.reviewed_by, pp.reviewed_at, 
                pp.created_at, pp.updated_at,
                u.full_name as intern_name, i.nim as intern_nim, 
                i.university as intern_university, u.email as intern_email
             FROM project_proposals pp
             LEFT JOIN interns i ON pp.intern_id = i.id
             LEFT JOIN users u ON i.user_id = u.id
             WHERE pp.intern_id = (SELECT id FROM interns WHERE user_id = ?)
             ORDER BY pp.tanggal_pengajuan DESC"
        } else {
            return Ok(Json(vec![]));
        }
    };

    let rows = if claims.role == "superadmin" || claims.role == "admin" || claims.role == "supervisor" {
        sqlx::query(query)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?
    } else {
        sqlx::query(query)
            .bind(&claims.sub)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?
    };

    let mut proposals = Vec::new();
    for row in rows {
        proposals.push(ProjectProposalWithIntern {
            id: row.try_get("id").map_err(AppError::Database)?,
            intern_id: row.try_get("intern_id").map_err(AppError::Database)?,
            judul_project: row.try_get("judul_project").map_err(AppError::Database)?,
            deskripsi_project: row.try_get("deskripsi_project").map_err(AppError::Database)?,
            catatan_mahasiswa: row.try_get("catatan_mahasiswa").map_err(AppError::Database)?,
            status: row.try_get("status").map_err(AppError::Database)?,
            tanggal_pengajuan: row.try_get("tanggal_pengajuan").map_err(AppError::Database)?,
            catatan_reviewer: row.try_get("catatan_reviewer").map_err(AppError::Database)?,
            reviewed_by: row.try_get("reviewed_by").map_err(AppError::Database)?,
            reviewed_at: row.try_get("reviewed_at").map_err(AppError::Database)?,
            created_at: row.try_get("created_at").map_err(AppError::Database)?,
            updated_at: row.try_get("updated_at").map_err(AppError::Database)?,
            intern_name: row.try_get("intern_name").map_err(AppError::Database)?,
            intern_nim: row.try_get("intern_nim").map_err(AppError::Database)?,
            intern_university: row.try_get("intern_university").map_err(AppError::Database)?,
            intern_email: row.try_get("intern_email").map_err(AppError::Database)?,
        });
    }

    tracing::info!("✅ Retrieved {} project proposals", proposals.len());
    Ok(Json(proposals))
}

// ==================== GET SINGLE PROJECT PROPOSAL ====================
pub async fn get_project_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proposal_id): Path<String>,
) -> Result<Json<ProjectProposalWithIntern>, AppError> {
    tracing::info!("📝 get_project_proposal called for proposal_id={}", proposal_id);

    let row = sqlx::query(
        "SELECT 
            pp.id, pp.intern_id, pp.judul_project, pp.deskripsi_project, 
            pp.catatan_mahasiswa, pp.status, pp.tanggal_pengajuan, 
            pp.catatan_reviewer, pp.reviewed_by, pp.reviewed_at, 
            pp.created_at, pp.updated_at,
            u.full_name as intern_name, i.nim as intern_nim, 
            i.university as intern_university, u.email as intern_email
         FROM project_proposals pp
         LEFT JOIN interns i ON pp.intern_id = i.id
         LEFT JOIN users u ON i.user_id = u.id
         WHERE pp.id = ?"
    )
    .bind(&proposal_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Proposal tidak ditemukan".into()))?;

    // Check permission: Intern hanya bisa lihat miliknya, Supervisor/Admin bisa lihat semua
    let intern_id: String = row.get("intern_id");
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

    let proposal = ProjectProposalWithIntern {
        id: row.try_get("id").map_err(AppError::Database)?,
        intern_id: row.try_get("intern_id").map_err(AppError::Database)?,
        judul_project: row.try_get("judul_project").map_err(AppError::Database)?,
        deskripsi_project: row.try_get("deskripsi_project").map_err(AppError::Database)?,
        catatan_mahasiswa: row.try_get("catatan_mahasiswa").map_err(AppError::Database)?,
        status: row.try_get("status").map_err(AppError::Database)?,
        tanggal_pengajuan: row.try_get("tanggal_pengajuan").map_err(AppError::Database)?,
        catatan_reviewer: row.try_get("catatan_reviewer").map_err(AppError::Database)?,
        reviewed_by: row.try_get("reviewed_by").map_err(AppError::Database)?,
        reviewed_at: row.try_get("reviewed_at").map_err(AppError::Database)?,
        created_at: row.try_get("created_at").map_err(AppError::Database)?,
        updated_at: row.try_get("updated_at").map_err(AppError::Database)?,
        intern_name: row.try_get("intern_name").map_err(AppError::Database)?,
        intern_nim: row.try_get("intern_nim").map_err(AppError::Database)?,
        intern_university: row.try_get("intern_university").map_err(AppError::Database)?,
        intern_email: row.try_get("intern_email").map_err(AppError::Database)?,
    };

    tracing::info!("✅ Retrieved project proposal");
    Ok(Json(proposal))
}

// ==================== CREATE PROJECT PROPOSAL ====================
pub async fn create_project_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateProjectProposalRequest>,
) -> Result<(StatusCode, Json<ProjectProposal>), AppError> {
    tracing::info!("📝 create_project_proposal called by user_id={}", claims.sub);

    // Only interns can create proposals
    if claims.role != "intern" {
        return Err(AppError::Unauthorized);
    }

    // Get intern_id for current user
    let user_intern: Option<(String,)> = sqlx::query_as("SELECT id FROM interns WHERE user_id = ?")
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    let intern_id = user_intern
        .ok_or_else(|| AppError::NotFound("Data pemagang tidak ditemukan".into()))?
        .0;

    // Check if proposal already exists and not rejected/revised
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM project_proposals 
         WHERE intern_id = ? AND status IN ('pending', 'approved')"
    )
    .bind(&intern_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Anda sudah memiliki proposal yang pending atau approved".to_string()));
    }

    let proposal_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO project_proposals 
         (id, intern_id, judul_project, deskripsi_project, catatan_mahasiswa, status, tanggal_pengajuan, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?)"
    )
    .bind(&proposal_id)
    .bind(&intern_id)
    .bind(&payload.judul_project)
    .bind(&payload.deskripsi_project)
    .bind(&payload.catatan_mahasiswa)
    .bind(now.date_naive())
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let proposal = ProjectProposal {
        id: proposal_id,
        intern_id,
        judul_project: payload.judul_project,
        deskripsi_project: payload.deskripsi_project,
        catatan_mahasiswa: payload.catatan_mahasiswa,
        status: "pending".to_string(),
        tanggal_pengajuan: now.date_naive(),
        catatan_reviewer: None,
        reviewed_by: None,
        reviewed_at: None,
        created_at: now,
        updated_at: now,
    };

    tracing::info!("✅ Created project proposal");
    Ok((StatusCode::CREATED, Json(proposal)))
}

// ==================== UPDATE PROJECT PROPOSAL (INTERN) ====================
pub async fn update_project_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proposal_id): Path<String>,
    Json(payload): Json<UpdateProjectProposalRequest>,
) -> Result<Json<ProjectProposal>, AppError> {
    tracing::info!("📝 update_project_proposal called for proposal_id={}", proposal_id);

    // Only interns can update
    if claims.role != "intern" {
        return Err(AppError::Unauthorized);
    }

    // Check ownership and status
    let proposal: (String, String) = sqlx::query_as(
        "SELECT intern_id, status FROM project_proposals WHERE id = ?"
    )
    .bind(&proposal_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Proposal tidak ditemukan".into()))?;

    let intern_id = proposal.0;
    let status = proposal.1;

    // Check if user owns this proposal
    let user_intern_id: String = sqlx::query_scalar("SELECT id FROM interns WHERE user_id = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    if user_intern_id != intern_id {
        return Err(AppError::Unauthorized);
    }

    // Can only update if status is pending or revised
    if status != "pending" && status != "revised" {
        return Err(AppError::BadRequest("Hanya proposal dengan status pending atau revised yang dapat diubah".to_string()));
    }

    let now = Utc::now();
    
    sqlx::query(
        "UPDATE project_proposals 
         SET judul_project = COALESCE(?, judul_project),
             deskripsi_project = COALESCE(?, deskripsi_project),
             catatan_mahasiswa = COALESCE(?, catatan_mahasiswa),
             updated_at = ?
         WHERE id = ?"
    )
    .bind(&payload.judul_project)
    .bind(&payload.deskripsi_project)
    .bind(&payload.catatan_mahasiswa)
    .bind(now)
    .bind(&proposal_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    // Fetch updated proposal
    let row = sqlx::query(
        "SELECT id, intern_id, judul_project, deskripsi_project, catatan_mahasiswa, 
                status, tanggal_pengajuan, catatan_reviewer, reviewed_by, reviewed_at, 
                created_at, updated_at FROM project_proposals WHERE id = ?"
    )
    .bind(&proposal_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let updated_proposal = ProjectProposal {
        id: row.try_get("id").map_err(AppError::Database)?,
        intern_id: row.try_get("intern_id").map_err(AppError::Database)?,
        judul_project: row.try_get("judul_project").map_err(AppError::Database)?,
        deskripsi_project: row.try_get("deskripsi_project").map_err(AppError::Database)?,
        catatan_mahasiswa: row.try_get("catatan_mahasiswa").map_err(AppError::Database)?,
        status: row.try_get("status").map_err(AppError::Database)?,
        tanggal_pengajuan: row.try_get("tanggal_pengajuan").map_err(AppError::Database)?,
        catatan_reviewer: row.try_get("catatan_reviewer").map_err(AppError::Database)?,
        reviewed_by: row.try_get("reviewed_by").map_err(AppError::Database)?,
        reviewed_at: row.try_get("reviewed_at").map_err(AppError::Database)?,
        created_at: row.try_get("created_at").map_err(AppError::Database)?,
        updated_at: row.try_get("updated_at").map_err(AppError::Database)?,
    };

    tracing::info!("✅ Updated project proposal");
    Ok(Json(updated_proposal))
}

// ==================== REVIEW PROJECT PROPOSAL (SUPERVISOR/ADMIN) ====================
pub async fn review_project_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proposal_id): Path<String>,
    Json(payload): Json<ReviewProjectProposalRequest>,
) -> Result<Json<ProjectProposal>, AppError> {
    tracing::info!("📝 review_project_proposal called for proposal_id={}", proposal_id);

    // Only supervisor/admin can review
    if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        return Err(AppError::Unauthorized);
    }

    // Validate status
    if !["approved", "rejected", "revised"].contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest("Status harus approved, rejected, atau revised".to_string()));
    }

    let now = Utc::now();

    sqlx::query(
        "UPDATE project_proposals 
         SET status = ?, catatan_reviewer = ?, reviewed_by = ?, reviewed_at = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(&payload.status)
    .bind(&payload.catatan_reviewer)
    .bind(&claims.sub)
    .bind(now)
    .bind(now)
    .bind(&proposal_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    // Fetch updated proposal
    let row = sqlx::query(
        "SELECT id, intern_id, judul_project, deskripsi_project, catatan_mahasiswa, 
                status, tanggal_pengajuan, catatan_reviewer, reviewed_by, reviewed_at, 
                created_at, updated_at FROM project_proposals WHERE id = ?"
    )
    .bind(&proposal_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let updated_proposal = ProjectProposal {
        id: row.try_get("id").map_err(AppError::Database)?,
        intern_id: row.try_get("intern_id").map_err(AppError::Database)?,
        judul_project: row.try_get("judul_project").map_err(AppError::Database)?,
        deskripsi_project: row.try_get("deskripsi_project").map_err(AppError::Database)?,
        catatan_mahasiswa: row.try_get("catatan_mahasiswa").map_err(AppError::Database)?,
        status: row.try_get("status").map_err(AppError::Database)?,
        tanggal_pengajuan: row.try_get("tanggal_pengajuan").map_err(AppError::Database)?,
        catatan_reviewer: row.try_get("catatan_reviewer").map_err(AppError::Database)?,
        reviewed_by: row.try_get("reviewed_by").map_err(AppError::Database)?,
        reviewed_at: row.try_get("reviewed_at").map_err(AppError::Database)?,
        created_at: row.try_get("created_at").map_err(AppError::Database)?,
        updated_at: row.try_get("updated_at").map_err(AppError::Database)?,
    };

    tracing::info!("✅ Reviewed project proposal with status: {}", payload.status);
    Ok(Json(updated_proposal))
}

// ==================== DELETE PROJECT PROPOSAL ====================
pub async fn delete_project_proposal(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(proposal_id): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("📝 delete_project_proposal called for proposal_id={}", proposal_id);

    // Get proposal
    let proposal: (String, String) = sqlx::query_as(
        "SELECT intern_id, status FROM project_proposals WHERE id = ?"
    )
    .bind(&proposal_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Proposal tidak ditemukan".into()))?;

    let intern_id = proposal.0;
    let status = proposal.1;

    // Permission check: Intern bisa delete miliknya jika pending/revised, Admin bisa delete apapun
    if claims.role == "intern" {
        let user_intern_id: String = sqlx::query_scalar("SELECT id FROM interns WHERE user_id = ?")
            .bind(&claims.sub)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        if user_intern_id != intern_id {
            return Err(AppError::Unauthorized);
        }

        // Intern hanya bisa delete jika pending atau revised
        if status != "pending" && status != "revised" {
            return Err(AppError::BadRequest("Hanya proposal dengan status pending atau revised yang dapat dihapus".to_string()));
        }
    } else if claims.role != "supervisor" && claims.role != "admin" && claims.role != "superadmin" {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("DELETE FROM project_proposals WHERE id = ?")
        .bind(&proposal_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!("✅ Deleted project proposal");
    Ok(StatusCode::NO_CONTENT)
}

// ==================== GET PROJECT PROPOSALS BY INTERN ====================
pub async fn get_project_proposals_by_intern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
) -> Result<Json<Vec<ProjectProposalWithIntern>>, AppError> {
    tracing::info!("📝 get_project_proposals_by_intern called for intern_id={}", intern_id);

    // ✅ SECURITY: Intern hanya bisa lihat proposal miliknya sendiri
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
        "SELECT 
            pp.id, pp.intern_id, pp.judul_project, pp.deskripsi_project, 
            pp.catatan_mahasiswa, pp.status, pp.tanggal_pengajuan, 
            pp.catatan_reviewer, pp.reviewed_by, pp.reviewed_at, 
            pp.created_at, pp.updated_at,
            u.full_name as intern_name, i.nim as intern_nim, 
            i.university as intern_university, u.email as intern_email
         FROM project_proposals pp
         LEFT JOIN interns i ON pp.intern_id = i.id
         LEFT JOIN users u ON i.user_id = u.id
         WHERE pp.intern_id = ?
         ORDER BY pp.tanggal_pengajuan DESC"
    )
    .bind(&intern_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    let mut proposals = Vec::new();
    for row in rows {
        proposals.push(ProjectProposalWithIntern {
            id: row.try_get("id").map_err(AppError::Database)?,
            intern_id: row.try_get("intern_id").map_err(AppError::Database)?,
            judul_project: row.try_get("judul_project").map_err(AppError::Database)?,
            deskripsi_project: row.try_get("deskripsi_project").map_err(AppError::Database)?,
            catatan_mahasiswa: row.try_get("catatan_mahasiswa").map_err(AppError::Database)?,
            status: row.try_get("status").map_err(AppError::Database)?,
            tanggal_pengajuan: row.try_get("tanggal_pengajuan").map_err(AppError::Database)?,
            catatan_reviewer: row.try_get("catatan_reviewer").map_err(AppError::Database)?,
            reviewed_by: row.try_get("reviewed_by").map_err(AppError::Database)?,
            reviewed_at: row.try_get("reviewed_at").map_err(AppError::Database)?,
            created_at: row.try_get("created_at").map_err(AppError::Database)?,
            updated_at: row.try_get("updated_at").map_err(AppError::Database)?,
            intern_name: row.try_get("intern_name").map_err(AppError::Database)?,
            intern_nim: row.try_get("intern_nim").map_err(AppError::Database)?,
            intern_university: row.try_get("intern_university").map_err(AppError::Database)?,
            intern_email: row.try_get("intern_email").map_err(AppError::Database)?,
        });
    }

    tracing::info!("✅ Retrieved {} project proposals for intern", proposals.len());
    Ok(Json(proposals))
}
