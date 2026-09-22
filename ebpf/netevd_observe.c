// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0
//
// Observe-only. Never returns XDP_DROP / TC_ACT_SHOT.
// Layout must match src/ebpf/events.rs :: ObsEvent.
//
//   make -C ebpf

#include <linux/bpf.h>
#include <linux/in.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL"; /* probe helpers need GPL-compat */

#define NETEVD_KIND_DROP 1
#define NETEVD_KIND_RTX  2
#define NETEVD_KIND_RST  3

#define ETH_P_IP   0x0800
#define ETH_P_IPV6 0x86DD

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

/* Userspace polls key 0 for ringbuf reserve failures. */
struct {
	__uint(type, BPF_MAP_TYPE_ARRAY);
	__uint(max_entries, 1);
	__type(key, __u32);
	__type(value, __u64);
} ring_lost SEC(".maps");

struct kfree_skb_tp {
	__u64 pad;
	void *skbaddr;
	void *location;
	__u16 protocol;
	__u16 pad2;
	__u32 reason;
};

struct sk_buff___netevd {
	int skb_iif;
	unsigned char *head;
	__u16 network_header;
	__u16 transport_header;
} __attribute__((preserve_access_index));

/* Minimal IPv4 / L4 headers for probe-read (no full vmlinux.h). */
struct iphdr_min {
	__u8 ihl_version;
	__u8 tos;
	__be16 tot_len;
	__be16 id;
	__be16 frag_off;
	__u8 ttl;
	__u8 protocol;
	__be16 check;
	__be32 saddr;
	__be32 daddr;
};

struct ipv6hdr_min {
	__u8 priority_version;
	__u8 flow_lbl[3];
	__be16 payload_len;
	__u8 nexthdr;
	__u8 hop_limit;
	__u8 saddr[16];
	__u8 daddr[16];
};

struct tcphdr_min {
	__be16 source;
	__be16 dest;
};

struct udphdr_min {
	__be16 source;
	__be16 dest;
};

/* Only fields we read — CO-RE relocates by BTF name. */
struct sock_common___netevd {
	__be16 skc_dport;
	__u16 skc_num;
} __attribute__((preserve_access_index));

struct sock___netevd {
	struct sock_common___netevd __sk_common;
} __attribute__((preserve_access_index));

static __always_inline void bump_lost(void)
{
	__u32 key = 0;
	__u64 *v = bpf_map_lookup_elem(&ring_lost, &key);
	if (v)
		__sync_fetch_and_add(v, 1);
}

static __always_inline __u32 skb_ifindex(struct sk_buff___netevd *skb)
{
	int iif = 0;

	if (!skb)
		return 0;
	iif = BPF_CORE_READ(skb, skb_iif);
	return iif > 0 ? (__u32)iif : 0;
}

static __always_inline void fill_l4_from_skb(struct sk_buff___netevd *skb, __u16 eth_proto,
					    __u8 *ip_proto, __u16 *sport, __u16 *dport)
{
	unsigned char *head;
	__u16 nh, th;
	__u8 proto = 0;

	*ip_proto = 0;
	*sport = 0;
	*dport = 0;
	if (!skb)
		return;

	head = BPF_CORE_READ(skb, head);
	nh = BPF_CORE_READ(skb, network_header);
	th = BPF_CORE_READ(skb, transport_header);
	if (!head)
		return;

	if (eth_proto == ETH_P_IP || eth_proto == bpf_htons(ETH_P_IP) || eth_proto == 0x0800) {
		struct iphdr_min iph = {};

		if (bpf_probe_read_kernel(&iph, sizeof(iph), head + nh))
			return;
		proto = iph.protocol;
	} else if (eth_proto == ETH_P_IPV6 || eth_proto == bpf_htons(ETH_P_IPV6) ||
		   eth_proto == 0x86dd) {
		struct ipv6hdr_min ip6 = {};

		if (bpf_probe_read_kernel(&ip6, sizeof(ip6), head + nh))
			return;
		proto = ip6.nexthdr;
	} else {
		return;
	}

	*ip_proto = proto;
	if (proto == IPPROTO_TCP || proto == IPPROTO_UDP) {
		struct tcphdr_min ports = {};

		if (bpf_probe_read_kernel(&ports, sizeof(ports), head + th))
			return;
		*sport = bpf_ntohs(ports.source);
		*dport = bpf_ntohs(ports.dest);
	}
}

