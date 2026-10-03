use dioxus::prelude::*;

use crate::components::card::{Card, CardDescription, CardHeader, CardTitle};

#[component]
pub fn Home() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Home" }
                CardDescription { "Welcome to your workspace." }
            }
        }
    }
}
