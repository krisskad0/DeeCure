#!/usr/bin/env bash
# Install or remove systemd user units that run the agent from this checkout (debug build).
# Usage: scripts/dev-agent-units.sh install|uninstall
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
units="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

case "${1:-}" in
  install)
    cargo build -p deecure-agent --manifest-path "$repo/Cargo.toml"
    mkdir -p "$units"
    cp "$repo/packaging/systemd/deecure-agent.socket" "$units/"
    sed "s|^ExecStart=.*|ExecStart=$repo/target/debug/deecure-agent|" \
      "$repo/packaging/systemd/deecure-agent.service" > "$units/deecure-agent.service"
    systemctl --user daemon-reload
    systemctl --user restart deecure-agent.socket
    echo "Installed. The agent starts on the first connection. Try:"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"version\"}' | socat - UNIX-CONNECT:\$XDG_RUNTIME_DIR/deecure/agent.sock"
    ;;
  uninstall)
    systemctl --user stop deecure-agent.service deecure-agent.socket 2>/dev/null || true
    rm -f "$units/deecure-agent.socket" "$units/deecure-agent.service"
    systemctl --user daemon-reload
    echo "Removed."
    ;;
  *)
    echo "Usage: $0 install|uninstall" >&2
    exit 2
    ;;
esac
