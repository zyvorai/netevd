// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Dispatch a versioned event to `/etc/netevd/<state>.d/` (or override root).

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::event::HookEventV1;
use crate::system::execute::{execute_scripts, execute_scripts_with_timeout};
use crate::system::validation::validate_state_name;

#[derive(Debug, Clone)]
pub struct HookDispatchOpts {
    pub config_root: PathBuf,
    pub timeout: Duration,
}

impl Default for HookDispatchOpts {
    fn default() -> Self {
        Self {
            config_root: PathBuf::from("/etc/netevd"),
            timeout: Duration::from_secs(30),
        }
    }
}

pub fn hook_dir(root: &Path, state: &str) -> Option<PathBuf> {
    if !validate_state_name(state) {
        return None;
    }
    Some(root.join(format!("{state}.d")))
}

pub async fn dispatch_event(event: &HookEventV1, opts: &HookDispatchOpts) -> Result<()> {
    let Some(dir) = hook_dir(&opts.config_root, &event.event) else {
        tracing::warn!(event = %event.event, "rejected unknown hook state");
        return Ok(());
    };
    let env = event.to_env();
    if opts.timeout == Duration::from_secs(30) {
        execute_scripts(&dir.to_string_lossy(), env).await
    } else {
        execute_scripts_with_timeout(&dir.to_string_lossy(), env, opts.timeout).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tokio::fs;

    #[test]
    fn hook_dir_accepts_new_states() {
        let root = Path::new("/etc/netevd");
        assert_eq!(
            hook_dir(root, "address-added").unwrap(),
            PathBuf::from("/etc/netevd/address-added.d")
        );
        assert_eq!(
            hook_dir(root, "mtu").unwrap(),
            PathBuf::from("/etc/netevd/mtu.d")
        );
        assert!(hook_dir(root, "not-a-state").is_none());
        assert!(hook_dir(root, "../etc").is_none());
    }

    #[tokio::test]
    async fn dispatch_runs_executable_and_sets_env() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("address-added.d");
        fs::create_dir_all(&dir).await.unwrap();
        let script = dir.join("01-record.sh");
        let out = tmp.path().join("out.txt");
        let body = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$LINK\" \"$STATE\" \"$ADDRESSES\" \"$EVENT\" > \"{}\"\n",
            out.display()
        );
        fs::write(&script, body).await.unwrap();
        let mut perms = fs::metadata(&script).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).await.unwrap();

        let ev = HookEventV1::new("address-added", "eth0", 2, "systemd-networkd")
            .with_addresses(vec!["192.168.1.10".into()]);
        let opts = HookDispatchOpts {
            config_root: tmp.path().to_path_buf(),
            timeout: Duration::from_secs(5),
        };
        dispatch_event(&ev, &opts).await.unwrap();

        let recorded = fs::read_to_string(&out).await.unwrap();
        let lines: Vec<&str> = recorded.trim().lines().collect();
        assert_eq!(
            lines,
            ["eth0", "address-added", "192.168.1.10", "address-added"]
        );
    }

    #[tokio::test]
    async fn missing_dir_is_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let ev = HookEventV1::new("mtu", "eth0", 2, "systemd-networkd");
        let opts = HookDispatchOpts {
            config_root: tmp.path().to_path_buf(),
            timeout: Duration::from_secs(1),
        };
        dispatch_event(&ev, &opts).await.unwrap();
    }
}
