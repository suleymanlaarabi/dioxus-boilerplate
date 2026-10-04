pub mod auth;
pub mod files;
pub mod mail;
pub mod rate_limit;
pub mod tokens;

use sqlx::PgPool;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("Set {0} in .env (see .env.example).")]
    Missing(&'static str),
    #[error("Invalid {0} in .env.")]
    Invalid(&'static str),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Files(#[from] files::FileError),
}

pub struct Config {
    pub database_url: String,
    pub app_url: Url,
    pub resend_key: String,
    pub mail_from: String,
    pub storage_endpoint: String,
    pub storage_region: String,
    pub storage_bucket: String,
    pub storage_access_key: String,
    pub storage_secret_key: String,
}

impl Config {
    fn load() -> Result<Self, StartupError> {
        dotenvy::dotenv().ok();
        let app_url =
            Url::parse(&required("APP_URL")?).map_err(|_| StartupError::Invalid("APP_URL"))?;
        if !matches!(app_url.scheme(), "http" | "https")
            || app_url.host_str().is_none()
            || !app_url.username().is_empty()
            || app_url.password().is_some()
        {
            return Err(StartupError::Invalid("APP_URL"));
        }
        let endpoint = env_or("RUSTFS_ENDPOINT", "http://127.0.0.1:9000");
        let url = Url::parse(&endpoint).map_err(|_| StartupError::Invalid("RUSTFS_ENDPOINT"))?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(StartupError::Invalid("RUSTFS_ENDPOINT"));
        }
        let mail_from = required("RESEND_FROM")?;
        if email_address::EmailAddress::parse_with_options(
            &mail_from,
            email_address::Options::default()
                .without_display_text()
                .with_required_tld(),
        )
        .is_err()
        {
            return Err(StartupError::Invalid("RESEND_FROM"));
        }
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            app_url,
            resend_key: required("RESEND")?,
            mail_from,
            storage_endpoint: endpoint,
            storage_region: env_or("RUSTFS_REGION", "us-east-1"),
            storage_bucket: env_or("RUSTFS_BUCKET", "boilerplate-files"),
            storage_access_key: env_or("RUSTFS_ACCESS_KEY", "boilerplate"),
            storage_secret_key: env_or("RUSTFS_SECRET_KEY", "boilerplate-local-storage-secret"),
        })
    }
}

fn required(name: &'static str) -> Result<String, StartupError> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(StartupError::Missing(name))
}
fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[derive(Clone)]
pub struct Services {
    pub db: PgPool,
    pub mail: mail::Mailer,
    pub files: files::FileStore,
}

impl Services {
    pub async fn new() -> Result<Self, StartupError> {
        let config = Config::load()?;
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;
        sqlx::query!("SELECT id, email_verified_at, pending_email FROM users LIMIT 0")
            .fetch_all(&db)
            .await?;
        let files = files::FileStore::new(db.clone(), &config).await?;
        let services = Self {
            db,
            mail: mail::Mailer::new(&config),
            files,
        };
        let background = services.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(error) = background.cleanup().await {
                    dioxus::logger::tracing::error!(%error, "Background cleanup failed");
                }
            }
        });
        Ok(services)
    }

    async fn cleanup(&self) -> Result<(), StartupError> {
        sqlx::query!("DELETE FROM sessions WHERE expires_at <= now()")
            .execute(&self.db)
            .await?;
        sqlx::query!("DELETE FROM auth_tokens WHERE expires_at <= now()")
            .execute(&self.db)
            .await?;
        sqlx::query!("DELETE FROM auth_rate_limits WHERE expires_at <= now()")
            .execute(&self.db)
            .await?;
        self.files.cleanup().await?;
        Ok(())
    }
}
