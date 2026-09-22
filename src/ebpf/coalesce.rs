// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Time-window coalesce so a flood of kfree_skb becomes one hook.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::events::{ObsEvent, SampleKey};

#[derive(Clone, Debug)]
pub struct Bucket {
    pub first: Instant,
    pub sample: ObsEvent,
    pub count: u64,
}

#[derive(Debug)]
pub struct Coalescer {
    window: Duration,
    min_count: u64,
    buckets: HashMap<SampleKey, Bucket>,
}

impl Coalescer {
    pub fn new(window: Duration, min_count: u64) -> Self {
        Self {
            window: window.max(Duration::from_millis(1)),
            min_count: min_count.max(1),
            buckets: HashMap::new(),
        }
    }

    pub fn push(&mut self, sample: ObsEvent) {
        let key = sample.key();
        self.buckets
            .entry(key)
            .and_modify(|b| b.count += 1)
            .or_insert(Bucket {
                first: Instant::now(),
                sample,
                count: 1,
            });
    }

    pub fn flush_due(&mut self, now: Instant) -> Vec<(ObsEvent, u64)> {
        let window = self.window;
        let min_count = self.min_count;
        let due: Vec<_> = self
            .buckets
            .iter()
            .filter(|(_, b)| now.duration_since(b.first) >= window && b.count >= min_count)
            .map(|(k, _)| k.clone())
            .collect();
        let mut out = Vec::with_capacity(due.len());
        for k in due {
            if let Some(b) = self.buckets.remove(&k) {
                out.push((b.sample, b.count));
            }
        }
        out
    }

    pub fn len(&self) -> usize {
        self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebpf::events::{KIND_DROP, ObsEvent};

    fn sample(ifindex: u32, reason: u32) -> ObsEvent {
        ObsEvent {
            ts_ns: 0,
            ifindex,
            reason,
            kind: KIND_DROP,
            ip_proto: 6,
            eth_proto: 0x0800,
            sport: 0,
            dport: 0,
        }
    }

    #[test]
    fn coalesces_same_key() {
        let mut c = Coalescer::new(Duration::from_millis(50), 3);
        c.push(sample(2, 46));
        c.push(sample(2, 46));
        c.push(sample(2, 46));
        c.push(sample(2, 7)); // different reason
        assert_eq!(c.len(), 2);
        let flushed = c.flush_due(Instant::now() + Duration::from_millis(60));
        assert_eq!(flushed.len(), 2);
        let qdisc = flushed
            .iter()
            .find(|(s, _)| s.reason == 46)
            .unwrap();
        assert_eq!(qdisc.1, 3);
    }

    #[test]
    fn respects_min_count() {
        let mut c = Coalescer::new(Duration::from_millis(1), 5);
        c.push(sample(1, 1));
        let flushed = c.flush_due(Instant::now() + Duration::from_secs(1));
        assert!(flushed.is_empty());
        assert_eq!(c.len(), 1);
    }
}
