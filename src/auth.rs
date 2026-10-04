use dioxus::{prelude::*, CapturedError};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::server::{
    auth::*,
    rate_limit,
    tokens::{self, TokenKind},
    Services,
};
#[cfg(feature = "server")]
use dioxus::fullstack::{axum::Extension, HeaderMap, HttpError, StatusCode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_verified: bool,
    pub pending_email: Option<String>,
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

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileData {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NameData {
    pub first_name: String,
    pub last_name: String,
}

impl From<&User> for NameData {
    fn from(user: &User) -> Self {
        Self {
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EmailDelivery {
    Sent,
    Failed,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Registration {
    pub user: User,
    pub verification: EmailDelivery,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailChange {
    pub user: User,
    pub delivery: EmailDelivery,
}

pub fn error_message(error: &CapturedError) -> String {
    match error.downcast_ref::<ServerFnError>() {
        Some(ServerFnError::ServerError { message, .. }) => message.clone(),
        _ => "Unable to reach the server. Please try again.".to_string(),
    }
}

#[post("/api/auth/register", services: Extension<Services>, headers: HeaderMap)]
pub async fn register(
    profile: ProfileData,
    password: String,
) -> Result<Registration, ServerFnError> {
    check_origin(&headers)?;
    rate_limit::global(&services.db).await?;
    let profile = normalize_profile(profile)?;
    rate_limit::sensitive(
        &services.db,
        rate_limit::Operation::Register,
        &profile.email,
    )
    .await?;
    rate_limit::email(&services.db, &profile.email).await?;
    validate_password(&password)?;
    let password_hash = hash_password(password).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let user = sqlx::query_as!(User,
        "INSERT INTO users (first_name, last_name, email, password_hash) VALUES ($1, $2, $3, $4)
         RETURNING id, first_name, last_name, email, email_verified_at IS NOT NULL AS \"email_verified!\", pending_email",
        profile.first_name, profile.last_name, profile.email, password_hash)
        .fetch_one(&mut *transaction).await.map_err(account_error)?;
    let verification = tokens::issue(
        &mut transaction,
        user.id,
        &user.email,
        TokenKind::VerifyEmail,
    )
    .await?;
    let session = create_session(&mut transaction, &headers, user.id).await?;
    transaction.commit().await.map_err(internal_error)?;
    set_session_cookie(&session, 7 * 24 * 60 * 60)?;
    let verification = match services
        .mail
        .send(&user.email, TokenKind::VerifyEmail, &verification)
        .await
    {
        Ok(()) => EmailDelivery::Sent,
        Err(_) => EmailDelivery::Failed,
    };
    Ok(Registration { user, verification })
}

#[post("/api/auth/login", services: Extension<Services>, headers: HeaderMap)]
pub async fn login(email: String, password: String) -> Result<User, ServerFnError> {
    check_origin(&headers)?;
    rate_limit::global(&services.db).await?;
    let email = normalize_email(email)?;
    rate_limit::sensitive(&services.db, rate_limit::Operation::Login, &email).await?;
    validate_password_size(&password)?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = sqlx::query_as!(Account,
        "SELECT id, first_name, last_name, email, password_hash, email_verified_at IS NOT NULL AS \"email_verified!\", pending_email
         FROM users WHERE email = $1 FOR UPDATE", email)
        .fetch_optional(&mut *transaction).await.map_err(internal_error)?;
    let Some(account) = account else {
        return Err(invalid_credentials());
    };
    if !verify_password(password, account.password_hash.clone()).await? {
        return Err(invalid_credentials());
    }
    let token = create_session(&mut transaction, &headers, account.id).await?;
    transaction.commit().await.map_err(internal_error)?;
    set_session_cookie(&token, 7 * 24 * 60 * 60)?;
    Ok(account.into_user())
}

#[cfg(feature = "server")]
fn invalid_credentials() -> ServerFnError {
    HttpError::new(StatusCode::UNAUTHORIZED, "Invalid email or password.").into()
}

#[get("/api/auth/me", services: Extension<Services>, headers: HeaderMap)]
pub async fn current_user() -> Result<Option<User>, ServerFnError> {
    if let Some(context) = dioxus::fullstack::FullstackContext::current() {
        context.add_response_header(
            dioxus::fullstack::http::header::CACHE_CONTROL,
            dioxus::fullstack::HeaderValue::from_static("private, no-store"),
        );
    }
    session_user(&services.db, &headers).await
}

#[post("/api/auth/logout", services: Extension<Services>, headers: HeaderMap)]
pub async fn logout() -> Result<(), ServerFnError> {
    check_origin(&headers)?;
    if let Some(token_hash) = session_hash(&headers) {
        sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
            .execute(&services.db)
            .await
            .map_err(internal_error)?;
    }
    set_session_cookie("", 0)
}

#[post("/api/profile", services: Extension<Services>, headers: HeaderMap)]
pub async fn update_profile(names: NameData) -> Result<User, ServerFnError> {
    let user = require_user(&services.db, &headers).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = locked_account(&mut transaction, &headers, user.id).await?;
    let profile = normalize_profile(ProfileData {
        first_name: names.first_name,
        last_name: names.last_name,
        email: account.email,
    })?;
    let user = sqlx::query_as!(User,
        "UPDATE users SET first_name = $1, last_name = $2 WHERE id = $3
         RETURNING id, first_name, last_name, email, email_verified_at IS NOT NULL AS \"email_verified!\", pending_email",
        profile.first_name, profile.last_name, user.id).fetch_one(&mut *transaction).await.map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    Ok(user)
}

#[post("/api/auth/resend-verification", services: Extension<Services>, headers: HeaderMap)]
pub async fn resend_verification() -> Result<(), ServerFnError> {
    let user = require_user(&services.db, &headers).await?;
    rate_limit::global(&services.db).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = locked_account(&mut transaction, &headers, user.id).await?;
    if account.email_verified {
        return Ok(());
    }
    rate_limit::email(&mut *transaction, &account.email).await?;
    let token = tokens::issue(
        &mut transaction,
        user.id,
        &account.email,
        TokenKind::VerifyEmail,
    )
    .await?;
    transaction.commit().await.map_err(internal_error)?;
    services
        .mail
        .send(&account.email, TokenKind::VerifyEmail, &token)
        .await
}

#[post("/api/auth/forgot-password", services: Extension<Services>, headers: HeaderMap)]
pub async fn request_password_reset(email: String) -> Result<(), ServerFnError> {
    check_origin(&headers)?;
    rate_limit::global(&services.db).await?;
    let email = normalize_email(email)?;
    rate_limit::email(&services.db, &email).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let user = sqlx::query!("SELECT id FROM users WHERE email = $1 FOR UPDATE", email)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(internal_error)?;
    if let Some(user) = user {
        let token =
            tokens::issue(&mut transaction, user.id, &email, TokenKind::PasswordReset).await?;
        transaction.commit().await.map_err(internal_error)?;
        // The same response is returned for missing accounts and delivery failures.
        let _ = services
            .mail
            .send(&email, TokenKind::PasswordReset, &token)
            .await;
    }
    Ok(())
}

#[post("/api/auth/reset-password", services: Extension<Services>, headers: HeaderMap)]
pub async fn reset_password(token: String, password: String) -> Result<(), ServerFnError> {
    check_origin(&headers)?;
    rate_limit::global(&services.db).await?;
    rate_limit::sensitive(
        &services.db,
        rate_limit::Operation::ResetPassword,
        &token_hash(&token),
    )
    .await?;
    validate_password(&password)?;
    let (mut transaction, record, user) =
        tokens::consume(&services.db, &token, &[TokenKind::PasswordReset.code()]).await?;
    if record.email != user.email {
        return Err(tokens::invalid_token());
    }
    let hash = hash_password(password).await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        hash,
        user.id
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    tokens::invalidate(&mut transaction, user.id).await?;
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user.id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    clear_account_cookie(&services.db, &headers, user.id).await?;
    Ok(())
}

#[post("/api/auth/confirm-email", services: Extension<Services>, headers: HeaderMap)]
pub async fn confirm_email(token: String) -> Result<(), ServerFnError> {
    check_origin(&headers)?;
    rate_limit::global(&services.db).await?;
    let current = session_user(&services.db, &headers).await?;
    let (mut transaction, record, user) = tokens::consume(
        &services.db,
        &token,
        &[TokenKind::VerifyEmail.code(), TokenKind::EmailChange.code()],
    )
    .await?;
    let changing = record.kind == TokenKind::EmailChange.code();
    if changing {
        if user.pending_email.as_deref() != Some(&record.email) {
            return Err(tokens::invalid_token());
        }
        sqlx::query!("UPDATE users SET email = $1, email_verified_at = now(), pending_email = NULL WHERE id = $2", record.email, user.id)
            .execute(&mut *transaction).await.map_err(account_error)?;
        tokens::invalidate(&mut transaction, user.id).await?;
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user.id)
            .execute(&mut *transaction)
            .await
            .map_err(internal_error)?;
    } else {
        if record.email != user.email {
            return Err(tokens::invalid_token());
        }
        sqlx::query!(
            "UPDATE users SET email_verified_at = now() WHERE id = $1",
            user.id
        )
        .execute(&mut *transaction)
        .await
        .map_err(internal_error)?;
    }
    transaction.commit().await.map_err(internal_error)?;
    if changing && current.is_some_and(|current| current.id == user.id) {
        set_session_cookie("", 0)?;
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn clear_account_cookie(
    db: &sqlx::PgPool,
    headers: &HeaderMap,
    user_id: i64,
) -> Result<(), ServerFnError> {
    // Expired sessions are already absent; do not clear a different account's cookie.
    if let Some(hash) = session_hash(headers) {
        let current =
            sqlx::query_scalar!("SELECT user_id FROM sessions WHERE token_hash = $1", hash)
                .fetch_optional(db)
                .await
                .map_err(internal_error)?;
        if current.is_none() || current == Some(user_id) {
            set_session_cookie("", 0)?;
        }
    }
    Ok(())
}

#[post("/api/auth/change-password", services: Extension<Services>, headers: HeaderMap)]
pub async fn change_password(
    current_password: String,
    password: String,
) -> Result<User, ServerFnError> {
    let user = require_user(&services.db, &headers).await?;
    rate_limit::global(&services.db).await?;
    rate_limit::sensitive(
        &services.db,
        rate_limit::Operation::ChangePassword,
        &user.id.to_string(),
    )
    .await?;
    validate_password(&password)?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = locked_account(&mut transaction, &headers, user.id).await?;
    check_password(&account, current_password).await?;
    let hash = hash_password(password).await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        hash,
        user.id
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    tokens::invalidate(&mut transaction, user.id).await?;
    sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND token_hash IS DISTINCT FROM $2",
        user.id,
        session_hash(&headers)
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    let mut user = account.into_user();
    user.pending_email = None;
    Ok(user)
}

#[post("/api/auth/change-email", services: Extension<Services>, headers: HeaderMap)]
pub async fn request_email_change(
    email: String,
    current_password: String,
) -> Result<EmailChange, ServerFnError> {
    let user = require_user(&services.db, &headers).await?;
    rate_limit::global(&services.db).await?;
    rate_limit::sensitive(
        &services.db,
        rate_limit::Operation::ChangeEmail,
        &user.id.to_string(),
    )
    .await?;
    let email = normalize_email(email)?;
    rate_limit::email(&services.db, &email).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = locked_account(&mut transaction, &headers, user.id).await?;
    check_password(&account, current_password).await?;
    if email == account.email {
        return Err(
            HttpError::new(StatusCode::BAD_REQUEST, "Enter a different email address.").into(),
        );
    }
    sqlx::query!(
        "UPDATE users SET pending_email = $1 WHERE id = $2",
        email,
        user.id
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    let token = tokens::issue(&mut transaction, user.id, &email, TokenKind::EmailChange).await?;
    transaction.commit().await.map_err(internal_error)?;
    let delivery = match services
        .mail
        .send(&email, TokenKind::EmailChange, &token)
        .await
    {
        Ok(()) => EmailDelivery::Sent,
        Err(_) => EmailDelivery::Failed,
    };
    let mut user = account.into_user();
    user.pending_email = Some(email);
    Ok(EmailChange { user, delivery })
}

#[post("/api/auth/cancel-email-change", services: Extension<Services>, headers: HeaderMap)]
pub async fn cancel_email_change() -> Result<User, ServerFnError> {
    let user = require_user(&services.db, &headers).await?;
    let mut transaction = services.db.begin().await.map_err(internal_error)?;
    let account = locked_account(&mut transaction, &headers, user.id).await?;
    sqlx::query!(
        "UPDATE users SET pending_email = NULL WHERE id = $1",
        user.id
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    sqlx::query!(
        "DELETE FROM auth_tokens WHERE user_id = $1 AND kind = $2",
        user.id,
        TokenKind::EmailChange.code()
    )
    .execute(&mut *transaction)
    .await
    .map_err(internal_error)?;
    transaction.commit().await.map_err(internal_error)?;
    let mut user = account.into_user();
    user.pending_email = None;
    Ok(user)
}
