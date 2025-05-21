use std::{fmt::Display, str::FromStr};

use axum::http::{StatusCode, header::LOCATION};
use axum_test::TestServer;
use chrono::{NaiveDateTime, Utc};
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
    custom_expires_at: Option<NaiveDateTime>,
) -> axum_test::TestResponse {
    server
        .post(Route::Links.as_str())
        .json(&CreateLinkRequest {
            target_url: target_url.to_string(),
            custom_id,
            custom_expires_at,
        })
        .await
}

/// Utility function to create and validate shortened link
#[inline]
async fn assert_create_link(
    server: &TestServer,
    target_url: impl Display,
    custom_id: Option<String>,
    custom_expires_at: Option<NaiveDateTime>,
) -> Link {
    let response = request_create_link(server, &target_url, custom_id, custom_expires_at).await;
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
    assert_create_link(&server, "https://crates.io", None, None).await;
    assert_create_link(&server, "sftp://host.io", None, None).await;
    assert_create_link(&server, "http://192.168.1.1", None, None).await;
    assert_create_link(&server, "http://www.test.site/abc123", None, None).await;
    assert_create_link(
        &server,
        "http://random.address/with?params=1&params=2",
        None,
        None,
    )
    .await;

    // FAILURES
    let target_url = Url::parse("https://crates.io/").unwrap();
    let link = assert_create_link(&server, &target_url, None, None).await;

    // Custom IDs
    assert_create_link(&server, "https://crates.io", Some("abc123".into()), None).await;
    assert_create_link(&server, "https://crates.io", Some("testing".into()), None).await;
    assert_create_link(&server, "https://crates.io", Some("1".into()), None).await;
    assert_create_link(
        &server,
        "https://crates.io",
        Some("kelkadskekwklakdsfiowoekklasdf".into()),
        None,
    )
    .await;

    assert_create_link(
        // Expiration time
        &server,
        "https://crates.io",
        Some("2".into()),
        Some(NaiveDateTime::from_str("3045-01-01T00:00:00").unwrap()),
    )
    .await;

    // Does not exist
    let response = server.get(&format!("/links/{}", "noid")).await;
    response.assert_status(StatusCode::NOT_FOUND);

    // Existing ID
    let response = request_create_link(&server, &target_url, Some(link.id.clone()), None).await;
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    server
        .get(&format!("/links/{}", link.id))
        .await
        .assert_status_ok();

    // Invalid IDs
    for invalid in ["", "links", "docs", "not-alphanumeric", "/route"] {
        let response = request_create_link(&server, &target_url, Some(invalid.into()), None).await;
        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Invalid URLs
    for invalid in ["", "//", "https//crates", "crates.io", "/absolute/path"] {
        let response = request_create_link(&server, invalid, None, None).await;
        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Invalid Expiration times
    for invalid in [NaiveDateTime::default(), Utc::now().naive_utc()] {
        let response = request_create_link(&server, "https://crates.io", None, Some(invalid)).await;
        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn test_get_link() {
    let (_db_container, server) = get_server().await;

    let target_url = Url::parse("https://crates.io/").unwrap();

    let link = assert_create_link(&server, &target_url, None, None).await;
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
        assert_create_link(&server, u, None, None).await;
    }

    let response = server.get(Route::Links.as_str()).await;
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
    assert_create_link(&server, "https://www.rust-lang.org/", None, None).await;
    let response = server.get(Route::Links.as_str()).await;
    response.assert_status(StatusCode::OK);
    let links = response.json::<Vec<Link>>();
    assert_eq!(links.len(), target_urls.len() + 1);
}

#[tokio::test]
async fn test_redirect_links() {
    let (_db_container, server) = get_server().await;

    let target_url = Url::parse("https://crates.io").unwrap();
    let link = assert_create_link(&server, &target_url, None, None).await;

    let response = server.get(&format!("/{}", link.id)).await;
    response.assert_status(StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(response.header(LOCATION), target_url.to_string());

    let target_url = Url::parse("https://www.rust-lang.org/").unwrap();
    let link = assert_create_link(&server, &target_url, None, None).await;

    let response = server.get(&format!("/{}", link.id)).await;
    response.assert_status(StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(response.header(LOCATION), target_url.to_string());

    let response = server.get(&format!("/{}", "noid")).await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_expired_links_not_found() {
    let (_db_container, server) = get_server().await;

    let target_url = Url::parse("https://crates.io").unwrap();

    let beginning = Utc::now().naive_utc() + chrono::Duration::seconds(1);

    // Create links
    let link_with_expiration =
        assert_create_link(&server, &target_url, None, Some(beginning.clone())).await;
    assert_create_link(&server, &target_url, None, Some(beginning.clone())).await;

    let link_without_expiration = assert_create_link(&server, &target_url, None, None).await;
    let link_with_later_expiration = assert_create_link(
        &server,
        &target_url,
        None,
        Some(beginning + chrono::Duration::seconds(20)),
    )
    .await;

    // Assert all links were created
    let links = server.get(Route::Links.as_str()).await.json::<Vec<Link>>();
    assert_eq!(links.len(), 4);

    // Await until some links are expired
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    interval.tick().await;
    interval.tick().await;

    // Assert that expired links are no longer returned
    let links = server.get(Route::Links.as_str()).await.json::<Vec<Link>>();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].id, link_without_expiration.id);
    assert_eq!(links[1].id, link_with_later_expiration.id);

    // Assert that redirection will not work for an expired link
    let response = server.get(&format!("/{}", link_with_expiration.id)).await;
    response.assert_status(StatusCode::NOT_FOUND);

    // Only specifically querying an expired link should work
    let response = server
        .get(&format!("/links/{}", link_with_expiration.id))
        .await;
    let link = response.json::<Link>();
    assert_eq!(link.id, link_with_expiration.id);
}
