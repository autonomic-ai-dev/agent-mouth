use clap::{Parser, Subcommand, ValueEnum};

use agent_body_core::cli::apply_progress_env;
use agent_body_core::ui::ProgressMode;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProgressArg {
    Auto,
    Plain,
    Quiet,
}

impl From<ProgressArg> for ProgressMode {
    fn from(value: ProgressArg) -> Self {
        match value {
            ProgressArg::Auto => ProgressMode::Auto,
            ProgressArg::Plain => ProgressMode::Plain,
            ProgressArg::Quiet => ProgressMode::Quiet,
        }
    }
}

#[derive(Parser)]
#[command(version)]
#[command(name = "agent-mouth", about = "Communication and notification daemon")]
struct Cli {
    /// Progress output style: auto, plain, or quiet
    #[arg(long, value_enum, global = true, default_value = "auto")]
    progress: ProgressArg,

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
    /// Validate a bash command/script against approval AST policy
    Validate {
        /// Inline command text
        #[arg(long)]
        command: Option<String>,
        /// Script path to validate
        #[arg(long)]
        script: Option<std::path::PathBuf>,
    },
    /// Summarize log input from stdin
    Summarize,
    /// Self-update to the latest GitHub release
    Update {
        /// Force update even if already at latest version
        #[arg(short, long)]
        force: bool,
    },
    /// Read or follow supervisor logs
    Log {
        /// List available log names
        #[arg(long)]
        list: bool,
        /// Follow log output (tail -f style)
        #[arg(short, long)]
        follow: bool,
        /// Name of the log (omitted with --list)
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    apply_progress_env(cli.progress.into());
    match cli.command {
        Commands::Serve => {
            let config = agent_mouth::config::Config::load()?;
            agent_mouth::serve::start(config).await?;
        }
        Commands::Send { url, message } => {
            let config = agent_mouth::config::Config::load()?;
            let webhook_url = url.unwrap_or_else(|| config.notifications.default_webhook.clone());
            agent_mouth::notify::send_webhook(&webhook_url, &message).await?;
        }
        Commands::Status => {
            let config = agent_mouth::config::Config::load()?;
            println!("agent-mouth status");
            println!(
                "  config: {}",
                agent_mouth::config::Config::config_path().display()
            );
            println!("  port: {}", config.server.port);
            println!("  spine: {}", config.spine.url);
        }
        Commands::Summarize => {
            agent_mouth::summarize::summarize()?;
        }
        Commands::Log { list, follow, name } => {
            if list {
                let names = agent_mouth::log::list_logs()?;
                if names.is_empty() {
                    println!("no logs found");
                } else {
                    for n in &names {
                        println!("{n}");
                    }
                }
            } else if let Some(n) = name {
                if follow {
                    agent_mouth::log::follow_log(&n)?;
                } else {
                    agent_mouth::log::print_log(&n)?;
                }
            } else {
                anyhow::bail!("provide --list or a log name");
            }
        }
        Commands::Update { force } => {
            agent_mouth::update::run_update(force)?;
        }
        Commands::Validate { command, script } => {
            let report = match (command, script) {
                (Some(cmd), None) => agent_mouth::approval::validate_command(&cmd),
                (None, Some(path)) => agent_mouth::approval::validate_script(&path)?,
                _ => anyhow::bail!("provide exactly one of --command or --script"),
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.approved {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
