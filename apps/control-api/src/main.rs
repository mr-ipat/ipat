//! Laboratory-only HTTP bootstrap; no authenticated data endpoints enabled.

use axum::{http::StatusCode, routing::get, Router};

fn bind_address(k3s_lab: bool) -> &'static str {
    if k3s_lab {
        "0.0.0.0:3000"
    } else {
        "127.0.0.1:3000"
    }
}

fn app() -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route(
            "/v1/devices/{device_id}",
            get(|| async {
                // No trusted OIDC adapter exists; fail closed, including ID enumeration.
                StatusCode::UNAUTHORIZED
            }),
        )
}

#[tokio::main]
async fn main() {
    let k3s_lab = std::env::var("IPAT_RUN_K3S_LAB").as_deref() == Ok("1");
    let listener = tokio::net::TcpListener::bind(bind_address(k3s_lab))
        .await
        .expect("bind control API laboratory listener");
    axum::serve(listener, app()).await.expect("serve API");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[test]
    fn network_bind_is_loopback_unless_explicit_k3s_lab() {
        assert_eq!(bind_address(false), "127.0.0.1:3000");
        assert_eq!(bind_address(true), "0.0.0.0:3000");
    }

    #[tokio::test]
    async fn health_endpoint_responds() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
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
