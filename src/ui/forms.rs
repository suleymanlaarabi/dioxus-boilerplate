use dioxus::prelude::*;

use crate::auth::{self, NameData};
use crate::components::{button::Button, input::Input, label::Label};

#[component]
pub fn FormField(id: String, label: String, children: Element) -> Element {
    rsx! { div { class: "field-stack", Label { html_for: id, "{label}" } {children} } }
}

#[component]
pub fn NameFields(
    mut data: Signal<NameData>,
    disabled: ReadSignal<bool>,
    onchange: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        FormField { id: "first-name", label: "First name",
            Input { id: "first-name", name: "first_name", autocomplete: "given-name", required: true,
                maxlength: 100, disabled: disabled(), value: data.read().first_name.clone(),
                oninput: move |event: FormEvent| {
                    data.write().first_name = event.value();
                    if let Some(onchange) = onchange { onchange.call(()); }
                }
            }
        }
        FormField { id: "last-name", label: "Last name",
            Input { id: "last-name", name: "last_name", autocomplete: "family-name", required: true,
                maxlength: 100, disabled: disabled(), value: data.read().last_name.clone(),
                oninput: move |event: FormEvent| {
                    data.write().last_name = event.value();
                    if let Some(onchange) = onchange { onchange.call(()); }
                }
            }
        }
    }
}

#[component]
pub fn EmailField(
    mut value: Signal<String>,
    disabled: ReadSignal<bool>,
    #[props(default = "Email".to_string())] label: String,
) -> Element {
    rsx! { FormField { id: "email", label,
        Input { id: "email", name: "email", r#type: "email", autocomplete: "email", required: true,
            maxlength: 254, disabled: disabled(), value,
            oninput: move |event: FormEvent| value.set(event.value()) }
    } }
}

#[component]
pub fn PasswordField(
    id: String,
    label: String,
    mut value: Signal<String>,
    disabled: ReadSignal<bool>,
    #[props(default)] new_password: bool,
) -> Element {
    rsx! { FormField { id: id.clone(), label,
        Input { id: id.clone(), name: id, r#type: "password", required: true,
            autocomplete: if new_password { "new-password" } else { "current-password" },
            minlength: if new_password { Some(8) } else { None },
            maxlength: 1024, disabled: disabled(), value,
            oninput: move |event: FormEvent| value.set(event.value()) }
    } }
}

#[component]
pub fn SubmitButton(pending: bool, label: String) -> Element {
    rsx! { Button { r#type: "submit", disabled: pending,
        if pending { "Please wait..." } else { "{label}" }
    } }
}

pub fn action_error<I: 'static, T: 'static>(action: Action<I, T>) -> Option<String> {
    action
        .value()
        .and_then(Result::err)
        .map(|error| auth::error_message(&error))
}

#[component]
pub fn ActionFeedback(error: Option<String>, success: Option<String>) -> Element {
    rsx! {
        if let Some(error) = error { p { role: "alert", "{error}" } }
        if let Some(success) = success { p { role: "status", "{success}" } }
    }
}
