# command-gate-and-notify

Summary: agent-mouth decides whether a shell command is safe to run by parsing it into a bash syntax tree, sends outbound notifications to a webhook, accepts Slack approval callbacks, and condenses piped logs.

## Sub-features

- `ast-validate-command`: Rejects destructive or dynamic shell execution in a command string.
  - Intent: Stop dangerous commands before they reach the host shell.
  - Behaviour: `agent-mouth validate --command "<text>"` parses the command with tree-sitter bash and applies the AST policy.
  - Outcome: Exit 0 for an allowed command, non-zero with the rule that matched for a rejected one.
- `ast-validate-script`: Applies the same policy to a script file.
  - Intent: Gate multi-line scripts as well as one-liners.
  - Behaviour: `agent-mouth validate --script <path>`.
  - Outcome: Same allow or reject result as the command form.
- `webhook-send`: Posts a message to the configured webhook.
  - Intent: Notify humans of a run or an approval request.
  - Behaviour: `agent-mouth send <message>` and `POST /webhook/send` (`agent-mouth/src/serve.rs` line 37).
  - Outcome: The webhook receives the payload. Failure returns an error.
- `slack-approval-callback`: Accepts a validated approval payload from Slack.
  - Intent: Let a person approve a gated action from Slack.
  - Behaviour: `POST /webhook/slack/approval` on port 3104 validates the payload before accepting it.
  - Outcome: A valid payload is accepted. An invalid one is rejected.
- `log-summarize`: Summarizes piped log input.
  - Behaviour: `agent-mouth summarize` reads stdin.
  - Outcome: A condensed summary on stdout.
- `mcp-server`: Serves the gate over MCP.
  - Behaviour: `agent-mouth mcp` (stdio) for the gateway. Tools: `mouth_validate_ast`, `mouth_request_approval`.

## Code

- `agent-mouth`

## How to get to it (user POV)

- CLI: `agent-mouth validate`, `send`, `summarize`, `status`, `serve`, `mcp`
- HTTP: `GET /health`, `POST /webhook/send`, `POST /webhook/slack/approval` on `http://localhost:3104` (port confirmed in `agent-mouth/src/config.rs`)
- MCP: `mouth_validate_ast` through agent-body, which runs this gate before every bash call

## Verification Suites (Targeted Test Commands)

- **Unit Tests:** `cargo test` (whole crate, 36 tests in the baseline)
- **Functional API Tests (Hurl):** not yet written
- **Allowlisted Verification:** none yet. This crate has no `.verify/` harness.
- **Preconditions:** Rust toolchain. `send` and the Slack callback need a webhook URL in config.

### Verification Steps

1. **mouth-unit-tests** (sub-features: all)
   - Request: `cargo test`
   - Why: Proves the AST policy and the webhook code paths.
   - Expected: 36 passed, 0 failed (baseline).
   - Impact: A failure means the command gate changed. Fix the cause. Never loosen the policy to pass a test.

## Gotchas

- `ast-validate-command` is the gate that blocked this session's Bash calls. A rule change here changes what every host-side command can run, so it needs a regression test for each newly allowed and newly rejected pattern.
- No test in this pass confirms the webhook, so `webhook-send` and the Slack callback are unproven end to end.
- This crate has no `.verify/` harness. Add one before the feature map can be run through the allowlist.
