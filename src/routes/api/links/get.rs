use axum::{extract::State, http::StatusCode};

use crate::{AppState, database::{Link, get_link}, error::{Error, ErrorResponse, Result}, extractors::{Json, Path}, routes::Route};

#[utoipa::path(
    get,
    tags = [ "links" ],
    description = "Get a specific link by the given ID",
    path = Route::LinkGet.as_str(),
    responses(
        (status = 200, description = "Successfully fetched request link", content(
            ("application/json", examples(
                ( "OK" = (summary="Shortened link found", value = json!(
                        Link::new(None, "https://crates.io/".into())
                )))
            )),
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
pub async fn get_specific_link(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> Result<(StatusCode, Json<Link>)> {
    // Increment count of redirects for the link
    let link = get_link(&state.db, &link_id)
        .await?
        // The link with the given ID could not be found
        .ok_or_else(|| Error::LinkNotFound(link_id.clone()))?;

    tracing::debug!("Found link with ID {}", link_id);

    Ok((StatusCode::OK, Json(link)))
}
