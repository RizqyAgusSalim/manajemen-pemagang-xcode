use axum::Router;
use tower_http::{cors::{CorsLayer, Any}, services::ServeDir};
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
    // ✅ Init tracing dengan level debug
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting server initialization...");

    // ✅ Load config
    let config = AppConfig::from_env();
    tracing::info!("📦 Config loaded: PORT={}, DATABASE_URL={}", 
        config.server_port, 
        if config.database_url.contains("root") { "mysql://root@..." } else { "mysql://..." }
    );

    // ✅ Init database
    let pool = init_db(&config).await;
    tracing::info!("✅ Database connected");

    let app_state = AppState::new(pool, config.clone());

    // ✅ CORS LAYER - FIX: Hapus allow_credentials untuk dev
    let cors = CorsLayer::new()
        .allow_origin(Any)  // Izinkan semua origin (untuk dev)
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
        ]);
        // ❌ HAPUS: .allow_credentials(true)  ← Ini yang menyebabkan panic!

    tracing::info!("🔧 CORS configured");

    // ✅ Build router
    let app = Router::new()
        .nest_service("/", ServeDir::new("static").append_index_html_on_directories(true))
        .nest("/api", create_router().with_state(app_state))
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("🚀 Server running at http://{}", addr);
    tracing::info!("📄 Frontend: http://localhost:{}", config.server_port);
    tracing::info!("🔌 API Base: http://localhost:{}/api", config.server_port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🎧 Listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}