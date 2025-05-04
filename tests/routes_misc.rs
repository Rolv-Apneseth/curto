use axum::http::header::CONTENT_TYPE;
use curto::routes::Route;
use pretty_assertions::assert_eq;

mod common;
use common::get_server;

#[tokio::test]
async fn test_routes_misc() {
    // Spin up a single DB container to run tests for misc routes
    let (_db_container, server) = get_server().await;

    // HEALTH
    let response = server.get(Route::Health.as_str()).await;
    response.assert_status_ok();
    response.assert_text_contains("OK");
    assert_eq!(response.header(CONTENT_TYPE), "text/plain; charset=utf-8");

    // 404
    let response = server.get("/not/a/route").await;
    response.assert_status_not_found();
    response.assert_text_contains("not found");
    assert_eq!(response.header(CONTENT_TYPE), "application/json");

    // METRICS
    let response = server.get(Route::Metrics.as_str()).await;
    response.assert_status_ok();
    assert_eq!(response.header(CONTENT_TYPE), "text/plain; charset=utf-8");
    assert!(
        response
            .header("content-length")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 1000
    );
}
