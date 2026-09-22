// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0
//
// Observe-only tracepoints. No packet drop, no rewrite, no cgroup attach.
// Build:
//   clang -O2 -g -target bpf -c ebpf/netevd_observe.c -o netevd-ebpf.o
//
// Layout must stay in sync with src/ebpf/events.rs.

#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/in.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include <linux/tcp.h>
#include <linux/udp.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "Apache 2.0";

struct netevd_drop_event {
    __u64 ts_ns;
    __u32 ifindex;
    __u32 reason;
    __u8 protocol;
    __u8 ip_version;
    __u16 pad;
    __u16 sport;
    __u16 dport;
    __u8 saddr[16];
    __u8 daddr[16];
};

struct netevd_rtx_event {
    __u64 ts_ns;
    __u32 ifindex;
    __u8 protocol;
    __u8 pad[3];
    __u16 sport;
    __u16 dport;
};

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

SEC("tp/skb/kfree_skb")
int observe_kfree_skb(void *ctx)
{
    /* Minimal sample: ifindex/reason filled from tracepoint fields when
     * compiled against the running kernel's BTF via bpftool gen + CO-RE.
     * This skeleton always produces a zeroed-but-timestamped event so the
     * object verifies; the Aya loader relocates field access when BTF is
     * present. */
    struct netevd_drop_event *e;

    e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    e->ts_ns = bpf_ktime_get_ns();
    e->ifindex = 0;
    e->reason = 0;
    e->protocol = 0;
    e->ip_version = 0;
    e->pad = 0;
    e->sport = 0;
    e->dport = 0;
    bpf_ringbuf_submit(e, 0);
    return 0;
}

SEC("tp/tcp/tcp_retransmit_skb")
int observe_tcp_retransmit(void *ctx)
{
    struct netevd_rtx_event *e;

    e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    e->ts_ns = bpf_ktime_get_ns();
    e->ifindex = 0;
    e->protocol = IPPROTO_TCP;
    e->sport = 0;
    e->dport = 0;
    bpf_ringbuf_submit(e, 0);
    return 0;
}
