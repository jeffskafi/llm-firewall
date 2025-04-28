// crates/http/src/lib.rs  – replace proxy()
use axum::{
    extract::{Json, Extension},
    http::StatusCode,
    response::IntoResponse,
    Router as HttpRouter,
    routing::post,
};
use firewall_core::{Firewall, FirewallInput, AppConfig};
use std::sync::Arc;

use gateway::{Router as ProviderRouter, UpReq};
use gateway::provider::{OpenAI, Anthropic, Ollama};

#[derive(serde::Deserialize)]
struct ProxyReq {
    prompt: String,
    model:  String,
}

async fn proxy(
    Extension(fw): Extension<Arc<Firewall>>,
    Extension(router): Extension<ProviderRouter>,
    Json(body): Json<ProxyReq>,
) -> impl IntoResponse {
    // 1. firewall check
    let verdict = fw.evaluate(&FirewallInput {
        prompt: body.prompt.clone(),
        model:  body.model.clone(),
        user_id: None,
    });

    if let Ok(v) = &verdict {
        if !v.allowed {
            return (
                StatusCode::FORBIDDEN,
                format!("Blocked – {}", v.reason.clone().unwrap_or_default()),
            )
                .into_response();
        }
    }

    // 2. forward to provider
    let provider = match router.route(&body.model) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "unknown model").into_response(),
    };

    let upstream_req = UpReq {
        model: body.model,
        messages: vec![/* build from body.prompt */],
        extra: serde_json::json!({}),
    };

    match provider.chat(upstream_req).await {
        Ok(res) => (
            StatusCode::from_u16(res.status).unwrap_or(StatusCode::OK),
            res.body,
        )
            .into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

pub async fn build_router(cfg: AppConfig) -> HttpRouter {
    let fw = Arc::new(Firewall::new(cfg).await.unwrap());

    // Build provider registry
    let mut r = ProviderRouter::new();
    r.add_prefix(
        "gpt-",
        OpenAI {
            base: "https://api.openai.com".into(),
            key: std::env::var("OPENAI_KEY")
                .expect("OPENAI_KEY not set"),
        },
    );
    r.add_prefix(
        "claude-",
        Anthropic {
            api_key: std::env::var("ANTHROPIC_KEY")
                .expect("ANTHROPIC_KEY not set"),
        },
    );
    r.add_prefix(
        "llama3-",
        Ollama {
            base: "http://127.0.0.1:11434".into(),
        },
    );

    HttpRouter::new()
        .route("/proxy", post(proxy))
        .layer(Extension(fw))
        .layer(Extension(r))
}