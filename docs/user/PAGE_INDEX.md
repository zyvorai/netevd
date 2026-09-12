---
hero:
  eyebrow: USER GUIDE
  title: netevd — Complete page index
---

Every primary navigable dashboard route.

_Generated: 2026-08-29 · 11 routes_

Regenerate: `node scripts/user-docs/generate-page-index.mjs`

## CLI

| Page | Route | Purpose | Guide |
|------|-------|---------|-------|
| CLI: events | `/events` | Tail normalized carrier, routable, routes, and manager events from the daemon API. | [Open](pages/cli/events.md) |
| CLI: list / show | `/list-show` | Inspect interfaces, routes, policy rules, and registered hook scripts. | [Open](pages/cli/list-show.md) |
| CLI: status | `/status` | Show daemon health, uptime, and counters via the REST API. | [Open](pages/cli/status.md) |
| CLI: validate / reload / test | `/validate-reload` | Validate YAML offline; reload config or dry-run hook scripts safely. | [Open](pages/cli/validate-reload.md) |

## Hooks

| Page | Route | Purpose | Guide |
|------|-------|---------|-------|
| Hook: carrier / no-carrier | `/carrier` | Run scripts when link layer goes UP (carrier.d) or DOWN (no-carrier.d). | [Open](pages/hooks/carrier.md) |
| Hook: manager / NetworkManager | `/manager` | React to NetworkManager device activation, disconnect, and manager state. | [Open](pages/hooks/manager.md) |
| Hook: routable | `/routable` | Fire when an interface has full L3 connectivity (systemd-networkd or dhclient). | [Open](pages/hooks/routable.md) |
| Hook: routes | `/routes` | React to routing-table changes via routes.d hooks. | [Open](pages/hooks/routes.md) |

## Ops

| Page | Route | Purpose | Guide |
|------|-------|---------|-------|
| Configuration YAML | `/config-yaml` | Tune backend, monitored interfaces, policy routing, API/metrics, filters, and audit. | [Open](pages/ops/config-yaml.md) |
| Prometheus metrics (:9091) | `/prometheus` | Scrape hook and event health metrics for fleet observability. | [Open](pages/ops/prometheus.md) |
| REST API (:9090) | `/rest-api` | Query status, interfaces, routes, rules, and events without SSH or ip(8). | [Open](pages/ops/rest-api.md) |

## Related

- [User docs home](README.md)
- [Page-by-page guides](pages/README.md)
