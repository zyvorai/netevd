#!/usr/bin/env bash
# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: Apache-2.0
# Deploy netevd to a remote host and prove a veth pair fires hooks.
#
# Usage (from repo root):
#   ./scripts/deploy-remote.sh <host> [user]
#   ./scripts/deploy-remote.sh 175.110.122.71 sus
#
# Builds on the remote with the user's cargo (including --features ebpf),
# installs the systemd unit + BPF object, and creates a veth pair named
# veth-netevd0/veth-netevd1. The installed config matches only that pair
# so other veths on the host are ignored.

set -euo pipefail

HOST="${1:-}"
USER="${2:-sus}"
[ -n "$HOST" ] || { echo "Usage: $0 <host> [user]" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SSH_OPTS=(-o StrictHostKeyChecking=accept-new -o ServerAliveInterval=15 -o ServerAliveCountMax=6)
REMOTE_DIR=".deployment/netevd"
VETH_A="veth-netevd0"
VETH_B="veth-netevd1"
VETH_IP_A="192.168.200.1/24"
VETH_IP_B="192.168.200.2/24"

info() { printf '==> %s\n' "$*"; }

info "Rsync source to ${USER}@${HOST}:~/${REMOTE_DIR}"
ssh "${SSH_OPTS[@]}" "${USER}@${HOST}" "mkdir -p \"\$HOME/${REMOTE_DIR}\""
rsync -az --delete \
  -e "ssh ${SSH_OPTS[*]}" \
  --exclude '.git/' \
  --exclude 'target/' \
  --exclude '.deployment/' \
  "${REPO_DIR}/" "${USER}@${HOST}:${REMOTE_DIR}/"

info "Build release binary + eBPF object on ${HOST}"
ssh "${SSH_OPTS[@]}" "${USER}@${HOST}" "bash -s" <<EOF
set -euo pipefail
export PATH="\$HOME/.cargo/bin:\$PATH"
cd "\$HOME/${REMOTE_DIR}"
sudo apt-get update -qq
sudo apt-get install -y -qq pkg-config libdbus-1-dev build-essential clang llvm \
  linux-libc-dev libbpf-dev >/dev/null
make -C ebpf
cargo build --release --features ebpf
EOF

info "Install binary, BPF object, unit, lab config, and hook"
ssh "${SSH_OPTS[@]}" "${USER}@${HOST}" "bash -s" <<EOF
set -euo pipefail
cd "\$HOME/${REMOTE_DIR}"
sudo useradd --system --no-create-home --shell /usr/sbin/nologin netevd 2>/dev/null || true
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 ebpf/netevd-ebpf.o /usr/lib/netevd/netevd-ebpf.o
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo mkdir -p /etc/netevd /etc/systemd/system/netevd.service.d
sudo install -Dm644 systemd/netevd-ebpf.conf /etc/systemd/system/netevd.service.d/ebpf.conf
sudo tee /etc/systemd/system/netevd.service.d/state.conf >/dev/null <<'UNIT'
[Service]
StateDirectory=netevd
ReadWritePaths=/var/lib/netevd
UNIT
sudo tee /etc/netevd/netevd.yaml >/dev/null <<'YAML'
system:
  log_level: "info"
  backend: "systemd-networkd"
monitoring:
  interfaces: []
  match_patterns:
    - "veth-netevd*"
  exclude:
    - "lo"
    - "docker*"
    - "br-*"
    - "cni*"
    - "flannel*"
    - "cilium*"
    - "virbr*"
hooks:
  debounce_ms: 50
  timeout_sec: 30
  max_parallel: 1
routing:
  policy_rules: []
backends:
  systemd_networkd:
    emit_json: true
  dhclient: {}
  networkmanager: {}
api:
  enabled: true
  bind_address: "127.0.0.1"
  port: 9090
metrics:
  enabled: false
audit:
  enabled: false
ebpf:
  enabled: true
  drops: true
  tcp_retransmit: false
  tcp_reset: true
  debounce_ms: 250
  min_count: 8
  object_path: ""
  skip_unknown_ifindex: true
  reasons_allow: []
  reasons_deny: []
YAML
sudo mkdir -p /etc/netevd/link-added.d /etc/netevd/address-added.d /etc/netevd/drops.d /etc/netevd/tcp-reset.d
sudo tee /etc/netevd/link-added.d/01-log.sh >/dev/null <<'HOOK'
#!/bin/sh
printf '%s %s %s\n' "\$(date -Is)" "\$EVENT" "\$LINK" >> /var/lib/netevd/events.log
HOOK
sudo tee /etc/netevd/drops.d/01-log.sh >/dev/null <<'HOOK'
#!/bin/sh
printf '%s %s %s reason=%s proto=%s sport=%s dport=%s n=%s\n' \
  "\$(date -Is)" "\$EVENT" "\$LINK" "\${DROP_REASON:-}" "\${PROTOCOL:-}" \
  "\${SPORT:-}" "\${DPORT:-}" "\${COUNT:-}" >> /var/lib/netevd/events.log
HOOK
sudo tee /etc/netevd/tcp-reset.d/01-log.sh >/dev/null <<'HOOK'
#!/bin/sh
printf '%s %s %s reason=%s sport=%s dport=%s n=%s\n' \
  "\$(date -Is)" "\$EVENT" "\$LINK" "\${DROP_REASON:-}" \
  "\${SPORT:-}" "\${DPORT:-}" "\${COUNT:-}" >> /var/lib/netevd/events.log
HOOK
sudo cp /etc/netevd/link-added.d/01-log.sh /etc/netevd/address-added.d/01-log.sh
sudo chmod 755 /etc/netevd/link-added.d/01-log.sh /etc/netevd/address-added.d/01-log.sh \
  /etc/netevd/drops.d/01-log.sh /etc/netevd/tcp-reset.d/01-log.sh
sudo systemctl daemon-reload
sudo systemctl enable --now netevd
sudo systemctl restart netevd
sleep 2
sudo systemctl is-active netevd
EOF

info "Create veth pair ${VETH_A} <-> ${VETH_B} and check hooks + eBPF attach"
ssh "${SSH_OPTS[@]}" "${USER}@${HOST}" "bash -s" <<EOF
set -euo pipefail
sudo -n ip netns del nvtest 2>/dev/null || true
sudo -n ip link del ${VETH_A} 2>/dev/null || true
sudo -n rm -f /var/lib/netevd/events.log
sudo -n ip link add ${VETH_A} type veth peer name ${VETH_B}
sudo -n ip netns add nvtest
sudo -n ip link set ${VETH_B} netns nvtest
sudo -n ip addr add ${VETH_IP_A} dev ${VETH_A}
sudo -n ip link set ${VETH_A} up
sudo -n ip netns exec nvtest ip addr add ${VETH_IP_B} dev ${VETH_B}
sudo -n ip netns exec nvtest ip link set ${VETH_B} up
sleep 2
sudo -n ip netns exec nvtest ping -c 2 -W 2 ${VETH_IP_A%/*}
# Allow coalesce window for observe hooks (debounce_ms * a few ticks).
sleep 6
echo '--- events.log (head) ---'
sudo -n head -n 40 /var/lib/netevd/events.log
echo '--- journal (attach) ---'
sudo -n journalctl -u netevd --no-pager -n 5 --grep='eBPF observe-only programs attached'
sudo -n grep -q "link-added ${VETH_A}" /var/lib/netevd/events.log
sudo -n grep -q "link-added ${VETH_B}" /var/lib/netevd/events.log
sudo -n grep -q "address-added ${VETH_A}" /var/lib/netevd/events.log
sudo -n grep -qE ' (drops|tcp-reset|tcp-retransmit) ' /var/lib/netevd/events.log
ATTACH="\$(sudo -n journalctl -u netevd --no-pager -n 1 --grep='eBPF observe-only programs attached' || true)"
echo "\$ATTACH" | grep -q 'eBPF observe-only programs attached'
echo "\$ATTACH" | grep -q 'attachments=3'
sudo -n ip netns del nvtest
echo 'veth + eBPF attach + observe drain test passed'
EOF

info "Deployed netevd on ${USER}@${HOST}; veth + eBPF attach test passed"
