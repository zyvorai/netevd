// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use netevd_hooks::debounce::{coalesce_count, HookDebouncer};
use netevd_hooks::dispatch::{dispatch_event, hook_dir, HookDispatchOpts};
use netevd_hooks::event::{HookEventV1, HOOK_STATES, SCHEMA_V1};
use netevd_hooks::match_iface::InterfaceSelector;
use netevd_hooks::validation::validate_state_name;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

#[test]
fn every_hook_state_valid() {
    for state in HOOK_STATES {
        assert!(validate_state_name(state), "{state}");
    }
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
}

#[tokio::test]
async fn tempdir_hook_writes_json_schema() {
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

    let ev = HookEventV1::new("link-added", "eth0", 2, "systemd-networkd").with_mtu(1500);
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

#[test]
fn selector_skips_cni() {
    let sel = InterfaceSelector::default().with_default_excludes();
    assert!(!sel.allows("vethabc"));
    assert!(sel.allows("eth0"));
}
