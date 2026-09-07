// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0
//
// Std-only copy of the hook contract so `rustc --test` can run without cargo.
// Production modules (event.rs etc.) keep serde/chrono for the daemon tree.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const SCHEMA_V1: &str = "netevd.event.v1";

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

pub fn validate_state_name(state: &str) -> bool {
    HOOK_STATES.contains(&state)
}

pub fn validate_interface_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 15
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

pub fn glob_matches(pattern: &str, name: &str) -> bool {
    glob_match_inner(pattern.as_bytes(), name.as_bytes())
}

fn glob_match_inner(pat: &[u8], name: &[u8]) -> bool {
    let mut pi = 0;
    let mut ni = 0;
    let mut star_p: Option<usize> = None;
    let mut star_n: usize = 0;
    while ni < name.len() {
        if pi < pat.len() && pat[pi] == b'*' {
            star_p = Some(pi);
            star_n = ni;
            pi += 1;
            continue;
        }
        if pi < pat.len() && pat[pi] == name[ni] {
            pi += 1;
            ni += 1;
            continue;
        }
        if let Some(sp) = star_p {
            pi = sp + 1;
            star_n += 1;
            ni = star_n;
            continue;
        }
        return false;
    }
    while pi < pat.len() && pat[pi] == b'*' {
        pi += 1;
    }
    pi == pat.len()
}

pub const DEFAULT_EXCLUDES: &[&str] = &[
    "lo", "docker*", "br-*", "cni*", "flannel*", "cilium*", "veth*", "virbr*",
];

pub struct InterfaceSelector {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl InterfaceSelector {
    pub fn with_default_excludes() -> Self {
        Self {
            include: vec![],
            exclude: DEFAULT_EXCLUDES.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    pub fn allows(&self, name: &str) -> bool {
        if self.exclude.iter().any(|p| glob_matches(p, name)) {
            return false;
        }
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|p| glob_matches(p, name))
    }
}

pub struct HookEventV1 {
    pub schema: String,
    pub event: String,
    pub link: String,
    pub index: u32,
    pub backend: String,
    pub addresses: Vec<String>,
    pub mtu: Option<u32>,
}

impl HookEventV1 {
    pub fn new(event: &str, link: &str, index: u32, backend: &str) -> Self {
        Self {
            schema: SCHEMA_V1.into(),
            event: event.into(),
            link: link.into(),
            index,
            backend: backend.into(),
            addresses: vec![],
            mtu: None,
        }
    }

    pub fn to_json(&self) -> String {
        let addrs = self
            .addresses
            .iter()
            .map(|a| format!("\"{a}\""))
            .collect::<Vec<_>>()
            .join(",");
        let mtu = self
            .mtu
            .map(|m| format!(",\"mtu\":{m}"))
            .unwrap_or_default();
        format!(
            "{{\"schema\":\"{}\",\"event\":\"{}\",\"link\":\"{}\",\"index\":{},\"state\":\"{}\",\"backend\":\"{}\",\"addresses\":[{}]{}}}",
            self.schema, self.event, self.link, self.index, self.event, self.backend, addrs, mtu
        )
    }

    pub fn to_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("LINK".into(), self.link.clone());
        env.insert("LINKINDEX".into(), self.index.to_string());
        env.insert("STATE".into(), self.event.clone());
        env.insert("EVENT".into(), self.event.clone());
        env.insert("BACKEND".into(), self.backend.clone());
        env.insert("ADDRESSES".into(), self.addresses.join(" "));
        env.insert("JSON".into(), self.to_json());
        env
    }
}

struct Pending {
    event: HookEventV1,
    first_seen: Instant,
    count: u32,
}

pub struct HookDebouncer {
    window: Duration,
    pending: HashMap<String, Pending>,
    pub coalesced_total: u64,
}

impl HookDebouncer {
    pub fn from_millis(ms: u64) -> Self {
        Self {
            window: Duration::from_millis(ms),
            pending: HashMap::new(),
            coalesced_total: 0,
        }
    }

    fn key(event: &HookEventV1) -> String {
        format!("{}:{}", event.link, event.event)
    }

    pub fn push(&mut self, event: HookEventV1) {
        let now = Instant::now();
        let key = Self::key(&event);
        if let Some(p) = self.pending.get_mut(&key) {
            if now.duration_since(p.first_seen) <= self.window {
                self.coalesced_total += 1;
                p.count += 1;
                p.event = event;
                return;
            }
        }
        self.pending.insert(
            key,
            Pending {
                event,
                first_seen: now,
                count: 1,
            },
        );
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn take_all(&mut self) -> Vec<HookEventV1> {
        self.pending.drain().map(|(_, p)| p.event).collect()
    }
}

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
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    #[test]
    fn states_accepted() {
        for s in HOOK_STATES {
            assert!(validate_state_name(s), "{s}");
        }
        assert!(!validate_state_name("../etc"));
        assert!(!validate_state_name("address-added;rm"));
    }