static __always_inline void fill_ports_from_sk(struct sock___netevd *sk, __u16 *sport_out,
					      __u16 *dport_out)
{
	__be16 sk_dport;
	__u16 sk_num;

	if (!sk)
		return;
	sk_dport = BPF_CORE_READ(sk, __sk_common.skc_dport);
	sk_num = BPF_CORE_READ(sk, __sk_common.skc_num);
	*sport_out = sk_num;
	*dport_out = bpf_ntohs(sk_dport);
}

static __always_inline int submit_event(__u32 ifindex, __u32 reason, __u8 kind, __u8 ip_proto,
					__u16 eth_proto, __u16 sport, __u16 dport)
{
	struct netevd_obs_event *e;

	e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
	if (!e) {
		bump_lost();
		return 0;
	}
	e->ts_ns = bpf_ktime_get_ns();
	e->ifindex = ifindex;
	e->reason = reason;
	e->kind = kind;
	e->ip_proto = ip_proto;
	e->eth_proto = eth_proto;
	e->sport = sport;
	e->dport = dport;
	bpf_ringbuf_submit(e, 0);
	return 0;
}

SEC("tp/skb/kfree_skb")
int observe_kfree_skb(struct kfree_skb_tp *ctx)
{
	__u32 ifindex = 0;
	__u32 reason = 0;
	__u16 eth_proto = 0;
	__u8 ip_proto = 0;
	__u16 sport = 0, dport = 0;
	void *skbaddr;

	if (!ctx)
		return 0;

	reason = ctx->reason;
	eth_proto = bpf_ntohs(ctx->protocol);
	skbaddr = ctx->skbaddr;
	if (skbaddr) {
		struct sk_buff___netevd *skb = skbaddr;

		ifindex = skb_ifindex(skb);
		fill_l4_from_skb(skb, eth_proto, &ip_proto, &sport, &dport);
	}
	return submit_event(ifindex, reason, NETEVD_KIND_DROP, ip_proto, eth_proto, sport, dport);
}

struct tcp_rtx_tp {
	__u64 pad;
	void *skbaddr;
	void *skaddr;
};

SEC("tp/tcp/tcp_retransmit_skb")
int observe_tcp_retransmit(struct tcp_rtx_tp *ctx)
{
	__u32 ifindex = 0;
	__u8 ip_proto = IPPROTO_TCP;
	__u16 sport = 0, dport = 0;

	if (!ctx)
		return 0;
	if (ctx->skbaddr) {
		struct sk_buff___netevd *skb = ctx->skbaddr;
		__u8 p = 0;
		__u16 s = 0, d = 0;

		ifindex = skb_ifindex(skb);
		fill_l4_from_skb(skb, 0x0800, &p, &s, &d);
		if (s || d) {
			sport = s;
			dport = d;
		}
		if (p)
			ip_proto = p;
	}
	if ((!sport && !dport) && ctx->skaddr)
		fill_ports_from_sk((struct sock___netevd *)ctx->skaddr, &sport, &dport);
	return submit_event(ifindex, 0, NETEVD_KIND_RTX, ip_proto, 0x0800, sport, dport);
}

/*
 * tcp:tcp_receive_reset / tcp:tcp_send_reset — typical arg is sk.
 * reason: 1 = receive RST, 2 = send RST (userspace maps names).
 */
struct tcp_reset_tp {
	__u64 pad;
	void *skaddr;
};

SEC("tp/tcp/tcp_receive_reset")
int observe_tcp_receive_reset(struct tcp_reset_tp *ctx)
{
	__u16 sport = 0, dport = 0;

	if (ctx && ctx->skaddr)
		fill_ports_from_sk((struct sock___netevd *)ctx->skaddr, &sport, &dport);
	return submit_event(0, 1, NETEVD_KIND_RST, IPPROTO_TCP, 0x0800, sport, dport);
}

SEC("tp/tcp/tcp_send_reset")
int observe_tcp_send_reset(struct tcp_reset_tp *ctx)
{
	__u16 sport = 0, dport = 0;

	if (ctx && ctx->skaddr)
		fill_ports_from_sk((struct sock___netevd *)ctx->skaddr, &sport, &dport);
	return submit_event(0, 2, NETEVD_KIND_RST, IPPROTO_TCP, 0x0800, sport, dport);
}
