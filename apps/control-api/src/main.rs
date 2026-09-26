//! Rust Control API laboratory bootstrap.
//! R5.9 adds an explicitly opted-in, read-only LOCAL web preview.
//! This is NOT an authenticated tenant dashboard or a production/public UI.

use axum::{
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::Html,
    routing::get,
    Router,
};

const LAB_INDEX: &str = include_str!("../../../web/lab/index.html");
const LAB_CSS: &str = include_str!("../../../web/lab/style.css");
const LAB_JS: &str = include_str!("../../../web/lab/app.js");
const LAB_STATUS: &str = r#"{"mode":"ssh-loopback-only","production_access":false,"authentication_enabled":false,"device_operations_enabled":false,"backend":"online"}"#;

fn bind_address(k3s_lab: bool) -> &'static str {
    if k3s_lab {
        "0.0.0.0:3000"
    } else {
        "127.0.0.1:3000"
    }
}

fn lab_web_enabled(k3s_lab: bool, lab_requested: bool) -> bool {
    lab_requested && !k3s_lab
}

fn private_lab_headers(content_type: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; \
             img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        ),
    );
    headers
}

async fn lab_index() -> (HeaderMap, Html<&'static str>) {
    (
        private_lab_headers("text/html; charset=utf-8"),
        Html(LAB_INDEX),
    )
}

async fn lab_css() -> (HeaderMap, &'static str) {
    (private_lab_headers("text/css; charset=utf-8"), LAB_CSS)
}

async fn lab_js() -> (HeaderMap, &'static str) {
    (
        private_lab_headers("text/javascript; charset=utf-8"),
        LAB_JS,
    )
}

async fn lab_status() -> (HeaderMap, &'static str) {
    (
        private_lab_headers("application/json; charset=utf-8"),
        LAB_STATUS,
    )
}

fn app_with_lab(lab_web_enabled: bool) -> Router {
    let api = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route(
            "/v1/devices/{device_id}",
            get(|| async {
                // Trusted OIDC/tenant binding is not implemented. No ID enumeration.
                StatusCode::UNAUTHORIZED
            }),
        );
    if lab_web_enabled {
        api.route("/lab", get(lab_index))
            .route("/lab/", get(lab_index))
            .route("/lab/style.css", get(lab_css))
            .route("/lab/app.js", get(lab_js))
            .route("/lab/status", get(lab_status))
    } else {
        api
    }
}

#[cfg(test)]
fn app() -> Router {
    app_with_lab(false)
}

#[tokio::main]
async fn main() {
    let k3s_lab = std::env::var("IPAT_RUN_K3S_LAB").as_deref() == Ok("1");
    let lab_requested = std::env::var("IPAT_LAB_WEB").as_deref() == Ok("1");
    // Fail closed: never expose unauthenticated lab HTML through the K3s pod bind.
    let lab_web_enabled = lab_web_enabled(k3s_lab, lab_requested);
    let listener = tokio::net::TcpListener::bind(bind_address(k3s_lab))
        .await
        .expect("bind control API laboratory listener");
    axum::serve(listener, app_with_lab(lab_web_enabled))
        .await
        .expect("serve API");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    async fn get_path(router: Router, uri: &str) -> axum::response::Response {
        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn network_bind_is_loopback_unless_explicit_k3s_lab() {
        assert_eq!(bind_address(false), "127.0.0.1:3000");
        assert_eq!(bind_address(true), "0.0.0.0:3000");
    }

    #[tokio::test]
    async fn health_endpoint_responds() {
        let response = get_path(app(), "/healthz").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn data_endpoint_denies_anonymous_requests_even_with_tenant_header() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/v1/devices/123")
                    .header("X-Tenant-Id", "kangnet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn lab_routes_are_missing_by_default_and_on_k3s_router() {
        for uri in [
            "/lab",
            "/lab/",
            "/lab/style.css",
            "/lab/app.js",
            "/lab/status",
        ] {
            let response = get_path(app(), uri).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        }
        assert!(!lab_web_enabled(false, false));
        assert!(lab_web_enabled(false, true));
        assert!(!lab_web_enabled(true, false));
        assert!(!lab_web_enabled(true, true));
    }

    #[tokio::test]
    async fn private_lab_has_browser_content_and_restrictive_headers() {
        let response = get_path(app_with_lab(true), "/lab").await;
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert_eq!(headers["x-content-type-options"], "nosniff");
        assert_eq!(headers["x-frame-options"], "DENY");
        assert!(headers["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("default-src 'none'"));
        assert!(LAB_INDEX.contains("Mode terbatas"));
        assert!(LAB_INDEX.contains("Mr. iPat"));
        assert!(!LAB_INDEX.contains("type=\"password\""));
    }

    #[tokio::test]
    async fn private_lab_assets_and_status_are_read_only() {
        for (uri, mime) in [
            ("/lab/style.css", "text/css; charset=utf-8"),
            ("/lab/app.js", "text/javascript; charset=utf-8"),
            ("/lab/status", "application/json; charset=utf-8"),
        ] {
            let response = get_path(app_with_lab(true), uri).await;
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(response.headers()[header::CONTENT_TYPE], mime);
        }
        assert!(LAB_STATUS.contains("\"production_access\":false"));
        assert!(LAB_STATUS.contains("\"authentication_enabled\":false"));
    }
}
