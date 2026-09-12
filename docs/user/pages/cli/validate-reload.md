---
hero:
  eyebrow: USER GUIDE
  title: 'CLI: validate / reload / test'
---

## Purpose

Validate YAML offline, request a config reload through the API, and dry-run hook scripts — the safe path before touching production networking.

## When to use it

- Before every edit to `/etc/netevd/netevd.yaml`
- After adding or renaming hook scripts (validate + restart/reload)
- When developing a new hook: preview environment variables without firing real events

## How to get there

- Commands: `netevd validate`, `netevd reload`, `netevd test`, `netevd version`
- Config path: `/etc/netevd/netevd.yaml` (override with `-c` / `--config`)
- Global flags on the daemon binary: `--dry-run`, `-v` (verbose)

## Operate from CLI

1. Validate the live config (default path):

```bash
sudo netevd validate
```

2. Validate a staged file before install:

```bash
netevd validate -c /tmp/netevd.yaml
```

3. Request reload via API (preferred when the daemon supports hot reload):

```bash
netevd reload --endpoint http://127.0.0.1:9090
curl -X POST http://127.0.0.1:9090/api/v1/reload -H 'Content-Type: application/json' -d '{}'
```

4. When reload is not yet applied server-side, restart the unit (config + hook directory changes):

```bash
sudo systemctl restart netevd
journalctl -u netevd -n 30 --no-pager
```

5. Dry-run the daemon (no script execution, no routing mutations):

```bash
sudo netevd --dry-run -c /etc/netevd/netevd.yaml start -f
```

6. Preview hook environment for a script under test:

```bash
netevd test /etc/netevd/routable.d/01-notify.sh -i eth0 -t routable --ip 192.168.1.10
```

7. Print build/feature matrix:

```bash
netevd version --detailed
```

8. **Empty / fail:** Validation errors name the YAML key; reload returns `success: false` → fall back to `systemctl restart netevd`; test mode prints env but does not execute — confirm with a real link bounce + `journalctl`.

9. **Success:** `✓ Configuration is valid` summary lists backend, interfaces, and policy rules; daemon restarts cleanly; test output shows expected `$LINK`, `$STATE`, `$ADDRESSES`.

## Related pages

- [Configuration YAML](../ops/config-yaml.md)
- [Admin basics](../../admin-basics.md)
- [CLI: events](events.md)
