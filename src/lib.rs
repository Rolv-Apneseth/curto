#![forbid(unsafe_code)]

pub mod config;
pub mod database;
pub mod error;
pub mod extractors;
pub mod routes;
pub mod utils;

use std::{sync::{Arc, OnceLock}, time::Duration};

use axum::{Router, http::Method};
use axum_prometheus::{GenericMetricLayer, Handle, PrometheusMetricLayer, metrics_exporter_prometheus::PrometheusHandle};
use config::Config;
use database::init_db;
use routes::{Route, api::{links, misc}};
use sqlx::{Pool, Postgres};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor};
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}, limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::routes::api::docs::{self, ApiDoc};

// HACK: workaround for being able to run integration tests
// See: https://github.com/Ptrskay3/axum-prometheus/issues/65
static PROMETHEUS: OnceLock<(
    GenericMetricLayer<'_, PrometheusHandle, Handle>,
    PrometheusHandle,
)> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct AppState {
    metric_handle: PrometheusHandle,
    db: Pool<Postgres>,
}

pub async fn get_app(config: Config) -> Router {
    // Setup database connection
    let db = init_db(&config.database)
        .await
        .expect("could not setup database connection");

    // Governor configuration for rate-limiting
    let governor_conf = Arc::new({
        let mut builder = GovernorConfigBuilder::default().key_extractor(SmartIpKeyExtractor);

        if config.application.shouldratelimit {
            builder.burst_size(10).per_millisecond(200);
        } else {
            builder.burst_size(u32::MAX).per_millisecond(1);
        };

        builder
            .finish()
            .expect("Failed setting up `tower_governor` configuration")
    });

    // CORS
    let cors = CorsLayer::default()
        .allow_methods([Method::GET])
        .allow_origin(Any)
        .max_age(Duration::from_secs(3600));

    // Metrics
    let (prometheus_layer, metric_handle) =
        PROMETHEUS.get_or_init(PrometheusMetricLayer::pair).clone();

    // Compresses response bodies
    let compression_layer = CompressionLayer::new();

    // Limit the size of request bodies to 100KB.
    let request_size_layer = RequestBodyLimitLayer::new(1000 * 100);

    // Application state
    let state = AppState { db, metric_handle };

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(links::routes())
        .merge(misc::routes())
        .fallback(async || error::Error::RouteNotFound)
        // Rate-limiting
        .layer(GovernorLayer {
            config: governor_conf,
        })
        // CORS
        .layer(cors)
        // Request body size limit
        .layer(compression_layer)
        // Response body compression
        .layer(request_size_layer)
        // Metrics
        .layer(prometheus_layer)
        // Logging
        .layer(TraceLayer::new_for_http())
        // Timeout
        .layer(TimeoutLayer::new(Duration::from_secs(8)))
        // STATE
        .with_state(state)
        .split_for_parts();

    // Add API doc routes separately
    router.nest(Route::Docs.into(), docs::routes(api))
}
