---
hero:
  eyebrow: USER GUIDE
  title: 'Hook: carrier / no-carrier'
---

## Purpose

Run scripts when the link layer goes UP (`carrier.d`) or DOWN (`no-carrier.d`) — cable plug, Wi‑Fi associate, or switch port flap — before L3 addresses exist.

## When to use it

- Notify ops when a physical link returns (PDU cycle, switch maintenance)
- Trigger low-level diagnostics that do not need an IP yet
- **Not** for “has an IP / default route” — use [Hook: routable](routable.md) for L3

## How to get there

- Directories: `/etc/netevd/carrier.d/`, `/etc/netevd/no-carrier.d/`
- Backends: all (`systemd-networkd`, NetworkManager, `dhclient`)
- Scripts must be executable; run in lexical order (`10-`, `20-`, …)

## Operate from CLI

1. Install a carrier-up logger:

```bash
sudo tee /etc/netevd/carrier.d/10-link-up.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "carrier UP on $LINK (index $LINKINDEX, backend $BACKEND)"
EOF
sudo chmod +x /etc/netevd/carrier.d/10-link-up.sh
```

2. Install a no-carrier handler:

```bash
sudo tee /etc/netevd/no-carrier.d/10-link-down.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "carrier DOWN on $LINK"
EOF
sudo chmod +x /etc/netevd/no-carrier.d/10-link-down.sh
```

3. Validate config and bounce the link to test:

```bash
netevd validate
sudo ip link set eth0 down && sudo ip link set eth0 up
netevd events -f -i eth0 -t carrier &
journalctl -u netevd -f
```

4. Dry-run hook env without executing:

```bash
netevd test /etc/netevd/carrier.d/10-link-up.sh -i eth0 -t carrier
```

5. **Empty / fail:** Script never runs → missing `+x`, wrong directory, or YAML filter with `action: ignore` on `docker*`/pattern match; check `journalctl -u netevd` for exit codes (non-zero exits are logged, other scripts still run).

6. **Success:** `logger` or script output appears on link up/down; `$LINK` and `$BACKEND` are set; carrier events visible in `netevd events`.

## Related pages

- [Hook: routable](routable.md)
- [CLI: events](../cli/events.md)
- [Admin basics](../../admin-basics.md)
