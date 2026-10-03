use dioxus::fullstack::AsStatusCode;
use dioxus::prelude::*;

use crate::auth::{self, User};
use crate::components::avatar::{Avatar, AvatarFallback};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::routes::Route;

#[component]
pub fn Profile() -> Element {
    let mut session = use_context::<Signal<Option<User>>>();
    let user = session().expect("Profile is rendered only for authenticated users");
    let navigator = use_navigator();
    let mut first_name = use_signal(|| user.first_name.clone());
    let mut last_name = use_signal(|| user.last_name.clone());
    let mut email = use_signal(|| user.email.clone());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);
    let name = user.full_name();
    let initials = user.initials();

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
                        if busy() { return; }
                        busy.set(true);
                        error.set(None);
                        saved.set(false);
                        spawn(async move {
                            match auth::update_profile(first_name(), last_name(), email()).await {
                                Ok(user) => {
                                    first_name.set(user.first_name.clone());
                                    last_name.set(user.last_name.clone());
                                    email.set(user.email.clone());
                                    session.set(Some(user));
                                    saved.set(true);
                                }
                                Err(failure) => {
                                    if failure.as_status_code() == dioxus::fullstack::StatusCode::UNAUTHORIZED { session.set(None); }
                                    error.set(Some(auth::error_message(failure)));
                                }
                            }
                            busy.set(false);
                        });
                    },
                    div { class: "profile-row",
                        Avatar { AvatarFallback { "{initials}" } }
                        span { "{name}" }
                    }
                    div { class: "field-stack",
                        Label { html_for: "first-name", "First name" }
                        Input {
                            id: "first-name", name: "first_name", autocomplete: "given-name", required: true,
                            maxlength: 100, disabled: busy(), value: first_name,
                            oninput: move |event: FormEvent| { first_name.set(event.value()); saved.set(false); },
                        }
                    }
                    div { class: "field-stack",
                        Label { html_for: "last-name", "Last name" }
                        Input {
                            id: "last-name", name: "last_name", autocomplete: "family-name", required: true,
                            maxlength: 100, disabled: busy(), value: last_name,
                            oninput: move |event: FormEvent| { last_name.set(event.value()); saved.set(false); },
                        }
                    }
                    div { class: "field-stack",
                        Label { html_for: "email", "Email" }
                        Input {
                            id: "email", name: "email", r#type: "email", autocomplete: "email", required: true,
                            maxlength: 254, disabled: busy(), value: email,
                            oninput: move |event: FormEvent| { email.set(event.value()); saved.set(false); },
                        }
                    }
                    if let Some(message) = error() {
                        p { role: "alert", "{message}" }
                    }
                    if saved() {
                        p { role: "status", "Profile saved." }
                    }
                    Button {
                        r#type: "submit", disabled: busy(),
                        if busy() { "Please wait..." } else { "Save changes" }
                    }
                }
            }
            CardFooter {
                Button {
                    r#type: "button", variant: ButtonVariant::Outline, disabled: busy(),
                    onclick: move |_| {
                        if busy() { return; }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            match auth::logout().await {
                                Ok(()) => {
                                    session.set(None);
                                    navigator.replace(Route::Login {});
                                }
                                Err(failure) => error.set(Some(auth::error_message(failure))),
                            }
                            busy.set(false);
                        });
                    },
                    "Sign out"
                }
            }
        }
    }
}
