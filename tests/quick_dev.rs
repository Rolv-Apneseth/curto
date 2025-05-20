use std::error::Error;

use curto::{config::{AppConfig, Config}, extractors::Json, routes::{Route, api::docs::ROUTE_API_FILE}};
use serde_json::json;

#[ignore = "Only used for convenient development"]
#[tokio::test]
async fn quick_dev() -> Result<(), Box<dyn Error>> {
    let AppConfig { host, port, .. } = Config::get_config()
        .expect("Failed to read configuration")
        .application;
    let hc = httpc_test::new_client(format!(
        "http://{}.{}.{}.{}:{port}",
        host[0], host[1], host[2], host[3]
    ))?;

    // HEALTH
    // hc.do_get(Route::Health.into()).await?.print().await?;
    // METRICS
    // hc.do_get(Route::Metrics.into()).await?.print().await?;
    // DOCS
    // hc.do_get(Route::Docs.into()).await?.print().await?;
    // hc.do_get(&format!("{}{}", Route::Docs.as_str(), ROUTE_API_FILE))
    //     .await?
    //     .print()
    //     .await?;

    // LINKS
    // Create a link
    // hc.do_post(
    //     Route::LinkCreate.into(),
    //     json!(Link::new(None, "https://crates.io".into())),
    // )
    // .await?
    // .print()
    // .await?;

    // List existing links
    hc.do_get(Route::Links.into()).await?.print().await?;

    Ok(())
}
