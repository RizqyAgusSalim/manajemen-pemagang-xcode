use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use crate::config::AppConfig;

pub async fn init_db(config: &AppConfig) -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(&config.database_url)
        .await
        .expect("❌ Gagal koneksi ke database MySQL")
}