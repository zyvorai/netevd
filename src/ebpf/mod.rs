// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Observe-only eBPF event source (ring-buffer drain).
//!
//! * `drops.d`          — `skb:kfree_skb`
//! * `tcp-retransmit.d` — `tcp:tcp_retransmit_skb`
//! * `tcp-reset.d`      — `tcp:tcp_receive_reset` / `tcp:tcp_send_reset`

pub mod coalesce;
pub mod decode;
pub mod events;
pub mod loader;
pub mod service;

pub use service::spawn;

#[cfg(test)]
pub use events::{ObsEvent, SampleKey};

use crate::config::EbpfConfig;

pub fn is_requested(cfg: &EbpfConfig) -> bool {
    cfg.enabled && (cfg.drops || cfg.tcp_retransmit || cfg.tcp_reset)
}
