use axum::{extract::State, http};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, routes::Route};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handle_health))
        .routes(routes!(handle_metrics))
}

#[utoipa::path(
    get,
    tags = [ "misc" ],
    path = Route::Health.as_str(),
    description="Simple API health check",
    responses(
        (status = 200, description = "Health check OK", content(
            ("text/plain", examples(
                ( "OK" = (summary="API is available", value = json!("OK") )),
            ))
        )),
    )
)]
async fn handle_health() -> (http::StatusCode, String) {
    (http::StatusCode::OK, "OK".into())
}

#[utoipa::path(
    get,
    tags = [ "misc" ],
    path = Route::Metrics.as_str(),
    description="API Prometheus metrics",
    responses(
        (status = 200, description = "Prometheus metrics found", content(
            ("text/plain", examples(
                ( "OK" = (summary="Prometheus metrics found", value = json!(
r#"# TYPE axum_http_requests_pending gauge
axum_http_requests_pending{method="GET",endpoint="/api/v0/metrics"} 1
axum_http_requests_total{method="GET",status="200",endpoint="/api/v0/health"} 1
..."#
                )))
            )),
        ))
    )
)]
async fn handle_metrics(State(state): State<AppState>) -> (http::StatusCode, String) {
    (http::StatusCode::OK, state.metric_handle.clone().render())
}
