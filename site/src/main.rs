use lsl_website::state::oauth_client;

cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
use axum::{
    body::Body as AxumBody,
    extract::{State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_session::{SessionConfig, SessionLayer};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::{SessionPgPool, SessionPgSessionStore};
use http::Request;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use lsl_website::{app::*, state::AppState};
use sqlx::PgPool;
use tower::ServiceBuilder;
use server::auth::ssr::connect_to_database;
use types::{leptos::AuthSession, api::User};

async fn leptos_handler(
    state: State<AppState>,
    session: AuthSession,
    req: Request<AxumBody>,
) -> Response {
    let pool = state.pool.clone();
    let options = state.leptos_options.clone();
    let handler = leptos_axum::render_route_with_context(
        state.routes.clone(),
        move || {
            provide_context(pool.clone());
            provide_context(session.clone());
        },
        move || shell(options.clone()),
    );
    handler(state, req).await.into_response()
}

async fn server_handler(
    State(state): State<AppState>,
    session: AuthSession,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(state.pool.clone());
            provide_context(state.oauth.clone());
            provide_context(session.clone());
        },
        request,
    )
    .await
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_env().expect("couldn't initialize logging");

    let pool = connect_to_database().await;
    let session_config = SessionConfig::default().with_table_name("session");
    let auth_config = AuthConfig::<i64>::default().with_session_id("user_id".to_string());
    let session_store = SessionPgSessionStore::new(Some(pool.clone().into()), session_config)
        .await
        .unwrap();

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options,
        pool: pool.clone(),
        routes: routes.clone(),
        oauth: oauth_client(),
    };

    // build our application with a route
    let app = Router::new()
        .route("/api/{*fn_name}", get(server_handler).post(server_handler))
        .leptos_routes_with_handler(routes.clone(), get(leptos_handler))
        .layer(ServiceBuilder::new()
            .layer(SessionLayer::new(session_store))
            .layer(AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(
                Some(pool.clone()),
            )
            .with_config(auth_config),
        ))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
} else {
    fn main() {

    }
}}
