// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Configuration parsing and management

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

use crate::filters::Filter;

const DEFAULT_CONFIG_PATH: &str = "/etc/netevd/netevd.yaml";
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_BACKEND: &str = "systemd-networkd";

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub system: SystemConfig,

    #[serde(default)]
    pub monitoring: MonitoringConfig,

    #[serde(default)]
    pub routing: RoutingConfig,

    #[serde(default)]
    pub backends: BackendsConfig,

    #[serde(default)]
    pub api: ApiConfig,

    #[serde(default)]
    pub metrics: MetricsConfig,

    #[serde(default)]
    pub audit: AuditConfig,

    #[serde(default)]
    pub filters: Vec<Filter>,

    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct HooksConfig {
    /// Coalesce netlink bursts for this many milliseconds (0 = off).
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Per-script timeout in seconds.
    #[serde(default = "default_hook_timeout_sec")]
    pub timeout_sec: u64,

    /// Max scripts to run in parallel in one directory (1 = sequential).
    #[serde(default = "default_hook_max_parallel")]
    pub max_parallel: usize,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce_ms(),
            timeout_sec: default_hook_timeout_sec(),
            max_parallel: default_hook_max_parallel(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_backend")]
    pub backend: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MonitoringConfig {
    #[serde(default)]
    pub interfaces: Vec<String>,

    /// Glob include list (`eth*`, `wg*`). Empty = all (minus exclude).
    #[serde(default)]
    pub match_patterns: Vec<String>,

    /// Glob exclude list. Empty = built-in virtual/CNI defaults when used
    /// through `InterfaceSelector::with_default_excludes`.
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RoutingConfig {
    #[serde(default)]
    pub policy_rules: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct BackendsConfig {
    #[serde(default)]
    pub systemd_networkd: SystemdNetworkdConfig,

    #[serde(default)]
    pub dhclient: DhclientConfig,

    #[serde(default)]
    pub networkmanager: NetworkManagerConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemdNetworkdConfig {
    #[serde(default = "default_true")]
    pub emit_json: bool,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DhclientConfig {
    #[serde(default)]
    pub use_dns: bool,

    #[serde(default)]
    pub use_domain: bool,

    #[serde(default)]
    pub use_hostname: bool,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct NetworkManagerConfig {
    // Placeholder for future NetworkManager-specific options
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    #[serde(default = "default_api_port")]
    pub port: u16,

    #[serde(default)]
    pub tls: TlsConfig,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub cert_file: Option<String>,

    #[serde(default)]
    pub key_file: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MetricsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_metrics_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_audit_path")]
    pub path: String,

    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            backend: DEFAULT_BACKEND.to_string(),
        }
    }
}

impl Default for SystemdNetworkdConfig {
    fn default() -> Self {
        Self { emit_json: true }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 9090,
            tls: TlsConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9091,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: "/var/log/netevd/audit.log".to_string(),
            retention_days: 90,
        }
    }
}

impl MonitoringConfig {
    /// Get interfaces as a vector
    pub fn get_interfaces(&self) -> Vec<String> {
        self.interfaces.clone()
    }
}

impl RoutingConfig {
    /// Get routing policy rule interfaces as a vector
    pub fn get_routing_policy_interfaces(&self) -> Vec<String> {
        self.policy_rules.clone()
    }
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
}

fn default_backend() -> String {
    DEFAULT_BACKEND.to_string()
}

fn default_true() -> bool {
    true
}

fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}

fn default_api_port() -> u16 {
    9090
}

fn default_metrics_port() -> u16 {
    9091
}

fn default_audit_path() -> String {
    "/var/log/netevd/audit.log".to_string()
}

fn default_retention_days() -> u32 {
    90
}

fn default_debounce_ms() -> u64 {
    50
}

fn default_hook_timeout_sec() -> u64 {
    30
}

fn default_hook_max_parallel() -> usize {
    1
}

impl Config {
    /// Parse configuration from file and environment variables
    pub fn parse() -> Result<Self> {
        Self::parse_from_path(DEFAULT_CONFIG_PATH)
    }

    /// Parse configuration from a specific path
    pub fn parse_from_path(path: &str) -> Result<Self> {
        // Try to read config file
        let mut config = if Path::new(path).exists() {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path))?;

            serde_yaml::from_str(&contents)
                .with_context(|| format!("Failed to parse YAML config: {}", path))?
        } else {
            // Use default config if file doesn't exist
            Config::default()
        };

        // Override with environment variables
        // Environment variable pattern: NETEVD_SYSTEM_LOG_LEVEL, etc.
        if let Ok(log_level) = env::var("NETEVD_LOG_LEVEL") {
            config.system.log_level = log_level;
        }
        if let Ok(backend) = env::var("NETEVD_BACKEND") {
            config.system.backend = backend;
        }
        if let Ok(api_enabled) = env::var("NETEVD_API_ENABLED") {
            config.api.enabled = api_enabled.parse().unwrap_or(true);
        }
        if let Ok(api_bind) = env::var("NETEVD_API_BIND_ADDRESS") {
            config.api.bind_address = api_bind;
        }
        if let Ok(api_port) = env::var("NETEVD_API_PORT") {
            if let Ok(port) = api_port.parse() {
                config.api.port = port;
            }
        }
        if let Ok(metrics_enabled) = env::var("NETEVD_METRICS_ENABLED") {
            config.metrics.enabled = metrics_enabled.parse().unwrap_or(true);
        }
        if let Ok(audit_enabled) = env::var("NETEVD_AUDIT_ENABLED") {
            config.audit.enabled = audit_enabled.parse().unwrap_or(false);
        }

        // Validate backend
        match config.system.backend.as_str() {
            "systemd-networkd" | "NetworkManager" | "dhclient" => {}
            other => {
                anyhow::bail!(
                    "Invalid backend '{}'. Valid options: systemd-networkd, NetworkManager, dhclient",
                    other
                );
            }
        }

        Ok(config)
    }

    /// Get links as a vector
    pub fn get_links(&self) -> Vec<String> {
        self.monitoring.interfaces.clone()
    }

    /// Get routing policy rule links as a vector
    pub fn get_routing_policy_links(&self) -> Vec<String> {
        self.routing.policy_rules.clone()
    }

    /// Check if a link should be monitored
    pub fn should_monitor_link(&self, link_name: &str) -> bool {
        let links = self.get_links();
        links.is_empty() || links.contains(&link_name.to_string())
    }

    /// Check if routing policy rules should be configured for a link
    pub fn should_configure_routing_rules(&self, link_name: &str) -> bool {
        let policy_links = self.get_routing_policy_links();
        policy_links.contains(&link_name.to_string())
    }

    /// Get emit_json setting
    pub fn get_emit_json(&self) -> bool {
        self.backends.systemd_networkd.emit_json
    }

    /// Get use_dns setting
    pub fn get_use_dns(&self) -> bool {
        self.backends.dhclient.use_dns
    }

    /// Get use_domain setting
    pub fn get_use_domain(&self) -> bool {
        self.backends.dhclient.use_domain
    }

    /// Get use_hostname setting
    pub fn get_use_hostname(&self) -> bool {
        self.backends.dhclient.use_hostname
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.system.log_level, "info");
        assert_eq!(config.system.backend, "systemd-networkd");
        assert!(config.backends.systemd_networkd.emit_json);
    }

    #[test]
    fn test_parse_new_config_format() {
        let yaml = r#"
system:
  log_level: "debug"
  backend: "systemd-networkd"

monitoring:
  interfaces:
    - eth0
    - eth1
    - wlan0

routing:
  policy_rules:
    - eth1

backends:
  systemd_networkd:
    emit_json: true

  dhclient:
    use_dns: true
    use_domain: false
    use_hostname: false
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.system.log_level, "debug");
        assert_eq!(config.get_links(), vec!["eth0", "eth1", "wlan0"]);
        assert_eq!(config.get_routing_policy_links(), vec!["eth1"]);
        assert!(config.get_emit_json());
        assert!(config.get_use_dns());
        assert!(!config.get_use_domain());
    }

    #[test]
    fn test_should_monitor_link() {
        let yaml = r#"
monitoring:
  interfaces:
    - eth0
    - eth1
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.should_monitor_link("eth0"));
        assert!(config.should_monitor_link("eth1"));
        assert!(!config.should_monitor_link("wlan0"));
    }

    #[test]
    fn test_should_monitor_all_links_when_empty() {
        let config = Config::default();
        assert!(config.should_monitor_link("eth0"));
        assert!(config.should_monitor_link("wlan0"));
    }

    #[test]
    fn test_routing_policy_rules() {
        let yaml = r#"
routing:
  policy_rules:
    - eth1
    - eth2
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.should_configure_routing_rules("eth1"));
        assert!(config.should_configure_routing_rules("eth2"));
        assert!(!config.should_configure_routing_rules("eth0"));
    }

    #[test]
    fn test_minimal_config() {
        let yaml = r#"
system:
  backend: "NetworkManager"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.system.backend, "NetworkManager");
        assert_eq!(config.system.log_level, "info");
        assert!(config.monitoring.interfaces.is_empty());
    }
}
