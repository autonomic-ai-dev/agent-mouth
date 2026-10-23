# agent-mouth features

## Baseline preconditions

- Rust toolchain matching `Cargo.lock`.
- `send` and the Slack callback need a webhook URL in config.

## Driving conventions

- Unit tests run through `cargo test`. Logs go to `.verify/artifacts/<RUN_ID>/`.

## Proof standards

- The command gate is proven when allowed and rejected commands both produce the documented result. Each new policy rule needs a regression test in both directions.

## Feature index

| Feature file | Sub-features | Primary surface |
|---|---|---|
| `command-gate-and-notify.md` | 6 | `validate --command/--script`, `send`, `summarize`, `mcp`, HTTP `/webhook/slack/approval`, `/webhook/send` |
