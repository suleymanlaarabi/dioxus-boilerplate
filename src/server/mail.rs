use std::time::Duration;

use resend_rs::{types::CreateEmailBaseOptions, Resend};
use url::Url;

use super::{
    auth::{internal_error, token_hash},
    tokens::TokenKind,
};
use dioxus::prelude::ServerFnError;

#[derive(Clone)]
pub struct Mailer {
    client: Resend,
    from: String,
    app_url: Url,
}

impl Mailer {
    pub fn new(config: &super::Config) -> Self {
        Self {
            client: Resend::new(&config.resend_key),
            from: config.mail_from.clone(),
            app_url: config.app_url.clone(),
        }
    }

    pub async fn send(
        &self,
        email: &str,
        kind: TokenKind,
        token: &str,
    ) -> Result<(), ServerFnError> {
        let mut link = self.app_url.join(kind.path()).map_err(internal_error)?;
        link.query_pairs_mut().append_pair("token", token);
        let subject = kind.subject();
        let text = format!("{subject}\n\nOpen this link to continue:\n{link}\n\nIf you did not request this, ignore this email.");
        let html = format!("<p>{subject}</p><p><a href=\"{link}\">Continue</a></p><p>If you did not request this, ignore this email.</p>");
        let message = CreateEmailBaseOptions::new(&self.from, [email], subject)
            .with_text(&text)
            .with_html(&html)
            .with_idempotency_key(&token_hash(token));
        tokio::time::timeout(Duration::from_secs(10), self.client.emails.send(message))
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;
        Ok(())
    }
}
