// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Network event watchers using real-time netlink events
//!
//! This implementation uses netlink multicast subscriptions for real-time
//! event notification with <100ms latency, replacing the previous polling
//! approach which had 5-second intervals.

use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;
use netlink_sys::{protocols::NETLINK_ROUTE, SocketAddr as NetlinkSocketAddr, TokioSocket};
use rtnetlink::packet_core::NetlinkMessage;
use rtnetlink::packet_route::RouteNetlinkMessage;
use rtnetlink::Handle;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::hooks::{HookEventV1, InterfaceSelector};

use super::{
    address::get_ipv4_addresses,
    route::{add_route, calculate_table_id, discover_gateway, remove_route},
    routing_rule::{add_routing_rule_from, add_routing_rule_to, remove_routing_rules},
    NetworkState,
};

type NetlinkEventReceiver =
    UnboundedReceiver<(NetlinkMessage<RouteNetlinkMessage>, NetlinkSocketAddr)>;

/// Create a netlink event receiver subscribed to the specified multicast groups.
/// This only returns a message receiver (no Handle), since the event watchers
/// only need to receive multicast notifications, not send requests.
fn new_event_receiver(
    groups: &[u32],
) -> std::io::Result<(impl std::future::Future<Output = ()>, NetlinkEventReceiver)> {
    use std::os::unix::io::{AsRawFd, FromRawFd};

    let mut socket = netlink_sys::Socket::new(NETLINK_ROUTE)?;
    let addr = NetlinkSocketAddr::new(0, 0);
    socket.bind(&addr)?;
    for group in groups {
        socket.add_membership(*group)?;
    }
    socket.set_non_blocking(true)?;

    // Transfer fd ownership: extract the raw fd before forgetting the socket
    // to prevent its destructor from closing it. TokioSocket takes ownership.
    // Note: if TokioSocket::from_raw_fd panics, the fd leaks — but this only
    // happens on tokio registration failure which is unrecoverable anyway.
    let raw_fd = socket.as_raw_fd();
    std::mem::forget(socket);
    let async_socket = unsafe { TokioSocket::from_raw_fd(raw_fd) };

    let (conn, _handle, messages) = rtnetlink::proto::from_socket_with_codec::<
        RouteNetlinkMessage,
        TokioSocket,
        rtnetlink::proto::NetlinkCodec,
    >(async_socket);
    Ok((
        async move {
            let _ = conn.await;
        },
        messages,
    ))
}

