// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Shared layout between the BPF object (`ebpf/netevd_observe.c`) and userspace.

use crate::hooks::HookEventV1;

/// Must match `struct netevd_drop_event` in `ebpf/netevd_observe.c`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DropSample {
    pub ts_ns: u64,
    pub ifindex: u32,
    pub reason: u32,
    pub protocol: u8,
    pub ip_version: u8,
    pub _pad: u16,
    pub sport: u16,
    pub dport: u16,
    pub saddr: [u8; 16],
    pub daddr: [u8; 16],
}

/// Must match `struct netevd_rtx_event` in `ebpf/netevd_observe.c`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetransmitSample {
    pub ts_ns: u64,
    pub ifindex: u32,
    pub protocol: u8,
    pub _pad: [u8; 3],
    pub sport: u16,
    pub dport: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SampleKey {
    pub ifindex: u32,
    pub kind: SampleKind,
    pub reason: u32,
    pub protocol: u8,
    pub sport: u16,
    pub dport: u16,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SampleKind {
    Drop,
    Retransmit,
}

/// Subset of `enum skb_drop_reason` (linux/skbuff.h) we surface by name.
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
        12 => "UNICAST_IN_L2_MULTICAST",
        13 => "XFRM_POLICY",
        14 => "IP_NOPROTO",
        15 => "SOCKET_RCVBUFF",
        16 => "PROTO_MEM",
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

pub fn proto_name(p: u8) -> &'static str {
    match p {
        1 => "icmp",
        6 => "tcp",
        17 => "udp",
        58 => "icmpv6",
        _ => "other",
    }
}

impl DropSample {
    pub fn key(&self) -> SampleKey {
        SampleKey {
            ifindex: self.ifindex,
            kind: SampleKind::Drop,
            reason: self.reason,
            protocol: self.protocol,
            sport: self.sport,
            dport: self.dport,
        }
    }

    pub fn to_hook(&self, link: &str, count: u64) -> HookEventV1 {
        let mut ev = HookEventV1::new("drops", link, self.ifindex, "ebpf");
        ev.drop_reason = Some(drop_reason_name(self.reason).to_string());
        ev.drop_location = Some("skb:kfree_skb".into());
        ev.protocol = Some(proto_name(self.protocol).into());
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

impl RetransmitSample {
    pub fn key(&self) -> SampleKey {
        SampleKey {
            ifindex: self.ifindex,
            kind: SampleKind::Retransmit,
            reason: 0,
            protocol: 6,
            sport: self.sport,
            dport: self.dport,
        }
    }

    pub fn to_hook(&self, link: &str, count: u64) -> HookEventV1 {
        let mut ev = HookEventV1::new("tcp-retransmit", link, self.ifindex, "ebpf");
        ev.protocol = Some("tcp".into());
        ev.sport = Some(self.sport);
        ev.dport = Some(self.dport);
        ev.count = Some(count);
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_reason_known() {
        assert_eq!(drop_reason_name(46), "QDISC_DROP");
        assert_eq!(drop_reason_name(7), "NETFILTER_DROP");
        assert_eq!(drop_reason_name(9999), "UNKNOWN");
    }

    #[test]
    fn drop_sample_hook_schema() {
        let s = DropSample {
            ts_ns: 1,
            ifindex: 2,
            reason: 46,
            protocol: 6,
            ip_version: 4,
            _pad: 0,
            sport: 443,
            dport: 51234,
            saddr: [0; 16],
            daddr: [0; 16],
        };
        let ev = s.to_hook("eth0", 12);
        assert_eq!(ev.event, "drops");
        assert_eq!(ev.backend, "ebpf");
        assert_eq!(ev.drop_reason.as_deref(), Some("QDISC_DROP"));
        assert_eq!(ev.count, Some(12));
        let env = ev.to_env();
        assert_eq!(env.get("DROP_REASON").unwrap(), "QDISC_DROP");
        assert_eq!(env.get("COUNT").unwrap(), "12");
        assert!(env.get("JSON").unwrap().contains("netevd.event.v1"));
    }
}
