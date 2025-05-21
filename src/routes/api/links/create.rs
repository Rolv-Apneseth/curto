use axum::{extract::State, http::StatusCode};
use axum_extra::extract::Host;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::{AppState, database::{Link, create_link}, error::{Error, ErrorResponse, Result}, extractors::Json, routes::Route};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateLinkRequest {
    /// The target URL which the new shortened link should redirect to
    pub target_url: String,
    /// An optional, unique ID for the new shortened link.
    ///
    /// This value will be randomly generated if omitted
    pub custom_id: Option<String>,
    /// An optional expiration time for the new shortened link, given in the
    /// form "yyyy-mm-ddTHH:MM:ss.SSS" (without a timezone).
    pub custom_expires_at: Option<NaiveDateTime>,
}

#[utoipa::path(
    post,
    path = Route::Links.as_str(),
    tags = [ "links" ],
    description = "Create new shortened links",
    request_body = CreateLinkRequest,
    responses(
        (status = 201, description = "Shortened link created successfully", content(
            ("application/json", examples(
                ( "OK" = (summary="Shortened link created", value = json!(
                    Link::new(None, "https://crates.io/".into())
                )))
            )),
        )),
        (status = 400, description = "Bad request", content(
            ("application/json", examples(
                ("Malformed URL" = (summary="User provided a malformed URL",
                    value=json!(ErrorResponse::from(Error::MalformedURL("hppts://googlecom".to_string())))))
            ))
        )),
        (status = 422, description = "Request parameter(s) invalid", content(
            ("application/json", examples(
                ("Provided ID not unique" = (summary="User provided an ID which is already in use",
                    value=json!(ErrorResponse::from(Error::LinkIdNotUnique("taken".to_string()))))),
                ("URL without host" = (summary="User provided a URL which does not have a host",
                    value=json!(ErrorResponse::from(Error::URLWithoutHost("/path/to/file".to_string()))))),
                ("URL has the same host as this service" = (summary="User provided a URL which has the same host as this service",
                    value=json!(ErrorResponse::from(Error::URLWithMatchingHosts("localhost".to_string()))))),
                ("URL is invalid" = (summary="User provided a URL which is invalid, either containing non-alphanumeric characters or matching a path in use by this service",
                    value=json!(ErrorResponse::from(Error::LinkIdNotValid("abc-xyz".to_string()))))),
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
pub async fn create_new_link(
    State(state): State<AppState>,
    Host(host): Host,
    Json(new_link): Json<CreateLinkRequest>,
) -> Result<(StatusCode, Json<Link>)> {
    let url = Url::parse(&new_link.target_url)?;

    // Deny URLs without a defined host
    let target_host = url
        .host()
        .ok_or_else(|| Error::URLWithoutHost(url.to_string()))?
        .to_string();

    // Attempt to deny URLs with a host that matches this service, to prevent a
    // circular redirect.
    if hosts_match(&host, &target_host) {
        return Err(Error::URLWithMatchingHosts(host));
    };

    // Create a new link
    let new_link = create_link(
        &state.db,
        url.to_string(),
        new_link.custom_id,
        new_link.custom_expires_at,
    )
    .await?;

    tracing::debug!("Created new link with id {} targeting {}", new_link.id, url);

    Ok((StatusCode::CREATED, Json(new_link)))
}

/// Utility function used to check if the request and target hosts match
fn hosts_match(request_host: &str, target_host: &str) -> bool {
    if request_host == target_host {
        return true;
    }

    const LOCALHOSTS: &[&str] = &["0.0.0.0", "localhost", "127.0.0.1"];

    if LOCALHOSTS.iter().any(|h| target_host.starts_with(h))
        && LOCALHOSTS.iter().any(|h| request_host.starts_with(h))
    {
        return match (target_host.split_once(":"), request_host.split_once(":")) {
            (Some(t), Some(r)) => t.1 == r.1,
            (None, None) => true,
            _ => false,
        };
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hosts_match() {
        assert!(hosts_match("0.0.0.0", "0.0.0.0"));
        assert!(hosts_match("localhost", "0.0.0.0"));
        assert!(hosts_match("127.0.0.1", "0.0.0.0"));
        assert!(hosts_match("localhost", "127.0.0.1"));
        assert!(hosts_match("localhost:7229", "127.0.0.1:7229"));
        assert!(hosts_match("real.site:420", "real.site:420"));

        assert!(!hosts_match("google.com", "bing.com"));
        assert!(!hosts_match("127.0.0.1", "127.0.0.1:7229"));
        assert!(!hosts_match("127.0.0.1:7229", "127.0.0.1"));
        assert!(!hosts_match("localhost:7229", "0.0.0.0:7228"));
        assert!(!hosts_match("localhost", "localhost:722"));
        assert!(!hosts_match("localhost", "crates.io"));
    }
}
