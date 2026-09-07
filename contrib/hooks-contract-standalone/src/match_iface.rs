// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! Interface include/exclude matching (`eth*`, exact names).

/// Shell-style glob: `*` → `.*`, everything else regex-escaped, anchored.
pub fn glob_matches(pattern: &str, name: &str) -> bool {
    let escaped = regex::escape(pattern).replace(r"\*", ".*");
    regex::Regex::new(&format!("^{escaped}$"))
        .map(|re| re.is_match(name))
        .unwrap_or(false)
}

/// Default virtual / CNI ifaces we do not hook unless opted in.
pub const DEFAULT_EXCLUDES: &[&str] = &[
    "lo",
    "docker*",
    "br-*",
    "cni*",
    "flannel*",
    "cilium*",
    "veth*",
    "virbr*",
];

#[derive(Debug, Clone, Default)]
pub struct InterfaceSelector {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl InterfaceSelector {
    pub fn from_lists(include: Vec<String>, exclude: Vec<String>) -> Self {
        Self { include, exclude }
    }

    pub fn with_default_excludes(mut self) -> Self {
        if self.exclude.is_empty() {
            self.exclude = DEFAULT_EXCLUDES
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        }
        self
    }

    pub fn allows(&self, name: &str) -> bool {
        if self.exclude.iter().any(|p| glob_matches(p, name)) {
            return false;
        }
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|p| glob_matches(p, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_exact_and_star() {
        assert!(glob_matches("eth0", "eth0"));
        assert!(!glob_matches("eth0", "eth1"));
        assert!(glob_matches("eth*", "eth0"));
        assert!(glob_matches("eth*", "eth12"));
        assert!(glob_matches("wg*", "wg0"));
        assert!(!glob_matches("wg*", "eth0"));
        assert!(glob_matches("enp*", "enp1s0"));
    }

    #[test]
    fn default_excludes_virtual() {
        let sel = InterfaceSelector::default().with_default_excludes();
        assert!(!sel.allows("lo"));
        assert!(!sel.allows("docker0"));
        assert!(!sel.allows("veth1a2b"));
        assert!(!sel.allows("cilium_host"));
        assert!(!sel.allows("cni0"));
        assert!(sel.allows("eth0"));
        assert!(sel.allows("enp3s0"));
        assert!(sel.allows("wg0"));
    }

    #[test]
    fn include_restricts() {
        let sel = InterfaceSelector::from_lists(
            vec!["eth*".into(), "wg*".into()],
            vec!["eth1".into()],
        );
        assert!(sel.allows("eth0"));
        assert!(!sel.allows("eth1"));
        assert!(sel.allows("wg0"));
        assert!(!sel.allows("enp0s3"));
    }

    #[test]
    fn exclude_wins() {
        let sel = InterfaceSelector::from_lists(vec!["*".into()], vec!["veth*".into()]);
        assert!(sel.allows("eth0"));
        assert!(!sel.allows("veth99"));
    }
}
