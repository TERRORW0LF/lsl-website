use std::env;

use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use leptos_axum::AxumRouteListing;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl,
};
use sqlx::PgPool;

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: PgPool,
    pub routes: Vec<AxumRouteListing>,
    pub oauth: BasicClient,
}

pub fn oauth_client() -> BasicClient {
    let client_id = env::var("DISCORD_ID").expect("Missing DISCORD_ID!");
    let client_secret = env::var("DISCORD_SECRET").expect("Missing DISCORD_SECRET!");
    let redirect_url = env::var("REDIRECT_URL").expect("Missing REDIRECT_URL!");

    let auth_url = env::var("AUTH_URL").unwrap_or_else(|_| {
        "https://discord.com/api/oauth2/authorize?response_type=code".to_string()
    });

    let token_url = env::var("TOKEN_URL")
        .unwrap_or_else(|_| "https://discord.com/api/oauth2/token".to_string());
    let revoke_url = env::var("REVOKE_URL")
        .unwrap_or_else(|_| "https://discord.com/api/oauth2/token/revoke".to_string());

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).expect("failed to create new authorization server URL"),
        Some(TokenUrl::new(token_url).expect("failed to create new token endpoint URL")),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("failed to create new redirection URL"))
    .set_revocation_uri(
        RevocationUrl::new(revoke_url).expect("failed to create new revokation URL"),
    )
}
