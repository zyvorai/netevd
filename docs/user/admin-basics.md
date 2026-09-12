---
hero:
  eyebrow: USER GUIDE
  title: Admin basics
---

| Topic | Detail |
|-------|--------|
| Config | `/etc/netevd/netevd.yaml` (see `config/netevd.example.yaml`) |
| Hooks | `/etc/netevd/{carrier,no-carrier,configured,degraded,routable,activated,disconnected,manager,routes}.d/` |
| Unit | `netevd.service` — `systemctl enable --now netevd` |
| Logs | `journalctl -u netevd -f` |
| REST | `api.bind_address` + `api.port` (default `127.0.0.1:9090`) |
| Metrics | `metrics.port` (default `9091`) |
| Backend | `system.backend`: `systemd-networkd` · `NetworkManager` · `dhclient` |
| Privilege | Drops to `netevd` user; retains `CAP_NET_ADMIN`; hooks inherit no caps |
| Dry-run | `netevd --dry-run …` skips script execution / mutations |
| Validate | `netevd validate` against the config file |
| Reload | `netevd reload` (API) or `systemctl restart netevd` after YAML/hook changes |
| Security | See repo `SECURITY.md`; restrict YAML permissions (`chmod 640`) |

## Hook environment (common)

| Variable | Content |
|----------|---------|
| `$LINK` | Interface name |
| `$LINKINDEX` | Kernel ifindex |
| `$STATE` | Normalized event state |
| `$BACKEND` | Source backend |
| `$ADDRESSES` | Address / gateway / DNS summary |
| `$JSON` | Full payload (systemd-networkd when enabled) |
| `$DHCP_*` | dhclient lease fields |

## Operate from CLI

1. **Config check:** `netevd validate && netevd status`
2. **Hook inventory:** `ls -la /etc/netevd/routable.d/ && netevd list scripts`
3. **Live debug:** `netevd events -f` + `journalctl -u netevd -f`
4. **Policy routing:** `netevd list rules` and `ip rule list`
5. **Remote API (tunnel):** `ssh -L 9090:127.0.0.1:9090 root@<host>` then `curl http://127.0.0.1:9090/health`
6. **Metrics scrape:** `curl http://<host>:9091/metrics | grep netevd_info`

Use `<host>` in runbooks — never hard-code lab IPs.

## Support

[GitHub issues](https://github.com/zyvorai/netevd/issues) · [Contact Zyvor](/contact) for Enterprise
