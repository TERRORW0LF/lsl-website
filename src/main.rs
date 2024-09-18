use axum_session::SessionLayer;
use axum_session_auth::AuthSessionLayer;
use lsl_website::{auth::ssr::AuthSession, fallback::file_and_error_handler, state::AppState};

cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
use axum::{
    body::Body as AxumBody,
    extract::{State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use axum_session_auth::{AuthConfig};
use axum_session_sqlx::SessionPgPool;
use http::Request;
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use leptos_router::RouteListing;
use lsl_website::{app::*, auth::ssr::User};
use rustls_acme::{caches::DirCache, AcmeConfig};
use sqlx::PgPool;
use tokio_stream::StreamExt;
use axum_session::SessionConfig;
use axum_session_sqlx::SessionPgSessionStore;
use lsl_website::auth::ssr::{connect_to_database};

async fn leptos_handler(
    State(state): State<AppState>,
    session: AuthSession,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        state.leptos_options.clone(),
        state.routes.clone(),
        move || {
            provide_context(state.pool.clone());
            provide_context(session.clone());
        },
        App,
    );
    handler(req).await.into_response()
}

async fn server_handler(
    State(state): State<AppState>,
    session: AuthSession,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(state.pool.clone());
            provide_context(session.clone());
        },
        request,
    )
    .await
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");
    // ACME setup for Let's Encrypt + TLS connection
    /*let email = env!("EMAIL");
    let cache = env!("CERT_CACHE");
    let domain = env!("DOMAIN");
    let mut state = AcmeConfig::new([domain.unwrap()])
        .contact_push(email.unwrap())
        .cache(DirCache::new(cache.unwrap()))
        .directory_lets_encrypt(false)
        .state();
    let acceptor = state.axum_acceptor(state.default_rustls_config());

    tokio::spawn(async move {
        loop {
            match state.next().await.unwrap() {
                Ok(ok) => log::info!("event: {:?}", ok),
                Err(err) => log::error!("error: {:?}", err),
            }
        }
    });*/

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
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options,
        pool: pool.clone(),
        routes: routes.clone(),
    };

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(server_handler))
        .leptos_routes_with_handler(routes.clone(), get(leptos_handler))
        .fallback(file_and_error_handler)
        .layer(AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(
                Some(pool.clone()),
            )
            .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    axum_server::bind(addr)
        //.acceptor(acceptor)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
} else {
    fn main() {

    }
}}
