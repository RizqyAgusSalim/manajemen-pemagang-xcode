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
    pub role: String,
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
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInternRequest {
    pub user_id: String,
    pub university: String,
    pub major: String,
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