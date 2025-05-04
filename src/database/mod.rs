mod links;
use std::str::FromStr;

use sqlx::{PgPool, postgres::{PgConnectOptions, PgPoolOptions, PgSslMode}};

pub use self::links::*;
use crate::config::DbConfig;

pub async fn init_db(config: &DbConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(config.url.as_str())
        .expect("Invalid Database URL")
        .ssl_mode(if config.requiressl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        });

    let pool = PgPoolOptions::new()
        .min_connections(2)
        .max_connections(20)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    Ok(pool)
}
