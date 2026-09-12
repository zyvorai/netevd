---
hero:
  eyebrow: USER GUIDE
  title: netevd — User Documentation
---

**netevd** is a Netlink-first Linux network event daemon (Rust). It watches carrier, address, route, and manager state — then runs your scripts with rich context. It also maintains per-interface policy routing on multi-homed hosts and exposes REST + Prometheus on `:9090` / `:9091`.

| You want to… | Open |
|--------------|------|
| Install and fire the first hook | [Getting Started](getting-started.md) |
| Orient around hooks / CLI / API | [Using the operator surfaces](using-the-dashboard.md) |
| Screen-by-screen / command guides | [Page-by-page guides](pages/README.md) |
| Look up hooks and CLI | [Complete page index](PAGE_INDEX.md) |
| YAML, systemd, ports, security | [Admin basics](admin-basics.md) |
| Multi-home, resume, observe | [Common workflows](workflows.md) |

**→ [Docs one-pager](/docs/netevd)** · **[GitHub](https://github.com/zyvorai/netevd)** · **[netctl](/docs/netctl)**

## Printable PDFs

```bash
node scripts/user-docs/build-user-pdfs.mjs
```

Output lands in [`pdf/`](pdf/):

- `netevd-User-README.pdf`
- `netevd-Getting-Started.pdf`
- `netevd-Page-by-Page.pdf`
- `netevd-Admin-Basics.pdf`

## Product at a glance

```text
  Hooks      →  /etc/netevd/{carrier,routable,routes,…}.d/
  Config     →  /etc/netevd/netevd.yaml
  REST/API   →  :9090  (default bind 127.0.0.1)
  Metrics    →  :9091
  Unit       →  netevd.service
  CLI        →  netevd status | list | show | events | validate | reload
```

## Operate from CLI

1. Install and enable: `sudo ./install.sh && sudo systemctl enable --now netevd`
2. Validate config: `netevd validate`
3. Inspect live state: `netevd status && netevd list interfaces`
4. Watch events during a link test: `netevd events -f -i eth0`
5. Fleet scrape: point Prometheus at `http://<host>:9091/metrics`

Never publish lab IPs in runbooks — use `<host>` for remote targets.

---

*Zyvor · [zyvor.dev](https://zyvor.dev) · netevd · Apache-2.0*
