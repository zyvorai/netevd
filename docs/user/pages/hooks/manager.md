---
hero:
  eyebrow: USER GUIDE
  title: 'Hook: manager / NetworkManager'
---

## Purpose

React to NetworkManager device and manager state via `activated.d`, `disconnected.d`, and `manager.d` — the NM equivalent of systemd-networkd's configured/routable hooks.

## When to use it

- Hosts where NetworkManager is primary (`system.backend: NetworkManager`)
- VPN or Wi‑Fi profiles that activate/disconnect without classic netlink carrier semantics
- Manager-wide scripts (restart dependent services when NM reconnects)

## How to get there

- Directories: `/etc/netevd/activated.d/`, `/etc/netevd/disconnected.d/`, `/etc/netevd/manager.d/`
- Config: set `system.backend: NetworkManager` in `/etc/netevd/netevd.yaml`
- Requires NM D-Bus signals; unit starts after `NetworkManager.service`

## Operate from CLI

1. Confirm backend and restart:

```bash
grep backend /etc/netevd/netevd.yaml
netevd validate
sudo systemctl restart netevd
```

2. Add an activation hook:

```bash
sudo tee /etc/netevd/activated.d/20-nm-up.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "NM activated $LINK state=$STATE addrs=$ADDRESSES"
EOF
sudo chmod +x /etc/netevd/activated.d/20-nm-up.sh
```

3. Add a disconnect hook:

```bash
sudo tee /etc/netevd/disconnected.d/10-nm-down.sh >/dev/null <<'EOF'
#!/bin/bash
logger -t netevd "NM disconnected $LINK"
EOF
sudo chmod +x /etc/netevd/disconnected.d/10-nm-down.sh
```

4. Manager-level script (all interfaces / manager events):

```bash
sudo install -m755 ./my-manager-hook.sh /etc/netevd/manager.d/05-manager.sh
```

5. Test with NM CLI while tailing events:

```bash
netevd events -f -t activated &
nmcli connection down my-wifi && nmcli connection up my-wifi
journalctl -u netevd -n 20 --no-pager
```

6. **Empty / fail:** Hooks silent on NM host → backend still `systemd-networkd`; NM device state changes but no scripts → check filters for `backend: NetworkManager` with `action: log` only.

7. **Success:** Activation/disconnect logged; `$STATE` reflects NM normalized state; events appear in `netevd events -t activated`.

## Related pages

- [Configuration YAML](../ops/config-yaml.md)
- [Hook: routable](routable.md)
- [Admin basics](../../admin-basics.md)
