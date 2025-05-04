use std::net::SocketAddr;

use axum_test::TestServer;
use curto::{config::{AppConfig, DbConfig}, get_app};
use testcontainers_modules::{postgres::{self, Postgres}, testcontainers::{ContainerAsync, runners::AsyncRunner}};
use url::Url;

/// Get a test server using the router that will be used for the actual server
pub async fn get_server() -> (ContainerAsync<Postgres>, TestServer) {
    // Setup test DB
    let container = postgres::Postgres::default()
        .with_password("postgres")
        .with_user("postgres")
        .with_db_name("curto-db")
        .start()
        .await
        .expect("Failed to start test db");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("could not get container port");

    // Test config
    let config = curto::config::Config {
        application: AppConfig {
            shouldratelimit: false,
            ..Default::default()
        },
        database: DbConfig {
            url: Url::parse(&format!(
                "postgresql://0.0.0.0:{port}/curto-db?user=postgres&password=postgres"
            ))
            .unwrap(),
            requiressl: false,
        },
    };

    // Setup application
    let app = get_app(config)
        .await
        .into_make_service_with_connect_info::<SocketAddr>();

    (container, TestServer::new(app).unwrap())
}
