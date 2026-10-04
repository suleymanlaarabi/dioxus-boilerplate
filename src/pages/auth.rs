use dioxus::prelude::*;

use crate::auth::{self, ProfileData};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::input::Input;
use crate::routes::Route;
use crate::session::use_session;
use crate::ui::{FormField, ProfileFields};

#[derive(Clone, Copy, PartialEq)]
enum AuthMode {
    Login,
    Register,
}

#[component]
pub fn Login() -> Element {
    rsx! { AuthForm { mode: AuthMode::Login } }
}

#[component]
pub fn Register() -> Element {
    rsx! { AuthForm { mode: AuthMode::Register } }
}

#[component]
fn AuthForm(mode: AuthMode) -> Element {
    let navigator = use_navigator();
    let session = use_session();
    let mut profile = use_signal(ProfileData::default);
    let mut password = use_signal(String::new);
    let mut submit = use_action(move |profile: ProfileData, secret: String| async move {
        let result = match mode {
            AuthMode::Login => auth::login(profile.email, secret).await,
            AuthMode::Register => auth::register(profile, secret).await,
        };
        password.set(String::new());
        session.set_user(Some(result?));
        navigator.replace(Route::Home {});
        Ok::<(), ServerFnError>(())
    });
    let (title, alternate_label, alternate_route) = match mode {
        AuthMode::Login => ("Sign in", "Create an account", Route::Register {}),
        AuthMode::Register => (
            "Create an account",
            "Already have an account? Sign in",
            Route::Login {},
        ),
    };

    rsx! {
        Card {
            CardHeader { CardTitle { "{title}" } }
            CardContent {
                form {
                    class: "form-stack",
                    onsubmit: move |event| {
                        event.prevent_default();
                        if !submit.pending() { submit.call(profile(), password()); }
                    },
                    if mode == AuthMode::Register {
                        ProfileFields { data: profile, disabled: submit.pending() }
                    } else {
                        FormField { id: "email", label: "Email",
                            Input {
                                id: "email", name: "email", r#type: "email", autocomplete: "email", required: true,
                                maxlength: 254, disabled: submit.pending(), value: profile.read().email.clone(),
                                oninput: move |event: FormEvent| profile.write().email = event.value(),
                            }
                        }
                    }
                    FormField { id: "password", label: "Password",
                        Input {
                            id: "password", name: "password", r#type: "password", required: true,
                            autocomplete: if mode == AuthMode::Register { "new-password" } else { "current-password" },
                            minlength: if mode == AuthMode::Register { Some(8) } else { None },
                            maxlength: 1024, disabled: submit.pending(), value: password,
                            oninput: move |event: FormEvent| password.set(event.value()),
                        }
                    }
                    if let Some(Err(error)) = submit.value() {
                        p { role: "alert", "{auth::error_message(&error)}" }
                    }
                    Button {
                        r#type: "submit", disabled: submit.pending(),
                        if submit.pending() { "Please wait..." } else { "{title}" }
                    }
                }
            }
            CardFooter {
                Button {
                    variant: ButtonVariant::Link,
                    r#type: "button", disabled: submit.pending(),
                    onclick: move |_| { navigator.push(alternate_route.clone()); },
                    "{alternate_label}"
                }
            }
        }
    }
}
