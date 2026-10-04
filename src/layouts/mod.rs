use dioxus::prelude::*;

use crate::components::sidebar::*;
use crate::routes::Route;
use crate::session::use_session;
use crate::ui::UserIdentity;

#[component]
fn SidebarMenuButtonLink(to: Route, children: Element) -> Element {
    let route = use_route::<Route>();

    rsx! {
        SidebarMenuButton {
            size: SidebarMenuButtonSize::Lg,
            is_active: to == route,
            as: move |attributes: Vec<Attribute>| rsx! {
                Link { to: to.clone(), attributes: attributes,
                    {children.clone()}
                }
            },
        }
    }
}

#[component]
pub fn AppLayout() -> Element {
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
                                SidebarMenuButtonLink {
                                    to: Route::Home {},
                                    "Home"
                                }
                            }
                        }
                    }
                }
                SidebarFooter {
                    SidebarMenu {
                        SidebarMenuItem {
                            SidebarMenuButtonLink {
                                to: Route::Profile {},
                                UserIdentity { user: user.clone() }
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
