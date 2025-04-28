// crates/http/src/lib.rs  – replace proxy()
use axum::{
    extract::{Json, Extension},
    http::StatusCode,
    response::IntoResponse,
    Router,
    routing::post,
};
use firewall_core::{Firewall, FirewallInput, AppConfig};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct ProxyReq {
    prompt: String,
    model:  String,
}

async fn proxy(
    Extension(fw): Extension<Arc<Firewall>>,
    Json(body): Json<ProxyReq>,
) -> impl IntoResponse {
    let verdict = fw.evaluate(&FirewallInput {
        prompt: body.prompt,
        model: body.model,
        user_id: None,
    });

    match verdict {
        Ok(v) if v.allowed => (
            StatusCode::OK,
            "🛡 Allowed (placeholder: forward to upstream LLM)".to_string(),
        ),
        Ok(v) => (
            StatusCode::FORBIDDEN,
            format!("Blocked – {}", v.reason.unwrap_or_default()),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

pub async fn build_router(cfg: AppConfig) -> Router {
    let fw = Firewall::new(cfg).await.unwrap();
    let fw = Arc::new(fw);
    Router::new()
        .route("/proxy", post(proxy))
        .layer(Extension(fw))
}
