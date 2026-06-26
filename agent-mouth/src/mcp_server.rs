use rmcp::model::{CallToolResult, Content, ErrorData as McpError, ServerInfo};
use rmcp::serve_server;
use rmcp::tool;
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::config::Config;

#[derive(Clone)]
pub struct MouthMcp {
    config: Config,
}

impl MouthMcp {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(config: Config) -> anyhow::Result<()> {
        let server = Self::new(config);
        let service = serve_server(server, rmcp::transport::io::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ValidateAstParams {
    command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RequestApprovalParams {
    message: String,
    action: String,
    #[serde(default)]
    webhook_url: Option<String>,
}

#[tool(tool_box)]
impl MouthMcp {
    #[tool(description = "Pass a bash command through tree-sitter AST validation to check if it violates security policy")]
    async fn mouth_validate_ast(
        &self,
        #[tool(aggr)] params: ValidateAstParams,
    ) -> Result<CallToolResult, McpError> {
        let report = crate::approval::validate_command(&params.command);
        let text =
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Request human approval before destructive operations via Slack/Discord webhook")]
    async fn mouth_request_approval(
        &self,
        #[tool(aggr)] params: RequestApprovalParams,
    ) -> Result<CallToolResult, McpError> {
        let webhook_url = params
            .webhook_url
            .unwrap_or_else(|| self.config.notifications.default_webhook.clone());

        if webhook_url.is_empty() {
            return Err(McpError::internal_error(
                "No webhook URL configured. Set notifications.default_webhook in config or pass webhook_url parameter.",
                None,
            ));
        }

        let message = format!(
            "[APPROVAL REQUIRED] Action: {}\n\nMessage: {}",
            params.action, params.message
        );

        match crate::notify::send_webhook(&webhook_url, &message).await {
            Ok(result) => {
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for MouthMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "agent-mouth MCP server. Tools: mouth_validate_ast (validate bash command against security policy), mouth_request_approval (request human approval via webhook)."
                    .into(),
            ),
            ..Default::default()
        }
    }
}
