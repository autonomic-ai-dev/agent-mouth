#!/usr/bin/env bash
# Build agent-mouth release and adhoc-sign on macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo build --release -p agent-mouth
"$ROOT/scripts/sign-macos.sh"
