use axum::{
    extract::{Path, State},
    Extension,
    response::Response,
    http::header,
};
use printpdf::*;
use std::io::Write;
use crate::state::AppState;
use crate::error::AppError;
use crate::middleware::auth::Claims;
use sqlx::Row;
use chrono::NaiveDate;

pub async fn generate_intern_report(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(intern_id): Path<String>,
) -> Result<Response, AppError> {
    tracing::info!("📄 generate_intern_report called for intern_id={}, by user_id={}", intern_id, claims.sub);

    // === RBAC CHECK ===
    if claims.role == "intern" {
        let check = sqlx::query("SELECT user_id FROM interns WHERE id = ?")
            .bind(&intern_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to check intern ownership: {:?}", e);
                AppError::Database(e)
            })?;
            
        if let Some(row) = check {
            let user_id: String = row.get("user_id");
            if user_id != claims.sub {
                tracing::warn!("⚠️ Unauthorized PDF access attempt for intern {} by user {}", intern_id, claims.sub);
                return Err(AppError::Unauthorized);
            }
        } else {
            tracing::error!("❌ Intern {} not found for ownership check", intern_id);
            return Err(AppError::Internal);
        }
    } else if claims.role != "admin" && claims.role != "supervisor" && claims.role != "superadmin" {
        tracing::warn!("⚠️ Unauthorized PDF access attempt by role={}", claims.role);
        return Err(AppError::Unauthorized);
    }

    tracing::debug!("📦 Fetching intern data for PDF generation");

    // === AMBIL DATA DARI DATABASE ===
    let intern = sqlx::query(
        "SELECT i.*, u.full_name, u.email FROM interns i JOIN users u ON u.id = i.user_id WHERE i.id = ?"
    )
    .bind(&intern_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch intern data: {:?}", e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::error!("❌ Intern {} not found for PDF generation", intern_id);
        AppError::Internal
    })?;

    let full_name: String = intern.get("full_name");
    let email: String = intern.get("email");
    let university: Option<String> = intern.get("university");
    let major: Option<String> = intern.get("major");
    let start_date: Option<NaiveDate> = intern.get("start_date");
    let end_date: Option<NaiveDate> = intern.get("end_date");
    let status: String = intern.get("status");

    let logbooks = sqlx::query(
        "SELECT date, activity, description, status FROM logbooks WHERE intern_id = ? ORDER BY date"
    )
    .bind(&intern_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch logbooks for PDF: {:?}", e);
        AppError::Database(e)
    })?;

    let evaluation = sqlx::query(
        "SELECT discipline_score, performance_score, attitude_score, final_score, feedback FROM evaluations WHERE intern_id = ?"
    )
    .bind(&intern_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("❌ Failed to fetch evaluation for PDF: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("✅ Data fetched, generating PDF for intern {}", intern_id);

    // === GENERATE PDF ===
    let (doc, page1, layer1) = PdfDocument::new(
        "Laporan Pemagang",
        Mm(210.0),
        Mm(297.0),
        "Layer 1"
    );

    let page = doc.get_page(page1);
    let mut layer = page.get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| {
        tracing::error!("❌ Failed to add font for PDF: {:?}", e);
        AppError::Internal
    })?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| {
        tracing::error!("❌ Failed to add bold font for PDF: {:?}", e);
        AppError::Internal
    })?;
    let font_italic = doc.add_builtin_font(BuiltinFont::HelveticaOblique).map_err(|e| {
        tracing::error!("❌ Failed to add italic font for PDF: {:?}", e);
        AppError::Internal
    })?;

    let mut y_pos = Mm(280.0);
    let line_height = Mm(5.0);

    // Title
    layer.use_text("LAPORAN KEGIATAN PEMAGANG", 20.0, Mm(50.0), y_pos, &font_bold);
    y_pos -= line_height * 3.0;

    // Info
    layer.use_text(&format!("Nama: {}", full_name), 12.0, Mm(20.0), y_pos, &font);
    y_pos -= line_height;
    layer.use_text(&format!("Email: {}", email), 12.0, Mm(20.0), y_pos, &font);
    y_pos -= line_height;
    
    if let Some(ref u) = university {
        layer.use_text(&format!("Universitas: {}", u), 12.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
    }
    if let Some(ref m) = major {
        layer.use_text(&format!("Jurusan: {}", m), 12.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
    }
    if let (Some(sd), Some(ed)) = (start_date, end_date) {
        layer.use_text(&format!("Periode: {} s/d {}", sd, ed), 12.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
    }
    layer.use_text(&format!("Status: {}", status), 12.0, Mm(20.0), y_pos, &font);
    y_pos -= line_height * 2.0;

    // Logbook
    layer.use_text("RIWAYAT LOGBOOK:", 14.0, Mm(20.0), y_pos, &font_bold);
    y_pos -= line_height * 1.5;

    for log in logbooks.iter().take(10) {
        let date: NaiveDate = log.get("date");
        let activity: String = log.get("activity");
        let status: String = log.get("status");
        
        layer.use_text(&format!("[{}] {} - [{}]", date, activity, status), 10.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
        if y_pos < Mm(30.0) { break; }
    }

    // Evaluation
    y_pos -= line_height;
    layer.use_text("HASIL EVALUASI:", 14.0, Mm(20.0), y_pos, &font_bold);
    y_pos -= line_height * 1.5;

    if let Some(eval) = evaluation {
        let disc: i32 = eval.get("discipline_score");
        let perf: i32 = eval.get("performance_score");
        let att: i32 = eval.get("attitude_score");
        let final_score: i32 = eval.get("final_score");
        let feedback: Option<String> = eval.get("feedback");

        layer.use_text(&format!("Kedisiplinan: {}/100", disc), 10.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
        layer.use_text(&format!("Kinerja: {}/100", perf), 10.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
        layer.use_text(&format!("Sikap: {}/100", att), 10.0, Mm(20.0), y_pos, &font);
        y_pos -= line_height;
        layer.use_text(&format!("Nilai Akhir: {}/100", final_score), 12.0, Mm(20.0), y_pos, &font_bold);
        y_pos -= line_height;
        
        if let Some(fb) = feedback {
            layer.use_text(&format!("Catatan: {}", fb), 10.0, Mm(20.0), y_pos, &font_italic);
        }
    } else {
        layer.use_text("Belum ada evaluasi", 10.0, Mm(20.0), y_pos, &font_italic);
    }

    layer.use_text("Dokumen ini dihasilkan otomatis oleh Sistem Manajemen Pemagang", 8.0, Mm(20.0), Mm(10.0), &font_italic);

    // === SAVE & RETURN ===
    let mut buffer = Vec::new();
    
    {
        let mut writer = std::io::BufWriter::new(&mut buffer);
        doc.save(&mut writer).map_err(|e| {
            tracing::error!("❌ Failed to save PDF to buffer: {:?}", e);
            AppError::Internal
        })?;
        writer.flush().map_err(|e| {
            tracing::error!("❌ Failed to flush PDF buffer: {:?}", e);
            AppError::Internal
        })?;
    }

    tracing::info!("✅ PDF generated successfully for intern {} ({} bytes)", intern_id, buffer.len());

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"laporan_{}.pdf\"", intern_id)
        )
        .body(axum::body::Body::from(buffer))
        .map_err(|e| {
            tracing::error!("❌ Failed to build HTTP response for PDF: {:?}", e);
            AppError::Internal
        })?)
}