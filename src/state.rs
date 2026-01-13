use crate::config::Config;
use crate::utils::cache::Cache;
use axum::extract::FromRef;
use sqlx::PgPool;

/// Shared application state passed to all handlers
/// Contains database connection pool, cache, and configuration
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub cache: Cache,
    pub config: Config,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.db_pool.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(app_state: &AppState) -> Config {
        app_state.config.clone()
    }
}

impl FromRef<AppState> for Cache {
    fn from_ref(app_state: &AppState) -> Cache {
        app_state.cache.clone()
    }
}
