// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Hook contract tests (no root, no live netlink).

use netevd::config::Config;
use netevd::hooks::debounce::{coalesce_count, HookDebouncer};
use netevd::hooks::dispatch::{dispatch_event, hook_dir, HookDispatchOpts};
use netevd::hooks::event::{HookEventV1, HOOK_STATES, SCHEMA_V1};
use netevd::hooks::match_iface::InterfaceSelector;
use netevd::system::paths::get_script_dir;
use netevd::system::validation::validate_state_name;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

#[test]
fn every_hook_state_is_a_valid_script_dir() {
    for state in HOOK_STATES {
        assert!(validate_state_name(state), "{state}");
        let dir = get_script_dir(state);
        assert!(
            dir.ends_with(&format!("/{state}.d")),
            "unexpected dir for {state}: {dir}"
        );
    }
}

#[test]
fn yaml_parses_hooks_and_match_fields() {
    let yaml = r#"
system:
  backend: systemd-networkd
monitoring:
  interfaces: [eth0]
  match_patterns: ["eth*", "wg*"]
  exclude: ["veth*"]
hooks:
  debounce_ms: 40
  timeout_sec: 5
  max_parallel: 1
"#;
    let cfg: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.hooks.debounce_ms, 40);
    assert_eq!(cfg.hooks.timeout_sec, 5);
    assert_eq!(cfg.monitoring.match_patterns, ["eth*", "wg*"]);
    let sel = InterfaceSelector::from_lists(
        cfg.monitoring.match_patterns.clone(),
        cfg.monitoring.exclude.clone(),
    );
    assert!(sel.allows("eth0"));
    assert!(sel.allows("wg0"));
    assert!(!sel.allows("veth0"));
}

#[test]
fn twenty_events_collapse_to_one() {
    let mut d = HookDebouncer::from_millis(50);
    for i in 0..20 {
        d.push(
            HookEventV1::new("routable", "eth0", 2, "systemd-networkd")
                .with_addresses(vec![format!("192.168.1.{i}")]),
        );
    }
    assert_eq!(d.pending_len(), 1);
    assert_eq!(d.coalesced_total(), 19);
    assert_eq!(coalesce_count(20), 1);
    let out = d.take_all();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].addresses, vec!["192.168.1.19"]);
}

#[tokio::test]
async fn end_to_end_tempdir_hook_writes_json_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("link-added.d");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("01-dump.sh");
    let out = tmp.path().join("json.txt");
    std::fs::write(
        &script,
        format!("#!/bin/sh\nprintf '%s' \"$JSON\" > \"{}\"\n", out.display()),
    )
    .unwrap();
    let mut p = std::fs::metadata(&script).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(&script, p).unwrap();

    let ev = HookEventV1::new("link-added", "eth0", 2, "systemd-networkd")
        .with_mtu(1500)
        .with_operstate("DOWN");
    let opts = HookDispatchOpts {
        config_root: tmp.path().to_path_buf(),
        timeout: Duration::from_secs(5),
    };
    dispatch_event(&ev, &opts).await.unwrap();
    let json = std::fs::read_to_string(&out).unwrap();
    assert!(json.contains(SCHEMA_V1));
    let parsed: HookEventV1 = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.event, "link-added");
    assert_eq!(parsed.mtu, Some(1500));
}

#[test]
fn hook_dir_rejects_traversal() {
    assert!(hook_dir(Path::new("/etc/netevd"), "../passwd").is_none());
}
