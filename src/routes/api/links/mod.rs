pub mod create;
pub mod get;
pub mod list;
pub mod redirect;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create::create_new_link))
        .routes(routes!(redirect::redirect_links))
        .routes(routes!(list::list_links))
        .routes(routes!(get::get_specific_link))
}
