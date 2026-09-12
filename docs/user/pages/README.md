---
hero:
  eyebrow: USER GUIDE
  title: Page-by-page guides
---

Each guide follows: Purpose → When to use it → How to get there → Operate from CLI → Related pages.

Every route is also listed in the [complete page index](../PAGE_INDEX.md).

## Cli

| Page | What it covers |
|------|----------------|
| [CLI: events](cli/events.md) | Tail normalized carrier, routable, routes, and manager events from the daemon API — the same stream hooks react to, but for operators debugging link flaps or missing scripts. |
| [CLI: list / show](cli/list-show.md) | Inspect interfaces, routes, policy rules, and registered hook scripts — the same objects netevd tracks for hook dispatch and automatic policy routing. |
| [CLI: status](cli/status.md) | Show daemon health, uptime, interface count, routing-rule count, and events processed — a single glance before you change hooks or YAML. |
| [CLI: validate / reload / test](cli/validate-reload.md) | Validate YAML offline, request a config reload through the API, and dry-run hook scripts — the safe path before touching production networking. |

## Hooks

| Page | What it covers |
|------|----------------|
| [Hook: carrier / no-carrier](hooks/carrier.md) | Run scripts when the link layer goes UP (`carrier.d`) or DOWN (`no-carrier.d`) — cable plug, Wi‑Fi associate, or switch port flap — before L3 addresses exist. |
| [Hook: manager / NetworkManager](hooks/manager.md) | React to NetworkManager device and manager state via `activated.d`, `disconnected.d`, and `manager.d` — the NM equivalent of systemd-networkd's configured/routable hooks. |
| [Hook: routable](hooks/routable.md) | Fire when an interface has full L3 connectivity — addresses and a usable path — on systemd-networkd and dhclient backends. This is the most common hook directory for “start my service when the network is actually up.” |
| [Hook: routes](hooks/routes.md) | React to routing-table changes via `/etc/netevd/routes.d/` — default-route moves, VPN injection, or policy-table updates that netevd detects through its route watcher. |

## Ops

| Page | What it covers |
|------|----------------|
| [Configuration YAML](ops/config-yaml.md) | Tune backend, monitored interfaces, policy routing, API/metrics bind, event filters, and audit logging — the control plane for every hook and CLI query. |
| [Prometheus metrics (:9091)](ops/prometheus.md) | Scrape hook execution, event throughput, routing-rule counts, and netlink health for fleet observability — complementing REST status checks on `:9090`. |
| [REST API (:9090)](ops/rest-api.md) | Query status, interfaces, routes, policy rules, and recent events over HTTP — the same data backing `netevd status`, `list`, `show`, and `events` without SSH or `ip(8)`. |

---

11 guides. Regenerate: `node scripts/user-docs/generate-guide-index.mjs`.
