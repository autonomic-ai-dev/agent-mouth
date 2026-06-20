use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-mouth", about = "Communication and notification daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the webhook listener daemon
    Serve,
    /// Send a notification via webhook
    Send {
        /// Webhook URL (or use from config)
        #[arg(short, long)]
        url: Option<String>,
        /// Message text to send
        message: String,
    },
    /// Show configuration and status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve => {
            println!("agent-mouth serve (not yet implemented)");
        }
        Commands::Send { url, message } => {
            let config = agent_mouth::config::Config::load()?;
            let webhook_url = url.unwrap_or_else(|| config.notifications.default_webhook.clone());
            agent_mouth::notify::send_webhook(&webhook_url, &message).await?;
        }
        Commands::Status => {
            let config = agent_mouth::config::Config::load()?;
            println!("agent-mouth status");
            println!("  config: {}", agent_mouth::config::Config::config_path().display());
            println!("  default_webhook: {}", config.notifications.default_webhook);
        }
    }
    Ok(())
}
