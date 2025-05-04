use tokio::signal;

/// Used for configuring graceful shutdown for the server.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("could not install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("could not install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Returns the default timeout duration for database actions.
pub fn get_default_db_timeout() -> tokio::time::Duration {
    tokio::time::Duration::from_millis(400)
}
