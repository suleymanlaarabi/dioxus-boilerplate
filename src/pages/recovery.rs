use crate::{
    auth,
    components::{
        button::{Button, ButtonVariant},
        card::{Card, CardContent, CardFooter, CardHeader, CardTitle},
    },
    routes::Route,
    session::use_session,
    ui::{action_error, ActionFeedback, EmailField, PasswordField, SubmitButton},
};
use dioxus::prelude::*;

#[component]
pub fn ForgotPassword() -> Element {
    let email = use_signal(String::new);
    let mut send = use_action(auth::request_password_reset);
    let navigator = use_navigator();
    rsx! { Card {
        CardHeader { CardTitle { "Forgot password" } }
        CardContent { form { class: "form-stack",
            onsubmit: move |event| { event.prevent_default(); if !send.pending() { send.call(email()); } },
            EmailField { value: email, disabled: send.pending() }
            ActionFeedback { error: action_error(send),
                success: send.value().is_some_and(|value| value.is_ok()).then(|| "If an account exists, a reset link will be sent to that address.".to_string()) }
            SubmitButton { pending: send.pending(), label: "Send reset link" }
        } }
        CardFooter { Button { variant: ButtonVariant::Link, r#type: "button", disabled: send.pending(),
            onclick: move |_| { navigator.push(Route::Login {}); }, "Back to sign in" } }
    } }
}

#[component]
pub fn ResetPassword(token: Option<String>) -> Element {
    let session = use_session();
    let navigator = use_navigator();
    let mut password = use_signal(String::new);
    let mut reset = use_action(move |token: String, secret: String| async move {
        let result = auth::reset_password(token, secret).await;
        password.set(String::new());
        result?;
        session.set_user(auth::current_user().await?);
        Ok::<(), ServerFnError>(())
    });
    let done = reset.value().is_some_and(|value| value.is_ok());
    rsx! { Card {
        CardHeader { CardTitle { "Reset password" } }
        CardContent {
            if token.as_ref().is_none_or(String::is_empty) {
                p { role: "alert", "This link is invalid. Request a new one." }
            } else if done {
                p { role: "status", "Your password has been reset. Sign in with your new password." }
            } else {
                form { class: "form-stack",
                    onsubmit: move |event| { event.prevent_default(); if !reset.pending() { if let Some(token) = token.clone() { reset.call(token, password()); } } },
                    PasswordField { id: "reset-password", label: "New password", value: password, disabled: reset.pending(), new_password: true }
                    ActionFeedback { error: action_error(reset) }
                    SubmitButton { pending: reset.pending(), label: "Reset password" }
                }
            }
        }
        CardFooter {
            Button { variant: ButtonVariant::Link, r#type: "button", disabled: reset.pending(),
                onclick: move |_| { navigator.push(Route::Login {}); }, "Back to sign in" }
            Button { variant: ButtonVariant::Link, r#type: "button", disabled: reset.pending(),
                onclick: move |_| { navigator.push(Route::ForgotPassword {}); }, "Request a new link" }
        }
    } }
}

#[component]
pub fn VerifyEmail(token: Option<String>) -> Element {
    let session = use_session();
    let navigator = use_navigator();
    let mut confirm = use_action(move |token: String| async move {
        auth::confirm_email(token).await?;
        session.set_user(auth::current_user().await?);
        session.set_delivery(None);
        Ok::<(), ServerFnError>(())
    });
    let done = confirm.value().is_some_and(|value| value.is_ok());
    rsx! { Card {
        CardHeader { CardTitle { "Verify email" } }
        CardContent {
            if token.as_ref().is_none_or(String::is_empty) {
                p { role: "alert", "This link is invalid. Request a new verification email from your account." }
            } else if done {
                p { role: "status", "Your email address has been verified." }
            } else {
                div { class: "form-stack",
                    p { "Confirm your email address to finish verification." }
                    ActionFeedback { error: action_error(confirm) }
                    Button { r#type: "button", disabled: confirm.pending(),
                        onclick: move |_| { if !confirm.pending() { if let Some(token) = token.clone() { confirm.call(token); } } },
                        if confirm.pending() { "Please wait..." } else { "Confirm email address" }
                    }
                }
            }
        }
        CardFooter { Button { variant: ButtonVariant::Link, r#type: "button", disabled: confirm.pending(),
            onclick: move |_| { navigator.push(if session.user().is_some() { Route::Home {} } else { Route::Login {} }); },
            if session.user().is_some() { "Back to Home" } else { "Back to sign in" }
        } }
    } }
}
