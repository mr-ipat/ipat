//! Laboratory-only HTTP bootstrap; no authenticated data endpoints enabled.

use axum::{http::StatusCode, routing::get, Router};

fn app() -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/v1/devices/{device_id}", get(|| async {
            // No trusted OIDC adapter exists; fail closed, including ID enumeration.
            StatusCode::UNAUTHORIZED
        }))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind local loopback");
    axum::serve(listener, app()).await.expect("serve API");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_responds() {
        let response = app()
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
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
}
