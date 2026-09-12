---
hero:
  eyebrow: USER GUIDE
  title: REST API (:9090)
---

## Purpose

Query status, interfaces, routes, policy rules, and recent events over HTTP — the same data backing `netevd status`, `list`, `show`, and `events` without SSH or `ip(8)`.

## When to use it

- Automation and config-management probes from a bastion
- Quick inspection when CLI is not installed on a jump host (curl only)
- Integrate with internal dashboards (JSON responses)

## How to get there

- Base URL: `http://127.0.0.1:9090` (default bind **localhost**)
- Config: `api.enabled`, `api.bind_address`, `api.port` in `/etc/netevd/netevd.yaml`
- Remote access: tunnel or set `bind_address: "0.0.0.0"` only on trusted management networks

## Operate from CLI

1. Health and version smoke test:

```bash
curl -sf http://127.0.0.1:9090/health | jq .
curl -sf http://127.0.0.1:9090/api/v1/status | jq .
```

2. List interfaces and drill into one:

```bash
curl -sf http://127.0.0.1:9090/api/v1/interfaces | jq .
curl -sf http://127.0.0.1:9090/api/v1/interfaces/eth0 | jq .
```

3. Routes and policy rules:

```bash
curl -sf http://127.0.0.1:9090/api/v1/routes | jq .
curl -sf http://127.0.0.1:9090/api/v1/rules | jq .
```

4. Recent events:

```bash
curl -sf 'http://127.0.0.1:9090/api/v1/events' | jq .
```

5. Request config reload (same as `netevd reload`):

```bash
curl -X POST http://127.0.0.1:9090/api/v1/reload \
  -H 'Content-Type: application/json' \
  -d '{"force":false}' | jq .
```

6. Remote host via SSH tunnel (keeps API on localhost):

```bash
ssh -L 9090:127.0.0.1:9090 root@<host>
curl -sf http://127.0.0.1:9090/api/v1/status | jq .
```

7. CLI equivalents:

```bash
netevd status -f json
netevd list interfaces -f json
netevd events --tail 20 -f json
```

8. **Empty / fail:** `Connection refused` → unit down or `api.enabled: false`; empty event history → normal on fresh install until links change; reload returns `success: false` → restart with `systemctl restart netevd`.

9. **Success:** JSON `success: true` with populated `data`; interface count matches `netevd list interfaces`; health reflects dbus/netlink/config checks.

## Endpoint reference

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness and subsystem checks |
| GET | `/metrics` | Prometheus text (also on `:9091`) |
| GET | `/api/v1/status` | Daemon summary |
| GET | `/api/v1/interfaces` | All interfaces |
| GET | `/api/v1/interfaces/{name}` | One interface |
| GET | `/api/v1/routes` | Routing table |
| GET | `/api/v1/rules` | Policy rules |
| GET | `/api/v1/events` | Event history |
| POST | `/api/v1/reload` | Config reload request |

## Related pages

- [CLI: status](../cli/status.md)
- [CLI: list / show](../cli/list-show.md)
- [Prometheus metrics](prometheus.md)
