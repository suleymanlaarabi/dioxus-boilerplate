use dioxus::prelude::*;

mod components;
mod layouts;
mod pages;
mod routes;

use routes::Route;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dioxus-base.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
