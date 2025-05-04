use std::net::SocketAddr;

use curto::{config::Config, get_app, routes::Route, utils::shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::get_config().expect("Failed to read configuration");

    let app = get_app(config.clone()).await;

    let addr = SocketAddr::from((config.application.host, config.application.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed binding listener");

    tracing::info!("Listening at http://{}", listener.local_addr().unwrap());
    tracing::info!(
        "API docs available at http://{}{}",
        listener.local_addr().unwrap(),
        Route::Docs.as_str()
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Failed to start server");
}
