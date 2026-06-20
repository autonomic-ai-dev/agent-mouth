# agent-mouth architecture documentation

## Design goals

agent-mouth is the **communication safety layer** for the Autonomic ecosystem. Every command that an agent generates passes through mouth's AST validator before execution. Every notification and approval flows through mouth's webhook system.

### AST command validation

```
agent-mouth validate --command "rm -rf /"
  1. Parse command string with tree-sitter bash grammar
  2. Walk the AST, check each node against the policy:
     - Forbidden commands: rm -rf /, dd, :(){ :|:& };:, etc.
     - Restricted patterns: > /dev/sd*, chmod -R 777, curl | bash
     - Allowed prefixes: cargo, npm, python, git, ls, echo, etc.
  3. Return: { approved: bool, issues: [string], reason: string }
```

The tree-sitter AST parser is chosen over regex for two reasons:

1. **Accuracy.** Regex cannot reliably parse nested bash constructs (pipes, subshells, variable expansion). tree-sitter produces a proper AST.
2. **Extensibility.** The policy is a configuration file, not hardcoded patterns. Adding a new forbidden pattern doesn't require code changes.

### Slack webhook approval

For commands that require human judgment (deploy, publish, delete), mouth sends a Slack webhook:

```
POST /webhook/slack/approval
  {
    "command": "cargo publish",
    "workflow": "release-pipeline",
    "execution_id": "...",
    "approval_url": "https://hooks.slack.com/approve/..."
  }
```

The approval URL connects to mouth's HTTP daemon. When the user clicks "Approve" in Slack, mouth resolves the approval gate in agent-spine, which resumes the paused workflow.

### Key design decisions

| Decision | Rationale |
|----------|-----------|
| **AST validation pre-execution** | Catches dangerous commands before any subprocess spawns. Shell-level protection would only catch failures, not prevent them. |
| **tree-sitter over regex** | Regex-based command safety is fragile. tree-sitter handles pipes, subshells, and complex bash expressions correctly. |
| **Slack webhook for HITL** | Slack is where most engineering teams already communicate. No separate dashboard needed. |
| **stdin-based log summarization** | Works with any pipeline (`command | agent-mouth summarize`). No file format dependencies. |

### Alternatives considered

| Option | Why rejected |
|--------|-------------|
| **Regex-based validation** | Fragile — `rm -rf /` can be obfuscated as `rm -rf "$ROOT"` where ROOT="/". tree-sitter handles variable evaluation. |
| **Deno/Node sandbox** | Heavy runtime dependency for what is fundamentally a bash parsing problem. |
| **Email approvals** | Too slow for agent workflows. Slack webhooks are near-instant. |
| **Built-in TUI approval** | Would require terminal interaction, breaking headless CI/CD. Slack decouples approval from execution context. |
