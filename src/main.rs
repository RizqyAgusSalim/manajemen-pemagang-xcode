use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod state;
mod models;
mod services;
mod middleware;
mod handlers;
mod routes;

use config::AppConfig;
use db::init_db;
use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() {
    // ✅ Init tracing — default ke info untuk production
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn,hyper=warn,tower=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting server initialization...");

    // ✅ Load config
    let config = AppConfig::from_env();
    tracing::info!("📦 Config loaded: PORT={}, ALLOWED_ORIGIN={}", 
        config.server_port, config.allowed_origin
    );

    // ✅ Init database
    let pool = init_db(&config).await;
    tracing::info!("✅ Database connected");

    // ✅ Auto-Migrate Database
    tracing::info!("🔄 Menjalankan migrasi database otomatis...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => tracing::info!("✅ Migrasi database berhasil! Struktur tabel sudah siap."),
        Err(e) => {
            tracing::error!("❌ Gagal menjalankan migrasi: {:?}", e);
            panic!("Tolong periksa koneksi atau file migrasi Anda.");
        }
    }

    let app_state = AppState::new(pool, config.clone());

    // ✅ CORS LAYER — Restrict origins untuk production
    let cors = CorsLayer::new()
        .allow_origin(
            config.allowed_origin.parse::<axum::http::HeaderValue>()
                .expect("ALLOWED_ORIGIN must be a valid URL")
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ])
        .allow_credentials(true);

    tracing::info!("🔧 CORS configured for origin: {}", config.allowed_origin);

    // ✅ Build router — uploads TIDAK disajikan secara publik
    let app = Router::new()
        .nest_service("/", ServeDir::new("static").append_index_html_on_directories(true))
        .nest("/api", create_router().with_state(app_state))
        .layer(cors);

    let addr = format!("{}:{}", config.bind_address, config.server_port);
    tracing::info!("🚀 Server running at http://{}", addr);
    tracing::info!("📄 Frontend: http://localhost:{}", config.server_port);
    tracing::info!("🔌 API Base: http://localhost:{}/api", config.server_port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🎧 Listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}