// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Drain the ring buffer, coalesce, dispatch hooks.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{info, warn};

use super::coalesce::Coalescer;
use super::decode;
use super::events::ObsEvent;
use super::loader;
use crate::config::Config;
use crate::hooks::{dispatch::dispatch_event, HookDispatchOpts};
use crate::network::NetworkState;
use tokio::sync::RwLock;

pub fn spawn(config: Config, state: Arc<RwLock<NetworkState>>, opts: HookDispatchOpts) {
    if !super::is_requested(&config.ebpf) {
        return;
    }
    tokio::spawn(async move {
        if let Err(e) = run(config, state, opts).await {
            warn!("eBPF observer stopped: {e:#}");
        }
    });
}

async fn run(
    config: Config,
    state: Arc<RwLock<NetworkState>>,
    opts: HookDispatchOpts,
) -> anyhow::Result<()> {
    #[cfg(not(feature = "ebpf"))]
    {
        let _ = (state, opts);
        loader::aya_attach::load_and_attach(&config.ebpf)?;
        return Ok(());
    }

    #[cfg(feature = "ebpf")]
    {
        let mut loaded = loader::aya_attach::load_and_attach(&config.ebpf)?;
        info!(
            attachments = loaded.attachments,
            "eBPF observe-only programs attached"
        );
        let mut ring = loader::aya_attach::take_ring(&mut loaded.bpf)?;
        let mut coal = Coalescer::new(
            Duration::from_millis(config.ebpf.debounce_ms),
            config.ebpf.min_count,
        );
        let mut tick = tokio::time::interval(Duration::from_millis(
            config.ebpf.debounce_ms.max(20),
        ));
        loop {
            let mut drained = 0u32;
            while let Some(item) = ring.next() {
                drained += 1;
                if let Some(ev) = decode::decode(&item) {
                    coal.push(ev);
                }
            }
            let _ = drained;
            tick.tick().await;
            for (sample, count) in coal.flush_due(Instant::now()) {
                dispatch_one(&state, &opts, sample, count).await;
            }
        }
    }
}

async fn dispatch_one(
    state: &Arc<RwLock<NetworkState>>,
    opts: &HookDispatchOpts,
    sample: ObsEvent,
    count: u64,
) {
    let link = resolve_link(state, sample.ifindex).await;
    let ev = sample.to_hook(&link, count);
    if let Err(e) = dispatch_event(&ev, opts).await {
        warn!(error = %e, event = %ev.event, "eBPF hook dispatch failed");
    }
}

async fn resolve_link(state: &Arc<RwLock<NetworkState>>, ifindex: u32) -> String {
    let guard = state.read().await;
    guard
        .get_link_name(ifindex)
        .cloned()
        .unwrap_or_else(|| format!("if{ifindex}"))
}
