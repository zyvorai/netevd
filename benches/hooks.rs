// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the hot-path functions on the hook dispatch path: input
//! validation (run per env var, per script) and the hook event payload
//! (built per netlink event, serialized per script directory dispatch).

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use netevd::hooks::event::HookEventV1;
use netevd::hooks::match_iface::InterfaceSelector;
use netevd::system::validation::{sanitize_env_value, validate_interface_name};

fn bench_validate_interface_name(c: &mut Criterion) {
    c.bench_function("validate_interface_name", |b| {
        b.iter(|| validate_interface_name(black_box("eth0")))
    });
}

fn bench_sanitize_env_value(c: &mut Criterion) {
    c.bench_function("sanitize_env_value", |b| {
        b.iter(|| sanitize_env_value(black_box("192.168.1.100 10.0.0.5")))
    });
}

fn bench_interface_selector_allows(c: &mut Criterion) {
    let selector =
        InterfaceSelector::from_lists(vec!["eth*".into(), "wg*".into()], vec!["veth*".into()])
            .with_default_excludes();
    c.bench_function("interface_selector_allows", |b| {
        b.iter(|| selector.allows(black_box("eth0")))
    });
}

fn bench_hook_event_to_json(c: &mut Criterion) {
    let event = HookEventV1::new("address-added", "eth0", 2, "netlink")
        .with_addresses(vec!["192.168.1.100".into(), "10.0.0.5".into()]);
    c.bench_function("hook_event_to_json", |b| {
        b.iter(|| black_box(&event).to_json())
    });
}

fn bench_hook_event_to_env(c: &mut Criterion) {
    let event = HookEventV1::new("address-added", "eth0", 2, "netlink")
        .with_addresses(vec!["192.168.1.100".into(), "10.0.0.5".into()]);
    c.bench_function("hook_event_to_env", |b| {
        b.iter(|| black_box(&event).to_env())
    });
}

criterion_group!(
    benches,
    bench_validate_interface_name,
    bench_sanitize_env_value,
    bench_interface_selector_allows,
    bench_hook_event_to_json,
    bench_hook_event_to_env,
);
criterion_main!(benches);
