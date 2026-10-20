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
    /// Start the MCP stdio server
    Mcp,
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
    /// Human approval queue for gated actions (requests wait for approve or deny)
    Approvals {
        #[command(subcommand)]
        action: ApprovalAction,
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

#[derive(Subcommand)]
enum ApprovalAction {
    /// Open a request and wait until it is approved, denied, or times out
    Request {
        /// Unique request id (letters, digits, '-' or '_')
        #[arg(long)]
        id: String,
        /// What is being approved, shown to the approver
        #[arg(long)]
        summary: String,
        /// Seconds to wait before the request times out (fails closed)
        #[arg(long, default_value_t = 300)]
        timeout_secs: u64,
    },
    /// Approve a pending request
    Approve { id: String },
    /// Deny a pending request
    Deny { id: String },
    /// List pending requests
    List,
}

fn approvals_dir() -> std::path::PathBuf {
    agent_body_core::organ_state_dir("mouth").join("approvals")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    apply_progress_env(cli.progress.into());
    match cli.command {
        Commands::Mcp => {
            let config = agent_mouth::config::Config::load()?;
            agent_mouth::mcp_server::MouthMcp::run(config).await?;
        }
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
        Commands::Approvals { action } => {
            let dir = approvals_dir();
            match action {
                ApprovalAction::Request {
                    id,
                    summary,
                    timeout_secs,
                } => {
                    agent_mouth::approvals::open(&dir, &id, &summary)?;
                    let config = agent_mouth::config::Config::load()?;
                    let webhook = &config.notifications.default_webhook;
                    if !webhook.is_empty() {
                        let message = format!(
                            "Approval needed: {id} — {summary}\nRun: agent-mouth approvals approve {id}"
                        );
                        agent_mouth::notify::send_webhook(webhook, &message).await?;
                    }
                    let outcome = agent_mouth::approvals::wait(
                        &dir,
                        &id,
                        std::time::Duration::from_secs(timeout_secs),
                        std::time::Duration::from_millis(500),
                    )?;
                    match outcome {
                        agent_mouth::approvals::Decision::Approved => println!("approved: {id}"),
                        agent_mouth::approvals::Decision::Denied => {
                            eprintln!("denied: {id}");
                            std::process::exit(2);
                        }
                        agent_mouth::approvals::Decision::TimedOut => {
                            eprintln!("timed out waiting for approval: {id} (fails closed)");
                            std::process::exit(3);
                        }
                    }
                }
                ApprovalAction::Approve { id } => {
                    agent_mouth::approvals::decide(
                        &dir,
                        &id,
                        agent_mouth::approvals::Status::Approved,
                    )?;
                    println!("approved: {id}");
                }
                ApprovalAction::Deny { id } => {
                    agent_mouth::approvals::decide(
                        &dir,
                        &id,
                        agent_mouth::approvals::Status::Denied,
                    )?;
                    println!("denied: {id}");
                }
                ApprovalAction::List => {
                    let pending = agent_mouth::approvals::list_pending(&dir)?;
                    if pending.is_empty() {
                        println!("no pending approvals");
                    }
                    for approval in pending {
                        println!("{}\t{}", approval.id, approval.summary);
                    }
                }
            }
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
