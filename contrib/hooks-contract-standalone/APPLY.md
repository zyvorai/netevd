# Apply this pack onto zyvorai/netevd

This zip is PR-1 code: versioned hook JSON, new dirs, debounce, glob match, hook timeout.

## Drop into the daemon tree

```
src/hooks/event.rs
src/hooks/debounce.rs
src/hooks/match_iface.rs
src/hooks/dispatch.rs
src/hooks/mod.rs
```

Wire:

* `src/lib.rs` — `pub mod hooks;`
* `src/system/validation.rs` — allow the new state names (see `src/validation.rs` in this crate)
* `src/system/execute.rs` — add `execute_scripts_with_timeout`
* `src/config/mod.rs` — `HooksConfig` + `monitoring.match_patterns` / `exclude`
* `config/netevd.example.yaml` — hooks + match/exclude examples

Listener call site (when an addr/link/route changes):

```rust
let ev = HookEventV1::new("address-added", &ifname, ifindex, backend)
    .with_addresses(addrs);
debouncer.push(ev);
// after debounce window:
for ev in debouncer.take_ready() {
    dispatch_event(&ev, &opts).await?;
}
```

## Test this pack standalone

```
cd netevd-hooks
cargo test --offline   # or cargo test if crates.io is reachable
```

## What was executed in this environment

See `TEST_RESULTS.txt` in the zip.
