// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Hook contract: versioned JSON events, interface match, debounce, dispatch.

pub mod debounce;
pub mod dispatch;
pub mod event;
pub mod match_iface;
pub mod service;

pub use debounce::HookDebouncer;
pub use dispatch::{dispatch_event, hook_dir, HookDispatchOpts};
pub use event::{HookEventV1, HOOK_STATES, SCHEMA_V1};
pub use match_iface::{glob_matches, InterfaceSelector, DEFAULT_EXCLUDES};
pub use service::spawn as spawn_service;
