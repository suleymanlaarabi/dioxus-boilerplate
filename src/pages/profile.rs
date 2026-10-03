use dioxus::prelude::*;

use crate::components::avatar::{Avatar, AvatarFallback};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::routes::Route;

#[component]
pub fn Profile() -> Element {
    let navigator = use_navigator();
    rsx! {
        Card {
            CardHeader { CardTitle { "Profile" } }
            CardContent {
                div { class: "profile-row",
                    Avatar { AvatarFallback { "FL" } }
                    span { "First Last" }
                }
            }
            CardFooter {
                Button {
                    variant: ButtonVariant::Link,
                    onclick: move |_| { navigator.push(Route::Login {}); },
                    "Sign in"
                }
            }
        }
    }
}
