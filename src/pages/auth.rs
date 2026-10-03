use dioxus::prelude::*;

use crate::auth::{self, User};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::routes::Route;

#[component]
pub fn Login() -> Element {
    rsx! { AuthForm {} }
}

#[component]
pub fn Register() -> Element {
    rsx! { AuthForm { register: true } }
}

#[component]
fn AuthForm(#[props(default)] register: bool) -> Element {
    let navigator = use_navigator();
    let mut session = use_context::<Signal<Option<User>>>();
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let title = if register {
        "Create an account"
    } else {
        "Sign in"
    };

    rsx! {
        Card {
            CardHeader { CardTitle { "{title}" } }
            CardContent {
                form {
                    class: "form-stack",
                    onsubmit: move |event| {
                        event.prevent_default();
                        if busy() { return; }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            let result = if register {
                                auth::register(first_name(), last_name(), email(), password()).await
                            } else {
                                auth::login(email(), password()).await
                            };
                            password.set(String::new());
                            match result {
                                Ok(user) => {
                                    session.set(Some(user));
                                    navigator.replace(Route::Home {});
                                }
                                Err(failure) => error.set(Some(auth::error_message(failure))),
                            }
                            busy.set(false);
                        });
                    },
                    if register {
                        div { class: "field-stack",
                            Label { html_for: "first-name", "First name" }
                            Input {
                                id: "first-name", name: "first_name", autocomplete: "given-name", required: true,
                                maxlength: 100, disabled: busy(), value: first_name,
                                oninput: move |event: FormEvent| first_name.set(event.value()),
                            }
                        }
                        div { class: "field-stack",
                            Label { html_for: "last-name", "Last name" }
                            Input {
                                id: "last-name", name: "last_name", autocomplete: "family-name", required: true,
                                maxlength: 100, disabled: busy(), value: last_name,
                                oninput: move |event: FormEvent| last_name.set(event.value()),
                            }
                        }
                    }
                    div { class: "field-stack",
                        Label { html_for: "email", "Email" }
                        Input {
                            id: "email", name: "email", r#type: "email", autocomplete: "email", required: true,
                            maxlength: 254, disabled: busy(), value: email,
                            oninput: move |event: FormEvent| email.set(event.value()),
                        }
                    }
                    div { class: "field-stack",
                        Label { html_for: "password", "Password" }
                        Input {
                            id: "password", name: "password", r#type: "password", required: true,
                            autocomplete: if register { "new-password" } else { "current-password" },
                            minlength: 8, maxlength: 1024, disabled: busy(), value: password,
                            oninput: move |event: FormEvent| password.set(event.value()),
                        }
                    }
                    if let Some(message) = error() {
                        p { role: "alert", "{message}" }
                    }
                    Button {
                        r#type: "submit", disabled: busy(),
                        if busy() { "Please wait..." } else { "{title}" }
                    }
                }
            }
            CardFooter {
                if register {
                    Button {
                        variant: ButtonVariant::Link,
                        r#type: "button", disabled: busy(),
                        onclick: move |_| { navigator.push(Route::Login {}); },
                        "Already have an account? Sign in"
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Link,
                        r#type: "button", disabled: busy(),
                        onclick: move |_| { navigator.push(Route::Register {}); },
                        "Create an account"
                    }
                }
            }
        }
    }
}
