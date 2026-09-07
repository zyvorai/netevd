// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Hook contract: versioned JSON events, interface match, debounce, dispatch.

pub mod debounce;
pub mod dispatch;
pub mod event;
pub mod match_iface;
pub mod service;

pub use dispatch::HookDispatchOpts;
pub use event::HookEventV1;
pub use match_iface::InterfaceSelector;
pub use service::spawn as spawn_service;
