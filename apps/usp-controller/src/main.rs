//! Loopback-only synthetic USP Controller process bootstrap.
//!
//! NOT a native USP Record/MTP endpoint. The actual controller identity,
//! agent trust and TR-369 protobuf/MQTT remain unimplemented. No public
//! service or unauthenticated agent intake is intentionally offered.

use axum::{http::StatusCode, routing::get, Router};

fn app() -> Router {
    Router::new()
        .route("/healthz", get(|| async { "synthetic-usp-lab-only" }))
        .fallback(|| async { StatusCode::SERVICE_UNAVAILABLE })
}

#[tokio::main]
async fn main() {
    if std::env::var("IPAT_RUN_OFFLINE_USP_LAB").as_deref() != Ok("1") {
        eprintln!("USP network/MTP not implemented. Set IPAT_RUN_OFFLINE_USP_LAB=1 only for local health checks.");
        std::process::exit(2);
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3100")
        .await
        .expect("bind loopback-only synthetic USP health listener");
    axum::serve(listener, app())
        .await
        .expect("serve loopback health only");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn private_lab_health_is_available() {
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
    async fn no_usp_transport_route_accepts_untrusted_agent_messages() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/usp")
                    .header("X-Tenant-Id", "kangnet")
                    .header("X-Agent-Id", "synthetic-agent-a")
                    .body(Body::from("fake-USP"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn no_tenant_or_controller_data_endpoint_enabled() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/v1/tenants")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
