use super::{action_error, ActionFeedback};
use crate::{
    auth::{self, EmailDelivery},
    components::{
        button::Button,
        card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    },
    session::use_session,
};
use dioxus::prelude::*;

#[component]
pub fn VerificationNotice() -> Element {
    let session = use_session();
    let mut resend = use_action(move || async move {
        session.check(auth::resend_verification().await)?;
        session.set_delivery(None);
        Ok::<(), ServerFnError>(())
    });
    let unverified = session.user().is_some_and(|user| !user.email_verified);
    rsx! {
        if unverified {
            Card { role: "alert",
                CardHeader {
                    CardTitle { "Email not verified" }
                    CardDescription {
                        if session.delivery() == Some(EmailDelivery::Failed) {
                            "Your account was created, but the verification email could not be sent. Try sending it again."
                        } else {
                            "Please verify your email address. You can continue using your account."
                        }
                    }
                }
                if resend.value().is_some() {
                    CardContent {
                        ActionFeedback { error: action_error(resend),
                            success: resend.value().is_some_and(|value| value.is_ok()).then(|| "Verification email sent.".to_string()) }
                    }
                }
                CardFooter {
                    Button { r#type: "button", disabled: resend.pending(),
                        onclick: move |_| { if !resend.pending() { resend.call(); } },
                        if resend.pending() { "Please wait..." } else { "Resend verification email" }
                    }
                }
            }
        }
    }
}
