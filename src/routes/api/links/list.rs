use axum::{extract::State, http::StatusCode};

use crate::{AppState, database::{Link, get_links}, error::{Error, ErrorResponse, Result}, extractors::Json, routes::Route};

#[utoipa::path(
    get,
    tags = [ "links" ],
    description = "Get all existing shortened links",
    path = Route::LinkList.as_str(),
    responses(
        (status = 200, description = "Successfully fetched all shortened links", content(
            ("application/json", examples(
                ( "OK" = (summary="Shortened links found", value = json!(
                    vec![
                        Link::new(None, "https://crates.io/".into()),
                        Link::new(None, "https://github.com/orgs/rust-lang".into())
                    ]
                )))
            )),
        )),
        (status = 500, description = "Internal server error", content(
            ("application/json", examples(
                ("Internal server error" =
                    (value=json!(ErrorResponse::from(Error::Internal(String::new())))))
            ))
        )),
    )
)]
pub async fn list_links(State(state): State<AppState>) -> Result<(StatusCode, Json<Vec<Link>>)> {
    let links = get_links(&state.db).await?;

    Ok((StatusCode::OK, Json(links)))
}
