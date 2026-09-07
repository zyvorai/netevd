# netevd

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![CI](https://github.com/zyvorai/netevd/actions/workflows/ci.yml/badge.svg)](https://github.com/zyvorai/netevd/actions/workflows/ci.yml)
[![Functional Tests](https://github.com/zyvorai/netevd/actions/workflows/functional-tests.yml/badge.svg)](https://github.com/zyvorai/netevd/actions/workflows/functional-tests.yml)
[![codecov](https://codecov.io/gh/zyvorai/netevd/branch/main/graph/badge.svg)](https://codecov.io/gh/zyvorai/netevd)
[![Release](https://img.shields.io/github/v/release/zyvorai/netevd?sort=semver)](https://github.com/zyvorai/netevd/releases)

<p align="center">
  <a href="https://zyvor.dev/demo?utm_source=github&utm_medium=netevd"><img src="https://img.shields.io/badge/Demo-F97316?style=flat-square" alt="Demo"/></a>
  <a href="https://zyvor.dev/docs?utm_source=github&utm_medium=netevd"><img src="https://img.shields.io/badge/Docs-2563EB?style=flat-square" alt="Docs"/></a>
  <a href="https://zyvor.dev/blog?utm_source=github&utm_medium=netevd"><img src="https://img.shields.io/badge/Blog-71717A?style=flat-square" alt="Blog"/></a>
  <a href="https://zyvor.dev/contact?utm_source=github&utm_medium=netevd"><img src="https://img.shields.io/badge/Contact_sales-22C55E?style=flat-square" alt="Contact"/></a>
</p>

**netevd** is a network event daemon that watches your Linux network interfaces and runs scripts when things change. Think of it as systemd path units, but purpose-built for networking: when an interface gets an IP, loses its link, or routes change, netevd executes your scripts with full context about what happened.

It bridges **systemd-networkd**, **NetworkManager**, and **dhclient** into a single, unified event system -- with automatic policy routing, a REST API, Prometheus metrics, and a defense-in-depth security model.

## Table of contents

- [Why netevd?](#why-netevd)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [Script Directories](#script-directories)
- [Automatic Policy Routing](#automatic-policy-routing)
- [Security](#security)
- [Performance](#performance)
- [REST API](#rest-api)
- [Enterprise](#enterprise)
- [License](#license)

## Why netevd?

| Problem | netevd solution |
|---------|----------------|
| Need scripts to run when network state changes | Drop scripts in `/etc/netevd/routable.d/` -- done |
| Multi-homed server with broken return-path routing | Automatic per-interface routing tables and policy rules |
| Want real-time network events, not polling | Netlink multicast: sub-100ms latency, zero polling |
| Need to support multiple network managers | One daemon handles networkd, NetworkManager, and dhclient |
| Security concerns with network daemons | Privilege separation, CAP_NET_ADMIN only, input validation |

## Quick Start

### GitHub Release (recommended)

```bash
curl -LO https://github.com/zyvorai/netevd/releases/download/v0.4.0/netevd-0.4.0-linux-amd64.tar.gz
tar xzf netevd-*-linux-amd64.tar.gz && cd netevd-*-linux-amd64
sudo ./install.sh
sudo systemctl enable --now netevd
```

### Build from source

```bash
git clone https://github.com/zyvorai/netevd.git && cd netevd
cargo build --release
sudo install -Dm755 target/release/netevd /usr/bin/netevd
sudo install -Dm644 systemd/netevd.service /lib/systemd/system/netevd.service
sudo install -Dm644 config/netevd.example.yaml /etc/netevd/netevd.yaml
sudo systemctl enable --now netevd
```

### First hook script

Create your first script — this runs whenever an interface becomes fully routable:

```bash
cat <<'EOF' | sudo tee /etc/netevd/routable.d/01-notify.sh && sudo chmod +x /etc/netevd/routable.d/01-notify.sh
#!/bin/bash
logger -t netevd "Interface $LINK is routable: $ADDRESSES"
EOF
```

Sample configuration: [config/netevd.example.yaml](config/netevd.example.yaml).

## How It Works

```
                    +------------------+
                    |   Linux Kernel   |
                    |  Netlink events  |
                    +--------+---------+
                             |
         +-------------------+-------------------+
         |                   |                   |
   +-----------+      +-----------+      +-----------+
   | Addresses |      |   Links   |      |  Routes   |
   |  watcher  |      |  watcher  |      |  watcher  |
   +-----+-----+      +-----+-----+      +-----+-----+
         |                   |                   |
         +-------------------+-------------------+
                             |
                    +--------+---------+
                    |  NetworkState    |
                    |  (Arc<RwLock>)   |
                    +--------+---------+
                             |
              +--------------+--------------+
              |              |              |
        +-----+-----+  +----+----+  +------+------+
        |  Routing   |  | Script  |  |    DBus     |
        |  policy    |  |  exec   |  |  resolved/  |
        |  rules     |  |         |  |  hostnamed  |
        +------------+  +---------+  +-------------+
```

**Event sources** -- netevd subscribes to kernel netlink multicast groups and listens for DBus signals from your chosen backend (systemd-networkd, NetworkManager) or watches dhclient lease files via inotify.

**State management** -- All state is held in a single `NetworkState` behind `Arc<RwLock>`, updated by concurrent Tokio tasks. Read locks for queries, write locks for mutations -- no races.

**Actions** -- On state changes, netevd configures routing policy rules, executes scripts from the matching event directory, and optionally pushes DNS/hostname updates via DBus.

## Configuration

```yaml
# /etc/netevd/netevd.yaml
system:
  log_level: "info"
  backend: "systemd-networkd"    # or "NetworkManager" or "dhclient"

monitoring:
  interfaces:                    # empty = monitor all
    - eth0
    - eth1
  match_patterns:                # globs for address/link/mtu hooks; empty = all
    - "eth*"
  exclude:                       # globs to skip; empty = built-in virtual/CNI defaults
    - "veth*"

hooks:
  debounce_ms: 50                # coalesce netlink bursts per (link, event)
  timeout_sec: 30                # per-script timeout

routing:
  policy_rules:                  # auto-create per-interface routing tables
    - eth1

backends:
  systemd_networkd:
    emit_json: true              # pass full JSON to scripts via $JSON
  dhclient:
    use_dns: false
    use_domain: false
    use_hostname: false
  networkmanager: {}
```

Full template: [config/netevd.example.yaml](config/netevd.example.yaml)

## Script Directories

Scripts are organized by the event that triggers them:

| Directory | Trigger | Backends |
|-----------|---------|----------|
| `carrier.d/` | Cable connected | All |
| `no-carrier.d/` | Cable disconnected | All |
| `configured.d/` | Interface has IP | systemd-networkd |
| `degraded.d/` | Partial configuration | systemd-networkd |
| `routable.d/` | Full connectivity | systemd-networkd, dhclient |
| `activated.d/` | Device activated | NetworkManager |
| `disconnected.d/` | Device disconnected | NetworkManager |
| `manager.d/` | Manager state change | All |
| `routes.d/` | Routing table change | All |
| `address-added.d/` | IP address added to an interface | All (netlink, backend-independent) |
| `address-removed.d/` | IP address removed from an interface | All (netlink, backend-independent) |
| `link-added.d/` | Interface appears (veth, tap, WireGuard, ...) | All (netlink, backend-independent) |
| `link-removed.d/` | Interface disappears | All (netlink, backend-independent) |
| `mtu.d/` | Interface MTU changes | All (netlink, backend-independent) |

Scripts run in alphabetical order. Use numeric prefixes (`01-`, `02-`) to control ordering. Non-zero exit codes are logged but don't block other scripts.

The `address-*`, `link-*`, and `mtu` hooks fire per interface based on `monitoring.match_patterns` / `monitoring.exclude` (glob lists, defaulting to excluding `lo`/`docker*`/`veth*`/`cni*`/`cilium*`), independent of whether that interface is in `routing.policy_rules`. They're also debounced (`hooks.debounce_ms`, default 50ms) so a burst of netlink events collapses into one script run per `(interface, event)` pair. See [docs/hooks-contract.md](docs/hooks-contract.md) for the full JSON schema and config keys.

### Environment Variables

Every script receives:

| Variable | Example |
|----------|---------|
| `$LINK` | `eth0` |
| `$LINKINDEX` | `2` |
| `$STATE` | `routable` |
| `$BACKEND` | `systemd-networkd` |
| `$ADDRESSES` | `192.168.1.100 10.0.0.5` |

**systemd-networkd** adds `$JSON` with full interface data (MTU, driver, DNS, routes).
**dhclient** adds `$DHCP_ADDRESS`, `$DHCP_GATEWAY`, `$DHCP_DNS`, `$DHCP_DOMAIN`, `$DHCP_HOSTNAME`.
**The netlink-driven hooks** (`address-*`, `link-*`, `mtu`, `routes`) always set `$JSON` to a versioned `netevd.event.v1` payload (`$BACKEND` is `netlink` for these).

## Automatic Policy Routing

For multi-homed servers, netevd solves the classic "wrong interface" problem automatically. When you list an interface under `routing.policy_rules`, netevd:

1. Creates a custom routing table (ID = 200 + interface index)
2. Adds `from <ip> lookup <table>` and `to <ip> lookup <table>` rules
3. Installs a default route via the interface's gateway in that table
4. Cleans up automatically when addresses are removed

```bash
# After netevd configures eth1 (index 3, IP 192.168.1.100):
$ ip rule list
32765: from 192.168.1.100 lookup 203
32766: to 192.168.1.100 lookup 203

$ ip route show table 203
default via 192.168.1.1 dev eth1
```

## Security

netevd follows a defense-in-depth model:

1. **Privilege separation** -- Starts as root, immediately drops to the `netevd` user via `setuid`/`setgid`
2. **Minimal capabilities** -- Retains only `CAP_NET_ADMIN`; child processes inherit nothing
3. **Input validation** -- All external data (interface names, IPs, hostnames) is validated; shell metacharacters are rejected
4. **No shell intermediary** -- Scripts are executed directly, not via `sh -c`
5. **systemd hardening** -- `NoNewPrivileges`, `ProtectSystem=strict`, `PrivateTmp`

Details: **[Security Policy](SECURITY.md)**

## Performance

| Metric | Value |
|--------|-------|
| Memory (idle) | 3-5 MB RSS |
| CPU (idle) | < 1% |
| Event latency | < 100ms (netlink multicast) |
| Event-to-script | < 10ms |
| Throughput | 1000+ events/sec |

## REST API

9 endpoints built on Axum for remote management and monitoring:

```bash
curl http://localhost:9090/api/v1/status       # Daemon status
curl http://localhost:9090/api/v1/interfaces    # List interfaces
curl http://localhost:9090/api/v1/routes        # Routing table
curl http://localhost:9090/api/v1/events        # Event history
curl http://localhost:9090/metrics              # Prometheus metrics
curl http://localhost:9090/health               # Health check
```

REST API and metrics use the daemon HTTP port (default `9090`). Tune behavior in `/etc/netevd/netevd.yaml` — start from [config/netevd.example.yaml](config/netevd.example.yaml).

## Development

```bash
cargo build && cargo test && cargo clippy -- -D warnings
```

## Enterprise

| | Community Edition (this repo) | Enterprise ([zyvor.dev](https://zyvor.dev/?utm_source=github&utm_medium=netevd)) |
|---|------------------------------|-------------------------------------------------------------------------------------|
| **Support** | [GitHub Issues](https://github.com/zyvorai/netevd/issues) | SLA, [sales@zyvor.dev](mailto:sales@zyvor.dev), professional services |
| **Scope** | Self-hosted event hooks | Production rollouts, platform integration |
| **Platform** | netevd daemon | Full networking stack with netctl, cloud-netconfig, HyperSDK |

**Next steps:** [Demo](https://zyvor.dev/demo?utm_source=github&utm_medium=netevd) · [ROI](https://zyvor.dev/roi?utm_source=github&utm_medium=netevd) · [Pricing](https://zyvor.dev/pricing?utm_source=github&utm_medium=netevd) · [Contact](https://zyvor.dev/contact?utm_source=github&utm_medium=netevd) · [sales@zyvor.dev](mailto:sales@zyvor.dev)

Community Edition covers self-hosted event hooks and policy routing. Production SLAs, supported deployments, and the full HyperSDK platform → contact Zyvor (not GitHub Issues). Full detail: [docs/enterprise.md](docs/enterprise.md).

## Support the project

netevd Community Edition is free and open source, maintained by **Susant Sahani** · [Zyvor AI Labs](https://zyvor.dev?utm_source=github&utm_medium=netevd)

- **Enterprise / production:** [zyvor.dev/contact](https://zyvor.dev/contact?utm_source=github&utm_medium=netevd) · [sales@zyvor.dev](mailto:sales@zyvor.dev)
- **Community help:** [GitHub Issues](https://github.com/zyvorai/netevd/issues) · [SECURITY.md](SECURITY.md)

## License

netevd is licensed under the Apache License, Version 2.0.

Copyright © 2026 Zyvor AI Labs Private Limited.

This repository contains only the netevd Community Edition source code.

Other Zyvor products, platforms, services, and commercial offerings are separate works and may be governed by different licenses and terms.

Enterprise: [sales@zyvor.dev](mailto:sales@zyvor.dev) · General: [info@zyvor.dev](mailto:info@zyvor.dev).

Related: [netctl](https://github.com/zyvorai/netctl) · [cloud-netconfig](https://github.com/zyvorai/cloud-netconfig)
