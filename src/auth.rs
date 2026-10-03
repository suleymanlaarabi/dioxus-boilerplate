use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::server::auth::*;
#[cfg(feature = "server")]
use dioxus::fullstack::{axum::Extension, HeaderMap, HttpError, StatusCode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn initials(&self) -> String {
        format!(
            "{}{}",
            self.first_name.chars().next().unwrap_or_default(),
            self.last_name.chars().next().unwrap_or_default()
        )
        .to_uppercase()
    }
}

pub fn error_message(error: ServerFnError) -> String {
    match error {
        ServerFnError::ServerError { message, .. } => message,
        _ => "Unable to reach the server. Please try again.".to_string(),
    }
}

#[post("/api/auth/register", db: Extension<sqlx::PgPool>, headers: HeaderMap)]
pub async fn register(
    first_name: String,
    last_name: String,
    email: String,
    password: String,
) -> Result<User, ServerFnError> {
    check_origin(&headers)?;
    let (first_name, last_name, email) = profile_fields(first_name, last_name, email)?;
    validate_password(&password)?;
    let password_hash = hash_password(password).await?;
    let mut transaction = db.begin().await.map_err(internal_error)?;
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (first_name, last_name, email, password_hash)
         VALUES ($1, $2, $3, $4) RETURNING id, first_name, last_name, email",
        first_name,
        last_name,
        email,
        password_hash
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(account_error)?;
    let token = create_session(&mut transaction, &headers, user.id).await?;
    transaction.commit().await.map_err(internal_error)?;
    set_session_cookie(&token, 7 * 24 * 60 * 60)?;
    Ok(user)
}

#[post("/api/auth/login", db: Extension<sqlx::PgPool>, headers: HeaderMap)]
pub async fn login(email: String, password: String) -> Result<User, ServerFnError> {
    check_origin(&headers)?;
    let email = normalize_email(email)?;
    validate_password(&password)?;
    let account = sqlx::query_scalar!("SELECT password_hash FROM users WHERE email = $1", &email)
        .fetch_optional(&*db)
        .await
        .map_err(internal_error)?;
    let Some(password_hash) = account else {
        return Err(HttpError::new(StatusCode::UNAUTHORIZED, "Invalid email or password.").into());
    };
    if !verify_password(password, password_hash).await? {
        return Err(HttpError::new(StatusCode::UNAUTHORIZED, "Invalid email or password.").into());
    }
    let mut transaction = db.begin().await.map_err(internal_error)?;
    let user = sqlx::query_as!(
        User,
        "SELECT id, first_name, last_name, email FROM users WHERE email = $1",
        email
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(internal_error)?;
    let token = create_session(&mut transaction, &headers, user.id).await?;
    transaction.commit().await.map_err(internal_error)?;
    set_session_cookie(&token, 7 * 24 * 60 * 60)?;
    Ok(user)
}

#[get("/api/auth/me", db: Extension<sqlx::PgPool>, headers: HeaderMap)]
pub async fn current_user() -> Result<Option<User>, ServerFnError> {
    if let Some(context) = dioxus::fullstack::FullstackContext::current() {
        context.add_response_header(
            dioxus::fullstack::http::header::CACHE_CONTROL,
            dioxus::fullstack::HeaderValue::from_static("private, no-store"),
        );
    }
    session_user(&db, &headers).await
}

#[post("/api/auth/logout", db: Extension<sqlx::PgPool>, headers: HeaderMap)]
pub async fn logout() -> Result<(), ServerFnError> {
    check_origin(&headers)?;
    if let Some(token_hash) = session_hash(&headers) {
        sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
            .execute(&*db)
            .await
            .map_err(internal_error)?;
    }
    set_session_cookie("", 0)?;
    Ok(())
}

#[post("/api/profile", db: Extension<sqlx::PgPool>, headers: HeaderMap)]
pub async fn update_profile(
    first_name: String,
    last_name: String,
    email: String,
) -> Result<User, ServerFnError> {
    check_origin(&headers)?;
    let user = session_user(&db, &headers)
        .await?
        .ok_or_else(|| HttpError::new(StatusCode::UNAUTHORIZED, "Please sign in again."))?;
    let (first_name, last_name, email) = profile_fields(first_name, last_name, email)?;
    sqlx::query_as!(
        User,
        "UPDATE users SET first_name = $1, last_name = $2, email = $3 WHERE id = $4
         RETURNING id, first_name, last_name, email",
        first_name,
        last_name,
        email,
        user.id
    )
    .fetch_one(&*db)
    .await
    .map_err(account_error)
}
