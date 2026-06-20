# Changelog

## [Unreleased]

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
