// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Shared layout with `ebpf/netevd_observe.c`.

use crate::hooks::HookEventV1;

pub const KIND_DROP: u8 = 1;
pub const KIND_RTX: u8 = 2;

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

pub fn drop_reason_name(code: u32) -> &'static str {
    match code {
        0 => "NOT_SPECIFIED",
        2 => "NO_SOCKET",
        3 => "PKT_TOO_SMALL",
        4 => "TCP_CSUM",
        5 => "SOCKET_FILTER",
        6 => "UDP_CSUM",
        7 => "NETFILTER_DROP",
        8 => "OTHERHOST",
        9 => "IP_CSUM",
        10 => "IP_INHDR",
        11 => "IP_RPFILTER",
        14 => "IP_NOPROTO",
        15 => "SOCKET_RCVBUFF",
        20 => "SOCKET_BACKLOG",
        21 => "TCP_FLAGS",
        29 => "TCP_RESET",
        38 => "IP_OUTNOROUTES",
        39 => "BPF_CGROUP_EGRESS",
        41 => "NEIGH_CREATEFAIL",
        42 => "NEIGH_FAILED",
        43 => "NEIGH_QUEUEFULL",
        44 => "NEIGH_DEAD",
        45 => "TC_EGRESS",
        46 => "QDISC_DROP",
        47 => "CPU_BACKLOG",
        48 => "XDP",
        52 => "TC_INGRESS",
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
            reason: 46,
            kind: KIND_DROP,
            ip_proto: 6,
            eth_proto: 0x0800,
            sport: 0,
            dport: 0,
        }
        .to_hook("eth0", 9);
        assert_eq!(ev.event, "drops");
        assert_eq!(ev.drop_reason.as_deref(), Some("QDISC_DROP"));
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
}