    #[test]
    fn json_schema_v1() {
        let mut ev = HookEventV1::new("address-added", "eth0", 2, "systemd-networkd");
        ev.addresses = vec!["192.168.1.10".into()];
        ev.mtu = Some(1500);
        let json = ev.to_json();
        assert!(json.contains("\"schema\":\"netevd.event.v1\""));
        assert!(json.contains("\"event\":\"address-added\""));
        assert!(json.contains("192.168.1.10"));
        assert!(json.contains("\"mtu\":1500"));
        let env = ev.to_env();
        assert_eq!(env.get("LINK").unwrap(), "eth0");
        assert_eq!(env.get("STATE").unwrap(), "address-added");
        assert_eq!(env.get("EVENT").unwrap(), "address-added");
        assert_eq!(env.get("ADDRESSES").unwrap(), "192.168.1.10");
    }

    #[test]
    fn glob_and_defaults() {
        assert!(glob_matches("eth*", "eth0"));
        assert!(glob_matches("enp*", "enp1s0"));
        assert!(!glob_matches("wg*", "eth0"));
        let sel = InterfaceSelector::with_default_excludes();
        assert!(!sel.allows("lo"));
        assert!(!sel.allows("docker0"));
        assert!(!sel.allows("veth12ab"));
        assert!(!sel.allows("cilium_host"));
        assert!(sel.allows("eth0"));
        assert!(sel.allows("wg0"));
    }

    #[test]
    fn burst_20_to_1() {
        let mut d = HookDebouncer::from_millis(80);
        for i in 0..20 {
            let mut ev = HookEventV1::new("routable", "eth0", 2, "systemd-networkd");
            ev.addresses = vec![format!("192.168.1.{i}")];
            d.push(ev);
        }
        assert_eq!(d.pending_len(), 1);
        assert_eq!(d.coalesced_total, 19);
        assert_eq!(coalesce_count(20), 1);
        let all = d.take_all();
        assert_eq!(all[0].addresses[0], "192.168.1.19");
    }

    #[test]
    fn different_keys_not_merged() {
        let mut d = HookDebouncer::from_millis(80);
        d.push(HookEventV1::new("routable", "eth0", 2, "systemd-networkd"));
        d.push(HookEventV1::new("routes", "eth0", 2, "systemd-networkd"));
        d.push(HookEventV1::new("routable", "eth1", 3, "systemd-networkd"));
        assert_eq!(d.pending_len(), 3);
    }

    #[test]
    fn script_gets_env_and_json() {
        let dir = std::env::temp_dir().join(format!("netevd-hook-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let script = dir.join("01.sh");
        let out = dir.join("out.txt");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$LINK\" \"$STATE\" \"$ADDRESSES\" \"$JSON\" > {}\n",
                out.display()
            ),
        )
        .unwrap();
        let mut p = fs::metadata(&script).unwrap().permissions();
        p.set_mode(0o755);
        fs::set_permissions(&script, p).unwrap();

        let mut ev = HookEventV1::new("link-added", "eth0", 2, "systemd-networkd");
        ev.addresses = vec!["10.0.0.8".into()];
        let env = ev.to_env();
        let mut cmd = Command::new(&script);
        cmd.env_clear().env("PATH", "/usr/bin:/bin").stdout(Stdio::null());
        for (k, v) in &env {
            if k == "LINK" && !validate_interface_name(v) {
                continue;
            }
            cmd.env(k, v);
        }
        let status = cmd.status().unwrap();
        assert!(status.success());
        let body = fs::read_to_string(&out).unwrap();
        let lines: Vec<&str> = body.trim().lines().collect();
        assert_eq!(lines[0], "eth0");
        assert_eq!(lines[1], "link-added");
        assert_eq!(lines[2], "10.0.0.8");
        assert!(lines[3].contains(SCHEMA_V1));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hanging_script_timeout() {
        let dir = std::env::temp_dir().join(format!("netevd-hang-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let script = dir.join("hang.sh");
        fs::write(&script, "#!/bin/sh\nsleep 30\n").unwrap();
        let mut p = fs::metadata(&script).unwrap().permissions();
        p.set_mode(0o755);
        fs::set_permissions(&script, p).unwrap();

        let mut child = Command::new(&script).spawn().unwrap();
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(250) {
                let _ = child.kill();
                break;
            }
            if child.try_wait().unwrap().is_some() {
                panic!("hanging script exited early");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = child.wait();
        assert!(start.elapsed() < Duration::from_secs(5));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dangerous_link_rejected() {
        assert!(!validate_interface_name("eth0;rm"));
        assert!(!validate_interface_name("eth$0"));
        assert!(validate_interface_name("eth0"));
        assert!(validate_interface_name("wg0"));
        assert!(validate_interface_name("enp1s0"));
    }
}
