# Changelog

## [v] - 2026-06-20

### Added
- Added Mermaid charts to README
- Added `--version` flag to CLI parser


## [Unreleased]

## [0.5.0] - 2026-06-20

### Added

- **Slack approval AST validation** — `POST /webhook/slack/approval` verifies signing secret and blocks destructive bash patterns
- **`agent-mouth validate`** — CLI for `--command` or `--script` approval checks
- Publishes `mouth.approval.validated` / `mouth.approval.rejected` to agent-spine

## [0.4.0] - 2026-06-20

### Added

- **Unified config** — loads from `~/.autonomic/config.toml` via `agent-body-core::organ_config::load("mouth")`

### Changed

- Version bumped from `0.3.0` to `0.4.0`

## [0.3.0] - 2026-06-20

### Added

- **Log summarizer** — `cat logs.txt | agent-mouth summarize` reads stdin, detects errors/warnings, and outputs a structured summary with bullet points
- **API summarization** — Uses OpenAI (via `OPENAI_API_KEY`) or Anthropic (via `ANTHROPIC_API_KEY`) when available for AI-powered summaries
- **Local fallback** — Keyword-based analysis when no API key is configured

### Changed

- Version bumped from `0.2.0` to `0.3.0`

## [0.2.0] - 2026-06-20

### Added

- **HTTP daemon** — `agent-mouth serve` now starts an axum HTTP server with `/health` and `/webhook/send` endpoints
- **Agent-spine integration** — registers with agent-spine event bus on startup, heartbeats every 30s, publishes `mouth.sent` events
- **Config extended** — `server.port` (default 3104) and `spine.url` (default `http://localhost:3100`) settings

### Changed

- Version bumped from `0.1.0` to `0.2.0`

## [0.1.0] - 2026-06-20

### Added

- **Initial project scaffold** — workspace, crate, config
- **Webhook sender** — sends JSON payload via HTTP POST with reqwest
- **CLI** — `agent-mouth serve` (daemon placeholder), `send <message>` (send webhook), `status`
- **CI pipeline** — test + build + release workflows
