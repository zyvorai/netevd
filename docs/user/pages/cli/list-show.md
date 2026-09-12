---
hero:
  eyebrow: USER GUIDE
  title: 'CLI: list / show'
---

## Purpose

Inspect interfaces, routes, policy rules, and registered hook scripts — the same objects netevd tracks for hook dispatch and automatic policy routing.

## When to use it

- Multi-homed hosts: confirm per-interface routing tables after `routing.policy_rules` changes
- Debugging: compare `netevd list rules` with `ip rule list`
- Inventory: see which hook scripts netevd knows about before a rollout

## How to get there

- Commands: `netevd list …`, `netevd show …`
- Default API: `http://127.0.0.1:9090` (`--endpoint` for remote hosts)
- Config reference: `/etc/netevd/netevd.yaml`

## Operate from CLI

1. List tracked interfaces:

```bash
netevd list interfaces
netevd list interfaces -f json
```

2. List kernel routes netevd is aware of:

```bash
netevd list routes
```

3. List policy routing rules (multi-homed verification):

```bash
netevd list rules
ip rule list
ip route show table $((200 + $(cat /sys/class/net/eth1/ifindex)))
```

4. List hook scripts discovered under `/etc/netevd/*.d/`:

```bash
netevd list scripts
```

5. Drill into one interface:

```bash
netevd show interface eth0
netevd show interface eth0 -f yaml --endpoint http://<host>:9090
```

6. **Empty / fail:** Lists empty but links exist → widen `monitoring.interfaces` (empty = all) or remove an overly broad `filters` ignore rule; API errors → see [CLI: status](status.md).

7. **Success:** Interface count matches expectations; `list rules` shows `from`/`to` pairs for each address on policy-routed interfaces.

## Related pages

- [CLI: events](events.md)
- [Hook: routes](../hooks/routes.md)
- [Configuration YAML](../ops/config-yaml.md)
- [Multi-homed workflow](../../workflows.md)
