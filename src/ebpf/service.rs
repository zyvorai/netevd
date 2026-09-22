// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Debounce eBPF samples and dispatch hook scripts.

use std::sync::Arc;

use tracing::warn;

use super::events::{DropSample, RetransmitSample};
use super::loader;
use crate::config::Config;
use crate::hooks::{HookDispatchOpts, HookEventV1};
use crate::network::NetworkState;
use tokio::sync::RwLock;

pub fn spawn(config: Config, state: Arc<RwLock<NetworkState>>, opts: HookDispatchOpts) {
    if !super::is_requested(&config.ebpf) {
        return;
    }
    tokio::spawn(async move {
        let attached = match loader::aya_attach::load_and_attach(&config.ebpf) {
            Ok(a) => {
                tracing::info!(
                    attachments = a.count,
                    "eBPF observe-only programs attached"
                );
                a
            }
            Err(e) => {
                warn!(
                    "eBPF requested but not attached ({e:#}); drops.d / tcp-retransmit.d stay idle"
                );
                return;
            }
        };
        // Hold the BPF object for the process lifetime so programs stay attached.
        // Ringbuf → debounce → dispatch lands in a follow-up.
        let _ = (state, opts, attached);
        std::future::pending::<()>().await;
    });
}

pub fn hook_from_drop(link: &str, reason: u32, proto: u8, count: u64) -> HookEventV1 {
    DropSample {
        ts_ns: 0,
        ifindex: 1,
        reason,
        protocol: proto,
        ip_version: 4,
        _pad: 0,
        sport: 0,
        dport: 0,
        saddr: [0; 16],
        daddr: [0; 16],
    }
    .to_hook(link, count)
}

pub fn hook_from_rtx(link: &str, sport: u16, dport: u16, count: u64) -> HookEventV1 {
    RetransmitSample {
        ts_ns: 0,
        ifindex: 1,
        protocol: 6,
        _pad: [0; 3],
        sport,
        dport,
    }
    .to_hook(link, count)
}
