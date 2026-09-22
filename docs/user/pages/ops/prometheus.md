---
hero:
  eyebrow: USER GUIDE
  title: Prometheus metrics
---

## Purpose

Scrape hook execution, event throughput, routing-rule counts, and netlink health for fleet observability — the same HTTP server as the REST API on `:9090`.

## When to use it

- Alert when `netevd_script_failures_total` rises after a hook deploy
- Dashboard event rates per interface/backend
- Detect silent daemon failure (uptime gauge flatlines)

## How to get there

- Enable: `metrics.enabled: true` in `/etc/netevd/netevd.yaml`
- Endpoint: `http://<host>:9090/metrics` on the API server (Prometheus text exposition)
- `metrics.port` (default 9091) is stored in the config and logged at startup; the daemon serves `/metrics` on `api.port`

## Operate from CLI

1. Confirm metrics are enabled:

```bash
grep -A3 '^metrics:' /etc/netevd/netevd.yaml
curl -sf http://127.0.0.1:9090/metrics | head -20
```

2. Inspect key series locally:

```bash
curl -sf http://127.0.0.1:9090/metrics | grep -E '^netevd_(info|events_total|script_|routing_rules|ebpf_)'
```

3. Prometheus scrape config (replace `<host>` with your edge/management target):

```yaml
scrape_configs:
  - job_name: netevd
    scrape_interval: 30s
    static_configs:
      - targets: ['<host>:9090']
        labels:
          role: network-events
```

4. Example alert rules (hook failures):

```yaml
groups:
  - name: netevd
    rules:
      - alert: NetevdHookFailures
        expr: increase(netevd_script_failures_total[5m]) > 0
        for: 2m
        labels:
          severity: warning
      - alert: NetevdDown
        expr: absent(netevd_info)
        for: 5m
        labels:
          severity: critical
```

5. Remote check from your workstation:

```bash
curl -sf http://<host>:9090/metrics | grep netevd_uptime_seconds
```

6. **Empty / fail:** Connection refused → `metrics.enabled: false` or firewall; series missing → daemon just started or metrics init failed (see `journalctl -u netevd`).

7. **Success:** `netevd_info{version="…"}` present; counters increment when you bounce a link and routable hooks run.

## Related pages

- [REST API](rest-api.md)
- [CLI: status](../cli/status.md)
- [Fleet workflow](../../workflows.md)
