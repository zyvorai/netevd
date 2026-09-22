// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Shared layout with `ebpf/netevd_observe.c`.

use crate::hooks::HookEventV1;

pub const KIND_DROP: u8 = 1;
pub const KIND_RTX: u8 = 2;
pub const KIND_RST: u8 = 3;

/// Must match `struct netevd_obs_event`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ObsEvent {
    pub ts_ns: u64,
    pub ifindex: u32,
    pub reason: u32,
    pub kind: u8,
    pub ip_proto: u8,
    pub eth_proto: u16,
    pub sport: u16,
    pub dport: u16,
}

impl ObsEvent {
    pub const SIZE: usize = core::mem::size_of::<Self>();

    pub fn key(&self) -> SampleKey {
        SampleKey {
            ifindex: self.ifindex,
            kind: self.kind,
            reason: self.reason,
            protocol: self.effective_proto(),
            sport: self.sport,
            dport: self.dport,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            KIND_RTX => "rtx",
            KIND_RST => "rst",
            _ => "drop",
        }
    }

    pub fn effective_proto(&self) -> u8 {
        if self.ip_proto != 0 {
            return self.ip_proto;
        }
        match self.eth_proto {
            0x0806 => 0, // arp
            _ => 0,
        }
    }

    pub fn to_hook(&self, link: &str, count: u64) -> HookEventV1 {
        match self.kind {
            KIND_RTX => {
                let mut ev = HookEventV1::new("tcp-retransmit", link, self.ifindex, "ebpf");
                ev.protocol = Some("tcp".into());
                if self.sport != 0 {
                    ev.sport = Some(self.sport);
                }
                if self.dport != 0 {
                    ev.dport = Some(self.dport);
                }
                ev.count = Some(count);
                ev
            }
            KIND_RST => {
                let mut ev = HookEventV1::new("tcp-reset", link, self.ifindex, "ebpf");
                ev.protocol = Some("tcp".into());
                ev.drop_reason = Some(tcp_reset_reason_name(self.reason).into());
                ev.drop_location = Some(
                    if self.reason == 2 {
                        "tcp:tcp_send_reset"
                    } else {
                        "tcp:tcp_receive_reset"
                    }
                    .into(),
                );
                if self.sport != 0 {
                    ev.sport = Some(self.sport);
                }
                if self.dport != 0 {
                    ev.dport = Some(self.dport);
                }
                ev.count = Some(count);
                ev
            }
            _ => {
                let mut ev = HookEventV1::new("drops", link, self.ifindex, "ebpf");
                ev.drop_reason = Some(drop_reason_name(self.reason).to_string());
                ev.drop_location = Some("skb:kfree_skb".into());
                ev.protocol = Some(proto_name(self.ip_proto, self.eth_proto).into());
                if self.sport != 0 {
                    ev.sport = Some(self.sport);
                }
                if self.dport != 0 {
                    ev.dport = Some(self.dport);
                }
                ev.count = Some(count);
                ev
            }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SampleKey {
    pub ifindex: u32,
    pub kind: u8,
    pub reason: u32,
    pub protocol: u8,
    pub sport: u16,
    pub dport: u16,
}

pub fn tcp_reset_reason_name(code: u32) -> &'static str {
    match code {
        1 => "TCP_RECEIVE_RESET",
        2 => "TCP_SEND_RESET",
        _ => "TCP_RESET",
    }
}

pub fn drop_reason_name(code: u32) -> &'static str {
    // Linux 6.8 `enum skb_drop_reason` (dropreason-core.h), including
    // NOT_DROPPED_YET / CONSUMED at 0/1.
    match code {
        0 => "NOT_DROPPED_YET",
        1 => "CONSUMED",
        2 => "NOT_SPECIFIED",
        3 => "NO_SOCKET",
        4 => "PKT_TOO_SMALL",
        5 => "TCP_CSUM",
        6 => "SOCKET_FILTER",
        7 => "UDP_CSUM",
        8 => "NETFILTER_DROP",
        9 => "OTHERHOST",
        10 => "IP_CSUM",
        11 => "IP_INHDR",
        12 => "IP_RPFILTER",
        13 => "UNICAST_IN_L2_MULTICAST",
        14 => "XFRM_POLICY",
        15 => "IP_NOPROTO",
        16 => "SOCKET_RCVBUFF",
        17 => "PROTO_MEM",
        18 => "TCP_AUTH_HDR",
        19 => "TCP_MD5NOTFOUND",
        20 => "TCP_MD5UNEXPECTED",
        21 => "TCP_MD5FAILURE",
        22 => "TCP_AONOTFOUND",
        23 => "TCP_AOUNEXPECTED",
        24 => "TCP_AOKEYNOTFOUND",
        25 => "TCP_AOFAILURE",
        26 => "SOCKET_BACKLOG",
        27 => "TCP_FLAGS",
        28 => "TCP_ZEROWINDOW",
        29 => "TCP_OLD_DATA",
        30 => "TCP_OVERWINDOW",
        31 => "TCP_OFOMERGE",
        32 => "TCP_RFC7323_PAWS",
        33 => "TCP_OLD_SEQUENCE",
        34 => "TCP_INVALID_SEQUENCE",
        35 => "TCP_RESET",
        36 => "TCP_INVALID_SYN",
        37 => "TCP_CLOSE",
        38 => "TCP_FASTOPEN",
        39 => "TCP_OLD_ACK",
        40 => "TCP_TOO_OLD_ACK",
        41 => "TCP_ACK_UNSENT_DATA",
        42 => "TCP_OFO_QUEUE_PRUNE",
        43 => "TCP_OFO_DROP",
        44 => "IP_OUTNOROUTES",
        45 => "BPF_CGROUP_EGRESS",
        46 => "IPV6DISABLED",
        47 => "NEIGH_CREATEFAIL",
        48 => "NEIGH_FAILED",
        49 => "NEIGH_QUEUEFULL",
        50 => "NEIGH_DEAD",
        51 => "TC_EGRESS",
        52 => "QDISC_DROP",
        53 => "CPU_BACKLOG",
        54 => "XDP",
        55 => "TC_INGRESS",
        56 => "UNHANDLED_PROTO",
        57 => "CSUM",
        58 => "GSO_SEG",
        59 => "UCOPY_FAULT",
        60 => "DEV_HDR",
        61 => "DEV_READY",
        62 => "FULL_RING",
        63 => "NOMEM",
        64 => "HDR_TRUNC",
        65 => "TAP_FILTER",
        66 => "TAP_TXFILTER",
        67 => "ICMP_CSUM",
        68 => "INVALID_PROTO",
        69 => "IP_INADDRERRORS",
        70 => "IP_INNOROUTES",
        71 => "PKT_TOO_BIG",
        72 => "DUP_FRAG",
        73 => "FRAG_REASM_TIMEOUT",
        74 => "FRAG_TOO_FAR",
        75 => "TCP_MINTTL",
        76 => "IPV6_BAD_EXTHDR",
        77 => "IPV6_NDISC_FRAG",
        78 => "IPV6_NDISC_HOP_LIMIT",
        79 => "IPV6_NDISC_BAD_CODE",
        80 => "IPV6_NDISC_BAD_OPTIONS",
        81 => "IPV6_NDISC_NS_OTHERHOST",
        82 => "QUEUE_PURGE",
        83 => "TC_COOKIE_ERROR",
        84 => "PACKET_SOCK_ERROR",
        85 => "TC_CHAIN_NOTFOUND",
        86 => "TC_RECLASSIFY_LOOP",
        _ => "UNKNOWN",
    }
}

pub fn proto_name(ip_proto: u8, eth_proto: u16) -> &'static str {
    match ip_proto {
        1 => "icmp",
        6 => "tcp",
        17 => "udp",
        58 => "icmpv6",
        _ => match eth_proto {
            0x0806 => "arp",
            0x86dd => "ipv6",
            0x0800 => "ip",
            _ => "other",
        },
    }
}

