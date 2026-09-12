---
hero:
  eyebrow: USER GUIDE
  title: Configuration YAML
---

## Purpose

Tune backend, monitored interfaces, policy routing, API/metrics bind, event filters, and audit logging — the control plane for every hook and CLI query.

## When to use it

- Initial install: copy from `config/netevd.example.yaml`
- Multi-homed servers: list interfaces under `routing.policy_rules`
- Noisy lab hosts: add `filters` to ignore `docker*`, `veth*`, `br-*`
- Fleet scrape: enable metrics and optionally widen API bind on trusted networks

## How to get there

- Live file: `/etc/netevd/netevd.yaml`
- Template: `config/netevd.example.yaml` in the repo or release tarball
- Validate before apply: `netevd validate`

## Operate from CLI

1. Copy the example and edit:

```bash
sudo install -Dm644 config/netevd.example.yaml /etc/netevd/netevd.yaml
sudo ${EDITOR:-vi} /etc/netevd/netevd.yaml
```

2. Set backend to match the host:

```yaml
system:
  backend: "systemd-networkd"   # or NetworkManager | dhclient
```

3. Limit monitoring scope (empty list = all interfaces):

```yaml
monitoring:
  interfaces:
    - eth0
    - eth1
```

4. Enable multi-homed policy routing:

```yaml
routing:
  policy_rules:
    - eth1
```

5. Open API/metrics for fleet hosts (trusted network only):

```yaml
api:
  enabled: true
  bind_address: "127.0.0.1"   # use 0.0.0.0 only on isolated management nets
  port: 9090
metrics:
  enabled: true
  port: 9091
```

6. Add a filter to ignore container interfaces:

```yaml
filters:
  - match_rule:
      interface_pattern: "docker*"
    action: ignore
```

7. Validate, apply, verify:

```bash
netevd validate -c /etc/netevd/netevd.yaml
sudo systemctl restart netevd
netevd status -f json | jq .
journalctl -u netevd -n 20 --no-pager
```

8. **Empty / fail:** Parse error → validator prints the offending key; hooks stop firing → check filter `action: ignore`; API unreachable → `api.enabled: false` or bind still `127.0.0.1` on remote scrape.

9. **Success:** Summary from `netevd validate` matches intent; daemon logs show chosen backend; hooks and `list rules` behave per YAML.

## Related pages

- [CLI: validate / reload](../cli/validate-reload.md)
- [Admin basics](../../admin-basics.md)
- [REST API](rest-api.md)
