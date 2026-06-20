use axum::{Json, Router, extract::State, routing::{get, post}};
use std::sync::Arc;
use crate::config::Config;
use crate::notify;
use crate::spine::SpineClient;

pub struct AppState {
    pub config: Config,
    pub spine: SpineClient,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting agent-mouth daemon...");
    let spine = SpineClient::new(&config.spine.url, "agent-mouth", env!("CARGO_PKG_VERSION"));
    spine.register().await?;
    let spine_clone = spine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let _ = spine_clone.heartbeat().await;
        }
    });
    let port = config.server.port;
    let state = Arc::new(AppState { config, spine });
    let app = Router::new()
        .route("/health", get(health))
        .route("/webhook/send", post(webhook_send))
        .with_state(state);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(_): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[derive(serde::Deserialize)]
struct WebhookSendRequest {
    url: Option<String>,
    message: String,
}

async fn webhook_send(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WebhookSendRequest>,
) -> Json<serde_json::Value> {
    let webhook_url = req.url.unwrap_or_else(|| state.config.notifications.default_webhook.clone());
    match notify::send_webhook(&webhook_url, &req.message).await {
        Ok(result) => {
            let _ = state.spine.publish("mouth.sent", &serde_json::json!({
                "url": result.url,
                "status": result.status,
                "success": result.success,
            })).await;
            Json(serde_json::to_value(&result).unwrap_or_default())
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}
