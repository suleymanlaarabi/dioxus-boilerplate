use crate::{
    auth::{self, EmailDelivery, NameData, User},
    components::{
        button::{Button, ButtonVariant},
        card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    },
    session::use_session,
    ui::{
        action_error, ActionFeedback, EmailField, NameFields, PasswordField, SubmitButton,
        UserIdentity,
    },
};
use dioxus::prelude::*;

#[component]
pub fn Profile() -> Element {
    let session = use_session();
    rsx! { if let Some(user) = session.user() {
        ProfileDetails { user: user.clone() }
        EmailSettings { user }
        PasswordSettings {}
    } }
}

#[component]
fn ProfileDetails(user: User) -> Element {
    let session = use_session();
    let mut names = use_signal(|| NameData::from(&user));
    let mut save = use_action(move |input: NameData| async move {
        let user = session.check(auth::update_profile(input).await)?;
        names.set(NameData::from(&user));
        session.set_user(Some(user));
        Ok::<(), ServerFnError>(())
    });
    let mut logout = use_action(move || async move {
        session.check(auth::logout().await)?;
        session.set_user(None);
        Ok::<(), ServerFnError>(())
    });
    let busy = save.pending() || logout.pending();
    rsx! { Card {
        CardHeader { CardTitle { "Profile" } CardDescription { "Manage your account details." } }
        CardContent { form { class: "form-stack",
            onsubmit: move |event| { event.prevent_default(); if !save.pending() && !logout.pending() { save.call(names()); } },
            div { class: "profile-row", UserIdentity { user } }
            NameFields { data: names, disabled: busy, onchange: move |_| { save.reset(); } }
            ActionFeedback { error: action_error(save).or_else(|| action_error(logout)),
                success: save.value().is_some_and(|value| value.is_ok()).then(|| "Profile saved.".to_string()) }
            div { class: "form-actions",
                SubmitButton { pending: busy, label: "Save changes" }
                Button { r#type: "button", variant: ButtonVariant::Outline, disabled: busy,
                    onclick: move |_| { if !save.pending() && !logout.pending() { logout.call(); } }, "Sign out" }
            }
        } }
    } }
}

#[component]
fn EmailSettings(user: User) -> Element {
    let session = use_session();
    let email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut request = use_action(move |email: String, secret: String| async move {
        let result = session.check(auth::request_email_change(email, secret).await);
        password.set(String::new());
        let result = result?;
        session.set_user(Some(result.user));
        Ok::<EmailDelivery, ServerFnError>(result.delivery)
    });
    let mut cancel = use_action(move || async move {
        session.set_user(Some(session.check(auth::cancel_email_change().await)?));
        Ok::<(), ServerFnError>(())
    });
    let busy = request.pending() || cancel.pending();
    let delivery = request.value().and_then(Result::ok).map(|value| value());
    rsx! { Card {
        CardHeader { CardTitle { "Email address" } CardDescription { "Current email: {user.email}" } }
        CardContent { form { class: "form-stack",
            onsubmit: move |event| { event.prevent_default(); if !request.pending() && !cancel.pending() { cancel.reset(); request.call(email(), password()); } },
            if let Some(pending) = &user.pending_email {
                p { "Waiting for confirmation: {pending}" }
                Button { r#type: "button", variant: ButtonVariant::Outline, disabled: busy,
                    onclick: move |_| { if !request.pending() && !cancel.pending() { request.reset(); cancel.call(); } }, "Cancel email change" }
                p { "To resend the confirmation, submit the same new email address again." }
            }
            EmailField { value: email, disabled: busy, label: "New email" }
            PasswordField { id: "email-password", label: "Current password", value: password, disabled: busy }
            ActionFeedback {
                error: action_error(request).or_else(|| action_error(cancel)).or_else(||
                    (delivery == Some(EmailDelivery::Failed)).then(|| "Your change is pending, but the email could not be sent. Try sending it again.".to_string())),
                success: (delivery == Some(EmailDelivery::Sent)).then(|| "Check your new email address to confirm the change.".to_string())
            }
            SubmitButton { pending: busy, label: "Send confirmation" }
        } }
    } }
}

#[component]
fn PasswordSettings() -> Element {
    let session = use_session();
    let mut current = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut change = use_action(move |old: String, new: String| async move {
        let result = session.check(auth::change_password(old, new).await);
        current.set(String::new());
        password.set(String::new());
        session.set_user(Some(result?));
        Ok::<(), ServerFnError>(())
    });
    rsx! { Card {
        CardHeader { CardTitle { "Password" } CardDescription { "Changing your password signs out your other sessions." } }
        CardContent { form { class: "form-stack",
            onsubmit: move |event| { event.prevent_default(); if !change.pending() { change.call(current(), password()); } },
            PasswordField { id: "current-password", label: "Current password", value: current, disabled: change.pending() }
            PasswordField { id: "new-password", label: "New password", value: password, disabled: change.pending(), new_password: true }
            ActionFeedback { error: action_error(change),
                success: change.value().is_some_and(|value| value.is_ok()).then(|| "Password changed.".to_string()) }
            SubmitButton { pending: change.pending(), label: "Change password" }
        } }
    } }
}
