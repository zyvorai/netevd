// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use prometheus::{Counter, CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,

    // Daemon metrics
    pub uptime_seconds: Gauge,
    pub events_total: CounterVec,
    pub events_duration: HistogramVec,

    // Interface metrics
    pub interfaces_total: Gauge,
    pub interface_state_changes: CounterVec,

    // Routing metrics
    pub routing_rules_total: Gauge,
    pub routes_total: Gauge,

    // Script execution metrics
    pub script_executions_total: CounterVec,
    pub script_duration: HistogramVec,
    pub script_failures_total: CounterVec,

    // DBus metrics
    pub dbus_calls_total: CounterVec,
    pub dbus_errors_total: Counter,

    // Netlink metrics
    pub netlink_messages_total: CounterVec,
    pub netlink_errors_total: Counter,

    // Hook contract metrics
    pub hooks_dispatched_total: CounterVec,
    pub hooks_coalesced_total: Counter,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // Daemon metrics
        let uptime_seconds = Gauge::with_opts(Opts::new(
            "netevd_uptime_seconds",
            "Uptime of netevd daemon in seconds",
        ))?;
        registry.register(Box::new(uptime_seconds.clone()))?;

        let events_total = CounterVec::new(
            Opts::new(
                "netevd_events_total",
                "Total number of network events processed",
            ),
            &["type", "interface", "backend"],
        )?;
        registry.register(Box::new(events_total.clone()))?;

        let events_duration = HistogramVec::new(
            HistogramOpts::new(
                "netevd_event_duration_seconds",
                "Time spent processing network events",
            )
            .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["type"],
        )?;
        registry.register(Box::new(events_duration.clone()))?;

        // Interface metrics
        let interfaces_total = Gauge::with_opts(Opts::new(
            "netevd_interfaces_total",
            "Total number of network interfaces",
        ))?;
        registry.register(Box::new(interfaces_total.clone()))?;

        let interface_state_changes = CounterVec::new(
            Opts::new(
                "netevd_interface_state_changes_total",
                "Total number of interface state changes",
            ),
            &["interface", "state"],
        )?;
        registry.register(Box::new(interface_state_changes.clone()))?;

        // Routing metrics
        let routing_rules_total = Gauge::with_opts(Opts::new(
            "netevd_routing_rules_total",
            "Total number of active routing policy rules",
        ))?;
        registry.register(Box::new(routing_rules_total.clone()))?;

        let routes_total = Gauge::with_opts(Opts::new(
            "netevd_routes_total",
            "Total number of routes managed",
        ))?;
        registry.register(Box::new(routes_total.clone()))?;

        // Script execution metrics
        let script_executions_total = CounterVec::new(
            Opts::new(
                "netevd_script_executions_total",
                "Total number of script executions",
            ),
            &["script", "event_type"],
        )?;
        registry.register(Box::new(script_executions_total.clone()))?;

        let script_duration = HistogramVec::new(
            HistogramOpts::new(
                "netevd_script_duration_seconds",
                "Time spent executing scripts",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]),
            &["script"],
        )?;
        registry.register(Box::new(script_duration.clone()))?;

        let script_failures_total = CounterVec::new(
            Opts::new(
                "netevd_script_failures_total",
                "Total number of script execution failures",
            ),
            &["script", "exit_code"],
        )?;
        registry.register(Box::new(script_failures_total.clone()))?;

        // DBus metrics
        let dbus_calls_total = CounterVec::new(
            Opts::new("netevd_dbus_calls_total", "Total number of DBus calls"),
            &["service", "method"],
        )?;
        registry.register(Box::new(dbus_calls_total.clone()))?;

        let dbus_errors_total = Counter::with_opts(Opts::new(
            "netevd_dbus_errors_total",
            "Total number of DBus errors",
        ))?;
        registry.register(Box::new(dbus_errors_total.clone()))?;

        // Netlink metrics
        let netlink_messages_total = CounterVec::new(
            Opts::new(
                "netevd_netlink_messages_total",
                "Total number of netlink messages",
            ),
            &["message_type"],
        )?;
        registry.register(Box::new(netlink_messages_total.clone()))?;

        let netlink_errors_total = Counter::with_opts(Opts::new(
            "netevd_netlink_errors_total",
            "Total number of netlink errors",
        ))?;
        registry.register(Box::new(netlink_errors_total.clone()))?;

        // Hook contract metrics
        let hooks_dispatched_total = CounterVec::new(
            Opts::new(
                "netevd_hooks_dispatched_total",
                "Total number of hook events dispatched to script directories",
            ),
            &["event", "link"],
        )?;
        registry.register(Box::new(hooks_dispatched_total.clone()))?;

        let hooks_coalesced_total = Counter::with_opts(Opts::new(
            "netevd_hooks_coalesced_total",
            "Total number of hook events coalesced by the debouncer",
        ))?;
        registry.register(Box::new(hooks_coalesced_total.clone()))?;

        Ok(Self {
            registry,
            uptime_seconds,
            events_total,
            events_duration,
            interfaces_total,
            interface_state_changes,
            routing_rules_total,
            routes_total,
            script_executions_total,
            script_duration,
            script_failures_total,
            dbus_calls_total,
            dbus_errors_total,
            netlink_messages_total,
            netlink_errors_total,
            hooks_dispatched_total,
            hooks_coalesced_total,
        })
    }

    pub fn gather(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            tracing::error!("Failed to encode metrics: {}", e);
            return String::new();
        }
        String::from_utf8(buffer).unwrap_or_default()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

pub type MetricsHandle = Arc<Metrics>;

use std::sync::OnceLock;

static GLOBAL_METRICS: OnceLock<MetricsHandle> = OnceLock::new();

/// Register a metrics handle globally so the API handler can access it
pub fn set_global_metrics(handle: MetricsHandle) {
    let _ = GLOBAL_METRICS.set(handle);
}

/// Get the global metrics handle (returns None if metrics are disabled)
pub fn get_global_metrics() -> Option<&'static MetricsHandle> {
    GLOBAL_METRICS.get()
}
