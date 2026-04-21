use axum::{Router, routing::{get, post, put, delete}, middleware};
use crate::{
    handlers::{auth, intern, task, logbook, evaluation, dashboard, report, user, profile, attendance},  // ✅ attendance sudah ada
    middleware::auth::auth_middleware, 
    state::AppState
};

pub fn create_router() -> Router<AppState> {
    // === PUBLIC ROUTES (No auth required) ===
    let public = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/forgot-password", post(auth::forgot_password))
        .route("/auth/reset-password", post(auth::reset_password));

    // === PROTECTED ROUTES (Require auth) ===
    let protected = Router::new()
        // === USER MANAGEMENT (Superadmin/Admin only) ===
        .route("/users", get(user::list_users).post(user::create_user))
        .route("/users/:id", get(user::get_user).put(user::update_user).delete(user::delete_user))
        
        // === DASHBOARD STATS ===
        .route("/dashboard/admin", get(dashboard::admin_stats))
        .route("/dashboard/supervisor", get(dashboard::supervisor_stats))
        .route("/dashboard/intern", get(dashboard::intern_stats))
        
        // === EXPORT PDF ===
        .route("/reports/intern/:id/pdf", get(report::generate_intern_report))
        
        // === CRUD: INTERNS ===
        .route("/interns", get(intern::list_interns).post(intern::create_intern))
        .route("/interns/:id", get(intern::get_intern).put(intern::update_intern))
        
        // === CRUD: TASKS ===
        .route("/tasks", get(task::list_tasks).post(task::create_task))
        .route("/tasks/:id", put(task::update_task))
        
        // === CRUD: LOGBOOKS ===
        .route("/logbooks", get(logbook::list_logbooks).post(logbook::create_logbook))
        .route("/logbooks/:id", get(logbook::get_logbook).put(logbook::update_logbook))
        .route("/logbooks/:id/approve", post(logbook::approve_logbook))
        
        // === CRUD: EVALUATIONS ===
        .route("/evaluations/:intern_id", get(evaluation::get_evaluation))
        .route("/evaluations", post(evaluation::create_evaluation))
        
        // === PROFILE (Semua role bisa akses profil sendiri) ===
        .route("/profile", get(profile::get_profile).put(profile::update_profile))
        
        // === ATTENDANCE / ABSEN (Intern: ajukan, Supervisor/Admin: konfirmasi) ✅ TAMBAH INI ===
        .route("/attendances", get(attendance::list_attendances).post(attendance::create_attendance))
        .route("/attendances/:id", get(attendance::get_attendance))
        .route("/attendances/:id/status", put(attendance::update_attendance_status))
        .route("/attendances/:id/end-time", put(attendance::update_end_time))

        // === TEST ENDPOINT ===
        .route("/dashboard", get(|| async { "✅ You are authenticated!" }))
        
        // ✅ Apply auth middleware to all protected routes
        .layer(middleware::from_fn(auth_middleware));

    // === MERGE ROUTES ===
    Router::new()
        .merge(public)
        .merge(protected)
}