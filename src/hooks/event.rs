// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Versioned hook event payload (`netevd.event.v1`).

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_V1: &str = "netevd.event.v1";

/// Hook directory stem / `$STATE` / `$EVENT`.
pub const HOOK_STATES: &[&str] = &[
    "carrier",
    "no-carrier",
    "configured",
    "degraded",
    "routable",
    "activated",
    "disconnected",
    "manager",
    "routes",
    "off",
    "dormant",
    "enslaved",
    "linger",
    "missing",
    "address-added",
    "address-removed",
    "link-added",
    "link-removed",
    "mtu",
    "neigh",
    "dns",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookEventV1 {
    pub schema: String,
    pub event: String,
    pub ts: String,
    pub link: String,
    pub index: u32,
    pub state: String,
    pub backend: String,
    #[serde(default)]
    pub addresses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operstate: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub routes_delta: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(default)]
    pub dns: Vec<String>,
}

impl HookEventV1 {
    pub fn new(event: &str, link: &str, index: u32, backend: &str) -> Self {
        Self {
            schema: SCHEMA_V1.to_string(),
            event: event.to_string(),
            ts: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            link: link.to_string(),
            index,
            state: event.to_string(),
            backend: backend.to_string(),
            addresses: Vec::new(),
            mtu: None,
            operstate: None,
            flags: Vec::new(),
            routes_delta: Vec::new(),
            gateway: None,
            dns: Vec::new(),
        }
    }

    pub fn with_addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = addresses;
        self
    }

    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    pub fn with_operstate(mut self, operstate: impl Into<String>) -> Self {
        self.operstate = Some(operstate.into());
        self
    }

    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }

    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = dns;
        self
    }

    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_routes_delta(mut self, routes_delta: Vec<String>) -> Self {
        self.routes_delta = routes_delta;
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Environment block scripts already understand, plus `$JSON` / `$EVENT`.
    pub fn to_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("LINK".into(), self.link.clone());
        env.insert("LINKINDEX".into(), self.index.to_string());
        env.insert("STATE".into(), self.state.clone());
        env.insert("EVENT".into(), self.event.clone());
        env.insert("BACKEND".into(), self.backend.clone());
        env.insert("ADDRESSES".into(), self.addresses.join(" "));
        if let Some(gw) = &self.gateway {
            env.insert("GATEWAY".into(), gw.clone());
        }
        if !self.dns.is_empty() {
            env.insert("DNS".into(), self.dns.join(" "));
        }
        if let Some(mtu) = self.mtu {
            env.insert("MTU".into(), mtu.to_string());
        }
        if let Ok(json) = self.to_json() {
            env.insert("JSON".into(), json);
        }
        env
    }

    pub fn is_known_event(name: &str) -> bool {
        HOOK_STATES.contains(&name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_constant() {
        let ev = HookEventV1::new("routable", "eth0", 2, "systemd-networkd")
            .with_addresses(vec!["192.168.1.100".into()])
            .with_mtu(1500)
            .with_gateway("192.168.1.1");
        let json = ev.to_json().unwrap();
        assert!(json.contains(SCHEMA_V1));
        let back: HookEventV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(back.link, "eth0");
        assert_eq!(back.addresses, vec!["192.168.1.100"]);
        assert_eq!(back.mtu, Some(1500));
        assert_eq!(back.gateway.as_deref(), Some("192.168.1.1"));
    }

    #[test]
    fn env_keeps_legacy_keys() {
        let ev = HookEventV1::new("address-added", "eth1", 3, "dhclient")
            .with_addresses(vec!["10.0.0.5".into()]);
        let env = ev.to_env();
        assert_eq!(env.get("LINK").unwrap(), "eth1");
        assert_eq!(env.get("LINKINDEX").unwrap(), "3");
        assert_eq!(env.get("STATE").unwrap(), "address-added");
        assert_eq!(env.get("EVENT").unwrap(), "address-added");
        assert_eq!(env.get("BACKEND").unwrap(), "dhclient");
        assert_eq!(env.get("ADDRESSES").unwrap(), "10.0.0.5");
        let json = env.get("JSON").unwrap();
        assert!(json.contains("\"schema\":\"netevd.event.v1\""));
    }

    #[test]
    fn known_events_cover_new_dirs() {
        for name in [
            "address-added",
            "address-removed",
            "link-added",
            "link-removed",
            "mtu",
            "neigh",
            "dns",
            "routable",
        ] {
            assert!(HookEventV1::is_known_event(name), "{name}");
        }
        assert!(!HookEventV1::is_known_event("rm -rf"));
        assert!(!HookEventV1::is_known_event("../etc"));
    }
}
