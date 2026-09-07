# netevd hook contract — offline sandbox

This is the standalone crate used to validate the `netevd.event.v1` hook
contract (versioned JSON event, debounce, glob interface matching, script
dispatch) in isolation, before it was wired into the daemon proper as
`src/hooks/` and `src/network/watcher.rs` in the main crate. See
[`../../docs/hooks-contract.md`](../../docs/hooks-contract.md) for how the
real thing is wired up, and [`../../docs/ROADMAP.md`](../../docs/ROADMAP.md)
for the PR-1/2/3 plan this came out of.

It's a fully independent Cargo package (own `Cargo.toml`/`Cargo.lock`,
`publish = false`) — not a workspace member of the root `netevd` crate, so it
never affects the main build.

## Re-run its tests

With cargo/network access:

```
cargo test
```

Or, matching how it was originally exercised in a sandbox with no crates.io
access (`src/contract_std.rs` is a hand-written, `std`-only reimplementation
of the same contract, kept in sync manually):

```
sh run_tests.sh
```

See `TEST_RESULTS.txt` for the last recorded run, and `APPLY.md` for the
original notes on carrying this contract onto the real daemon tree (now
already applied — see `docs/hooks-contract.md` instead for current state).
