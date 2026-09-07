// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! netevd - Network Event Daemon
//!
//! Library components for network event monitoring and handling

// This crate is a library (`[lib]`) with a thin binary (`[[bin]] netevd`) on
// top. Clippy's --all-targets dead-code pass only sees what main.rs and the
// test suites actually call, so most of this public API — used by its own
// unit tests, or reserved for callers outside the current binary — reads as
// "never used" even though it's real, working, intentionally public surface.
#![allow(dead_code)]

pub mod bus;
pub mod config;
pub mod hooks;
pub mod listeners;
pub mod network;
pub mod system;

// New modules for enhanced functionality
pub mod api;
pub mod audit;
pub mod cli;
pub mod cloud;
pub mod filters;
pub mod metrics;
