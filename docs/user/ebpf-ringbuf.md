# Observe ring buffer — drain path

```
kfree_skb / tcp_retransmit_skb / tcp_*_reset
        │
        ▼
  netevd_obs_event  ── ringbuf `events` (256 KiB)
        │
        ▼
  decode → filter (iface + reasons) → Coalescer
        │
        ▼
  drops.d / tcp-retransmit.d / tcp-reset.d
```

## Fields

| Field | Source | Notes |
|-------|--------|--------|
| `reason` | TP arg | `$DROP_REASON` (Linux 6.8 `skb_drop_reason` names) |
| `eth_proto` / `ip_proto` | TP + skb IP header | `$PROTOCOL` |
| `ifindex` | `skb->skb_iif` CO-RE | `0` skipped when `skip_unknown_ifindex` |
| `kind` | 1 drop / 2 rtx / 3 rst | Hook directory |
| `sport` / `dport` | skb L4 or `inet_sock` CO-RE | `$SPORT` / `$DPORT` |

Buckets that never reach `min_count` expire at the end of `debounce_ms`
(they do not linger forever).

## Metrics

```
netevd_ebpf_samples_total{kind="drop|rtx|rst"}
netevd_ebpf_hooks_total{event="drops|tcp-retransmit|tcp-reset"}
netevd_ebpf_ring_lost_total
netevd_ebpf_attached
```

## Caps

Install `systemd/netevd-ebpf.conf` as
`/etc/systemd/system/netevd.service.d/ebpf.conf`.

## Still not in Community

XDP shield, cgroup deny, DNS qname, process `comm` — that is Netra.
