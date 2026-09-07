# netevd — next features (PR pack)

Repo: https://github.com/zyvorai/netevd  
Status checked: 2026-09-07 — no open PRs, no open issues on zyvorai/netevd.  
Current surface: netlink watchers (addr/link/route), systemd-networkd / NetworkManager / dhclient backends, hook dirs, automatic policy routing, Axum REST + Prometheus, privilege drop + CAP_NET_ADMIN.

This pack is a single PR *plan*, not a dump of unrelated ideas. Keep Community Edition small. Push SLA / multi-cluster / GUI into Enterprise.

---

## Verdict

Yes — add more features, but only ones that stay on the daemon’s job: **observe kernel + manager events, keep NetworkState correct, run hooks, fix return-path routing**. Do not grow netevd into Fabric, Cilium, or NetPredator.

Ship as **3 stacked PRs**, not one mega-PR.

| PR | Title | Why first |
|----|--------|-----------|
| 1 | Event completeness + hook contract | Users already depend on dirs + `$JSON`. Gaps here are bugs, not “nice to have”. |
| 2 | Policy-routing v2 | Multi-homed / dual-WAN / WireGuard / KubeVirt is the actual lab pain. |
| 3 | Live stream + reload | Makes REST useful for Fleet/Fabric without turning netevd into a control plane. |

---

## PR-1 — Event completeness and hook contract

### Already have
`carrier.d`, `no-carrier.d`, `configured.d`, `degraded.d`, `routable.d`, `activated.d`, `disconnected.d`, `manager.d`, `routes.d`.  
Env: `$LINK`, `$LINKINDEX`, `$STATE`, `$BACKEND`, `$ADDRESSES` (+ `$JSON` / DHCP vars).

### Add

1. **Missing hook dirs (all backends)**
   - `address-added.d` / `address-removed.d` — today address changes only show up as configured/routable.
   - `link-added.d` / `link-removed.d` — veth, tap, macvtap, WireGuard appear and vanish on KVM hosts.
   - `mtu.d` — MTU flaps break overlay / jumbo paths.
   - `neigh.d` (optional, off by default) — gateway MAC change / incomplete NUD.
   - `dns.d` — resolv.conf / resolved link DNS change (you already talk to resolved).

2. **Stable JSON schema for every hook**
   - Always set `$JSON` (not only networkd).
   - Schema version field: `"schema": "netevd.event.v1"`.
   - Fields: `event`, `ts`, `link`, `index`, `state`, `backend`, `addresses[]`, `mtu`, `operstate`, `flags`, `routes_delta`, `gateway`, `dns`.
   - Keep env vars. Scripts should not parse `$JSON` if they only need `$LINK`.

3. **Debounce + coalesce**
   - Kernel often emits addr+route+link in a burst.
   - Config: `hooks.debounce_ms: 50` (default), coalesce into one `routable` / `routes` run.
   - Counter metric: `netevd_hooks_coalesced_total`.

4. **Hook timeout + concurrency cap**
   - `hooks.timeout_sec`, `hooks.max_parallel`.
   - Kill + log + metric on hang. Alphabet order stays.

5. **Interface match beyond exact name**
   - `monitoring.match: ["eth*", "enp*", "wg*", "tap*"]`
   - Exclude list: `lo`, `docker*`, `cni*`, `flannel*`, `cilium*`, `veth*` (default exclude virtual unless opted in).

### Tests
- Unit: JSON schema fixtures per event type.
- Functional: create dummy iface + addr, assert hook env + JSON.
- Burst of 20 netlink msgs → one coalesced hook.

### Docs
- README table of dirs.
- `docs/user/hooks.md` with schema example.

---

## PR-2 — Policy routing v2 (multi-homed that actually survives)

### Already have
Table ID `200 + ifindex`, `from <ip> lookup`, `to <ip> lookup`, default via gateway, cleanup on addr delete.

### Add

1. **Named tables + reserved IDs**
   - `routing.tables.eth1: 203` instead of magic `200+index` (index reuse after nic replace is a footgun).
   - Persist mapping in `/var/lib/netevd/tables.json`.

2. **IPv6 policy rules**
   - Same from/to rules for GUA and (optional) ULA.
   - Skip link-local by default.

3. **fwmark / suppress_prefixlength**
   - Needed when Cilium / kube-proxy / WireGuard already use tables 100–200.
   - `routing.fwmark: 0x100` optional.

