use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub app_host: String,
    pub app_port: u16,
    pub app_cors_origin: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let app_host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let app_port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("APP_PORT must be a valid u16")?;
        let app_cors_origin = std::env::var("APP_CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        Ok(Self {
            database_url,
            app_host,
            app_port,
            app_cors_origin,
        })
    }
}
