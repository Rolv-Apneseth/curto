use axum::{body::Body, extract::{RawQuery, State}, http::{HeaderMap, HeaderValue, StatusCode, header::{ACCEPT_LANGUAGE, CONTENT_SECURITY_POLICY, CONTENT_TYPE, COOKIE, HOST, SET_COOKIE, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS, X_XSS_PROTECTION}, response::Builder}, response::Response};
use url::Url;

use crate::{AppState, database::{Link, increment_link_redirect_count}, error::{Error, ErrorResponse, Result}, extractors::Path, routes::Route};

const DEFAULT_CACHE_CONTROL_HEADER_VALUE: &str =
    "public, max-age=300, s-maxage=300, stale-while-revalidate=300, stale-if-error=300";

#[utoipa::path(
    get,
    tags = [ "links" ],
    description = "Redirect from a link matching the given ID to its target URL",
    path = Route::LinkRedirect.as_str(),
    responses(
        (status = 307, description = "Successful redirect", headers(
            ("Cache-Control"),
            ("Location"),
        )),
        (status = 404, description = "Link matching ID not found", content(
            ("application/json", examples(
                ("Link not found" = (summary="No link matching the specified ID could be found",
                    value=json!(ErrorResponse::from(Error::LinkNotFound("bmdkw".to_string())))))
            ))
        )),
        (status = 500, description = "Internal server error", content(
            ("application/json", examples(
                ("Internal server error" =
                    (value=json!(ErrorResponse::from(Error::Internal(String::new())))))
            ))
        )),
    )
)]
pub async fn redirect_links(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
    raw_query: RawQuery,
    headers: HeaderMap,
) -> Result<Response> {
    // Increment count of redirects for the link
    let link = increment_link_redirect_count(&state.db, &link_id)
        .await?
        // The link with the given ID could not be found
        .ok_or_else(|| Error::LinkNotFound(link_id.clone()))?;

    tracing::debug!("Redirecting link ID {} to {}", link_id, link.target_url);

    let mut resp = Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", forward_query_params(&link, raw_query))
        .header("Cache-Control", DEFAULT_CACHE_CONTROL_HEADER_VALUE);

    resp = forward_headers(resp, headers);

    Ok(resp
        .body(Body::empty())
        .expect("This response should always be constructable"))
}

/// Forward certain headers from the request on to the response
fn forward_headers(mut resp: Builder, request_headers: HeaderMap) -> Builder {
    let existing_headers = resp
        .headers_mut()
        .expect("builder has an error prior to reaching the forward headers step");

    // Forward certain headers
    for key in [
        // Basic
        HOST,
        ACCEPT_LANGUAGE,
        CONTENT_TYPE,
        // Security
        CONTENT_SECURITY_POLICY,
        X_CONTENT_TYPE_OPTIONS,
        X_FRAME_OPTIONS,
        X_XSS_PROTECTION,
        // Cookies
        COOKIE,
        SET_COOKIE,
    ] {
        let Some(Ok(Ok(value))) = request_headers
            .get(&key)
            .map(|v| v.to_str().map(|s| s.parse::<HeaderValue>()))
        else {
            continue;
        };

        // Don't replace existing keys
        if existing_headers.contains_key(&key) {
            continue;
        }

        existing_headers.insert(key, value);
    }

    resp
}

/// Build the target URL from the base target URL and any received query
/// parameters
fn forward_query_params(link: &Link, raw_query: RawQuery) -> String {
    let mut url = Url::parse(&link.target_url).unwrap_or_else(|e| {
        tracing::error!(
            "Invalid URL stored in database for link with ID '{}'. Error: {e}",
            link.id
        );
        panic!("Invalid URL somehow got into the database");
    });

    if let Some(q) = raw_query.0 {
        url.set_query(Some(q.as_str()));
    }

    url.to_string()
}

#[cfg(test)]
mod test {
    use axum::http::header::HOST;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_forward_headers() {
        let resp = Response::builder().status(StatusCode::TEMPORARY_REDIRECT);
        let mut headers = HeaderMap::new();

        let resp = forward_headers(resp, headers.clone());
        assert_eq!(resp.headers_ref().unwrap().len(), 0);

        headers.insert(HOST, "host".parse().unwrap());
        let resp = forward_headers(resp, headers.clone());
        assert!(resp.headers_ref().unwrap().contains_key(HOST));

        // Inserts new headers but does not duplicate existing headers
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        let resp = forward_headers(resp, headers.clone());
        assert_eq!(resp.headers_ref().unwrap().len(), 2);
        assert!(resp.headers_ref().unwrap().contains_key(HOST));
        assert_eq!(resp.headers_ref().unwrap().get(HOST).unwrap(), "host");
        assert!(resp.headers_ref().unwrap().contains_key(ACCEPT_LANGUAGE));

        // Does not replace existing header
        headers.insert(HOST, "new_host".parse().unwrap());
        let resp = forward_headers(resp, headers.clone());
        assert_eq!(resp.headers_ref().unwrap().len(), 2);
        assert_eq!(resp.headers_ref().unwrap().get(HOST).unwrap(), "host");
    }

    #[test]
    fn test_forward_query_params() {
        let base_url = "https://github.com/".to_string();
        let link = Link {
            target_url: base_url.clone(),
            ..Default::default()
        };

        assert_eq!(
            forward_query_params(&link, RawQuery(None)),
            base_url.clone()
        );

        assert_eq!(
            forward_query_params(&link, RawQuery(Some("test=value".into()))),
            format!("{}?{}", base_url, "test=value")
        );
        assert_eq!(
            forward_query_params(&link, RawQuery(Some("test=value&test2=value".into()))),
            format!("{}?{}", base_url, "test=value&test2=value")
        );
    }
}
