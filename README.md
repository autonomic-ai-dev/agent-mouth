# agent-mouth

**Communication and approvals — Slack webhooks with AST-safe command validation.**

`agent-mouth` sends webhook notifications and validates approval payloads (e.g. blocking `rm -rf` in shell snippets) before agent-spine or muscle act on them.

Standalone: `agent-mouth send` · Integrated: `POST /webhook/slack/approval`, spine registration on **3104**.

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-mouth/master/scripts/install.sh | bash
```

---

## Quick start

```bash
agent-mouth status
agent-mouth send "deploy complete"
agent-mouth validate --command "cargo test"
agent-mouth serve
```

---

## Commands

| Command | Description |
|---------|-------------|
| `serve` | HTTP daemon with webhook routes |
| `send <message>` | POST to configured webhook URL |
| `validate --command\|--script` | tree-sitter bash AST gate |
| `status` | Config and webhook target |

---

## HTTP API

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Daemon health |
| `POST /webhook/slack/approval` | Validated approval payload |
| `POST /send` | Outbound notification |

---

## Configuration

Section `[mouth]` in `~/.autonomic/config.toml` (default port **3104**).

---

## Development

```bash
cargo test --release -p agent-mouth
```

---

## License

MIT
