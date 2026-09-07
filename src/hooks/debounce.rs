// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Coalesce bursty netlink events per (link, event) key.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::event::HookEventV1;

#[derive(Debug, Clone)]
struct Pending {
    event: HookEventV1,
    first_seen: Instant,
    last_seen: Instant,
    count: u32,
}

#[derive(Debug)]
pub struct HookDebouncer {
    window: Duration,
    pending: HashMap<String, Pending>,
    coalesced_total: u64,
}

impl HookDebouncer {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            pending: HashMap::new(),
            coalesced_total: 0,
        }
    }

    pub fn from_millis(ms: u64) -> Self {
        Self::new(Duration::from_millis(ms))
    }

    fn key(event: &HookEventV1) -> String {
        format!("{}:{}", event.link, event.event)
    }

    /// Push an event. Returns the previous pending event if this push
    /// starts a *new* window after the old one already expired (caller
    /// should dispatch that previous one immediately).
    pub fn push(&mut self, event: HookEventV1) -> Option<HookEventV1> {
        let now = Instant::now();
        let key = Self::key(&event);
        match self.pending.get_mut(&key) {
            Some(p) if now.duration_since(p.first_seen) <= self.window => {
                self.coalesced_total += 1;
                p.count += 1;
                p.last_seen = now;
                p.event = event;
                None
            }
            Some(_) => {
                let old = self.pending.remove(&key).unwrap();
                self.pending.insert(
                    key,
                    Pending {
                        event,
                        first_seen: now,
                        last_seen: now,
                        count: 1,
                    },
                );
                Some(old.event)
            }
            None => {
                self.pending.insert(
                    key,
                    Pending {
                        event,
                        first_seen: now,
                        last_seen: now,
                        count: 1,
                    },
                );
                None
            }
        }
    }

    /// Drain events whose window has elapsed.
    pub fn take_ready(&mut self) -> Vec<HookEventV1> {
        let now = Instant::now();
        let window = self.window;
        let mut ready_keys = Vec::new();
        for (k, p) in &self.pending {
            if now.duration_since(p.first_seen) >= window {
                ready_keys.push(k.clone());
            }
        }
        let mut out = Vec::with_capacity(ready_keys.len());
        for k in ready_keys {
            if let Some(p) = self.pending.remove(&k) {
                out.push(p.event);
            }
        }
        out
    }

    /// Drain everything (shutdown / tests).
    pub fn take_all(&mut self) -> Vec<HookEventV1> {
        self.pending.drain().map(|(_, p)| p.event).collect()
    }

    pub fn coalesced_total(&self) -> u64 {
        self.coalesced_total
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

/// Pure helper used by tests: N events in one window collapse to 1.
pub fn coalesce_count(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::event::HookEventV1;
    use std::thread;

    fn ev(link: &str, event: &str) -> HookEventV1 {
        HookEventV1::new(event, link, 2, "systemd-networkd")
    }

    #[test]
    fn burst_coalesces_same_key() {
        let mut d = HookDebouncer::from_millis(80);
        for _ in 0..20 {
            let flushed = d.push(ev("eth0", "routable"));
            assert!(flushed.is_none());
        }
        assert_eq!(d.pending_len(), 1);
        assert_eq!(d.coalesced_total(), 19);
        assert_eq!(coalesce_count(20), 1);
    }

    #[test]
    fn different_keys_stay_separate() {
        let mut d = HookDebouncer::from_millis(80);
        d.push(ev("eth0", "routable"));
        d.push(ev("eth0", "routes"));
        d.push(ev("eth1", "routable"));
        assert_eq!(d.pending_len(), 3);
        assert_eq!(d.coalesced_total(), 0);
    }

    #[test]
    fn take_ready_after_window() {
        let mut d = HookDebouncer::from_millis(20);
        d.push(ev("eth0", "address-added"));
        thread::sleep(Duration::from_millis(35));
        let ready = d.take_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event, "address-added");
        assert_eq!(d.pending_len(), 0);
    }

    #[test]
    fn last_payload_wins() {
        let mut d = HookDebouncer::from_millis(80);
        d.push(ev("eth0", "mtu").with_mtu(1500));
        d.push(ev("eth0", "mtu").with_mtu(9000));
        let all = d.take_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].mtu, Some(9000));
    }
}
