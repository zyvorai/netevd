# FAQ

Questions people evaluating netevd actually ask. For "why netevd over X",
see the README's [Instead of...](https://github.com/zyvorai/netevd#instead-of) section — this
FAQ covers what that section doesn't.

## Licensing & cost

**Is the Community Edition really free?** Yes. Apache-2.0 — use, modify,
and run it for personal, lab, and commercial production use at no charge.
See the README's [License](https://github.com/zyvorai/netevd#license) section.

**What's the difference between Community and Enterprise?** See the
README's [Enterprise](https://github.com/zyvorai/netevd#enterprise) section and
[`docs/enterprise.md`](enterprise.md) for the full comparison table —
briefly: Community is the self-hosted daemon with GitHub Issues support;
Enterprise adds SLA-backed support and the broader HyperSDK
platform (`netctl`, `cloud-netconfig`).

## Support

**What if I find a bug?** Open a GitHub issue.

**What if I find a security vulnerability?** See [`SECURITY.md`](https://github.com/zyvorai/netevd/blob/main/SECURITY.md)
for private reporting to legal@zyvor.dev — response SLA is documented
there (initial response within 48 hours; critical fixes within 7 days).

**Is there a support SLA for Community Edition?** No — that's specifically
what the Enterprise tier adds. Community support is via GitHub Issues.

## Production readiness

**Is this production-ready?** The README's own License section already
states it's licensed for "commercial production use," and `SECURITY.md`
documents a real defense-in-depth model (privilege drop, `CAP_NET_ADMIN`
only, input validation, systemd hardening) plus an explicit "what netevd
does NOT protect against" section — read that before deploying, since it's
the honest boundary of the security model (e.g. it does not protect
against a malicious script you've already placed in a hook directory, or a
root compromise).

**What's the current version?** Check the latest git tag/release —
`CONTRIBUTING.md`/`CHANGELOG.md` don't exist in this repo yet, so releases
and their notes live in GitHub Releases directly.

## Platform & compatibility

**Which network managers does it support?** systemd-networkd,
NetworkManager, and dhclient — one daemon handles all three, per the
README's "Why netevd?" table.

**Is there a web dashboard?** No shipped browser console —
[`docs/user/using-the-dashboard.md`](user/using-the-dashboard.md) is
explicit that operators use the CLI, REST API, Prometheus metrics, and the
systemd journal as the primary surfaces; a static dashboard HTML exists
but isn't the primary way to operate netevd.

## Performance

**Are the performance numbers (sub-100ms, 1000+ events/sec) benchmarked
against a specific environment?** The README's Performance table states
the figures; check `.github/workflows/benchmark.yml` for the CI benchmark
job if you need the methodology behind them before relying on the numbers
for capacity planning.
