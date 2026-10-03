use dioxus::prelude::*;

use crate::auth::User;
use crate::components::avatar::{Avatar, AvatarFallback};
use crate::components::sidebar::*;
use crate::routes::Route;

#[component]
pub fn AppLayout() -> Element {
    let route = use_route::<Route>();
    let session = use_context::<Signal<Option<User>>>();
    let navigator = use_navigator();
    use_effect(move || {
        if session().is_none() {
            navigator.replace(Route::Login {});
        }
    });
    let Some(user) = session() else {
        return rsx! { main { class: "auth-layout", "Redirecting to sign in..." } };
    };
    let name = user.full_name();
    let initials = user.initials();

    rsx! {
        SidebarProvider {
            Sidebar {
                SidebarContent {
                    SidebarGroup {
                        SidebarMenu {
                            SidebarMenuItem {
                                SidebarMenuButton {
                                    is_active: route == Route::Home {},
                                    as: move |attributes: Vec<Attribute>| rsx! {
                                        Link { to: Route::Home {}, attributes: attributes, "Home" }
                                    },
                                }
                            }
                        }
                    }
                }
                SidebarFooter {
                    SidebarMenu {
                        SidebarMenuItem {
                            SidebarMenuButton {
                                size: SidebarMenuButtonSize::Lg,
                                is_active: route == Route::Profile {},
                                as: move |attributes: Vec<Attribute>| rsx! {
                                    Link { to: Route::Profile {}, attributes: attributes,
                                        Avatar { AvatarFallback { "{initials}" } }
                                        span { "{name}" }
                                    }
                                },
                            }
                        }
                    }
                }
            }
            SidebarInset {
                div { class: "page-stack",
                    header { SidebarTrigger {} }
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
pub fn AuthLayout() -> Element {
    let session = use_context::<Signal<Option<User>>>();
    let navigator = use_navigator();
    use_effect(move || {
        if session().is_some() {
            navigator.replace(Route::Home {});
        }
    });

    rsx! {
        main { class: "auth-layout",
            div { class: "auth-panel", Outlet::<Route> {} }
        }
    }
}
