use axum::Router;
use utoipa::{OpenApi, openapi};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub const ROUTE_SWAGGER_UI: &str = "/swagger/";
pub const ROUTE_API_FILE: &str = "/api.json";

#[derive(OpenApi)]
#[openapi(info(
    title = "Curto API",
    description = "Easy-to-use URL shortener",
    license(name = "AGPLv3"),
))]
pub struct ApiDoc;

pub fn routes(api: openapi::OpenApi) -> Router {
    Router::new()
        // TODO: figure out how to make the swagger UI work with the API file in this nested route
        .merge(SwaggerUi::new(ROUTE_SWAGGER_UI).url(ROUTE_API_FILE, api.clone()))
        .merge(Scalar::with_url("/", api).custom_html(include_str!("./scalar.html")))
}
