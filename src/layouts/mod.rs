use dioxus::prelude::*;

use crate::components::sidebar::*;
use crate::routes::Route;
use crate::session::use_session;
use crate::ui::UserIdentity;

#[component]
pub fn AppLayout() -> Element {
    let route = use_route::<Route>();
    let session = use_session();
    let navigator = use_navigator();
    use_effect(move || {
        if session.user().is_none() {
            navigator.replace(Route::Login {});
        }
    });
    let Some(user) = session.user() else {
        return rsx! { main { class: "auth-layout", "Redirecting to sign in..." } };
    };

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
                                        UserIdentity { user: user.clone() }
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
    let session = use_session();
    let navigator = use_navigator();
    use_effect(move || {
        if session.user().is_some() {
            navigator.replace(Route::Home {});
        }
    });

    rsx! {
        main { class: "auth-layout",
            div { class: "auth-panel", Outlet::<Route> {} }
        }
    }
}
