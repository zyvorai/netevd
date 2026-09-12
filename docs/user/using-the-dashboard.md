---
hero:
  eyebrow: USER GUIDE
  title: Using the operator surfaces
---

netevd has **no shipped browser console** for day-2 work. Operators use hook directories, the CLI, REST/Prometheus, and systemd journals. A static dashboard HTML exists in the repo for future embedding but is not the primary surface.

## Mental model

```text
Kernel / managers  →  netevd watchers  →  NetworkState
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
           hooks.d/*      REST :9090     metrics :9091
              │               │
              ▼               ▼
         your scripts     netevd status/list/events
```

## Where to look

| Surface | Role |
|---------|------|
| `/etc/netevd/*.d/` | Event scripts (primary UX) |
| `/etc/netevd/netevd.yaml` | Backend, filters, API bind, metrics |
| `netevd status\|list\|show\|events` | Live inspection via API |
| `journalctl -u netevd` | Daemon + hook exit logs |
| `:9090` / `:9091` | Automation scrape / status on `<host>` |

## Operate from CLI

1. **Hooks first:** add executable scripts under the event directory that matches your backend ([Page guides](pages/README.md)).
2. **Validate:** `netevd validate -c /etc/netevd/netevd.yaml`
3. **Observe:** `netevd events -f` while bouncing a link (`ip link set eth0 down/up`).
4. **Inspect state:** `netevd list interfaces`, `netevd list rules`, `netevd status -f json`.
5. **Fleet:** scrape `http://<host>:9091/metrics` or curl `http://<host>:9090/api/v1/status` over SSH tunnel.

## Tips

1. Start with one `routable.d` logger before complex automation.
2. Use `netevd events -f` while bouncing a link to confirm events.
3. On multi-homed hosts, confirm policy rules with `netevd list rules` or `ip rule`.
4. Keep hooks small and executable; non-zero exits are logged and do not stop other scripts.

Next: [Page-by-page guides](pages/README.md) · [Workflows](workflows.md)
