---
hero:
  eyebrow: HOOK CONTRACT
  title: Hook contract (netevd.event.v1)
  lead: "Reference for the src/hooks module: the versioned JSON event payload, the hook directories it dispatches to, and the config keys that control debouncing and interface selection."
  highlights:
    - {value: "14", label: "hook directories"}
    - {value: "v1", label: "event schema version"}
    - {value: "50ms", label: "default debounce window"}
    - {value: "30s", label: "default script timeout"}
    - {value: "2", label: "hook metrics exported"}
---

Reference for the `src/hooks` module: the versioned JSON event payload, the
new hook directories it dispatches to, and the config keys that control
debouncing and interface selection. See [`ROADMAP.md`](ROADMAP.md) for the
larger PR-1/2/3 plan this belongs to.

## Hook directories

All directories live under `/etc/netevd/<state>.d/` (or `CONFIG_DIR`).
Scripts run alphabetically, with `$LINK`/`$LINKINDEX`/`$STATE`/`$EVENT`/
`$BACKEND`/`$ADDRESSES`/`$JSON` in their environment (plus event-specific
extras noted below).

| Directory | Source | Notes |
|---|---|---|
| `carrier.d` / `no-carrier.d` | systemd-networkd / NetworkManager | link layer up/down |
| `configured.d` / `degraded.d` / `routable.d` | systemd-networkd | operational state |
| `activated.d` / `disconnected.d` / `manager.d` | NetworkManager | device/manager state |
| `routes.d` | netlink route watcher | now versioned + debounced; `$ROUTES_DELTA` via JSON |
| `address-added.d` / `address-removed.d` | netlink address watcher | fires per interface matched by `monitoring.match_patterns`/`exclude`, independent of `routing.policy_rules` |
| `link-added.d` / `link-removed.d` | netlink link watcher | fires only on a genuine add/remove, not on every attribute update |
| `mtu.d` | netlink link watcher | fires only when MTU actually changes (first sighting just seeds the baseline) |

`neigh.d` and `dns.d` are accepted as valid state names (for forward
compatibility) but nothing emits them yet — see `ROADMAP.md` for why they're
deferred.

## Take a closer look

=== "Link"

    `link-added.d` / `link-removed.d` fire only on a genuine interface add or
    remove — not on every attribute update — for interfaces like veth, tap,
    or WireGuard devices appearing and disappearing. `mtu.d` fires only when
    MTU actually changes; the first sighting just seeds the baseline.

=== "Address"

    `address-added.d` / `address-removed.d` fire per interface matched by
    `monitoring.match_patterns` / `exclude`, independent of whether that
    interface is listed under `routing.policy_rules`.

=== "Route"

    `routes.d` reacts to routing-table changes — default-route moves, VPN
    injection, or policy-table updates — detected by the netlink route
    watcher. It's versioned and debounced, with `$ROUTES_DELTA` delivered via
    `$JSON`.

## JSON schema (`netevd.event.v1`)

Every dispatch also sets `$JSON` to a `HookEventV1` (see
`src/hooks/event.rs`):

```json
{
  "schema": "netevd.event.v1",
  "event": "address-added",
  "ts": "2026-09-07T12:00:00.000Z",
  "link": "eth0",
  "index": 2,
  "state": "address-added",
  "backend": "netlink",
  "addresses": ["192.168.1.10"],
  "mtu": null,
  "operstate": null,
  "flags": [],
  "routes_delta": [],
  "gateway": null,
  "dns": []
}
```

`backend` is `"netlink"` for the address/link/mtu/routes hooks fired
directly from `src/network/watcher.rs` (backend-agnostic — they fire
regardless of the configured `system.backend`), or the configured backend
name (`systemd-networkd`, `NetworkManager`, `dhclient`) for the existing
operstate-driven hooks.

## Config

```yaml
monitoring:
  match_patterns: ["eth*", "enp*", "wg*"]   # glob include list; empty = all
  exclude: ["lo", "docker*", "veth*"]        # glob exclude list; empty = built-in defaults

hooks:
  debounce_ms: 50   # coalesce bursts per (link, event) key
  timeout_sec: 30   # per-script timeout
  max_parallel: 1   # reserved; scripts currently always run sequentially
```

`InterfaceSelector` (`src/hooks/match_iface.rs`) applies `exclude` first,
then `match_patterns` (empty include list = allow everything not excluded).
When `exclude` is empty it falls back to `DEFAULT_EXCLUDES`
(`lo`, `docker*`, `br-*`, `cni*`, `flannel*`, `cilium*`, `veth*`, `virbr*`).

## Metrics

- `netevd_hooks_dispatched_total{event,link}` — every successful or failed
  `dispatch_event` call.
- `netevd_hooks_coalesced_total` — events merged into a pending window
  instead of causing a separate dispatch.

## Offline contract sandbox

`contrib/hooks-contract-standalone/` is a standalone Cargo package (its own
lockfile, `publish = false`) used to validate this contract's logic in
isolation before it was wired into the daemon. It's not part of the main
build; see its own `README.md`.
