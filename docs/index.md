# netevd

**netevd** runs your scripts the moment something changes on a Linux network interface — link up/down, a new IP, a route change — instead of you writing a NetworkManager dispatcher script, a systemd-networkd `ExecStartPost` hack, or a cron job that polls `ip addr` every few seconds. It bridges **systemd-networkd**, **NetworkManager**, and **dhclient** into one event system, with sub-100ms netlink-driven latency, automatic policy routing for multi-homed hosts, a REST API, Prometheus metrics, and a defense-in-depth security model.

See the full [README on GitHub](https://github.com/zyvorai/netevd) for the complete feature tour, quick start, configuration reference, and security/performance details.

## Start here

- [FAQ](FAQ.md) — licensing, support, production readiness, and platform questions
- [Community Edition vs Enterprise](enterprise.md) — feature and support comparison
- [User documentation](user/README.md) — installation, CLI, hooks, and the operator surfaces
- [Using the operator surfaces](user/using-the-dashboard.md) — CLI, REST API, Prometheus, and journal-based operation
- [Getting started](user/getting-started.md) — install netevd and fire your first hook
- [Admin basics](user/admin-basics.md) — config, hooks, ports, privilege, and validation at a glance
