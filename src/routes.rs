use crate::layouts::{AppLayout, AuthLayout, PublicAuthLayout};
use crate::pages::{ForgotPassword, Home, Login, Profile, Register, ResetPassword, VerifyEmail};
use dioxus::prelude::*;

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
        #[route("/forgot-password")]
        ForgotPassword {},
    #[end_layout]
    #[layout(PublicAuthLayout)]
        #[route("/reset-password?:token")]
        ResetPassword { token: Option<String> },
        #[route("/verify-email?:token")]
        VerifyEmail { token: Option<String> },
}
