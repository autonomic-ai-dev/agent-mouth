#!/usr/bin/env bash
# Build agent-mouth release and Developer ID–sign on macOS (Apple Silicon).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo build --release -p agent-mouth
"$ROOT/scripts/sign-macos.sh"
