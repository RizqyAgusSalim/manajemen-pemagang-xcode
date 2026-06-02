use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate, NaiveTime};

// ==================== USER & AUTH ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,  // MySQL CHAR(36) -> Rust String
    pub email: String,
    pub role: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    // role dihapus — selalu 'intern' untuk registrasi publik
    pub university: Option<String>,
    pub major: Option<String>, 
    pub division: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,

    #[serde(default)]
    pub nim: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// ==================== INTERN ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Intern {
    pub id: String,
    pub user_id: String,
    pub university: Option<String>,  // Nullable -> Option
    pub major: Option<String>,
    pub divisi: Option<String>,
    pub division: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub nama_lengkap: Option<String>,
    pub nim: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInternRequest {
    pub user_id: String,
    pub university: String,
    pub major: String,
    pub division: Option<String>,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInternRequest {
    pub university: Option<String>,
    pub major: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
}

// ==================== TASK ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub intern_id: String,
    pub supervisor_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub deadline: Option<NaiveDate>,
    pub submission_file: Option<String>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub intern_id: String,
    pub title: String,
    pub description: Option<String>,
    pub deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
    pub description: Option<String>,
}

// ==================== LOGBOOK ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logbook {
    pub id: String,
    pub intern_id: String,
    pub date: NaiveDate,
    pub activity: String,
    pub description: Option<String>,
    pub status: String,
    pub supervisor_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLogbookRequest {
    pub date: String,
    pub activity: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLogbookRequest {
    pub activity: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveLogbookRequest {
    pub status: String,
    pub notes: Option<String>,
}

// ==================== EVALUATION ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evaluation {
    pub id: String,
    pub intern_id: String,
    pub supervisor_id: String,
    pub discipline_score: i32,
    pub performance_score: i32,
    pub attitude_score: i32,
    pub final_score: i32,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvaluationRequest {
    pub intern_id: String,
    pub discipline_score: i32,
    pub performance_score: i32,
    pub attitude_score: i32,
    pub feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvaluationRequest {
    pub discipline_score: i32,
    pub performance_score: i32,
    pub attitude_score: i32,
    pub feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

// ==================== ATTENDANCE / ABSEN ====================
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Attendance {
    pub id: String,
    pub intern_id: String,
    pub date: NaiveDate,
    pub attendance_time: Option<chrono::NaiveTime>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub description: Option<String>,
    pub status: String,
    pub confirmed_by: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ✅ STRUCT BARU: Attendance dengan data intern (untuk supervisor/admin)
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AttendanceWithIntern {
    pub id: String,
    pub intern_id: String,
    pub date: NaiveDate,
    pub attendance_time: Option<chrono::NaiveTime>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub description: Option<String>,
    pub status: String,
    pub confirmed_by: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    
    // ✅ Data tambahan dari tabel interns
    pub intern_name: Option<String>,  // nama_lengkap
    pub intern_nim: Option<String>,   // nim
    pub intern_email: Option<String>, // email dari users
}

#[derive(Debug, Deserialize)]
pub struct CreateAttendanceRequest {
    pub intern_id: String,
    pub date: NaiveDate,
    pub attendance_time: Option<chrono::NaiveTime>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceStatusRequest {
    pub status: String,  // "approved" or "rejected"
}

#[derive(Debug, Deserialize)]
pub struct UpdateEndTimeRequest {
    pub end_time: NaiveTime,
}

// ==================== PROJECT PROPOSALS / PENGAJUAN JUDUL ====================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectProposal {
    pub id: String,
    pub intern_id: String,
    pub judul_project: String,
    pub deskripsi_project: Option<String>,
    pub catatan_mahasiswa: Option<String>,
    pub status: String,
    pub tanggal_pengajuan: NaiveDate,
    pub catatan_reviewer: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectProposalWithIntern {
    pub id: String,
    pub intern_id: String,
    pub judul_project: String,
    pub deskripsi_project: Option<String>,
    pub catatan_mahasiswa: Option<String>,
    pub status: String,
    pub tanggal_pengajuan: NaiveDate,
    pub catatan_reviewer: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Data dari tabel interns
    pub intern_name: Option<String>,
    pub intern_nim: Option<String>,
    pub intern_university: Option<String>,
    pub intern_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectProposalRequest {
    pub judul_project: String,
    pub deskripsi_project: Option<String>,
    pub catatan_mahasiswa: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectProposalRequest {
    pub judul_project: Option<String>,
    pub deskripsi_project: Option<String>,
    pub catatan_mahasiswa: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewProjectProposalRequest {
    pub status: String,  // "approved", "rejected", or "revised"
    pub catatan_reviewer: Option<String>,
}