// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Drain the ring buffer, coalesce, dispatch hooks.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{info, warn};

use super::coalesce::Coalescer;
use super::decode;
use super::events::{self, ObsEvent, KIND_RST};
use super::loader;
use crate::config::Config;
use crate::hooks::match_iface::InterfaceSelector;
use crate::hooks::{dispatch::dispatch_event, HookDispatchOpts};
use crate::metrics;
use crate::network::NetworkState;
use tokio::sync::RwLock;

pub fn spawn(config: Config, state: Arc<RwLock<NetworkState>>, opts: HookDispatchOpts) {
    if !super::is_requested(&config.ebpf) {
        return;
    }
    tokio::spawn(async move {
        if let Err(e) = run(config, state, opts).await {
            warn!("eBPF observer stopped: {e:#}");
            if let Some(m) = metrics::get_global_metrics() {
                m.ebpf_attached.set(0.0);
            }
        }
    });
}

struct ReasonFilter {
    allow: HashSet<u32>,
    deny: HashSet<u32>,
}

impl ReasonFilter {
    fn from_config(cfg: &crate::config::EbpfConfig) -> Self {
        let mut allow = HashSet::new();
        for t in &cfg.reasons_allow {
            if let Some(c) = events::parse_reason_token(t) {
                allow.insert(c);
            }
        }
        let mut deny = HashSet::new();
        for t in &cfg.reasons_deny {
            if let Some(c) = events::parse_reason_token(t) {
                deny.insert(c);
            }
        }
        Self { allow, deny }
    }

    fn allows(&self, sample: &ObsEvent) -> bool {
        let code = sample.reason;
        if !self.deny.is_empty() && self.deny.contains(&code) {
            return false;
        }
        if !self.allow.is_empty() && !self.allow.contains(&code) {
            return false;
        }
        true
    }
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
        let selector = InterfaceSelector::from_lists(
            config.monitoring.match_patterns.clone(),
            config.monitoring.exclude.clone(),
        )
        .with_default_excludes();
        let reasons = ReasonFilter::from_config(&config.ebpf);
        let skip_unknown = config.ebpf.skip_unknown_ifindex;

        let mut loaded = loader::aya_attach::load_and_attach(&config.ebpf)?;
        info!(
            attachments = loaded.attachments,
            "eBPF observe-only programs attached"
        );
        if let Some(m) = metrics::get_global_metrics() {
            m.ebpf_attached.set(loaded.attachments as f64);
        }
        let mut ring = loader::aya_attach::take_ring(&mut loaded.bpf)?;
        let mut coal = Coalescer::new(
            Duration::from_millis(config.ebpf.debounce_ms),
            config.ebpf.min_count,
        );
        let mut tick = tokio::time::interval(Duration::from_millis(
            config.ebpf.debounce_ms.max(20),
        ));
        let mut last_lost = 0u64;
        loop {
            while let Some(item) = ring.next() {
                if let Some(ev) = decode::decode(&item) {
                    if let Some(m) = metrics::get_global_metrics() {
                        m.ebpf_samples_total
                            .with_label_values(&[ev.kind_label()])
                            .inc();
                    }
                    if skip_unknown && ev.ifindex == 0 && ev.kind != KIND_RST {
                        continue;
                    }
                    if !reasons.allows(&ev) {
                        continue;
                    }
                    // tcp-reset TPs often lack ifindex; skip iface filter for those.
                    if ev.kind != KIND_RST {
                        let link = resolve_link(&state, ev.ifindex).await;
                        if !selector.allows(&link) {
                            continue;
                        }
                    }
                    coal.push(ev);
                }
            }
            let lost = loader::aya_attach::read_ring_lost(&mut loaded.bpf);
            if lost > last_lost {
                if let Some(m) = metrics::get_global_metrics() {
                    m.ebpf_ring_lost_total.inc_by((lost - last_lost) as f64);
                }
                last_lost = lost;
            }
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
    if let Some(m) = metrics::get_global_metrics() {
        m.ebpf_hooks_total.with_label_values(&[&ev.event]).inc();
    }
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
