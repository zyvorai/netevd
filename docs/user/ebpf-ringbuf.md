# PR-2 — drain the observe ring buffer

PR-1 loaded programs. This document is the live path.

```
kfree_skb / tcp_retransmit_skb
        │
        ▼
  netevd_obs_event  ── ringbuf `events` (256 KiB)
        │
        ▼
  decode.rs  →  Coalescer (debounce_ms / min_count)
        │
        ▼
  drops.d / tcp-retransmit.d   +  $JSON netevd.event.v1
```

## Fields the kernel actually fills

| Field | Source | Notes |
|-------|--------|--------|
| `reason` | `kfree_skb` TP arg | Maps to `$DROP_REASON` |
| `eth_proto` | `kfree_skb` TP `protocol` | Ethertype; `$PROTOCOL` falls back to this |
| `ifindex` | `skb->skb_iif` via CO-RE / probe-read | `0` → hook `$LINK=if0` until BTF is good |
| `kind` | 1 drop / 2 rtx | Picks hook dir |
| `sport` / `dport` | reserved | Filled in a later CO-RE inet_sock pass |

## Caps

Install `systemd/netevd-ebpf.conf` as
`/etc/systemd/system/netevd.service.d/ebpf.conf`.

Needs kernel 5.8+ (`CAP_BPF`, `CAP_PERFMON`). On 5.15-era RHEL-likes
`CAP_SYS_ADMIN` is the fallback if the unit still fails to load programs.

## Metrics (add to the existing registry when you wire this in)

```
netevd_ebpf_samples_total{kind="drop|rtx"}
netevd_ebpf_hooks_total{event="drops|tcp-retransmit"}
netevd_ebpf_ring_lost_total
netevd_ebpf_attached
```

Keep them next to the other Prometheus counters in `src/metrics`. This pack
does not touch that module so the overlay stays small.

## What is still not in Community

XDP shield, cgroup deny, DNS qname, process `comm`. That is Netra.
