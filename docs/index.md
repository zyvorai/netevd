---
hero:
  eyebrow: PRODUCT
  title: netevd
  lead: "netevd runs your scripts the moment something changes on a Linux network interface — link up/down, a new IP, a route change — instead of a NetworkManager dispatcher script, a systemd-networkd ExecStartPost hack, or a cron job polling ip addr."
  highlights:
    - {value: "3", label: "backends, one event system"}
    - {value: "14", label: "hook directories", footnote: "1"}
    - {value: "<100ms", label: "netlink event latency"}
    - {value: "1000+", label: "events / sec", footnote: "2"}
    - {value: "9", label: "REST API endpoints", footnote: "3"}
  hub_bands:
    - {icon: "❓", title: "FAQ", description: "Licensing, support, production readiness, and platform questions.", href: FAQ.md}
    - {icon: "🏢", title: "Community Edition vs Enterprise", description: "Feature and support comparison.", href: enterprise.md}
    - {icon: "📘", title: "User documentation", description: "Installation, CLI, hooks, and the operator surfaces.", href: user/README.md}
    - {icon: "🖥️", title: "Using the operator surfaces", description: "CLI, REST API, Prometheus, and journal-based operation.", href: user/using-the-dashboard.md}
    - {icon: "🚀", title: "Getting started", description: "Install netevd and fire your first hook.", href: user/getting-started.md}
    - {icon: "⚙️", title: "Admin basics", description: "Config, hooks, ports, privilege, and validation at a glance.", href: user/admin-basics.md}
footnotes:
  - {marker: "1", text: "14 hook directories span carrier/link, address, route, MTU, and NetworkManager/systemd-networkd manager state.", href: hooks-contract.md, href_label: "See the Hook Contract."}
  - {marker: "2", text: "1000+ events/sec sustained throughput, per the README's Performance table.", href: "https://github.com/zyvorai/netevd#performance", href_label: "See Performance in the README."}
  - {marker: "3", text: "9 REST endpoints for status, interfaces, routes, events, and metrics.", href: "https://github.com/zyvorai/netevd#rest-api", href_label: "See REST API in the README."}
---

**netevd** runs your scripts the moment something changes on a Linux network interface — link up/down, a new IP, a route change — instead of you writing a NetworkManager dispatcher script, a systemd-networkd `ExecStartPost` hack, or a cron job that polls `ip addr` every few seconds. It bridges **systemd-networkd**, **NetworkManager**, and **dhclient** into one event system, with sub-100ms netlink-driven latency, automatic policy routing for multi-homed hosts, a REST API, Prometheus metrics, and a defense-in-depth security model.

See the full [README on GitHub](https://github.com/zyvorai/netevd) for the complete feature tour, quick start, configuration reference, and security/performance details.

<div class="icon-badge-list" markdown="1">

- 🔌 Netlink-driven events — sub-100ms latency, zero polling
- 🛣️ Automatic policy routing for multi-homed hosts
- 📊 Prometheus metrics
- 🌐 REST API for status, interfaces, routes, and events
- 🔒 Defense-in-depth security model

</div>

## Start here

- [FAQ](FAQ.md) — licensing, support, production readiness, and platform questions
- [Community Edition vs Enterprise](enterprise.md) — feature and support comparison
- [User documentation](user/README.md) — installation, CLI, hooks, and the operator surfaces
- [Using the operator surfaces](user/using-the-dashboard.md) — CLI, REST API, Prometheus, and journal-based operation
- [Getting started](user/getting-started.md) — install netevd and fire your first hook
- [Admin basics](user/admin-basics.md) — config, hooks, ports, privilege, and validation at a glance
