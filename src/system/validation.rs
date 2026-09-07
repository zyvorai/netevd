// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Input validation for untrusted data
//!
//! This module provides validation functions to sanitize data from untrusted
//! sources (DHCP servers, network configurations, DBus signals) before passing
//! them to shell scripts or system APIs.

use std::net::IpAddr;

/// Validate an interface name
///
/// Interface names must be:
/// - 1-15 characters long (IFNAMSIZ - 1)
/// - Alphanumeric plus underscore, hyphen, and period
/// - Not containing shell metacharacters
pub fn validate_interface_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 15 {
        return false;
    }

    // Only allow safe characters
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

/// Validate a hostname
///
/// Hostnames must be:
/// - 1-253 characters total
/// - Each label 1-63 characters
/// - Only alphanumeric, hyphen (not at start/end of label)
/// - Labels separated by dots
pub fn validate_hostname(hostname: &str) -> bool {
    if hostname.is_empty() || hostname.len() > 253 {
        return false;
    }

    for label in hostname.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }

        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }

    true
}

/// Validate a domain name (similar to hostname but allows wildcards)
pub fn validate_domain_name(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    for (i, label) in domain.split('.').enumerate() {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        // Allow wildcard prefix only on the first label
        let label_to_check = if i == 0 {
            label.strip_prefix('*').unwrap_or(label)
        } else if label.contains('*') {
            return false; // wildcard not allowed on non-first labels
        } else {
            label
        };

        if label_to_check.is_empty() {
            continue; // bare wildcard as first label is valid
        }

        if label_to_check.starts_with('-') || label_to_check.ends_with('-') {
            return false;
        }

        if !label_to_check
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return false;
        }
    }

    true
}

/// Validate an IP address string
pub fn validate_ip_address(addr: &str) -> bool {
    addr.parse::<IpAddr>().is_ok()
}

/// Validate a space-separated list of IP addresses
pub fn validate_ip_list(list: &str) -> bool {
    if list.is_empty() {
        return true; // Empty list is valid
    }

    list.split_whitespace().all(validate_ip_address)
}

/// Sanitize a string for safe use in environment variables
///
/// Removes or escapes potentially dangerous characters that could be
/// interpreted by shells. This is defense-in-depth; scripts should
/// still properly quote variables.
///
/// Returns None if the string contains dangerous patterns.
pub fn sanitize_env_value(value: &str) -> Option<String> {
    // Reject values with shell metacharacters
    const DANGEROUS_CHARS: &[char] = &[
        '$', '`', '\\', '"', '\'', ';', '&', '|', '<', '>', '\n', '\r', '\0', '(', ')', '{', '}',
        '[', ']', '!', '~', '?', '*', '#', '%',
    ];

    if value.chars().any(|c| DANGEROUS_CHARS.contains(&c)) {
        return None;
    }

    Some(value.to_string())
}

/// Validate a network state string (used for script directory names)
pub fn validate_state_name(state: &str) -> bool {
    // Only allow known safe state names
    // Includes all systemd-networkd operational states and NetworkManager states
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_interface_name() {
        assert!(validate_interface_name("eth0"));
        assert!(validate_interface_name("wlan0"));
        assert!(validate_interface_name("br-1234"));
        assert!(validate_interface_name("veth_test"));

        assert!(!validate_interface_name(""));
        assert!(!validate_interface_name("a".repeat(16).as_str()));
        assert!(!validate_interface_name("eth0; rm -rf /"));
        assert!(!validate_interface_name("eth$0"));
    }

    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("localhost"));
        assert!(validate_hostname("example.com"));
        assert!(validate_hostname("sub.example.com"));
        assert!(validate_hostname("my-host"));

        assert!(!validate_hostname(""));
        assert!(!validate_hostname("-invalid"));
        assert!(!validate_hostname("invalid-"));
        assert!(!validate_hostname("in valid"));
        assert!(!validate_hostname(&"a".repeat(64)));
    }

    #[test]
    fn test_validate_domain_name() {
        assert!(validate_domain_name("example.com"));
        assert!(validate_domain_name("*.example.com"));
        assert!(validate_domain_name("sub.example.com"));

        assert!(!validate_domain_name(""));
        assert!(!validate_domain_name("invalid domain"));
        assert!(!validate_domain_name("foo.*.com")); // wildcard only on first label
    }

    #[test]
    fn test_validate_state_name_new_hook_dirs() {
        for s in [
            "address-added",
            "address-removed",
            "link-added",
            "link-removed",
            "mtu",
            "neigh",
            "dns",
            "routable",
        ] {
            assert!(validate_state_name(s), "{s}");
        }
        assert!(!validate_state_name(""));
        assert!(!validate_state_name("../etc"));
        assert!(!validate_state_name("address-added;rm"));
    }

    #[test]
    fn test_validate_ip_address() {
        assert!(validate_ip_address("192.168.1.1"));
        assert!(validate_ip_address("10.0.0.1"));
        assert!(validate_ip_address("::1"));
        assert!(validate_ip_address("2001:db8::1"));

        assert!(!validate_ip_address(""));
        assert!(!validate_ip_address("256.256.256.256"));
        assert!(!validate_ip_address("not-an-ip"));
    }

    #[test]
    fn test_validate_ip_list() {
        assert!(validate_ip_list(""));
        assert!(validate_ip_list("192.168.1.1"));
        assert!(validate_ip_list("192.168.1.1 10.0.0.1"));
        assert!(validate_ip_list("192.168.1.1 2001:db8::1"));

        assert!(!validate_ip_list("192.168.1.1 invalid"));
        assert!(!validate_ip_list("not an ip list"));
    }

    #[test]
    fn test_sanitize_env_value() {
        assert_eq!(
            sanitize_env_value("safe_value-123"),
            Some("safe_value-123".to_string())
        );
        assert_eq!(
            sanitize_env_value("192.168.1.1"),
            Some("192.168.1.1".to_string())
        );

        // Should reject dangerous patterns
        assert_eq!(sanitize_env_value("value; rm -rf /"), None);
        assert_eq!(sanitize_env_value("$(whoami)"), None);
        assert_eq!(sanitize_env_value("`whoami`"), None);
        assert_eq!(sanitize_env_value("value && malicious"), None);
        assert_eq!(sanitize_env_value("val$ue"), None);
    }

    #[test]
    fn test_validate_state_name() {
        assert!(validate_state_name("routable"));
        assert!(validate_state_name("activated"));
        assert!(validate_state_name("no-carrier"));

        assert!(!validate_state_name(""));
        assert!(!validate_state_name("invalid-state"));
        assert!(!validate_state_name("../../../etc/passwd"));
    }
}
