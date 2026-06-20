use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub notifications: NotificationConfig,
    pub slack: SlackConfig,
    pub spine: SpineConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub default_webhook: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Slack signing secret for interactive webhook verification.
    #[serde(default)]
    pub signing_secret: String,
    /// Optional default channel id for approval responses.
    #[serde(default)]
    pub approval_channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpineConfig {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig { port: 3104 },
            notifications: NotificationConfig {
                default_webhook: String::new(),
            },
            slack: SlackConfig {
                signing_secret: String::new(),
                approval_channel: String::new(),
            },
            spine: SpineConfig {
                url: "http://localhost:3100".into(),
            },
            logging: LoggingConfig {
                level: "info".into(),
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        agent_body_core::config_path()
    }

    pub fn load() -> Result<Self> {
        agent_body_core::organ_config::load("mouth")
    }
}
