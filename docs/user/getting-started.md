---
hero:
  eyebrow: USER GUIDE
  title: Getting started
---

## Prerequisites

- Linux host with Netlink
- One of: `systemd-networkd`, NetworkManager, or `dhclient`
- Root/`sudo` to install the binary and unit
- Optional: Rust toolchain to build from source

## 1. Install

**Release tarball (recommended):**

```bash
curl -LO https://github.com/zyvorai/netevd/releases/download/v0.4.1/netevd-0.4.1-linux-amd64.tar.gz
tar xzf netevd-*-linux-amd64.tar.gz && cd netevd-*-linux-amd64
sudo ./install.sh
sudo systemctl enable --now netevd
```

arm64 asset: `netevd-0.4.1-linux-arm64.tar.gz`.

**Containers** (`ghcr.io/zyvorai/netevd`):

| Tag | Runtime |
|-----|---------|
| `latest-ubuntu` | glibc (Ubuntu 26.04) |
| `latest-alpine` | musl (Alpine 3.23) |

**Remote host:**

```bash
./scripts/deploy-remote.sh <host> [user]
```

That builds on the target, installs `netevd.service`, and checks hooks with a `veth-netevd*` pair. The lab config matches only that pair.

**From source:**

```bash
git clone https://github.com/zyvorai/netevd.git && cd netevd
cargo build --release
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 config/netevd.example.yaml /etc/netevd/netevd.yaml
sudo systemctl enable --now netevd
```

## 2. First hook

```bash
sudo tee /etc/netevd/routable.d/01-notify.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "Interface $LINK is routable: $ADDRESSES"
EOF
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

Prefer `routable.d` when you need L3 connectivity; use `carrier.d` only for link-layer up.

## 3. Verify

```bash
systemctl status netevd
journalctl -u netevd -f
netevd validate
netevd status
curl -sf http://127.0.0.1:9090/api/v1/status | jq .
curl -sf http://127.0.0.1:9090/health | jq .
```

## Operate from CLI

1. Confirm unit active: `systemctl is-active netevd`
2. Validate YAML before edits: `netevd validate -c /etc/netevd/netevd.yaml`
3. Smoke-test API: `netevd status -f json`
4. Bounce an interface and tail events: `netevd events -f -i eth0 -t routable`
5. Remote host: `ssh root@<host> netevd status --endpoint http://127.0.0.1:9090`

## Troubleshooting

| Symptom | Check |
|---------|--------|
| Hook never fires | Executable bit; correct `*.d` directory; filters in YAML |
| No API on `:9090` | `api.enabled` / bind address in `netevd.yaml`; unit active |
| No events | Backend matches host (`systemd-networkd` vs NM vs dhclient) |

## Next

- [Using the operator surfaces](using-the-dashboard.md)
- [Admin basics](admin-basics.md)
- [Workflows](workflows.md)
