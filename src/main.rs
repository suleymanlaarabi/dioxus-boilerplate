use dioxus::prelude::*;

mod auth;
mod components;
mod layouts;
mod pages;
mod routes;
#[cfg(feature = "server")]
pub mod server;
mod session;
mod ui;

use routes::Route;

fn main() {
    dioxus::logger::initialize_default();
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        let services = server::Services::new().await?;
        Ok(dioxus::server::router(App).layer(dioxus::server::axum::Extension(services)))
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dioxus-base.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        ErrorBoundary {
            handle_error: |_| rsx! {
                main { class: "auth-layout",
                    p { role: "alert", "Unable to load your account. Please reload the page to try again." }
                }
            },
            SuspenseBoundary {
                fallback: |_| rsx! { main { class: "auth-layout", "Loading your account..." } },
                SessionRouter {}
            }
        }
    }
}

#[component]
fn SessionRouter() -> Element {
    let session = use_server_future(auth::current_user)?;
    let initial_user = session.suspend()?.cloned()?;
    rsx! { AppRouter { initial_user } }
}

#[component]
fn AppRouter(initial_user: Option<auth::User>) -> Element {
    use_context_provider(|| session::Session::new(initial_user));
    rsx! { Router::<Route> {} }
}
