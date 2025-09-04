use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, DatabaseConnection};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub key: Key,
}

/* State for the admin endpoints */

impl AppState {
    pub async fn new() -> Self {
        dotenv().ok();

        let key = if std::env::var("COOKIE_SECRET").is_err() {
            let _key = Key::generate();
            info!(
                "COOKIE_SECRET not set, generated one: {}",
                _key.master()
                    .iter()
                    .map(|c| { c.to_ascii_lowercase().to_string() })
                    .collect::<String>()
            );
            _key
        } else {
            let _key = Key::from(
                std::env::var("COOKIE_SECRET")
                    .expect("COOKIE_SECRET must be set")
                    .as_bytes(),
            );
            info!(
                "COOKIE_SECRET set to: {}",
                _key.master()
                    .iter()
                    .map(|c| { c.to_ascii_lowercase().to_string() })
                    .collect::<String>()
            );
            _key
        };

        let mut connection_opts =
            ConnectOptions::new(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"));

        connection_opts
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Warn)
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(std::time::Duration::from_secs(30))
            .acquire_timeout(std::time::Duration::from_secs(30));

        let db = sea_orm::Database::connect(connection_opts)
            .await
            .expect("Failed to connect to database");

        Self { db, key }
    }
}
