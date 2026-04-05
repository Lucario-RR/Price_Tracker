use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub allow_public_admin_bootstrap: bool,
}

impl AppState {
    pub fn new(db: PgPool, allow_public_admin_bootstrap: bool) -> Self {
        Self {
            db,
            allow_public_admin_bootstrap,
        }
    }
}
