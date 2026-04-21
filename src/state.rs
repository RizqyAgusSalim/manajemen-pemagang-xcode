use sqlx::mysql::MySqlPool;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(pool: MySqlPool, config: AppConfig) -> Self {
        Self { pool, config }
    }
}