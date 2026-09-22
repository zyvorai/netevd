// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Safe decode of ring-buffer bytes into `ObsEvent`.

use super::events::ObsEvent;

pub fn decode(bytes: &[u8]) -> Option<ObsEvent> {
    if bytes.len() < ObsEvent::SIZE {
        return None;
    }
    // Manual little-endian read — ringbuf payload is packed C layout on
    // the same endianness as the kernel (LE on every arch we ship).
    let ts_ns = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
    let ifindex = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    let reason = u32::from_le_bytes(bytes[12..16].try_into().ok()?);
    let kind = bytes[16];
    let ip_proto = bytes[17];
    let eth_proto = u16::from_le_bytes(bytes[18..20].try_into().ok()?);
    let sport = u16::from_le_bytes(bytes[20..22].try_into().ok()?);
    let dport = u16::from_le_bytes(bytes[22..24].try_into().ok()?);
    Some(ObsEvent {
        ts_ns,
        ifindex,
        reason,
        kind,
        ip_proto,
        eth_proto,
        sport,
        dport,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebpf::events::{KIND_DROP, ObsEvent};

    #[test]
    fn roundtrip_bytes() {
        let mut raw = [0u8; 24];
        raw[0..8].copy_from_slice(&0x1111_u64.to_le_bytes());
        raw[8..12].copy_from_slice(&2u32.to_le_bytes());
        raw[12..16].copy_from_slice(&46u32.to_le_bytes());
        raw[16] = KIND_DROP;
        raw[17] = 6;
        raw[18..20].copy_from_slice(&0x0800u16.to_le_bytes());
        let ev = decode(&raw).unwrap();
        assert_eq!(ev.ifindex, 2);
        assert_eq!(ev.reason, 46);
        assert_eq!(ev.kind, KIND_DROP);
        assert_eq!(ObsEvent::SIZE, raw.len());
    }

    #[test]
    fn short_slice_none() {
        assert!(decode(&[0u8; 8]).is_none());
    }
}
