use dioxus::fullstack::{HttpError, ServerFnError, StatusCode};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};

use super::auth::{internal_error, random_token, token_hash};
use crate::auth::User;

#[derive(Clone, Copy)]
pub enum TokenKind {
    VerifyEmail,
    PasswordReset,
    EmailChange,
}

impl TokenKind {
    pub fn code(self) -> i16 {
        match self {
            Self::VerifyEmail => 1,
            Self::PasswordReset => 2,
            Self::EmailChange => 3,
        }
    }
    pub fn path(self) -> &'static str {
        match self {
            Self::PasswordReset => "/reset-password",
            _ => "/verify-email",
        }
    }
    pub fn subject(self) -> &'static str {
        match self {
            Self::VerifyEmail => "Verify your email address",
            Self::PasswordReset => "Reset your password",
            Self::EmailChange => "Confirm your new email address",
        }
    }
}

pub async fn issue(
    db: &mut PgConnection,
    user_id: i64,
    email: &str,
    kind: TokenKind,
) -> Result<String, ServerFnError> {
    let token = random_token().map_err(internal_error)?;
    let hash = token_hash(&token);
    let seconds = match kind {
        TokenKind::PasswordReset => 1800.0,
        _ => 86400.0,
    };
    sqlx::query!(
        "INSERT INTO auth_tokens (token_hash, user_id, kind, email, expires_at)
         VALUES ($1, $2, $3, $4, now() + make_interval(secs => $5))
         ON CONFLICT (user_id, kind) DO UPDATE SET token_hash = EXCLUDED.token_hash,
         email = EXCLUDED.email, expires_at = EXCLUDED.expires_at",
        hash,
        user_id,
        kind.code(),
        email,
        seconds
    )
    .execute(db)
    .await
    .map_err(internal_error)?;
    Ok(token)
}

pub struct TokenRecord {
    pub kind: i16,
    pub email: String,
}

pub async fn consume<'a>(
    db: &'a PgPool,
    token: &str,
    kinds: &[i16],
) -> Result<(Transaction<'a, Postgres>, TokenRecord, User), ServerFnError> {
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid_token());
    }
    let hash = token_hash(token);
    let user_id = sqlx::query_scalar!(
        "SELECT user_id FROM auth_tokens WHERE token_hash = $1 AND expires_at > now() AND kind = ANY($2)", hash, kinds)
        .fetch_optional(db).await.map_err(internal_error)?.ok_or_else(invalid_token)?;
    let mut transaction = db.begin().await.map_err(internal_error)?;
    // Every account mutation locks the user first, including token replacement and consumption.
    let user = sqlx::query_as!(User,
        "SELECT id, first_name, last_name, email, email_verified_at IS NOT NULL AS \"email_verified!\", pending_email
         FROM users WHERE id = $1 FOR UPDATE", user_id)
        .fetch_optional(&mut *transaction).await.map_err(internal_error)?.ok_or_else(invalid_token)?;
    let record = sqlx::query_as!(TokenRecord,
        "DELETE FROM auth_tokens WHERE token_hash = $1 AND expires_at > clock_timestamp() AND kind = ANY($2) RETURNING kind, email",
        hash, kinds).fetch_optional(&mut *transaction).await.map_err(internal_error)?.ok_or_else(invalid_token)?;
    Ok((transaction, record, user))
}

pub async fn invalidate(db: &mut PgConnection, user_id: i64) -> Result<(), ServerFnError> {
    sqlx::query!("DELETE FROM auth_tokens WHERE user_id = $1", user_id)
        .execute(&mut *db)
        .await
        .map_err(internal_error)?;
    sqlx::query!(
        "UPDATE users SET pending_email = NULL WHERE id = $1",
        user_id
    )
    .execute(db)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub fn invalid_token() -> ServerFnError {
    HttpError::new(
        StatusCode::BAD_REQUEST,
        "This link is invalid or has expired. Request a new one.",
    )
    .into()
}
