# Observe-only eBPF events

netevd's job is **kernel + manager events → hooks**. Netlink already covers
link, address, and route. Classes of failure that never show up there:

| Gap | Kernel source | Hook dir |
|-----|---------------|----------|
| Packet dropped in the stack | `skb:kfree_skb` (`skb_drop_reason`) | `/etc/netevd/drops.d/` |
| TCP retransmission | `tcp:tcp_retransmit_skb` | `/etc/netevd/tcp-retransmit.d/` |
| TCP RST send/receive | `tcp:tcp_send_reset` / `tcp:tcp_receive_reset` | `/etc/netevd/tcp-reset.d/` |

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
sudo install -Dm644 systemd/netevd-ebpf.conf \
  /etc/systemd/system/netevd.service.d/ebpf.conf
```

2. Config (`/etc/netevd/netevd.yaml`):

```yaml
ebpf:
  enabled: true
  drops: true
  tcp_retransmit: false
  tcp_reset: false
  debounce_ms: 250
  min_count: 8
  object_path: ""
  skip_unknown_ifindex: true
  reasons_allow: []          # e.g. ["QDISC_DROP", "52"]
  reasons_deny: ["NO_SOCKET"]
```

Interface include/exclude reuses `monitoring.match_patterns` / `exclude`.

3. Caps: `CAP_NET_ADMIN` plus `CAP_BPF`, `CAP_PERFMON`, and
   `CAP_DAC_READ_SEARCH` (tracefs is often mode 0700). See the shipped unit
   and `systemd/netevd-ebpf.conf`.

Live ring-buffer drain, CO-RE ports, and metrics: [ebpf-ringbuf.md](ebpf-ringbuf.md).

## Hook environment

| Variable | Meaning |
|----------|---------|
| `$BACKEND` | always `ebpf` |
| `$DROP_REASON` | `QDISC_DROP`, `NETFILTER_DROP`, `TCP_RECEIVE_RESET`, … |
| `$DROP_LOCATION` | tracepoint name |
| `$PROTOCOL` | `tcp` / `udp` / `icmp` / `other` |
| `$SPORT` / `$DPORT` | from skb L4 or inet sock when CO-RE succeeds |
| `$COUNT` | samples coalesced in `debounce_ms` |
| `$JSON` | full `netevd.event.v1` |

## What we will not add here

- XDP/TC allow/deny maps
- process `comm` / UID / cgroup enforcement
- DNS qname / SNI inspection
- replacing Cilium or nftables
