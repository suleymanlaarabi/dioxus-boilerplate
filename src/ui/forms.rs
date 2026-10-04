use dioxus::prelude::*;

use crate::auth::ProfileData;
use crate::components::input::Input;
use crate::components::label::Label;

#[component]
pub fn FormField(id: String, label: String, children: Element) -> Element {
    rsx! {
        div { class: "field-stack",
            Label { html_for: id, "{label}" }
            {children}
        }
    }
}

#[component]
pub fn ProfileFields(
    mut data: Signal<ProfileData>,
    disabled: ReadSignal<bool>,
    onchange: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        FormField { id: "first-name", label: "First name",
            Input {
                id: "first-name", name: "first_name", autocomplete: "given-name", required: true,
                maxlength: 100, disabled: disabled(), value: data.read().first_name.clone(),
                oninput: move |event: FormEvent| {
                    data.write().first_name = event.value();
                    if let Some(onchange) = onchange { onchange.call(()); }
                },
            }
        }
        FormField { id: "last-name", label: "Last name",
            Input {
                id: "last-name", name: "last_name", autocomplete: "family-name", required: true,
                maxlength: 100, disabled: disabled(), value: data.read().last_name.clone(),
                oninput: move |event: FormEvent| {
                    data.write().last_name = event.value();
                    if let Some(onchange) = onchange { onchange.call(()); }
                },
            }
        }
        FormField { id: "email", label: "Email",
            Input {
                id: "email", name: "email", r#type: "email", autocomplete: "email", required: true,
                maxlength: 254, disabled: disabled(), value: data.read().email.clone(),
                oninput: move |event: FormEvent| {
                    data.write().email = event.value();
                    if let Some(onchange) = onchange { onchange.call(()); }
                },
            }
        }
    }
}
