// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

//! BPF object resolution + optional Aya attach / ringbuf poll.

use std::path::{Path, PathBuf};

use crate::config::EbpfConfig;

pub fn resolve_object(cfg: &EbpfConfig) -> Option<PathBuf> {
    if !cfg.object_path.is_empty() {
        let p = PathBuf::from(&cfg.object_path);
        return p.is_file().then_some(p);
    }
    [
        PathBuf::from("/usr/lib/netevd/netevd-ebpf.o"),
        PathBuf::from("/usr/libexec/netevd/netevd-ebpf.o"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

pub fn object_exists(path: &Path) -> bool {
    path.is_file()
}

#[cfg(feature = "ebpf")]
pub mod aya_attach {
    use super::*;
    use anyhow::{bail, Context, Result};
    use aya::maps::RingBuf;
    use aya::programs::TracePoint;
    use aya::Ebpf;

    pub struct Loaded {
        pub bpf: Ebpf,
        pub attachments: usize,
    }

    pub fn load_and_attach(cfg: &EbpfConfig) -> Result<Loaded> {
        let Some(path) = resolve_object(cfg) else {
            bail!("eBPF enabled but netevd-ebpf.o not found");
        };
        let mut bpf = Ebpf::load_file(&path).with_context(|| path.display().to_string())?;
        let mut attachments = 0usize;
        if cfg.drops {
            attachments += attach_tp(&mut bpf, "observe_kfree_skb", "skb", "kfree_skb")?;
        }
        if cfg.tcp_retransmit {
            attachments += attach_tp(
                &mut bpf,
                "observe_tcp_retransmit",
                "tcp",
                "tcp_retransmit_skb",
            )?;
        }
        Ok(Loaded { bpf, attachments })
    }

    fn attach_tp(bpf: &mut Ebpf, prog: &str, category: &str, name: &str) -> Result<usize> {
        let p = bpf
            .program_mut(prog)
            .with_context(|| format!("missing program {prog}"))?;
        let tp: &mut TracePoint = p.try_into()?;
        tp.load()?;
        tp.attach(category, name)?;
        Ok(1)
    }

    pub fn take_ring(bpf: &mut Ebpf) -> Result<RingBuf<aya::maps::MapData>> {
        let map = bpf
            .take_map("events")
            .context("missing ringbuf map `events`")?;
        RingBuf::try_from(map).context("map `events` is not a ringbuf")
    }
}

#[cfg(not(feature = "ebpf"))]
pub mod aya_attach {
    use super::*;
    use anyhow::{bail, Result};

    pub fn load_and_attach(_cfg: &EbpfConfig) -> Result<()> {
        bail!("netevd was built without the `ebpf` cargo feature")
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
