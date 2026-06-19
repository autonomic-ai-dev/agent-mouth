# agent-mouth

**Communication and ChatOps layer for AI agents — translating complex JSON workflows into human-readable Slack/Discord summaries.**

agent-mouth is the voice of the organism. It bridges the gap between deterministic machine execution and human developers by summarizing logs, requesting approvals, and listening for ChatOps triggers.

Rust is the vocal cord; Slack/Discord are the megaphone.

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-mouth/master/scripts/install.sh | bash -s -- --global
agent-mouth serve --daemon
```

**MCP is live immediately** — the agent can dynamically send messages or request explicit Human-In-The-Loop approvals via chat.

---

## Why agent-mouth?

As autonomous agents become highly capable, developers shift from writing code to reviewing code and monitoring workflows. 

1. **Information Overload:** When an agent runs a massive pipeline, it generates thousands of lines of JSON logs. Humans cannot read this quickly.
2. **Approval Bottlenecks:** If a workflow hits an `ApprovalGate` and pauses, the developer needs to know *immediately*, not the next time they check the terminal.
3. **Remote Control:** Developers want to trigger workflows from their phone while on the train (e.g., "Rollback the deployment").

**agent-mouth fixes this with an asynchronous communication bridge:**

| Problem | agent-mouth answer |
|---------|-------------------|
| "I have to read 500 lines of JSON logs" | **Summarization** — uses a highly quantized local LLM to turn raw stack traces into concise, plain-English executive summaries. |
| "The agent needs my permission to deploy" | **Slack/Discord Integration** — posts proactive updates and renders interactive HITL (Human-In-The-Loop) approval buttons directly in chat. |
| "I want to trigger the agent from my phone" | **ChatOps Triggers** — listens for human commands in Slack and translates them into `agent-spine` DAG workflow triggers. |

---

## Architectural Deep Dive

`agent-mouth` is essentially a bi-directional translation engine sitting between `agent-nerves` and external webhook APIs.

### 1. Inbound: Executive Summarization
When `agent-spine` completes a complex 50-node workflow, it publishes a massive JSON state payload.
- `agent-mouth` consumes this payload.
- It queries `agent-brain` to retrieve the original user intent.
- It passes the JSON and the intent through a local LLM with a strict system prompt: *"Summarize this workflow outcome in 3 bullet points."*
- It posts the human-readable summary to a Slack channel.

### 2. Outbound: ChatOps to DAG
When a developer tags `@AutonomicAI deploy the staging branch` in Slack:
- `agent-mouth` intercepts the Slack webhook.
- It parses the intent and looks up the corresponding declarative YAML workflow.
- It publishes a trigger event to `agent-nerves`, causing `agent-spine` to wake up and start executing.

---

## Complete Setup (Copy & Paste)

### 1. Install the binary

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-mouth/master/scripts/install.sh | bash -s -- --global
```

### 2. Configuration (`~/.agent_mouth/config.yaml`)

```yaml
integrations:
  slack:
    enabled: true
    bot_token_env: "SLACK_BOT_TOKEN"
    channel: "#ai-ops"
  discord:
    enabled: false

summarization:
  engine: "local_llm"  # or "openai"
  model: "qwen2.5-1.5b"
```

### 3. Verify

```bash
agent-mouth version
agent-mouth ping --slack
```

---

## Commands

| Command | Description |
|---------|-------------|
| `agent-mouth serve` | Start the bi-directional chat daemon |
| `agent-mouth broadcast <msg>` | Send a manual test message to channels |
| `agent-mouth log tail` | Watch the translation stream locally |

---

## Development

```bash
cargo test --release -p agent-mouth
cargo build --release -p agent-mouth
```

## License
MIT