4. **WireGuard / tunnel awareness**
   - On `wg*` / `tun*` / `gre*` becoming routable: optional AllowedIPs-style “don’t steal default” mode.
   - Config: `routing.default_route: auto | never | only-if-no-other`.

5. **Conflict detect**
   - On start, dump existing `ip rule` / tables that netevd does not own.
   - Metric + `/api/v1/routing/conflicts`.
   - Never delete rules it did not create (owner tag via pref range 32000–32700 or comment if available).

6. **Bond / VLAN / bridge parent**
   - Policy on `vlan10` should not also install a competing default on `bond0` unless asked.

### Tests
- Two dummy ifaces, two tables, ping-style “from src IP uses correct table” via `ip route get`.
- Ifindex recycle: rename/recreate iface, table id stays.

---

## PR-3 — Live events + config reload (Fleet/Fabric glue)

### Already have
REST: status, interfaces, routes, events, metrics, health on `:9090`.

### Add

1. **`GET /api/v1/events/stream` SSE** (and optional WS)
   - Same payload as hook JSON.
   - Auth: unix socket default + optional bearer for TCP. Do not leave 9090 open on 0.0.0.0 without token.

2. **`SIGHUP` / `POST /api/v1/reload`**
   - Reload yaml without dropping netlink sockets.
   - Report which section changed.

3. **Unix socket API**
   - `/run/netevd/netevd.sock` for local `netevdctl`.
   - Keep TCP for metrics scrape only if bound to localhost.

4. **`netevdctl`**
   - `status`, `ifaces`, `events -f`, `hooks list`, `routing show`, `reload`.
   - Thin client over the unix socket. Fits how you operate AlmaLinux labs.

5. **Journald structured fields**
   - `INTERFACE=`, `EVENT=`, `BACKEND=` so `journalctl -u netevd INTERFACE=eth1` works.

### Out of this PR
Web UI beyond a tiny status page. Machina/Fabric already own glass UI.

---

## Do **not** add in Community (or park as Enterprise)

| Idea | Why not now |
|------|-------------|
| Full eBPF packet path / process attribution | That is NetPredator. netevd should emit events, not inspect payloads. |
| Become a CNI or replace Cilium | Wrong layer. Optional *exclude* Cilium ifaces is enough. |
| Configure addresses / DHCP client | You observe managers; you do not replace them. |
| Multi-node control plane | Fleet. |
| Heavy dashboard | Fabric / Machina. |
| nftables policy engine | Easy to fight NetworkManager/firewalld. Hook scripts can call nft if the site wants it. |

A thin **optional** `src/cloud` metadata refresh (AWS/GCP/Azure IMDS on routable) is OK if the module already exists and stays off-by-default.

---

## Suggested hook extras (examples/, not daemon code)

Ship as example scripts only:

- `01-logger.sh` — journald
- `10-update-resolv.sh` — guarded, off by default
- `20-wg-up.sh` — `wg-quick` / `systemctl start wg-quick@…` on carrier
- `30-notify-fabric.sh` — POST event JSON to local Fabric
- `40-kubevirt-bridge.sh` — when `br0` becomes routable, log for KubeVirt node

---

## Acceptance for merge

- `cargo test && cargo clippy -- -D warnings`
- Functional: dummy interface lifecycle hits the new dirs
- Policy rules survive daemon restart
- No default bind of authenticated API on `0.0.0.0`
- README + example yaml updated in the same PR as the code
- Hook schema versioned; old env vars still work

---

## Proposed commit / PR titles

```
feat(hooks): address/link/mtu events, stable JSON v1, debounce
feat(routing): named tables, IPv6 rules, owner-safe cleanup
feat(api): SSE event stream, unix socket, SIGHUP reload, netevdctl
docs: hook contract and routing v2
```

Open PR-1 first on `zyvorai/netevd` (canonical). Keep hypersdk release notes in sync; README still points at hypersdk release tarballs.

---

## One-line GitHub PR body (PR-1)

```
## Summary
Add missing hook directories (address/link/mtu), a versioned `$JSON` schema on every backend, debounce/coalesce for netlink bursts, hook timeouts, and glob/exclude interface matching.

## Why
Address and link churn on KVM/WireGuard/veth hosts never got a dedicated dir. `$JSON` was networkd-only. Bursts double-fire scripts.

## Test plan
- dummy iface add/del → address-*.d and link-*.d
- 20 netlink events in 10ms → one coalesced hook
- clippy -D warnings, existing functional workflow
```
