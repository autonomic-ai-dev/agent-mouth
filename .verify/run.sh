#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/.." && pwd)"

echo "==> [agent-mouth] Starting verification suite..."
cd "$REPO_ROOT"

echo "==> Running cargo test -p agent-mouth..."
cargo test -p agent-mouth

echo "==> Running cargo clippy -p agent-mouth (warnings are errors)..."
cargo clippy -p agent-mouth --all-targets -- -D warnings

echo "==> [agent-mouth] Verification passed cleanly (exit 0)."
