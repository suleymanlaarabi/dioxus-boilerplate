use dioxus::prelude::*;

mod auth;
mod components;
mod layouts;
mod pages;
mod routes;
#[cfg(feature = "server")]
mod server;

use routes::Route;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        let database = server::database().await?;
        Ok(dioxus::server::router(App).layer(dioxus::server::axum::Extension(database)))
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let session = use_server_future(auth::current_user)?;
    let initial_user = session().unwrap_or(Ok(None))?;
    use_context_provider(|| Signal::new(initial_user));

    rsx! {
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/dioxus-base.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
