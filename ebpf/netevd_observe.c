// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0
//
// Observe-only. Never returns XDP_DROP / TC_ACT_SHOT.
// Layout must match src/ebpf/events.rs :: ObsEvent.
//
//   clang -O2 -g -target bpf -c ebpf/netevd_observe.c -o netevd-ebpf.o

#include <linux/bpf.h>
#include <linux/in.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL"; /* helper helpers need GPL-compat */

#define NETEVD_KIND_DROP 1
#define NETEVD_KIND_RTX  2

/* Packed the same way on every arch we care about. */
struct netevd_obs_event {
	__u64 ts_ns;
	__u32 ifindex;
	__u32 reason;
	__u8 kind;
	__u8 ip_proto;
	__u16 eth_proto;
	__u16 sport;
	__u16 dport;
};

struct {
	__uint(type, BPF_MAP_TYPE_RINGBUF);
	__uint(max_entries, 256 * 1024);
} events SEC(".maps");

/*
 * Generated layout of tracepoint:skb:kfree_skb (stable enough on 5.17+).
 * Fields after the common trace_entry (8+ bytes of type/flags/preempt/pid,
 * padded to 8 on 64-bit).
 *
 *   skbaddr, location, protocol (ethertype), reason
 */
struct kfree_skb_tp {
	__u64 pad;          /* common fields, ignored */
	void *skbaddr;
	void *location;
	__u16 protocol;
	__u16 pad2;
	__u32 reason;
};

/* Minimal sk_buff slice: skb_iif sits at a well-known offset on 64-bit
 * kernels we target; CO-RE relocates it when vmlinux BTF is present. */
struct sk_buff___netevd {
	int skb_iif;
} __attribute__((preserve_access_index));

static __always_inline void submit_drop(__u32 ifindex, __u32 reason, __u16 eth_proto)
{
	struct netevd_obs_event *e;

	e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
	if (!e)
		return;
	e->ts_ns = bpf_ktime_get_ns();
	e->ifindex = ifindex;
	e->reason = reason;
	e->kind = NETEVD_KIND_DROP;
	e->ip_proto = 0;
	e->eth_proto = bpf_ntohs(eth_proto);
	e->sport = 0;
	e->dport = 0;
	bpf_ringbuf_submit(e, 0);
}

SEC("tp/skb/kfree_skb")
int observe_kfree_skb(struct kfree_skb_tp *ctx)
{
	__u32 ifindex = 0;
	__u32 reason = 0;
	__u16 eth_proto = 0;
	void *skbaddr = NULL;

	if (!ctx)
		return 0;

	reason = ctx->reason;
	eth_proto = ctx->protocol;
	skbaddr = ctx->skbaddr;

	if (skbaddr) {
		struct sk_buff___netevd *skb = skbaddr;
		int iif = 0;

		/* CO-RE when BTF exists; plain probe-read otherwise. */
#ifdef __has_builtin
		iif = BPF_CORE_READ(skb, skb_iif);
#else
		bpf_probe_read_kernel(&iif, sizeof(iif), &skb->skb_iif);
#endif
		if (iif > 0)
			ifindex = (__u32)iif;
	}

	submit_drop(ifindex, reason, eth_proto);
	return 0;
}

/*
 * tracepoint:tcp:tcp_retransmit_skb
 * Typical args: skbaddr, skaddr. Ports live on inet_sock; we leave them 0
 * unless a later PR adds CO-RE inet_sport / dport. ifindex from skb_iif.
 */
struct tcp_rtx_tp {
	__u64 pad;
	void *skbaddr;
	void *skaddr;
};

SEC("tp/tcp/tcp_retransmit_skb")
int observe_tcp_retransmit(struct tcp_rtx_tp *ctx)
{
	struct netevd_obs_event *e;
	__u32 ifindex = 0;

	if (ctx && ctx->skbaddr) {
		struct sk_buff___netevd *skb = ctx->skbaddr;
		int iif = 0;
#ifdef __has_builtin
		iif = BPF_CORE_READ(skb, skb_iif);
#else
		bpf_probe_read_kernel(&iif, sizeof(iif), &skb->skb_iif);
#endif
		if (iif > 0)
			ifindex = (__u32)iif;
	}

	e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
	if (!e)
		return 0;
	e->ts_ns = bpf_ktime_get_ns();
	e->ifindex = ifindex;
	e->reason = 0;
	e->kind = NETEVD_KIND_RTX;
	e->ip_proto = IPPROTO_TCP;
	e->eth_proto = 0x0800;
	e->sport = 0;
	e->dport = 0;
	bpf_ringbuf_submit(e, 0);
	return 0;
}
