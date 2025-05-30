use crate::api::*;
use leptos::prelude::*;
use web_sys::FormData;

pub type UserResource = Resource<Result<User, ApiError>>;
pub type UpdatePfpAction = Action<FormData, Result<(), ApiError>>;

#[cfg(feature = "ssr")]
pub type AuthSession =
    axum_session_auth::AuthSession<User, i64, axum_session_sqlx::SessionPgPool, sqlx::PgPool>;
