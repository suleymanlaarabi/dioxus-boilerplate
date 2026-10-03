use dioxus::prelude::*;

use crate::components::avatar::{Avatar, AvatarFallback};
use crate::components::sidebar::*;
use crate::routes::Route;

#[component]
pub fn AppLayout() -> Element {
    let route = use_route::<Route>();

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
                                        Avatar { AvatarFallback { "FL" } }
                                        span { "First Last" }
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
    rsx! {
        main { class: "auth-layout",
            div { class: "auth-panel", Outlet::<Route> {} }
        }
    }
}
