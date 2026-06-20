#!/usr/bin/env bash
# Install agent-mouth from latest GitHub release.
set -euo pipefail

REPO="autonomic-ai-dev/agent-mouth"

detect_arch() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) echo "unsupported-arch-$arch" >&2; exit 1 ;;
  esac
}

detect_os() {
  local os
  os="$(uname -s)"
  case "$os" in
    Darwin) echo "apple-darwin" ;;
    Linux)  echo "unknown-linux-gnu" ;;
    *)      echo "unsupported-os-$os" >&2; exit 1 ;;
  esac
}

main() {
  local os arch asset tag url
  os="$(detect_os)"
  arch="$(detect_arch)"
  asset="agent-mouth-${arch}-${os}"

  echo "Fetching latest release for $os/$arch ..." >&2
  tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | awk -F'"' '/"tag_name"/{print $4}')"
  if [[ -z "$tag" ]]; then
    echo "Failed to fetch latest tag" >&2
    exit 1
  fi
  url="https://github.com/$REPO/releases/download/$tag/$asset"

  echo "Downloading $asset ($tag) ..." >&2
  install -d "$HOME/.local/bin"
  curl -fsSL "$url" -o "$HOME/.local/bin/agent-mouth"
  chmod +x "$HOME/.local/bin/agent-mouth"

  echo "Installed agent-mouth $tag to $HOME/.local/bin/agent-mouth" >&2
}

main "$@"
