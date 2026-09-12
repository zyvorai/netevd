---
hero:
  eyebrow: USER GUIDE
  title: 'Hook: routable'
---

## Purpose

Fire when an interface has full L3 connectivity — addresses and a usable path — on systemd-networkd and dhclient backends. This is the most common hook directory for “start my service when the network is actually up.”

## When to use it

- Register DNS, announce presence, or start app stacks when `$ADDRESSES` is meaningful
- DHCP renewals on edge nodes (dhclient path)
- Consume rich `$JSON` from systemd-networkd (MTU, DNS, routes) when `emit_json: true`

## How to get there

- Directory: `/etc/netevd/routable.d/`
- Backends: `systemd-networkd`, `dhclient` (not NM — use [Hook: manager](manager.md))
- Example env: `$LINK`, `$ADDRESSES`, `$JSON` (networkd), `$DHCP_*` (dhclient)

## Operate from CLI

1. Install the canonical first hook from [Getting Started](../../getting-started.md):

```bash
sudo tee /etc/netevd/routable.d/01-notify.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "Interface $LINK is routable: $ADDRESSES (backend=$BACKEND)"
EOF
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
```

2. Enable JSON payload for networkd scripts in YAML:

```bash
sudo sed -n '/systemd_networkd:/,/networkmanager:/p' /etc/netevd/netevd.yaml
# ensure: emit_json: true under backends.systemd_networkd
netevd validate && sudo systemctl restart netevd
```

3. Trigger a routable event (networkd host):

```bash
sudo networkctl renew eth0   # or reapply your .network unit
netevd events -f -i eth0 -t routable
```

4. dhclient host — renew lease:

```bash
sudo dhclient -r eth0 && sudo dhclient eth0
echo "check \$DHCP_ADDRESS \$DHCP_GATEWAY in journal"
```

5. Inspect script inventory:

```bash
netevd list scripts
ls -la /etc/netevd/routable.d/
```

6. **Empty / fail:** Hook runs on carrier but not routable → script is in wrong `*.d` dir; `$JSON` empty → `emit_json: false`; dhclient vars missing → event fired before lease parse completes.

7. **Success:** `logger` shows addresses; dependent service starts; `netevd_script_executions_total` increments (see [Prometheus](../ops/prometheus.md)).

## Related pages

- [Hook: carrier](carrier.md)
- [Common workflows](../../workflows.md)
- [CLI: validate / reload](../cli/validate-reload.md)
