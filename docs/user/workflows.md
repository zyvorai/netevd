---
hero:
  eyebrow: USER GUIDE
  title: Common workflows
---

## A. First routable notify

1. Install and start `netevd`.
2. Drop an executable script in `/etc/netevd/routable.d/`.
3. Bounce the interface or renew DHCP.
4. Confirm `journalctl -u netevd`, `netevd events --tail 10`, and script side effects.

```bash
sudo systemctl enable --now netevd
sudo chmod +x /etc/netevd/routable.d/01-notify.sh
sudo ip link set eth0 down && sudo ip link set eth0 up
journalctl -u netevd -n 20 --no-pager
```

## B. Multi-homed policy routing

1. List secondary interfaces under `routing.policy_rules` in YAML.
2. Acquire addresses on each uplink.
3. Verify:

```bash
netevd validate
sudo systemctl restart netevd
netevd list rules
ip rule list
ip route show table $((200 + $(cat /sys/class/net/eth1/ifindex)))
```

Traffic sourced from an address on `eth1` should leave via `eth1`'s gateway.

## C. Watch events live

```bash
netevd events -f
# or filter:
netevd events -f -i eth0 -t routable
```

## D. Fleet observability

1. Set `api.bind_address: "0.0.0.0"` only on trusted management networks (or scrape via SSH tunnel).
2. Point Prometheus at `http://<host>:9091/metrics` and/or REST `http://<host>:9090/api/v1/status`.
3. Alert when `netevd_script_executions_total` goes quiet (daemon likely down).

## E. Safe change window

```bash
netevd validate -c /etc/netevd/netevd.yaml
sudo systemctl restart netevd
# or when hot reload is available:
netevd reload --endpoint http://127.0.0.1:9090
journalctl -u netevd -n 50 --no-pager
netevd status
```

## F. Ignore noisy interfaces

Add a YAML filter with `action: ignore` for `docker*`, `veth*`, etc. (see `config/netevd.example.yaml`), then:

```bash
netevd validate && sudo systemctl restart netevd
netevd events -f   # confirm docker events no longer dispatch hooks
```

## Operate from CLI

Every workflow above is CLI-first: hooks in `/etc/netevd/`, validation via `netevd validate`, observation via `netevd events` and `journalctl`, fleet checks via curl to `<host>:9090` / `:9091`. See [Page-by-page guides](pages/README.md) for command-level detail per surface.
