//! Standalone private-only Site/POP master UI. The owner pilot currently shares
//! one atomic snapshot for its site-to-device delete constraints; production
//! MUST use a distinct tenant-scoped PostgreSQL site master and device FK.
use axum::{http::HeaderMap, response::Html, routing::get, Router};
const HTML: &str = include_str!("../../../web/lab/site-manager.html");
const JS: &str = include_str!("../../../web/lab/site-manager.js");
const CSS: &str = include_str!("../../../web/lab/site-manager.css");
async fn page() -> (HeaderMap, Html<&'static str>) {
    (
        super::private_lab_headers("text/html; charset=utf-8"),
        Html(HTML),
    )
}
async fn script() -> (HeaderMap, &'static str) {
    (
        super::private_lab_headers("text/javascript; charset=utf-8"),
        JS,
    )
}
async fn style() -> (HeaderMap, &'static str) {
    (super::private_lab_headers("text/css; charset=utf-8"), CSS)
}
pub(super) fn router() -> Router {
    Router::new()
        .route("/lab/sites", get(page))
        .route("/lab/sites.js", get(script))
        .route("/lab/sites.css", get(style))
}
