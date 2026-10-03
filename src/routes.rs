use dioxus::prelude::*;

use crate::layouts::{AppLayout, AuthLayout};
use crate::pages::{Home, Login, Profile, Register};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Home {},
        #[route("/profile")]
        Profile {},
    #[end_layout]
    #[layout(AuthLayout)]
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
}
