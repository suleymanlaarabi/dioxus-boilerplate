use dioxus::prelude::*;

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
                        navigator.push(Route::Home {});
                    },
                    if register {
                        div { class: "field-stack",
                            Label { html_for: "first-name", "First name" }
                            Input { id: "first-name", name: "first_name", autocomplete: "given-name", required: true }
                        }
                        div { class: "field-stack",
                            Label { html_for: "last-name", "Last name" }
                            Input { id: "last-name", name: "last_name", autocomplete: "family-name", required: true }
                        }
                    }
                    div { class: "field-stack",
                        Label { html_for: "email", "Email" }
                        Input { id: "email", name: "email", r#type: "email", autocomplete: "email", required: true }
                    }
                    div { class: "field-stack",
                        Label { html_for: "password", "Password" }
                        Input {
                            id: "password", name: "password", r#type: "password", required: true,
                            autocomplete: if register { "new-password" } else { "current-password" },
                        }
                    }
                    Button { r#type: "submit", "{title}" }
                }
            }
            CardFooter {
                if register {
                    Button {
                        variant: ButtonVariant::Link,
                        onclick: move |_| { navigator.push(Route::Login {}); },
                        "Already have an account? Sign in"
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Link,
                        onclick: move |_| { navigator.push(Route::Register {}); },
                        "Create an account"
                    }
                }
            }
        }
    }
}
