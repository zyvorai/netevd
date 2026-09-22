// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Observe-only eBPF event source.
//!
//! netevd is not a packet datapath (that is Netra / PacketWolf). This module
//! attaches **tracepoints** the kernel already emits and turns them into the
//! same hook contract as netlink:
//!
//! * `drops.d`          — `skb:kfree_skb` / `skb:kfree_skb_reason`
//! * `tcp-retransmit.d` — `tcp:tcp_retransmit_skb`
//!
//! Compiled always so config + event schema stay consistent. Loading a BPF
//! object requires the `ebpf` cargo feature (Aya) and `CAP_BPF` /
//! `CAP_PERFMON`.

mod events;
mod loader;
mod service;

pub use service::spawn;

#[cfg(test)]
pub use events::{DropSample, RetransmitSample, SampleKey};

use crate::config::EbpfConfig;

pub fn is_requested(cfg: &EbpfConfig) -> bool {
    cfg.enabled && (cfg.drops || cfg.tcp_retransmit)
}
