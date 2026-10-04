use dioxus::prelude::*;

use crate::auth::{self, ProfileData, User};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::session::use_session;
use crate::ui::{ProfileFields, UserIdentity};

#[component]
pub fn Profile() -> Element {
    let session = use_session();
    rsx! {
        if let Some(user) = session.user() {
            ProfileForm { user }
        }
    }
}

#[component]
fn ProfileForm(user: User) -> Element {
    let session = use_session();
    let mut profile = use_signal(|| ProfileData::from(&user));
    let mut save = use_action(move |input: ProfileData| async move {
        let user = session.check(auth::update_profile(input).await)?;
        profile.set(ProfileData::from(&user));
        session.set_user(Some(user));
        Ok::<(), ServerFnError>(())
    });
    let mut logout = use_action(move || async move {
        session.check(auth::logout().await)?;
        session.set_user(None);
        Ok::<(), ServerFnError>(())
    });
    let busy = save.pending() || logout.pending();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Profile" }
                CardDescription { "Manage your account details." }
            }
            CardContent {
                form {
                    class: "form-stack",
                    onsubmit: move |event| {
                        event.prevent_default();
                        if !save.pending() && !logout.pending() {
                            logout.reset();
                            save.call(profile());
                        }
                    },
                    div { class: "profile-row", UserIdentity { user } }
                    ProfileFields {
                        data: profile, disabled: busy,
                        onchange: move |_| { save.reset(); logout.reset(); },
                    }
                    if let Some(Err(error)) = save.value().or_else(|| logout.value()) {
                        p { role: "alert", "{auth::error_message(&error)}" }
                    }
                    if let Some(Ok(_)) = save.value() {
                        p { role: "status", "Profile saved." }
                    }
                    Button {
                        r#type: "submit", disabled: busy,
                        if save.pending() { "Please wait..." } else { "Save changes" }
                    }
                }
            }
            CardFooter {
                Button {
                    r#type: "button", variant: ButtonVariant::Outline, disabled: busy,
                    onclick: move |_| {
                        if !save.pending() && !logout.pending() {
                            save.reset();
                            logout.call();
                        }
                    },
                    if logout.pending() { "Please wait..." } else { "Sign out" }
                }
            }
        }
    }
}
