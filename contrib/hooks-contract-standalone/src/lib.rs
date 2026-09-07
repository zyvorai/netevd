// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Standalone, drop-in hook contract for netevd.
//!
//! Copy `src/hooks/` into the netevd tree and wire `pub mod hooks;` in lib.rs.
//! This crate is the same modules plus a tiny local validation/execute so the
//! contract can be cargo-tested without the full daemon lockfile.

pub mod debounce;
pub mod dispatch;
pub mod event;
pub mod execute;
pub mod match_iface;
pub mod validation;

pub use debounce::HookDebouncer;
pub use dispatch::{dispatch_event, hook_dir, HookDispatchOpts};
pub use event::{HookEventV1, HOOK_STATES, SCHEMA_V1};
pub use match_iface::{glob_matches, InterfaceSelector, DEFAULT_EXCLUDES};
