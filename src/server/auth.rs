use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use dioxus::fullstack::{
    FullstackContext, HeaderMap, HeaderValue, HttpError, ServerFnError, StatusCode,
};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};

use crate::auth::User;

pub fn internal_error(error: impl std::fmt::Display) -> ServerFnError {
    eprintln!("Authentication error: {error}");
    HttpError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong. Please try again.",
    )
    .into()
}

pub fn account_error(error: sqlx::Error) -> ServerFnError {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
    {
        HttpError::new(
            StatusCode::CONFLICT,
            "An account with this email already exists.",
        )
        .into()
    } else {
        internal_error(error)
    }
}

pub fn normalize_email(email: String) -> Result<String, ServerFnError> {
    let email = email.trim().to_lowercase();
    let valid = email.split_once('@').is_some_and(|(name, domain)| {
        !name.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !domain.contains('@')
    });
    if !valid || email.len() > 254 || email.chars().any(char::is_whitespace) {
        return Err(HttpError::new(StatusCode::BAD_REQUEST, "Enter a valid email address.").into());
    }
    Ok(email)
}

pub fn profile_fields(
    first_name: String,
    last_name: String,
    email: String,
) -> Result<(String, String, String), ServerFnError> {
    let first_name = first_name.trim().to_string();
    let last_name = last_name.trim().to_string();
    if [&first_name, &last_name]
        .iter()
        .any(|name| !(1..=100).contains(&name.chars().count()))
    {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Names must contain 1 to 100 characters.",
        )
        .into());
    }
    Ok((first_name, last_name, normalize_email(email)?))
}

pub fn validate_password(password: &str) -> Result<(), ServerFnError> {
    if password.chars().count() < 8 || password.len() > 1024 {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Use at least 8 characters and at most 1024 bytes for your password.",
        )
        .into());
    }
    Ok(())
}

// Keep CPU-intensive password hashing off the async request threads.
pub async fn hash_password(password: String) -> Result<String, ServerFnError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(internal_error)
    })
    .await
    .map_err(internal_error)?
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, ServerFnError> {
    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash).map_err(internal_error)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok())
    })
    .await
    .map_err(internal_error)?
}

pub fn check_origin(headers: &HeaderMap) -> Result<(), ServerFnError> {
    let site = headers.get("sec-fetch-site");
    let cross_origin = site.is_some_and(|value| value == "cross-site" || value == "same-site");
    // Browser metadata preserves the original origin when the Dioxus dev proxy rewrites Host.
    let same_origin = site.is_some_and(|value| value == "same-origin");
    let invalid_origin = !same_origin
        && headers.get("origin").is_some_and(|origin| {
            let origin = origin.to_str().unwrap_or_default();
            let host = headers.get("host").and_then(|host| host.to_str().ok());
            origin
                .strip_prefix("http://")
                .or_else(|| origin.strip_prefix("https://"))
                != host
        });
    if cross_origin || invalid_origin {
        return Err(HttpError::new(StatusCode::FORBIDDEN, "Invalid request origin.").into());
    }
    Ok(())
}

pub fn session_hash(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get("cookie")?.to_str().ok()?;
    let token = cookie
        .split(';')
        .find_map(|part| part.trim().strip_prefix("session="))?;
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(hex::encode(Sha256::digest(token.as_bytes())))
}

pub async fn session_user(db: &PgPool, headers: &HeaderMap) -> Result<Option<User>, ServerFnError> {
    let Some(token_hash) = session_hash(headers) else {
        return Ok(None);
    };
    sqlx::query_as!(
        User,
        "SELECT users.id, first_name, last_name, email FROM users
         JOIN sessions ON sessions.user_id = users.id
         WHERE sessions.token_hash = $1 AND sessions.expires_at > now()",
        token_hash
    )
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn create_session(
    db: &mut PgConnection,
    headers: &HeaderMap,
    user_id: i64,
) -> Result<String, ServerFnError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(internal_error)?;
    let token = hex::encode(bytes);
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    sqlx::query!(
        "DELETE FROM sessions WHERE expires_at <= now() OR token_hash = $1",
        session_hash(headers)
    )
    .execute(&mut *db)
    .await
    .map_err(internal_error)?;
    sqlx::query!(
        "INSERT INTO sessions (token_hash, user_id) VALUES ($1, $2)",
        token_hash,
        user_id
    )
    .execute(db)
    .await
    .map_err(internal_error)?;
    Ok(token)
}

pub fn set_session_cookie(token: &str, max_age: u32) -> Result<(), ServerFnError> {
    let secure = if std::env::var("SESSION_COOKIE_SECURE").as_deref() == Ok("false") {
        ""
    } else {
        "; Secure"
    };
    let cookie =
        format!("session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}");
    let context =
        FullstackContext::current().ok_or_else(|| internal_error("Missing request context"))?;
    context.add_response_header(
        dioxus::fullstack::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(internal_error)?,
    );
    Ok(())
}
