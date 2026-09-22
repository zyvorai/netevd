// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! BPF object resolution. Actual Aya attach lives behind `feature = "ebpf"`.

use std::path::{Path, PathBuf};

use crate::config::EbpfConfig;

pub fn resolve_object(cfg: &EbpfConfig) -> Option<PathBuf> {
    if !cfg.object_path.is_empty() {
        let p = PathBuf::from(&cfg.object_path);
        return p.is_file().then_some(p);
    }
    let candidates = [
        PathBuf::from("/usr/lib/netevd/netevd-ebpf.o"),
        PathBuf::from("/usr/libexec/netevd/netevd-ebpf.o"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn object_exists(path: &Path) -> bool {
    path.is_file()
}

/// Attached observe-only programs. Dropping this unloads them from the kernel.
pub struct AttachedObserve {
    pub count: usize,
    #[cfg(feature = "ebpf")]
    _bpf: aya::Ebpf,
}

#[cfg(feature = "ebpf")]
pub mod aya_attach {
    use super::*;
    use anyhow::{bail, Context, Result};

    pub fn load_and_attach(cfg: &EbpfConfig) -> Result<AttachedObserve> {
        let Some(path) = resolve_object(cfg) else {
            bail!("eBPF enabled but netevd-ebpf.o not found");
        };
        let mut bpf = aya::Ebpf::load_file(&path).with_context(|| path.display().to_string())?;
        let mut attached = 0usize;
        if cfg.drops {
            attached += attach_tracepoint(&mut bpf, "observe_kfree_skb", "skb", "kfree_skb")?;
        }
        if cfg.tcp_retransmit {
            attached += attach_tracepoint(
                &mut bpf,
                "observe_tcp_retransmit",
                "tcp",
                "tcp_retransmit_skb",
            )?;
        }
        Ok(AttachedObserve {
            count: attached,
            _bpf: bpf,
        })
    }

    fn attach_tracepoint(
        bpf: &mut aya::Ebpf,
        prog: &str,
        category: &str,
        name: &str,
    ) -> Result<usize> {
        let p = bpf
            .program_mut(prog)
            .with_context(|| format!("missing program {prog}"))?;
        let tp: &mut aya::programs::TracePoint = p.try_into()?;
        tp.load()?;
        tp.attach(category, name)?;
        Ok(1)
    }
}

#[cfg(not(feature = "ebpf"))]
pub mod aya_attach {
    use super::*;
    use anyhow::{bail, Result};

    pub fn load_and_attach(_cfg: &EbpfConfig) -> Result<AttachedObserve> {
        bail!("netevd was built without the `ebpf` cargo feature");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EbpfConfig;

    #[test]
    fn resolve_empty_when_missing() {
        let mut cfg = EbpfConfig::default();
        cfg.object_path = "/no/such/netevd-ebpf.o".into();
        assert!(resolve_object(&cfg).is_none());
    }
}
