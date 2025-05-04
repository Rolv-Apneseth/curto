use std::fmt::Display;

use axum::http::StatusCode;
use axum_test::TestServer;
use curto::{database::Link, routes::{Route, api::links::create::CreateLinkRequest}};
use pretty_assertions::assert_eq;

mod common;
use common::get_server;
use url::Url;

#[inline]
async fn request_create_link(
    server: &TestServer,
    target_url: impl Display,
    custom_id: Option<String>,
) -> axum_test::TestResponse {
    server
        .post(Route::LinkCreate.as_str())
        .json(&CreateLinkRequest {
            target_url: target_url.to_string(),
            custom_id,
        })
        .await
}

/// Utility function to create and validate shortened link
#[inline]
async fn assert_create_link(
    server: &TestServer,
    target_url: impl Display,
    custom_id: Option<String>,
) -> Link {
    let response = request_create_link(server, &target_url, custom_id).await;
    response.assert_status(StatusCode::CREATED);
    assert_eq!(response.header("content-type"), "application/json");

    let link = response.json::<Link>();
    assert_eq!(
        Url::parse(&link.target_url).unwrap(),
        target_url.to_string().parse().unwrap()
    );
    assert!(!link.id.is_empty());

    link
}

#[tokio::test]
async fn test_create_links() {
    let (_db_container, server) = get_server().await;

    // SUCCESS
    assert_create_link(&server, "https://crates.io", None).await;
    assert_create_link(&server, "sftp://host.io", None).await;
    assert_create_link(&server, "http://192.168.1.1", None).await;
    assert_create_link(&server, "http://www.test.site/abc123", None).await;
    assert_create_link(
        &server,
        "http://random.address/with?params=1&params=2",
        None,
    )
    .await;

    // FAILURES
    let target_url = Url::parse("https://crates.io/").unwrap();
    let link = assert_create_link(&server, &target_url, None).await;

    // Does not exist
    let response = server.get(&format!("/links/{}", "noid")).await;
    response.assert_status(StatusCode::NOT_FOUND);

    // Existing ID
    let response = request_create_link(&server, &target_url, Some(link.id.clone())).await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    server
        .get(&format!("/links/{}", link.id))
        .await
        .assert_status_ok();

    // Invalid IDs
    for invalid in ["", "links", "docs", "not-alphanumeric", "/route"] {
        let response = request_create_link(&server, &target_url, Some(invalid.into())).await;
        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Invalid URLs
    for invalid in ["", "//", "https//crates", "crates.io", "/absolute/path"] {
        let response = request_create_link(&server, invalid, None).await;
        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn test_get_link() {
    let (_db_container, server) = get_server().await;

    let target_url = Url::parse("https://crates.io/").unwrap();

    let link = assert_create_link(&server, &target_url, None).await;
    let id = link.id.clone();

    let response = server.get(&format!("/links/{}", id)).await;
    response.assert_status(StatusCode::OK);
    assert_eq!(response.header("content-type"), "application/json");

    let link = response.json::<Link>();
    assert_eq!(Url::parse(&link.target_url).unwrap(), target_url);
    assert_eq!(link.id, id);
}

#[tokio::test]
async fn test_list_links() {
    let (_db_container, server) = get_server().await;

    let target_urls = [
        Url::parse("https://crates.io/").unwrap(),
        Url::parse("https://www.rust-lang.org/").unwrap(),
        Url::parse("https://github.com/rust-lang").unwrap(),
        Url::parse("https://github.com/orgs/rust-lang/projects?query=is%3Aopen").unwrap(),
    ];

    // Create shortened links
    for u in target_urls.iter() {
        assert_create_link(&server, u, None).await;
    }

    let response = server.get(Route::LinkList.as_str()).await;
    response.assert_status(StatusCode::OK);
    assert_eq!(response.header("content-type"), "application/json");

    let links = response.json::<Vec<Link>>();
    assert_eq!(links.len(), target_urls.len());

    for u in target_urls.iter() {
        links
            .iter()
            .find(|l| l.target_url == u.to_string())
            .unwrap();
    }

    // Create a new link and ensure num of returned links increased
    assert_create_link(&server, "https://www.rust-lang.org/", None).await;
    let response = server.get(Route::LinkList.as_str()).await;
    response.assert_status(StatusCode::OK);
    let links = response.json::<Vec<Link>>();
    assert_eq!(links.len(), target_urls.len() + 1);
}

#[tokio::test]
async fn test_redirect_links() {
    let (_db_container, server) = get_server().await;

    let target_url = Url::parse("https://crates.io").unwrap();
    let link = assert_create_link(&server, &target_url, None).await;

    let response = server.get(&format!("/{}", link.id)).await;
    response.assert_status(StatusCode::TEMPORARY_REDIRECT);

    let target_url = Url::parse("https://www.rust-lang.org/").unwrap();
    let link = assert_create_link(&server, &target_url, None).await;

    let response = server.get(&format!("/{}", link.id)).await;
    response.assert_status(StatusCode::TEMPORARY_REDIRECT);

    let response = server.get(&format!("/{}", "noid")).await;
    response.assert_status(StatusCode::NOT_FOUND);
}
