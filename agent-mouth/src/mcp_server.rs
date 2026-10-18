use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorData as McpError, Implementation, ServerCapabilities, ServerInfo,
};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use serde::Deserialize;

use crate::config::Config;

#[derive(Clone)]
pub struct MouthMcp {
    config: Config,
    #[allow(dead_code)]
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl MouthMcp {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    pub async fn run(config: Config) -> anyhow::Result<()> {
        let server = Self::new(config);
        let service = server.serve(rmcp::transport::io::stdio()).await?;
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

#[tool_router]
impl MouthMcp {
    #[tool(
        description = "Pass a bash command through tree-sitter AST validation to check if it violates security policy"
    )]
    async fn mouth_validate_ast(
        &self,
        params: Parameters<ValidateAstParams>,
    ) -> Result<CallToolResult, McpError> {
        let report = crate::approval::validate_command(&params.0.command);
        let text = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(
        description = "Request human approval before destructive operations via Slack/Discord webhook"
    )]
    async fn mouth_request_approval(
        &self,
        params: Parameters<RequestApprovalParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let webhook_url = p
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
            p.action, p.message
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

#[tool_handler]
impl ServerHandler for MouthMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.instructions = Some(
            "agent-mouth MCP server. Tools: mouth_validate_ast (validate bash command against security policy), mouth_request_approval (request human approval via webhook)."
                .into(),
        );
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        let mut impl_info = Implementation::default();
        impl_info.name = "agent-mouth".into();
        impl_info.version = env!("CARGO_PKG_VERSION").into();
        info.server_info = impl_info;
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[tokio::test]
    async fn mouth_validate_ast_blocks_dangerous_command() {
        let mcp = MouthMcp::new(Config::default());
        let params = Parameters(ValidateAstParams {
            command: "rm -rf /".into(),
        });
        let result = mcp.mouth_validate_ast(params).await.unwrap();
        let text = result
            .content
            .iter()
            .filter_map(|c| {
                if let RawContent::Text(t) = &c.raw {
                    Some(&t.text[..])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("false"), "expected rejected: {text}");
    }

    #[tokio::test]
    async fn mouth_validate_ast_passes_safe_command() {
        let mcp = MouthMcp::new(Config::default());
        let params = Parameters(ValidateAstParams {
            command: "echo hello".into(),
        });
        let result = mcp.mouth_validate_ast(params).await.unwrap();
        let text = result
            .content
            .iter()
            .filter_map(|c| {
                if let RawContent::Text(t) = &c.raw {
                    Some(&t.text[..])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("true"), "expected approved: {text}");
    }

    #[tokio::test]
    async fn mouth_request_approval_fails_without_webhook() {
        let mcp = MouthMcp::new(Config::default());
        let params = Parameters(RequestApprovalParams {
            action: "delete_cluster".into(),
            message: "Request to delete production cluster".into(),
            webhook_url: None,
        });
        let result = mcp.mouth_request_approval(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("No webhook URL configured"));
    }

    #[test]
    fn mouth_server_info_metadata() {
        let mcp = MouthMcp::new(Config::default());
        let info = mcp.get_info();
        assert_eq!(info.server_info.name, "agent-mouth");
        assert!(info
            .instructions
            .unwrap_or_default()
            .contains("mouth_validate_ast"));
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn mouth_param_schemas_enforce_required_fields() {
        let ast_schema = schemars::schema_for!(ValidateAstParams);
        let ast_json = serde_json::to_value(&ast_schema).expect("schema");
        let req = ast_json
            .get("required")
            .and_then(|v| v.as_array())
            .expect("required array");
        assert!(req.iter().any(|v| v == "command"));

        let approval_schema = schemars::schema_for!(RequestApprovalParams);
        let approval_json = serde_json::to_value(&approval_schema).expect("schema");
        let req2 = approval_json
            .get("required")
            .and_then(|v| v.as_array())
            .expect("required array");
        assert!(req2.iter().any(|v| v == "action"));
        assert!(req2.iter().any(|v| v == "message"));
    }
}
