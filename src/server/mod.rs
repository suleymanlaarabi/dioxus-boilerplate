pub mod auth;

pub async fn database() -> Result<sqlx::PgPool, sqlx::Error> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")
        .expect("Set DATABASE_URL in .env (see .env.example) before starting the app");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    // Fail at startup if the schema has not been initialized.
    sqlx::query!("SELECT id FROM users LIMIT 0")
        .fetch_all(&pool)
        .await?;
    Ok(pool)
}
