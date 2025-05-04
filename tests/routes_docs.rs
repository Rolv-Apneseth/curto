mod common;
use axum::http::header::CONTENT_TYPE;
use common::get_server;
use curto::routes::Route;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_routes_docs() {
    let (_db_container, server) = get_server().await;

    let response = server.get(Route::Docs.into()).await;
    response.assert_status_ok();
    assert_eq!(response.header(CONTENT_TYPE), "text/html; charset=utf-8");
    assert!(
        response
            .header("content-length")
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 4000
    );
}
