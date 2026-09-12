---
hero:
  eyebrow: USER GUIDE
  title: 'Hook: routes'
---

## Purpose

React to routing-table changes via `/etc/netevd/routes.d/` — default-route moves, VPN injection, or policy-table updates that netevd detects through its route watcher.

## When to use it

- Audit when the default gateway shifts between uplinks
- Side effects after netevd installs per-interface policy tables (see [Multi-homed workflow](../../workflows.md))
- Integrate with SD-WAN or VPN clients that mutate `ip route` frequently

## How to get there

- Directory: `/etc/netevd/routes.d/`
- Backends: all
- Complements automatic policy routing from `routing.policy_rules` in YAML

## Operate from CLI

1. Install a route-change auditor:

```bash
sudo tee /etc/netevd/routes.d/30-audit-routes.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "route change on $LINK state=$STATE backend=$BACKEND"
EOF
sudo chmod +x /etc/netevd/routes.d/30-audit-routes.sh
```

2. Enable policy routing for a secondary uplink in YAML:

```bash
grep -A5 '^routing:' /etc/netevd/netevd.yaml
netevd validate
sudo systemctl restart netevd
```

3. Verify rules and routes after address acquisition:

```bash
netevd list rules
netevd list routes
ip route show table all | head -30
```

4. Watch route events during a controlled change:

```bash
netevd events -f -t routes &
sudo ip route replace default via 192.168.1.1 dev eth1   # lab only
```

5. **Empty / fail:** No route hooks despite table changes → script not executable; route events filtered; compare with `journalctl -u netevd` for netlink errors.

6. **Success:** Hook logs on table mutation; `list rules` reflects policy tables; traffic sourced from secondary IP uses correct table (`ip rule list`).

## Related pages

- [CLI: list / show](../cli/list-show.md)
- [Configuration YAML](../ops/config-yaml.md)
- [Multi-homed workflow](../../workflows.md)
