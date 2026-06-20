# agent-mouth

**Communication and notification daemon — webhook sending and event listening.**

agent-mouth sends notifications via webhooks and listens for events to relay to chat platforms.

---

## Why agent-mouth?

| Problem | agent-mouth answer |
|---------|-------------------|
| "I want my agent to notify Slack" | **Webhook sender** — POST JSON payload to any webhook URL |
| "How do I hook events into ChatOps?" | **Event listening** — future NATS subscriber for automated notifications |

## Commands

| Command | Description |
|---------|-------------|
| `agent-mouth serve` | Start daemon (future: event listener) |
| `agent-mouth send <message>` | Send a notification via webhook |
| `agent-mouth status` | Show config |

---

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-mouth/master/scripts/install.sh | bash
```

## Development

```bash
cargo build --release -p agent-mouth
cargo test --release -p agent-mouth
```

## License

MIT
