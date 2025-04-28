// crates/cli/src/main.rs
use anyhow::Result;
use firewall_core::AppConfig;
use http::build_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cfg     = AppConfig::default();
    let router  = build_router(cfg).await;

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}