// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Linux capability management

use anyhow::{Context, Result};
use caps::{CapSet, Capability};
use nix::sys::prctl;
use std::collections::HashSet;

/// Apply necessary capabilities for network operations
///
/// - CAP_NET_ADMIN: routing tables, policy rules, netlink, interface config
/// - CAP_BPF / CAP_PERFMON: observe-only eBPF load/attach (kernel 5.8+)
/// - CAP_DAC_READ_SEARCH: open root-only tracefs to resolve tracepoint IDs
///
/// CAP_SYS_ADMIN is intentionally omitted (too broad).
///
/// This should be called after privilege dropping.
pub fn apply_capabilities() -> Result<()> {
    let mut capabilities = HashSet::new();
    capabilities.insert(Capability::CAP_NET_ADMIN);
    // Include BPF / tracefs caps when present in the bounding set (systemd unit ships them).
    for cap in [
        Capability::CAP_BPF,
        Capability::CAP_PERFMON,
        Capability::CAP_DAC_READ_SEARCH,
        Capability::CAP_NET_RAW,
    ] {
        if caps::has_cap(None, CapSet::Bounding, cap).unwrap_or(false) {
            capabilities.insert(cap);
        }
    }

    // Set in permitted set (capability pool we can draw from)
    caps::set(None, CapSet::Permitted, &capabilities)
        .context("Failed to set capabilities in permitted set")?;

    // Set in effective set (actually active capabilities)
    caps::set(None, CapSet::Effective, &capabilities)
        .context("Failed to set capabilities in effective set")?;

    // Do NOT set Inheritable - child processes (scripts) should not
    // inherit network admin capabilities for security

    tracing::info!("Applied capabilities: {:?}", capabilities);

    // Verify the capability was actually acquired
    let effective =
        caps::read(None, CapSet::Effective).context("Failed to read effective capabilities")?;

    if !effective.contains(&Capability::CAP_NET_ADMIN) {
        anyhow::bail!("Failed to acquire CAP_NET_ADMIN capability");
    }

    Ok(())
}

/// Enable keeping capabilities across setuid
pub fn keep_capabilities() -> Result<()> {
    prctl::set_keepcaps(true).context("Failed to set PR_SET_KEEPCAPS to true")?;
    Ok(())
}

/// Disable keeping capabilities (should be called after setuid)
pub fn clear_keep_capabilities() -> Result<()> {
    prctl::set_keepcaps(false).context("Failed to set PR_SET_KEEPCAPS to false")?;
    Ok(())
}

/// Check if we have a specific capability
pub fn has_capability(cap: Capability) -> Result<bool> {
    caps::has_cap(None, CapSet::Effective, cap)
        .with_context(|| format!("Failed to check for capability {:?}", cap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_enum() {
        // Just ensure the capability enums work
        let _net_admin = Capability::CAP_NET_ADMIN;
        let _sys_admin = Capability::CAP_SYS_ADMIN;
    }

    #[test]
    fn test_has_capability() {
        // Test that has_capability doesn't panic
        let result = has_capability(Capability::CAP_NET_ADMIN);
        assert!(result.is_ok());
    }

    #[test]
    fn test_capability_check_multiple() {
        // Check multiple capabilities
        let caps_to_check = vec![
            Capability::CAP_NET_ADMIN,
            Capability::CAP_SYS_ADMIN,
            Capability::CAP_DAC_OVERRIDE,
        ];

        for cap in caps_to_check {
            let result = has_capability(cap);
            assert!(result.is_ok(), "Failed to check capability {:?}", cap);
        }
    }

    #[test]
    fn test_keep_capabilities_toggles() {
        // If running as root, test PR_SET_KEEPCAPS toggling
        if nix::unistd::Uid::effective().is_root() {
            // Enable keep capabilities
            let result = keep_capabilities();
            assert!(result.is_ok(), "Failed to enable keep capabilities");

            // Disable keep capabilities
            let result = clear_keep_capabilities();
            assert!(result.is_ok(), "Failed to disable keep capabilities");
        }
    }

    #[test]
    fn test_apply_capabilities_non_root() {
        // If not running as root, apply_capabilities should handle gracefully
        if !nix::unistd::Uid::effective().is_root() {
            // This will likely fail, but shouldn't panic
            let result = apply_capabilities();
            // Just ensure it returns a result (either Ok or Err)
            let _is_ok = result.is_ok();
        }
    }
}
