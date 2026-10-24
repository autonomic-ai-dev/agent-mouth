# agent-mouth

`agent-mouth` checks shell commands against the shared safety policy, sends approval requests and notifications to Slack, and condenses log output for operators.

Docs: [autonomic-ai-dev.vercel.app](https://autonomic-ai-dev.vercel.app)

## Quick start

```bash
brew tap autonomic-ai-dev/tap && brew install agent-mouth
agent-mouth validate --command "cargo test"
agent-mouth validate --command "rm -rf /"
```

## Commands

| Command | What it does |
|---------|--------------|
| `agent-mouth validate --command\|--script` | Checks a bash command or script with the AST safety policy |
| `agent-mouth send <message>` | Posts a message to the configured webhook |
| `agent-mouth summarize` | Condenses log text piped on stdin |
| `agent-mouth serve` | Runs the HTTP daemon on port 3104 |
| `agent-mouth mcp` | Runs the MCP stdio server only |
| `agent-mouth status` | Shows the config, webhook targets, and policy |

## HTTP API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Daemon health |
| `POST` | `/webhook/slack/approval` | Validated approval payload |
| `POST` | `/webhook/send` | Outbound notification |

## Configuration

Settings live in the `[mouth]` section of `~/.autonomic/config.toml`. The default port is 3104. Set `webhook_url` to the Slack incoming webhook.

## Develop

```bash
cargo test -p agent-mouth
cargo clippy -p agent-mouth -- -D warnings
```

Feature map and knowledge base: `.verify/features/` and `.verify/knowledge-base/`.
