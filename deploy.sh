#!/usr/bin/env bash
set -euo pipefail

BIN="${BIN:-target/release/rpc-monitor}"
REMOTE_BIN="${REMOTE_BIN:-/usr/local/bin/rpc-monitor}"
SSH_USER="${SSH_USER:-root}"
HOSTS=(
  "mon-us-east.rpc-providers.net"
  "mon-eu-central.rpc-providers.net"
)

if [[ ! -x "$BIN" ]]; then
  echo "Missing executable binary: $BIN" >&2
  echo "Build it first with: cargo build --release" >&2
  exit 1
fi

for host in "${HOSTS[@]}"; do
  target="${SSH_USER}@${host}"
  tmp="/tmp/rpc-monitor.$$.new"

  echo "Deploying $BIN to ${target}:${REMOTE_BIN}"
  scp "$BIN" "${target}:${tmp}"
  ssh "$target" "install -m 0755 '$tmp' '$REMOTE_BIN' && rm -f '$tmp' && '$REMOTE_BIN' --help >/dev/null"
done

echo "Deploy complete."
