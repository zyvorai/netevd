// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! netevd - Network Event Daemon
//!
//! A daemon that monitors systemd-networkd and dhclient events,
//! executes scripts on network state changes, and manages routing policy rules.

// main.rs re-declares the same modules as lib.rs (its own separate crate
// root), so lib.rs's #![allow(dead_code)] doesn't cover this compilation —
// see the identical note there for why this is a real library API surface,
// not sloppy dead code.
#![allow(dead_code)]

use anyhow::{Context, Result};
use clap::Parser;
use rtnetlink::Handle;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod audit;
mod bus;
mod cli;
mod config;
mod filters;
mod hooks;
mod listeners;
mod metrics;
mod network;
mod system;

use audit::AuditLogger;
use cli::{handler, Cli, Commands};
use config::Config;
use hooks::HookDispatchOpts;
use metrics::{Metrics, MetricsHandle};
use network::{link, watcher, NetworkState};
use std::path::PathBuf;
use system::user;

const DEFAULT_USER: &str = "netevd";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref cmd) = cli.command {
        if !matches!(cmd, Commands::Start { .. }) {
            return handler::handle_command(cli).await;
        }
    }

    let config_path = cli.config.to_str().context("Invalid configuration path")?;
    let config = Config::parse_from_path(config_path).context("Failed to parse configuration")?;

    // Initialize logging with config level (RUST_LOG env takes precedence)
    init_logging(&config.system.log_level);

    info!("Starting netevd - Network Event Daemon");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    info!(
        "Configuration loaded: backend={}, log_level={}",
        config.system.backend, config.system.log_level
    );

    // Initialize metrics if enabled
    let metrics: Option<MetricsHandle> = if config.metrics.enabled {
        match Metrics::new() {
            Ok(m) => {
                let metrics_handle = Arc::new(m);
                metrics::set_global_metrics(metrics_handle.clone());
                info!("Metrics collection enabled on port {}", config.metrics.port);
                Some(metrics_handle)
            }
            Err(e) => {
                warn!(
                    "Failed to initialize metrics: {}, continuing without metrics",
                    e
                );
                None
            }
        }
    } else {
        info!("Metrics collection disabled");
        None
    };

    // Initialize audit logger
    let audit_logger = Arc::new(AuditLogger::new(
        Some(PathBuf::from(&config.audit.path)),
        config.audit.enabled,
    ));
    if config.audit.enabled {
        info!("Audit logging enabled: {}", config.audit.path);
    } else {
        info!("Audit logging disabled");
    }

    // Drop privileges if running as root
    if user::is_root() {
        info!(
            "Running as root, attempting to drop privileges to user '{}'",
            DEFAULT_USER
        );
        user::drop_privileges(DEFAULT_USER).context("Failed to drop privileges")?;
    } else {
        info!("Not running as root, continuing with current user");
    }

    // Initialize network state
    let state = Arc::new(RwLock::new(NetworkState::new()));
    info!("Network state initialized");

    // Get netlink handle
    let handle = link::get_netlink_handle()
        .await
        .context("Failed to get netlink handle")?;
    info!("Netlink handle acquired");

    // Acquire initial links
    {
        let mut state_write = state.write().await;
        link::acquire_links(&mut state_write, &handle)
            .await
            .context("Failed to acquire initial links")?;
    }
    info!("Initial network links acquired");

    // Get routing policy interfaces from config
    let routing_policy_interfaces = config.routing.get_routing_policy_interfaces();

    // Hook contract: interface selector + dispatch service (versioned JSON,
    // debounced, script directories under CONFIG_DIR)
    let hook_selector = hooks::InterfaceSelector::from_lists(
        config.monitoring.match_patterns.clone(),
        config.monitoring.exclude.clone(),
    )
    .with_default_excludes();
    let hook_opts = HookDispatchOpts {
        config_root: PathBuf::from(system::paths::CONFIG_DIR),
        timeout: Duration::from_secs(config.hooks.timeout_sec),
    };
    let (hook_tx, hook_service) = hooks::spawn_service(
        hook_opts,
        Duration::from_millis(config.hooks.debounce_ms),
        metrics.clone(),
    );

    // Clone handles for async tasks
    let state_addr = state.clone();
    let state_route = state.clone();
    let state_link = state.clone();
    let state_listener = state.clone();
    let handle_addr = handle.clone();
    let handle_route = handle.clone();
    let handle_link = handle.clone();
    let handle_listener = handle.clone();
    let config_listener = config.clone();
    let metrics_listener = metrics.clone();
    let audit_listener = audit_logger.clone();
    let hook_selector_addr = hook_selector.clone();
    let hook_selector_link = hook_selector;
    let hook_tx_addr = hook_tx.clone();
    let hook_tx_route = hook_tx.clone();
    let hook_tx_link = hook_tx;

    // Set up signal handlers
    let mut sigterm =
        signal(SignalKind::terminate()).context("Failed to set up SIGTERM handler")?;
    let mut sigint = signal(SignalKind::interrupt()).context("Failed to set up SIGINT handler")?;

    info!("netevd initialized successfully, waiting for events...");

    // Main event loop with async watchers
    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C), shutting down gracefully");
        }
        result = watcher::watch_addresses(handle_addr, state_addr, routing_policy_interfaces, hook_selector_addr, hook_tx_addr) => {
            warn!("Address watcher exited: {:?}", result);
        }
        result = watcher::watch_routes(handle_route, state_route, hook_tx_route) => {
            warn!("Route watcher exited: {:?}", result);
        }
        result = watcher::watch_links(handle_link, state_link, hook_selector_link, hook_tx_link) => {
            warn!("Link watcher exited: {:?}", result);
        }
        result = spawn_listener(config_listener, handle_listener, state_listener, metrics_listener, audit_listener) => {
            warn!("Backend listener exited: {:?}", result);
        }
        result = hook_service => {
            warn!("Hook dispatch service exited: {:?}", result);
        }
    }

    info!("netevd shutdown complete");
    Ok(())
}

/// Initialize logging with the given default level (RUST_LOG env takes precedence)
fn init_logging(default_level: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .init();
}

/// Spawn the appropriate listener based on the configured backend
async fn spawn_listener(
    config: Config,
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    metrics: Option<MetricsHandle>,
    audit: Arc<AuditLogger>,
) -> Result<()> {
    match config.system.backend.as_str() {
        "systemd-networkd" => {
            info!("Starting systemd-networkd listener");
            listeners::networkd::listen_networkd(config, handle, state, metrics, audit).await
        }
        "NetworkManager" => {
            info!("Starting NetworkManager listener");
            listeners::networkmanager::listen_networkmanager(config, handle, state, metrics, audit)
                .await
        }
        "dhclient" => {
            info!("Starting dhclient listener");
            listeners::dhclient::watch_lease_file(config, handle, state, metrics, audit).await
        }
        _ => anyhow::bail!("Unknown backend: {}", config.system.backend),
    }
}
