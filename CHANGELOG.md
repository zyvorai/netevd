# Changelog

## 0.4.1 — 2026-09-22

- **Observe-only eBPF** (`--features ebpf`): `kfree_skb` → `drops.d`, `tcp_retransmit_skb` → `tcp-retransmit.d`, `tcp_*_reset` → `tcp-reset.d`, with ringbuf drain, CO-RE ports/L4, coalesce, iface/reason filters, and Prometheus `netevd_ebpf_*` metrics.
- glibc container image is Ubuntu 26.04 (`ghcr.io/zyvorai/netevd:latest-ubuntu`). The Alpine image is pinned to 3.23 (`:latest-alpine`).
- `scripts/deploy-remote.sh` builds with `--features ebpf`, installs the BPF object + unit drop-in, and checks attach plus observe hooks.
- Hooks-contract sandbox: tokio 1.52.3, which includes the fix for GHSA-rr8g-9fpq-6wmg.
- Dependency updates since 0.4.0, including tokio 1.53, hyper 1.11.1, zbus 5.19, rtnetlink 0.23, and tower-http 0.7.
- Docs: README eBPF highlight; install tarball name matches the release asset; Prometheus on the API port (`:9090/metrics`).

## 0.4.0 — 2026-09-07

- Release tarball, hook contract, and the operator docs shipped with tag `v0.4.0`.
