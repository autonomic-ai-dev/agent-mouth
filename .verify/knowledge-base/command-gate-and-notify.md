# Command gate and notify

## What it does

agent-mouth decides whether a shell command is safe to run. It parses the command into a bash syntax tree and applies an AST policy. It also sends outbound notifications to a webhook, accepts Slack approval callbacks, and condenses piped logs.

## How it works

1. **Validate a command.** `agent-mouth validate --command "<text>"` parses the text with tree-sitter bash and applies the policy. An allowed command exits 0. A rejected command exits non-zero and names the rule that matched.
2. **Validate a script.** `agent-mouth validate --script <path>` applies the same policy to a multi-line file.
3. **Webhook.** `agent-mouth send <message>` and `POST /webhook/send` post a payload to the configured webhook. A failed post returns an error.
4. **Slack approval.** `POST /webhook/slack/approval` on port 3104 accepts an approval payload from Slack after validating it. An invalid payload is rejected.
5. **Summarize.** `agent-mouth summarize` reads stdin and prints a condensed summary.
6. **MCP.** `mouth_validate_ast` is the gate that agent-body runs before every bash call. `mouth_request_approval` asks for a human approval.

## Where the code is

- Routes: `agent-mouth/agent-mouth/src/serve.rs` lines 36 to 38
- Port: `agent-mouth/src/config.rs`
- Policy: the AST policy in `agent-mouth/src/approval.rs`. The approval check is the second AST policy in the stack, next to agent-muscle's `ast_guard.rs`.

## How to use it

- `agent-mouth validate --command "rm -rf /"` should be rejected
- `agent-mouth send "build finished"` once a webhook URL is in config
- `GET /health` on port 3104 for supervisor probes

## How it is verified

- `cargo test` for the whole crate (36 tests in the recorded baseline), covering the AST policy and webhook code paths
- `route_drift.sh`: the README route table matches source

## Known limits

- A policy rule change changes what every host-side command can run. Each newly allowed or newly rejected pattern needs a regression test.
- The policy is duplicated in agent-muscle's `ast_guard.rs`. Keep the two in step, or move both to one crate.
- No test confirms the webhook end to end, so `send` and the Slack callback are unproven.
- This crate has no `.verify/` allowlist yet, so its feature map cannot run through the gate.

## Related

- Feature map: `agent-mouth/.verify/features/command-gate-and-notify.md`
- Muscle's guard: [trace-and-adaptation](../../../agent-muscle/.verify/knowledge-base/trace-and-adaptation.md)
