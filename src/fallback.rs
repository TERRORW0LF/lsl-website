use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::{
        body::Body,
        extract::State,
        response::IntoResponse,
        http::{Request, Response, StatusCode},
    };
    use axum::response::Response as AxumResponse;
    use tower::ServiceExt;
    use tower_http::services::ServeDir;
    use leptos::*;
    use crate::app::App;

    pub async fn file_and_error_handler(
        State(options): State<LeptosOptions>,
        req: Request<Body>,
    ) -> AxumResponse {
        let root = options.site_root.clone();
        let (parts, body) = req.into_parts();

        let mut static_parts = parts.clone();
        static_parts.headers.clear();
        if let Some(encodings) = parts.headers.get("accept-encoding") {
            static_parts
                .headers
                .insert("accept-encoding", encodings.clone());
        }

        let res = get_static_file(Request::from_parts(static_parts, Body::empty()), &root)
            .await;

        if res.status() == StatusCode::OK {
            res.into_response()
        } else {
            let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
            handler(Request::from_parts(parts, body))
                .await
                .into_response()
        }
    }


    async fn get_static_file(request: Request<Body>, root: &str) -> Response<Body> {
        // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
        // This path is relative to the cargo root
        ServeDir::new(root).precompressed_gzip().precompressed_br().oneshot(request).await.into_response()
    }
}}
