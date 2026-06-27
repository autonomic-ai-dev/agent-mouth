# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.6] - 2026-06-27

### Added

- **MCP server** (`agent-mouth mcp`) — starts an MCP stdio server for gateway aggregation. Used by `agent-body serve-mcp`.
- **Organ tool definitions** — `mouth_validate_ast`, `mouth_request_approval` with `#[tool(tool_box)]` naming convention
- **Integration tests** — AST validation tool handler with safe/dangerous commands

## [0.5.5] - 2026-06-23

### Added

- Global `--progress` CLI flag for structured ProgressTree output (also `AGENT_PROGRESS=1`)

## [0.5.4] - 2026-06-21

### Added

- `agent-mouth update [--force]` — self-update subcommand that checks GitHub releases, compares versions, and downloads the latest binary

## [0.5.3] - 2026-06-21

### Added

- `agent-mouth log <name> [--follow] [--list]` — read daemon logs from the supervisor log directory

## [0.5.2] - 2026-06-21

### Fixed

- agent-spine registration is now non-fatal — daemon starts even without spine available

## [0.5.1] - 2026-06-20

### Added

- `--version` CLI flag (`1c99357`)
- Mermaid architecture charts in README (`b28e6ee`)

### Changed

- Professional README with standalone and integrated usage (`5f78c79`)

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
