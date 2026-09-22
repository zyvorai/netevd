# Observe-only eBPF events

netevd's job is **kernel + manager events → hooks**. Netlink already covers
link, address, and route. Two classes of failure never show up there:

| Gap | Kernel source | Hook dir |
|-----|---------------|----------|
| Packet dropped in the stack | `skb:kfree_skb` (`skb_drop_reason`) | `/etc/netevd/drops.d/` |
| TCP retransmission | `tcp:tcp_retransmit_skb` | `/etc/netevd/tcp-retransmit.d/` |

This is **not** a datapath. Programs never return `XDP_DROP` / `TC_ACT_SHOT`.
They only sample and hand the same `netevd.event.v1` payload to scripts.

Enforcement, process attribution, DNS/SNI, and cgroup policy belong in
[Netra](https://github.com/zyvorai/netra) / PacketWolf — see `docs/ROADMAP.md`.

## Enable

1. Build with the feature and the BPF object:

```bash
make -C ebpf
cargo build --release --features ebpf
sudo install -Dm644 ebpf/netevd-ebpf.o /usr/lib/netevd/netevd-ebpf.o
```

2. Config (`/etc/netevd/netevd.yaml`):

```yaml
ebpf:
  enabled: true
  drops: true
  tcp_retransmit: false
  debounce_ms: 250
  min_count: 8          # ignore one-off drops
  object_path: ""       # default search path
```

3. Caps: the daemon keeps `CAP_NET_ADMIN`. Loading programs also needs
   `CAP_BPF` and `CAP_PERFMON` (kernel 5.8+). Tracepoint attach reads
   `/sys/kernel/tracing` (often mode 0700), so the unit also grants
   `CAP_DAC_READ_SEARCH`. The shipped systemd unit includes all four.

## Hook environment

Same contract as netlink hooks, plus:

| Variable | Meaning |
|----------|---------|
| `$BACKEND` | always `ebpf` |
| `$DROP_REASON` | `QDISC_DROP`, `NETFILTER_DROP`, `NEIGH_FAILED`, … |
| `$DROP_LOCATION` | tracepoint name |
| `$PROTOCOL` | `tcp` / `udp` / `icmp` / `other` |
| `$SPORT` / `$DPORT` | if parsed from the skb |
| `$COUNT` | samples coalesced in `debounce_ms` |
| `$JSON` | full `netevd.event.v1` |

Example:

```bash
cat >/etc/netevd/drops.d/10-log.sh <<'SH'
#!/bin/bash
logger -t netevd "drop if=$LINK reason=$DROP_REASON proto=$PROTOCOL n=$COUNT"
SH
chmod +x /etc/netevd/drops.d/10-log.sh
```

## What we will not add here

- XDP/TC allow/deny maps
- process `comm` / UID / cgroup enforcement
- DNS qname / SNI inspection
- replacing Cilium or nftables

Those collide with Netra and with the Community-edition size limit in the
roadmap.
