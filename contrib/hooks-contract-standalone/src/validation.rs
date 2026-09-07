// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Subset of netevd::system::validation used by the hook contract.

pub fn validate_state_name(state: &str) -> bool {
    matches!(
        state,
        "carrier"
            | "no-carrier"
            | "configured"
            | "degraded"
            | "routable"
            | "activated"
            | "disconnected"
            | "manager"
            | "routes"
            | "off"
            | "dormant"
            | "enslaved"
            | "linger"
            | "missing"
            | "address-added"
            | "address-removed"
            | "link-added"
            | "link-removed"
            | "mtu"
            | "neigh"
            | "dns"
    )
}

pub fn validate_interface_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 15 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

pub fn validate_ip_address(addr: &str) -> bool {
    addr.parse::<std::net::IpAddr>().is_ok()
}

pub fn validate_ip_list(list: &str) -> bool {
    list.is_empty() || list.split_whitespace().all(validate_ip_address)
}

pub fn sanitize_env_value(value: &str) -> Option<String> {
    const DANGEROUS_CHARS: &[char] = &[
        '$', '`', '\\', '"', '\'', ';', '&', '|', '<', '>', '\n', '\r', '\0', '(', ')', '{', '}',
        '[', ']', '!', '~', '?', '*', '#', '%',
    ];
    if value.chars().any(|c| DANGEROUS_CHARS.contains(&c)) {
        return None;
    }
    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dirs_ok() {
        for s in [
            "address-added",
            "address-removed",
            "link-added",
            "link-removed",
            "mtu",
            "neigh",
            "dns",
        ] {
            assert!(validate_state_name(s), "{s}");
        }
        assert!(!validate_state_name("../etc"));
    }
}
