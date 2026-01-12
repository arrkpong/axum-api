use crate::config::Config;
use axum::extract::FromRef;
use sqlx::PgPool;

/// Shared application state passed to all handlers
/// Contains database connection pool and configuration
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
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
