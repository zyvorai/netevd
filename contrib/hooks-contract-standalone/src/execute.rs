// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::validation;

const SCRIPT_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn execute_scripts(directory: &str, env_vars: HashMap<String, String>) -> Result<()> {
    execute_scripts_with_timeout(directory, env_vars, SCRIPT_TIMEOUT).await
}

pub async fn execute_scripts_with_timeout(
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

    let mut entries = fs::read_dir(dir_path)
        .await
        .with_context(|| format!("Failed to read directory: {}", directory))?;
    let mut scripts = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let metadata = fs::metadata(&path).await?;
        if metadata.permissions().mode() & 0o111 == 0 {
            continue;
        }
        scripts.push(path);
    }
    scripts.sort();
    if scripts.is_empty() {
        return Ok(());
    }
    info!("Executing {} scripts in: {}", scripts.len(), directory);
    for script_path in scripts {
        if let Err(e) = execute_script(&script_path, &env_vars, timeout).await {
            warn!("Failed to execute script {:?}: {}", script_path, e);
        }
    }
    Ok(())
}

async fn execute_script(
    script_path: &Path,
    env_vars: &HashMap<String, String>,
    timeout: Duration,
) -> Result<()> {
    let mut cmd = Command::new(script_path);
    cmd.env_clear();
    cmd.env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");

    for (key, value) in env_vars {
        let is_safe = match key.as_str() {
            "LINK" => validation::validate_interface_name(value),
            "ADDRESSES" | "DNS" | "GATEWAY" | "DHCP_ADDRESS" | "DHCP_DNS" | "DHCP_GATEWAY" => {
                validation::validate_ip_list(value)
            }
            "STATE" | "EVENT" => validation::validate_state_name(value),
            "LINKINDEX" | "MTU" => value.chars().all(|c| c.is_ascii_digit()),
            "JSON" => true,
            "BACKEND" => validation::sanitize_env_value(value).is_some(),
            _ => validation::sanitize_env_value(value).is_some(),
        };
        if is_safe {
            cmd.env(key, value);
        }
    }

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

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
        })??;

    if !output.status.success() {
        anyhow::bail!("Script {:?} exited with status: {}", script_path, output.status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn timeout_kills_hanging_script() {
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("hang.sh");
        std::fs::write(&script, "#!/bin/sh\nsleep 30\n").unwrap();
        let mut p = std::fs::metadata(&script).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&script, p).unwrap();

        let err = execute_script(&script, &HashMap::new(), Duration::from_millis(200))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("timed out"), "{err}");
    }

    #[tokio::test]
    async fn rejects_dangerous_link() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        let script = tmp.path().join("echo.sh");
        std::fs::write(
            &script,
            format!("#!/bin/sh\nprintf '%s' \"$LINK\" > {}\n", out.display()),
        )
        .unwrap();
        let mut p = std::fs::metadata(&script).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&script, p).unwrap();

        let mut env = HashMap::new();
        env.insert("LINK".into(), "eth0;rm".into());
        execute_script(&script, &env, Duration::from_secs(2))
            .await
            .unwrap();
        let body = std::fs::read_to_string(&out).unwrap_or_default();
        assert!(body.is_empty(), "dangerous LINK must not be exported: {body}");
    }
}
