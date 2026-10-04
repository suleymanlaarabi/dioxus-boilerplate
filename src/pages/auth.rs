use crate::{
    auth::{self, NameData, ProfileData},
    components::{
        button::{Button, ButtonVariant},
        card::{Card, CardContent, CardFooter, CardHeader, CardTitle},
    },
    routes::Route,
    session::use_session,
    ui::{action_error, ActionFeedback, EmailField, NameFields, PasswordField, SubmitButton},
};
use dioxus::prelude::*;

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
    let names = use_signal(NameData::default);
    let email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut submit = use_action(
        move |names: NameData, email: String, secret: String| async move {
            let result = match mode {
                AuthMode::Login => auth::login(email, secret).await,
                AuthMode::Register => auth::register(
                    ProfileData {
                        first_name: names.first_name,
                        last_name: names.last_name,
                        email,
                    },
                    secret,
                )
                .await
                .map(|registration| {
                    session.set_delivery(Some(registration.verification));
                    registration.user
                }),
            };
            password.set(String::new());
            session.set_user(Some(result?));
            navigator.replace(Route::Home {});
            Ok::<(), ServerFnError>(())
        },
    );
    let (title, alternate_label, alternate_route) = match mode {
        AuthMode::Login => ("Sign in", "Create an account", Route::Register {}),
        AuthMode::Register => (
            "Create an account",
            "Already have an account? Sign in",
            Route::Login {},
        ),
    };
    rsx! { Card {
        CardHeader { CardTitle { "{title}" } }
        CardContent { form { class: "form-stack",
            onsubmit: move |event| { event.prevent_default(); if !submit.pending() { submit.call(names(), email(), password()); } },
            if mode == AuthMode::Register { NameFields { data: names, disabled: submit.pending() } }
            EmailField { value: email, disabled: submit.pending() }
            PasswordField { id: "password", label: "Password", value: password, disabled: submit.pending(), new_password: mode == AuthMode::Register }
            ActionFeedback { error: action_error(submit) }
            SubmitButton { pending: submit.pending(), label: title }
        } }
        CardFooter {
            Button { variant: ButtonVariant::Link, r#type: "button", disabled: submit.pending(),
                onclick: move |_| { navigator.push(alternate_route.clone()); }, "{alternate_label}" }
            if mode == AuthMode::Login {
                Button { variant: ButtonVariant::Link, r#type: "button", disabled: submit.pending(),
                    onclick: move |_| { navigator.push(Route::ForgotPassword {}); }, "Forgot password?" }
            }
        }
    } }
}
