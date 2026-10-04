use dioxus::fullstack::{AsStatusCode, StatusCode};
use dioxus::prelude::*;

use crate::auth::User;

#[derive(Clone, Copy)]
pub struct Session {
    user: Signal<Option<User>>,
}

impl Session {
    pub fn new(user: Option<User>) -> Self {
        Self {
            user: Signal::new(user),
        }
    }

    pub fn user(self) -> Option<User> {
        (self.user)()
    }

    pub fn set_user(mut self, user: Option<User>) {
        self.user.set(user);
    }

    pub fn check<T>(self, result: Result<T, ServerFnError>) -> Result<T, ServerFnError> {
        if result
            .as_ref()
            .is_err_and(|error| error.as_status_code() == StatusCode::UNAUTHORIZED)
        {
            self.set_user(None);
        }
        result
    }
}

pub fn use_session() -> Session {
    use_context::<Session>()
}
