---
hero:
  eyebrow: USER GUIDE
  title: 'CLI: status'
---

## Purpose

Show daemon health, uptime, interface count, routing-rule count, and events processed — a single glance before you change hooks or YAML.

## When to use it

- Post-install smoke test on a new host
- Change-window entry: confirm netevd is running before editing `/etc/netevd/`
- Fleet checks from cron or a monitoring probe (JSON output)

## How to get there

- Command: `netevd status`
- Equivalent HTTP: `GET http://127.0.0.1:9090/api/v1/status`
- Unit: `systemctl status netevd`

## Operate from CLI

1. Quick text summary (default):

```bash
netevd status
```

2. JSON for scripts and dashboards:

```bash
netevd status -f json
netevd status -f json --endpoint http://<host>:9090 | jq '.data'
```

3. YAML output:

```bash
netevd status -f yaml
```

4. Cross-check systemd and health endpoint:

```bash
systemctl status netevd --no-pager
curl -sf http://127.0.0.1:9090/health | jq .
curl -sf http://127.0.0.1:9090/api/v1/status | jq .
```

5. **Empty / fail:** `Connection refused` → daemon down or `api.enabled: false`; status shows zero interfaces on a busy host → backend mismatch or netlink permission issue (check unit logs).

6. **Success:** `status: running`, non-zero uptime, interface count matches `ip link`, routing-rule count matches policy-routed addresses.

## Related pages

- [REST API](../ops/rest-api.md)
- [Prometheus metrics](../ops/prometheus.md)
- [Admin basics](../../admin-basics.md)
