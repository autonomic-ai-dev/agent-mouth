# agent-mouth — Context for agents

Ingress / API gateway for Autonomic (AST validation, webhooks, summarize). Part of [agent-body](https://github.com/autonomic-ai-dev/agent-body).

| Field | Value |
|-------|-------|
| Role | Ingress / API gateway |
| Binary | `agent-mouth` |
| HTTP | `:3104` |
| Version | 0.6.0+ |

## When to use

Pre-execution command AST checks, Slack approval webhooks for spine `ApprovalGate`, log summarization (template → local SLM → cloud).

## Key verbs

- AST validate before muscle execute
- Webhook approve/deny for HITL
- `summarize` with hard `max_tokens` cap

## Verification

Approval evidence is mouth's job; test outcomes still go through:

```bash
agent-brain verify --source orchestrator -- <cmd>
```

## CI

Path filter on `agent-mouth/**` + Cargo/Makefile/scripts/`.github`. Docs-only PRs skip compose.

## Related

Spine (gates) · muscle (execute after approve) · brain (verify ledger)
