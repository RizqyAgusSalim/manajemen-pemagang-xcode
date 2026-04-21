use sqlx::mysql::MySqlPool;  // ✅ Ganti dari sqlx::PgPool
use crate::config::AppConfig;

pub async fn init_db(config: &AppConfig) -> MySqlPool {
    MySqlPool::connect(&config.database_url)
        .await
        .expect("❌ Gagal koneksi ke database MySQL")
}