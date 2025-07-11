use crate::api::*;
use leptos::{prelude::*, tachys::view::fragment::IntoFragment};
use web_sys::FormData;

pub struct ViewFragment(Box<dyn FnOnce() -> Fragment + Send + 'static>);

impl Default for ViewFragment {
    fn default() -> Self {
        Self(Box::new(|| Fragment::new(vec![])))
    }
}

impl<F, C> From<F> for ViewFragment
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoFragment + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_fragment()))
    }
}

impl ViewFragment {
    /// Execute the wrapped function
    pub fn run(self) -> Fragment {
        (self.0)()
    }
}

pub type UserResource = Resource<Result<User, ApiError>>;
pub type UpdatePfpAction = Action<FormData, Result<(), ApiError>>;

#[cfg(feature = "ssr")]
pub type AuthSession = axum_session_auth::AuthSession<User, i64, axum_session_sqlx::SessionPgPool, sqlx::PgPool>;
