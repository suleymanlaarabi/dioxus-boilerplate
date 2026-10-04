use dioxus::fullstack::{HttpError, ServerFnError, StatusCode};
use sqlx::{Executor, PgPool, Postgres};

use super::auth::{internal_error, token_hash};

pub enum Operation {
    Login,
    Register,
    ResetPassword,
    ChangePassword,
    ChangeEmail,
}

impl Operation {
    fn scope(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Register => "register",
            Self::ResetPassword => "reset",
            Self::ChangePassword => "password",
            Self::ChangeEmail => "email-change",
        }
    }
}

pub async fn global(db: &PgPool) -> Result<(), ServerFnError> {
    check(db, "global", "auth", 100, 60.0).await
}

pub async fn sensitive(
    db: &PgPool,
    operation: Operation,
    subject: &str,
) -> Result<(), ServerFnError> {
    check(db, operation.scope(), subject, 10, 900.0).await
}

pub async fn email<'a>(
    db: impl Executor<'a, Database = Postgres>,
    address: &str,
) -> Result<(), ServerFnError> {
    check(db, "email", address, 3, 900.0).await
}

async fn check<'a>(
    db: impl Executor<'a, Database = Postgres>,
    scope: &str,
    subject: &str,
    limit: i32,
    seconds: f64,
) -> Result<(), ServerFnError> {
    let subject = token_hash(subject);
    let result = sqlx::query!(
        "INSERT INTO auth_rate_limits (scope, subject_hash, attempts, expires_at)
         VALUES ($1, $2, 1, now() + make_interval(secs => $3))
         ON CONFLICT (scope, subject_hash) DO UPDATE SET
           attempts = CASE WHEN auth_rate_limits.expires_at <= now() THEN 1 ELSE auth_rate_limits.attempts + 1 END,
           expires_at = CASE WHEN auth_rate_limits.expires_at <= now() THEN EXCLUDED.expires_at ELSE auth_rate_limits.expires_at END
         WHERE auth_rate_limits.expires_at <= now() OR auth_rate_limits.attempts < $4
         RETURNING attempts", scope, subject, seconds, limit)
        .fetch_optional(db).await.map_err(internal_error)?;
    if result.is_none() {
        return Err(HttpError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.",
        )
        .into());
    }
    Ok(())
}
