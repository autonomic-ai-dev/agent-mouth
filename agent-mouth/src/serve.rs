use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::approval;
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
    if let Err(e) = spine.register().await {
        tracing::warn!(error = %e, "Failed to register with agent-spine, continuing without registration");
    }
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
        .route("/webhook/slack/approval", post(slack_approval))
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
    let webhook_url = req
        .url
        .unwrap_or_else(|| state.config.notifications.default_webhook.clone());
    match notify::send_webhook(&webhook_url, &req.message).await {
        Ok(result) => {
            let _ = state
                .spine
                .publish(
                    "mouth.sent",
                    &serde_json::json!({
                        "url": result.url,
                        "status": result.status,
                        "success": result.success,
                    }),
                )
                .await;
            Json(serde_json::to_value(&result).unwrap_or_default())
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn slack_approval(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let signing_secret = &state.config.slack.signing_secret;
    if !signing_secret.is_empty() {
        let timestamp = headers
            .get("x-slack-request-timestamp")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let signature = headers
            .get("x-slack-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !approval::verify_slack_signature(signing_secret, timestamp, &body, signature) {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let body_str = String::from_utf8_lossy(&body);
    let command = extract_slack_command(&body_str);
    let report = approval::validate_command(&command);

    let subject = if report.approved {
        "mouth.approval.validated"
    } else {
        "mouth.approval.rejected"
    };
    let _ = state
        .spine
        .publish(
            subject,
            &serde_json::json!({
                "approved": report.approved,
                "reason": report.reason,
                "command": report.command,
                "issues": report.issues,
            }),
        )
        .await;

    Ok(Json(serde_json::json!({
        "response_type": "in_channel",
        "text": if report.approved {
            format!("✅ Approved: {}", report.reason)
        } else {
            format!("❌ Rejected: {}", report.reason)
        },
        "report": report,
    })))
}

fn extract_slack_command(body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
            return text.to_string();
        }
        if let Some(actions) = json.get("actions").and_then(|v| v.as_array()) {
            if let Some(value) = actions
                .first()
                .and_then(|a| a.get("value"))
                .and_then(|v| v.as_str())
            {
                return value.to_string();
            }
        }
        if let Some(message) = json.pointer("/message/text").and_then(|v| v.as_str()) {
            return message.to_string();
        }
    }

    if body.starts_with("payload=") {
        if let Ok(decoded) = urlencoding::decode(&body["payload=".len()..]) {
            return extract_slack_command(&decoded);
        }
    }

    body.lines().next().unwrap_or(body).to_string()
}
