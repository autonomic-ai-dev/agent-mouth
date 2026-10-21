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

pub async fn send_slack(api_base: &str, token: &str, channel: &str, message: &str) -> Result<()> {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(format!("{api_base}/chat.postMessage"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "channel": channel, "text": message }))
        .send()
        .await?
        .json()
        .await?;
    if resp["ok"].as_bool() != Some(true) {
        anyhow::bail!(
            "slack chat.postMessage failed: {}",
            resp["error"].as_str().unwrap_or("unknown error")
        );
    }
    println!("posted to slack channel {channel}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn mock_slack(reply: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 65536];
            let n = socket.read(&mut buf).await.unwrap();
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                reply.len()
            );
            socket.write_all(head.as_bytes()).await.unwrap();
            socket.write_all(reply.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (base, server)
    }

    #[tokio::test]
    async fn send_slack_posts_to_chat_postmessage_with_bearer_token() {
        let (base, server) = mock_slack(r#"{"ok":true,"channel":"C1"}"#).await;
        send_slack(&base, "xoxb-test", "C1", "hello").await.unwrap();
        let request = server.await.unwrap();
        assert!(request.starts_with("POST /chat.postMessage"));
        assert!(request.contains("authorization: Bearer xoxb-test"));
        assert!(request.contains(r#""text":"hello""#));
    }

    #[tokio::test]
    async fn send_slack_fails_when_slack_rejects_the_message() {
        let (base, server) = mock_slack(r#"{"ok":false,"error":"not_in_channel"}"#).await;
        let err = send_slack(&base, "xoxb-test", "C1", "hello")
            .await
            .unwrap_err();
        server.await.unwrap();
        assert!(err.to_string().contains("not_in_channel"));
    }
}