/// Resolve a config reason token (`QDISC_DROP` or numeric) to a code.
pub fn parse_reason_token(token: &str) -> Option<u32> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(n) = t.parse::<u32>() {
        return Some(n);
    }
    let upper = t.to_ascii_uppercase();
    for code in 0u32..=86 {
        if drop_reason_name(code) == upper {
            return Some(code);
        }
    }
    match upper.as_str() {
        "TCP_RECEIVE_RESET" => Some(1),
        "TCP_SEND_RESET" => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_size() {
        assert_eq!(ObsEvent::SIZE, 24);
    }

    #[test]
    fn drop_hook() {
        let ev = ObsEvent {
            ts_ns: 1,
            ifindex: 2,
            reason: 52,
            kind: KIND_DROP,
            ip_proto: 6,
            eth_proto: 0x0800,
            sport: 443,
            dport: 12345,
        }
        .to_hook("eth0", 9);
        assert_eq!(ev.event, "drops");
        assert_eq!(ev.drop_reason.as_deref(), Some("QDISC_DROP"));
        assert_eq!(ev.sport, Some(443));
        assert_eq!(ev.count, Some(9));
    }

    #[test]
    fn rtx_hook() {
        let ev = ObsEvent {
            ts_ns: 1,
            ifindex: 3,
            reason: 0,
            kind: KIND_RTX,
            ip_proto: 6,
            eth_proto: 0x0800,
            sport: 443,
            dport: 51234,
        }
        .to_hook("wg0", 2);
        assert_eq!(ev.event, "tcp-retransmit");
        assert_eq!(ev.sport, Some(443));
    }

    #[test]
    fn rst_hook() {
        let ev = ObsEvent {
            ts_ns: 1,
            ifindex: 0,
            reason: 1,
            kind: KIND_RST,
            ip_proto: 6,
            eth_proto: 0x0800,
            sport: 80,
            dport: 40000,
        }
        .to_hook("if0", 1);
        assert_eq!(ev.event, "tcp-reset");
        assert_eq!(ev.drop_reason.as_deref(), Some("TCP_RECEIVE_RESET"));
    }

    #[test]
    fn parse_reason_names() {
        assert_eq!(parse_reason_token("QDISC_DROP"), Some(52));
        assert_eq!(parse_reason_token("46"), Some(46));
        assert_eq!(parse_reason_token("nope"), None);
    }
}
