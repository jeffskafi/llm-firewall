// crates/lambda/src/main.rs
use firewall_core::AppConfig;
use http::build_router;
use lambda_http::{run, service_fn, Error as LambdaError, Request};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let app = build_router(AppConfig::default()).await;   // ← await here

    run(service_fn(move |event: Request| {
        let app = app.clone();
        async move {
            app.oneshot(event).await.map_err(LambdaError::from)
        }
    }))
    .await
}
