use super::Shared;
use axum::{response::Html, routing::get, Router};

pub fn router(state: Shared) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../../web/viewer.html")) }),
        )
        .route(
            "/viewer/main.js",
            get(|| async {
                (
                    [("content-type", "text/javascript")],
                    include_str!("../../web/viewer/main.js"),
                )
            }),
        )
        .route(
            "/viewer/fullscreen.js",
            get(|| async {
                (
                    [("content-type", "text/javascript")],
                    include_str!("../../web/viewer/fullscreen.js"),
                )
            }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [("content-type", "text/css")],
                    include_str!("../../web/style.css"),
                )
            }),
        )
        .route("/ws", get(super::upgrade))
        .with_state(state)
}
