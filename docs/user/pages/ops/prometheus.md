---
hero:
  eyebrow: USER GUIDE
  title: Prometheus metrics (:9091)
---

## Purpose

Scrape hook execution, event throughput, routing-rule counts, and netlink health for fleet observability — complementing REST status checks on `:9090`.

## When to use it

- Alert when `netevd_script_failures_total` rises after a hook deploy
- Dashboard event rates per interface/backend
- Detect silent daemon failure (uptime gauge flatlines)

## How to get there

- Metrics port: `metrics.port` in `/etc/netevd/netevd.yaml` (default **9091**)
- Enable: `metrics.enabled: true`
- Endpoint: `http://<host>:9091/metrics` (Prometheus text exposition)
- Note: `/metrics` also exists on the API router at `:9090` when API is enabled; prefer the dedicated metrics port for scrape jobs

## Operate from CLI

1. Confirm metrics are enabled:

```bash
grep -A3 '^metrics:' /etc/netevd/netevd.yaml
curl -sf http://127.0.0.1:9091/metrics | head -20
```

2. Inspect key series locally:

```bash
curl -sf http://127.0.0.1:9091/metrics | grep -E '^netevd_(info|events_total|script_|routing_rules)'
```

3. Prometheus scrape config (replace `<host>` with your edge/management target):

```yaml
scrape_configs:
  - job_name: netevd
    scrape_interval: 30s
    static_configs:
      - targets: ['<host>:9091']
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
curl -sf http://<host>:9091/metrics | grep netevd_uptime_seconds
```

6. **Empty / fail:** Connection refused → `metrics.enabled: false` or firewall; series missing → daemon just started or metrics init failed (see `journalctl -u netevd`).

7. **Success:** `netevd_info{version="…"}` present; counters increment when you bounce a link and routable hooks run.

## Related pages

- [REST API](rest-api.md)
- [CLI: status](../cli/status.md)
- [Fleet workflow](../../workflows.md)
