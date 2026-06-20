use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WebhookResult {
    pub url: String,
    pub status: u16,
    pub success: bool,
}

pub async fn send_webhook(url: &str, message: &str) -> Result<WebhookResult> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "text": message,
        "source": "agent-mouth",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let resp = client.post(url).json(&payload).send().await?;
    let status = resp.status().as_u16();
    let success = resp.status().is_success();

    let result = WebhookResult {
        url: url.to_string(),
        status,
        success,
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(result)
}