/// Watch for address changes using real-time netlink events
pub async fn watch_addresses(
    handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    routing_policy_interfaces: Vec<String>,
    hook_selector: InterfaceSelector,
    hook_tx: UnboundedSender<HookEventV1>,
) -> Result<()> {
    info!("Starting address watcher (real-time netlink events)");

    // Track addresses we've seen before
    let mut last_seen_addresses: HashSet<(u32, IpAddr)> = HashSet::new();

    // Subscribe to IPv4 address change notifications only
    // (IPv6 policy routing is handled separately via the ipv6 module)
    let (connection, mut messages) = new_event_receiver(&[libc::RTNLGRP_IPV4_IFADDR])?;
    tokio::spawn(connection);

    info!("Address watcher subscribed to netlink multicast groups");

    // Process address change events in real-time
    while let Some((message, _)) = messages.next().await {
        use rtnetlink::packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(
                RouteNetlinkMessage::NewAddress(msg),
            ) => ("new", msg),
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(
                RouteNetlinkMessage::DelAddress(msg),
            ) => ("del", msg),
            _ => continue,
        };
        let ifindex = msg.header.index;

        debug!("Address {} event on interface {}", event_type, ifindex);

        // Get interface name
        let link_name = {
            let state_read = state.read().await;
            state_read
                .get_link_name(ifindex)
                .cloned()
                .unwrap_or_default()
        };

        // Routing-policy interfaces get route configuration; hook-selector
        // interfaces get address-added/address-removed hooks. A link can be
        // both, either, or neither.
        let is_routing_policy_interface = {
            let state_read = state.read().await;
            routing_policy_interfaces
                .iter()
                .any(|name| state_read.get_link_index(name) == Some(ifindex))
        };
        let hook_allowed = hook_selector.allows(&link_name);

        if !is_routing_policy_interface && !hook_allowed {
            continue;
        }

        // Get current addresses for this interface
        match get_ipv4_addresses(&handle, ifindex).await {
            Ok(addresses) => {
                let current_addrs: HashSet<(u32, IpAddr)> =
                    addresses.iter().map(|addr| (ifindex, *addr)).collect();

                // Detect changes for this interface
                let old_addrs: HashSet<(u32, IpAddr)> = last_seen_addresses
                    .iter()
                    .filter(|(idx, _)| *idx == ifindex)
                    .copied()
                    .collect();

                if current_addrs != old_addrs {
                    info!(
                        "Address change detected on interface {} ({}): {} -> {} addresses",
                        link_name,
                        ifindex,
                        old_addrs.len(),
                        addresses.len()
                    );

                    if hook_allowed {
                        let added: Vec<String> = current_addrs
                            .iter()
                            .filter(|(_, addr)| !old_addrs.contains(&(ifindex, *addr)))
                            .map(|(_, addr)| addr.to_string())
                            .collect();
                        let removed: Vec<String> = old_addrs
                            .iter()
                            .filter(|(_, addr)| !current_addrs.contains(&(ifindex, *addr)))
                            .map(|(_, addr)| addr.to_string())
                            .collect();
                        if !added.is_empty() {
                            let _ = hook_tx.send(
                                HookEventV1::new("address-added", &link_name, ifindex, "netlink")
                                    .with_addresses(added),
                            );
                        }
                        if !removed.is_empty() {
                            let _ = hook_tx.send(
                                HookEventV1::new("address-removed", &link_name, ifindex, "netlink")
                                    .with_addresses(removed),
                            );
                        }
                    }

                    if is_routing_policy_interface {
                        if addresses.is_empty() {
                            info!(
                                "No addresses on interface {}, cleaning up routing configuration",
                                link_name
                            );
                            if let Err(e) = drop_configuration(&handle, &state, ifindex).await {
                                warn!("Failed to drop configuration: {}", e);
                            }

                            // Remove old addresses from tracking
                            last_seen_addresses.retain(|(idx, _)| *idx != ifindex);
                        } else {
                            // Clean up rules for removed addresses before adding new ones
                            let removed_addrs: Vec<IpAddr> = old_addrs
                                .iter()
                                .filter(|(_, addr)| !current_addrs.contains(&(ifindex, *addr)))
                                .map(|(_, addr)| *addr)
                                .collect();
                            if !removed_addrs.is_empty() {
                                let table = calculate_table_id(ifindex);
                                for addr in &removed_addrs {
                                    let _ = remove_routing_rules(&handle, *addr, table).await;
                                    state.write().await.routing_rules_from.remove(addr);
                                    state.write().await.routing_rules_to.remove(addr);
                                }
                            }

                            info!(
                                "Configuring routing rules for interface {} with {} addresses",
                                link_name,
                                addresses.len()
                            );
                            if let Err(e) =
                                configure_network(&handle, &state, ifindex, &addresses).await
                            {
                                warn!("Failed to configure network: {}", e);
                            }

                            // Update tracking
                            last_seen_addresses.retain(|(idx, _)| *idx != ifindex);
                            last_seen_addresses.extend(current_addrs);
                        }
                    } else {
                        // Hook-only interface: keep the diff baseline current
                        // without touching routing configuration.
                        last_seen_addresses.retain(|(idx, _)| *idx != ifindex);
                        last_seen_addresses.extend(current_addrs);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get addresses for interface {}: {}", ifindex, e);
            }
        }
    }

    Ok(())
}

/// Watch for route changes using real-time netlink events
pub async fn watch_routes(
    _handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    hook_tx: UnboundedSender<HookEventV1>,
) -> Result<()> {
    info!("Starting route watcher (real-time netlink events)");

    // Subscribe to route change notifications via multicast groups
    let (connection, mut messages) =
        new_event_receiver(&[libc::RTNLGRP_IPV4_ROUTE, libc::RTNLGRP_IPV6_ROUTE])?;
    tokio::spawn(connection);

    info!("Route watcher subscribed to netlink multicast groups");

    // Process route change events
    while let Some((message, _)) = messages.next().await {
        use rtnetlink::packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(
                RouteNetlinkMessage::NewRoute(msg),
            ) => ("new", msg),
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(
                RouteNetlinkMessage::DelRoute(msg),
            ) => ("del", msg),
            _ => continue,
        };

        // Extract output interface from attributes
        use rtnetlink::packet_route::route::RouteAttribute;
        let ifindex = msg
            .attributes
            .iter()
            .find_map(|attr| {
                if let RouteAttribute::Oif(idx) = attr {
                    Some(*idx)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        if ifindex == 0 {
            continue; // Skip routes without interface
        }

        debug!("Route {} event on interface {}", event_type, ifindex);

        // Get interface name
        let link_name = {
            let state_read = state.read().await;
            state_read
                .get_link_name(ifindex)
                .cloned()
                .unwrap_or_default()
        };

        info!(
            "Route {} on interface {} ({})",
            event_type, link_name, ifindex
        );

        let _ = hook_tx.send(
            HookEventV1::new("routes", &link_name, ifindex, "netlink")
                .with_routes_delta(vec![event_type.to_string()]),
        );
    }

    Ok(())
}

/// Watch for link changes using real-time netlink events
pub async fn watch_links(
    _handle: Handle,
    state: Arc<RwLock<NetworkState>>,
    hook_selector: InterfaceSelector,
    hook_tx: UnboundedSender<HookEventV1>,
) -> Result<()> {
    info!("Starting link watcher (real-time netlink events)");

    // Track last-seen MTU per ifindex so we only fire mtu.d on real changes
    let mut last_mtu: HashMap<u32, u32> = HashMap::new();

    // Subscribe to link change notifications via multicast groups
    let (connection, mut messages) = new_event_receiver(&[libc::RTNLGRP_LINK])?;
    tokio::spawn(connection);

    info!("Link watcher subscribed to netlink multicast groups");

    // Process link change events
    while let Some((message, _)) = messages.next().await {
        use rtnetlink::packet_route::RouteNetlinkMessage;

        let (event_type, msg) = match message.payload {
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(
                msg,
            )) => ("new", msg),
            rtnetlink::packet_core::NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelLink(
                msg,
            )) => ("del", msg),
            _ => continue,
        };
        let ifindex = msg.header.index;

        debug!("Link {} event on interface {}", event_type, ifindex);

        // For link additions, extract link name (and MTU) from the message and update state
        if event_type == "new" {
            use rtnetlink::packet_route::link::LinkAttribute;
            let mut new_name = None;
            let mut mtu = None;
            for attr in &msg.attributes {
                match attr {
                    LinkAttribute::IfName(name) => new_name = Some(name.clone()),
                    LinkAttribute::Mtu(m) => mtu = Some(*m),
                    _ => {}
                }
            }

            if let Some(name) = new_name {
                let already_known = state.read().await.get_link_name(ifindex).is_some();
                info!("Link added: {} ({})", name, ifindex);
                state.write().await.add_link(name.clone(), ifindex);

                if !already_known && hook_selector.allows(&name) {
                    let _ = hook_tx.send(HookEventV1::new("link-added", &name, ifindex, "netlink"));
                }

                if let Some(m) = mtu {
                    let prev = last_mtu.insert(ifindex, m);
                    let changed = matches!(prev, Some(old) if old != m);
                    if changed && hook_selector.allows(&name) {
                        let _ = hook_tx
                            .send(HookEventV1::new("mtu", &name, ifindex, "netlink").with_mtu(m));
                    }
                }
            } else {
                debug!(
                    "Link added with ifindex {} but no name in attributes",
                    ifindex
                );
            }
        }

        // For link deletions, clean up our state
        if event_type == "del" {
            let mut state_write = state.write().await;
            let link_name = state_write
                .get_link_name(ifindex)
                .cloned()
                .unwrap_or_default();
            info!("Link removed: {} ({})", link_name, ifindex);
            state_write.remove_link(ifindex);
            drop(state_write);
            last_mtu.remove(&ifindex);

            if hook_selector.allows(&link_name) {
                let _ = hook_tx.send(HookEventV1::new(
                    "link-removed",
                    &link_name,
                    ifindex,
                    "netlink",
                ));
            }
        }
    }

    Ok(())
}

/// Configure routing rules and routes for an interface
async fn configure_network(
    handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
    ifindex: u32,
    addresses: &[IpAddr],
) -> Result<()> {
    let table = calculate_table_id(ifindex);

    // Discover gateway for this interface
    let gateway = match discover_gateway(handle, ifindex).await? {
        Some(gw) => gw,
        None => {
            warn!("No gateway found for interface {}", ifindex);
            return Ok(());
        }
    };

    // Add default route to custom table
    add_route(handle, ifindex, gateway, table).await?;

    // Add routing rules for each address
    for address in addresses {
        // Add "from" rule
        add_routing_rule_from(handle, *address, table).await?;

        // Add "to" rule
        add_routing_rule_to(handle, *address, table).await?;
    }

    // Update state in a single atomic write operation
    // This prevents race conditions where another watcher could modify state
    // between individual updates
    {
        let mut state_write = state.write().await;

        // Add all routing rules to state
        for address in addresses {
            state_write.add_routing_rule_from(*address, table);
            state_write.add_routing_rule_to(*address, table);
        }

        // Add route to state
        state_write.add_route(ifindex, table, Some(gateway));
    }

    info!(
        "Successfully configured routing for interface {} with {} addresses",
        ifindex,
        addresses.len()
    );

    Ok(())
}

/// Remove routing configuration for an interface
async fn drop_configuration(
    handle: &Handle,
    state: &Arc<RwLock<NetworkState>>,
    ifindex: u32,
) -> Result<()> {
    let table = calculate_table_id(ifindex);

    // Get addresses that need to be cleaned up (deduplicated)
    let addresses_to_clean: Vec<IpAddr> = {
        let state_read = state.read().await;
        let addrs: HashSet<IpAddr> = state_read
            .routing_rules_from
            .keys()
            .chain(state_read.routing_rules_to.keys())
            .filter(|addr| {
                state_read
                    .routing_rules_from
                    .get(addr)
                    .map(|rule| rule.table == table)
                    .unwrap_or(false)
                    || state_read
                        .routing_rules_to
                        .get(addr)
                        .map(|rule| rule.table == table)
                        .unwrap_or(false)
            })
            .copied()
            .collect();
        addrs.into_iter().collect()
    };

    // Remove routing rules
    for address in &addresses_to_clean {
        if let Err(e) = remove_routing_rules(handle, *address, table).await {
            warn!("Failed to remove routing rules for {}: {}", address, e);
        }
    }

    // Remove route
    if let Err(e) = remove_route(handle, ifindex, table).await {
        warn!("Failed to remove route for interface {}: {}", ifindex, e);
    }

    // Update state
    {
        let mut state_write = state.write().await;
        for address in &addresses_to_clean {
            state_write.routing_rules_from.remove(address);
            state_write.routing_rules_to.remove(address);
        }
        state_write.routes.remove(&(ifindex, table));
    }

    info!(
        "Cleaned up routing configuration for interface {} ({} addresses)",
        ifindex,
        addresses_to_clean.len()
    );

    Ok(())
}
