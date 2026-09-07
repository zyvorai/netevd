// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Script execution with input validation
//!
//! This module executes user scripts with environment variables from potentially
//! untrusted sources (DHCP, network configurations). All environment values are
//! validated and sanitized before being passed to scripts.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};

const SCRIPT_TIMEOUT: Duration = Duration::from_secs(30);

use crate::system::validation;

/// Execute all scripts in a directory with a caller-supplied timeout.
pub async fn execute_scripts_with_timeout(
    directory: &str,
    env_vars: HashMap<String, String>,
    timeout: Duration,
) -> Result<()> {
    execute_scripts_inner(directory, env_vars, timeout).await
}

/// Execute all scripts in a directory with provided environment variables
pub async fn execute_scripts(directory: &str, env_vars: HashMap<String, String>) -> Result<()> {
    execute_scripts_inner(directory, env_vars, SCRIPT_TIMEOUT).await
}

async fn execute_scripts_inner(
    directory: &str,
    env_vars: HashMap<String, String>,
    timeout: Duration,
) -> Result<()> {
    let dir_path = Path::new(directory);

    if !dir_path.exists() {
        debug!("Script directory does not exist: {}", directory);
        return Ok(());
    }

    if !dir_path.is_dir() {
        warn!("Script path is not a directory: {}", directory);
        return Ok(());
    }

    // Read directory entries
    let mut entries = fs::read_dir(dir_path)
        .await
        .with_context(|| format!("Failed to read directory: {}", directory))?;

    let mut scripts = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .with_context(|| format!("Failed to read directory entry in: {}", directory))?
    {
        let path = entry.path();

        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        // Check if file is executable
        let metadata = fs::metadata(&path)
            .await
            .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            debug!("Skipping non-executable file: {:?}", path);
            continue;
        }

        scripts.push(path);
    }

    // Sort scripts by name for deterministic execution order
    scripts.sort();

    if scripts.is_empty() {
        debug!("No executable scripts found in: {}", directory);
        return Ok(());
    }

    info!("Executing {} scripts in: {}", scripts.len(), directory);

    // Execute each script
    for script_path in scripts {
        match execute_script(&script_path, &env_vars, timeout).await {
            Ok(_) => {
                info!("Successfully executed script: {:?}", script_path);
            }
            Err(e) => {
                warn!("Failed to execute script {:?}: {}", script_path, e);
            }
        }
    }

    Ok(())
}

/// Execute a single script with environment variables
///
/// All environment variable values are validated and sanitized to prevent
/// command injection attacks. Dangerous values are rejected with warnings.
async fn execute_script(
    script_path: &Path,
    env_vars: &HashMap<String, String>,
    timeout: Duration,
) -> Result<()> {
    debug!("Executing script: {:?}", script_path);

    let mut cmd = Command::new(script_path);

    // Start with a clean environment to prevent leaking daemon env vars,
    // but preserve essential variables scripts may need
    cmd.env_clear();
    cmd.env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");
    if let Ok(home) = std::env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = std::env::var("USER") {
        cmd.env("USER", user);
    }
    if let Ok(lang) = std::env::var("LANG") {
        cmd.env("LANG", lang);
    }
    if let Ok(tz) = std::env::var("TZ") {
        cmd.env("TZ", tz);
    }

    // Validate and set environment variables
    // This is defense-in-depth: even though scripts should quote variables,
    // we prevent passing potentially malicious data
    let mut rejected_vars = Vec::new();

    for (key, value) in env_vars {
        // Perform context-specific validation
        let is_safe = match key.as_str() {
            "LINK" => validation::validate_interface_name(value),
            "DHCP_HOSTNAME" | "HOSTNAME" => validation::validate_hostname(value),
            "DHCP_DOMAIN" | "DOMAIN" => validation::validate_domain_name(value),
            "DHCP_ADDRESS" | "ADDRESSES" => validation::validate_ip_list(value),
            "DHCP_DNS" | "DNS" => validation::validate_ip_list(value),
            "DHCP_GATEWAY" => validation::validate_ip_list(value),
            "STATE" => validation::validate_state_name(value),
            "LINKINDEX" => value.chars().all(|c| c.is_ascii_digit()),
            "JSON" => true, // Trusted: serialized by our own code
            "BACKEND" | "EVENT" => validation::sanitize_env_value(value).is_some(),
            // For other variables, apply general sanitization
            _ => validation::sanitize_env_value(value).is_some(),
        };

        if is_safe {
            cmd.env(key, value);
        } else {
            rejected_vars.push((key.clone(), value.clone()));
        }
    }

    // Log rejected variables for security monitoring
    if !rejected_vars.is_empty() {
        warn!(
            "Rejected {} environment variables with suspicious content for script {:?}",
            rejected_vars.len(),
            script_path
        );
        for (key, value) in &rejected_vars {
            debug!("Rejected: {}={}", key, value);
        }
    }

    // Configure stdio
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Spawn the child with kill_on_drop so it is killed if we bail on timeout
    cmd.kill_on_drop(true);
    let child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn script: {:?}", script_path))?;

    let output = tokio::time::timeout(timeout, child.wait_with_output())
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Script {:?} timed out after {}s (process killed)",
                script_path,
                timeout.as_secs()
            )
        })?
        .with_context(|| format!("Failed to execute script: {:?}", script_path))?;

    // Log output
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Script stdout: {}", stdout.trim());
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            debug!("Script stderr: {}", stderr.trim());
        } else {
            warn!("Script stderr: {}", stderr.trim());
        }
    }

    // Check exit status
    if !output.status.success() {
        anyhow::bail!(
            "Script {:?} exited with status: {}",
            script_path,
            output.status
        );
    }

    Ok(())
}
