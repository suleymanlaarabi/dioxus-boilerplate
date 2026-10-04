use dioxus::prelude::*;

use crate::auth::User;
use crate::components::avatar::{Avatar, AvatarFallback};

#[component]
pub fn UserIdentity(user: User) -> Element {
    rsx! {
        Avatar { AvatarFallback { "{user.initials()}" } }
        span { "{user.full_name()}" }
    }
}
