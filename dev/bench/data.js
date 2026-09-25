window.BENCHMARK_DATA = {
  "lastUpdate": 1790368230435,
  "repoUrl": "https://github.com/zyvorai/netevd",
  "entries": {
    "Rust Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c0815525a49e1f9d40edf2d1d56a45c022e43afc",
          "message": "build(deps): Bump netlink-packet-core from 0.8.1 to 0.9.0 (#96)\n\nBumps [netlink-packet-core](https://github.com/rust-netlink/netlink-packet-core) from 0.8.1 to 0.9.0.\n- [Release notes](https://github.com/rust-netlink/netlink-packet-core/releases)\n- [Changelog](https://github.com/rust-netlink/netlink-packet-core/blob/main/CHANGELOG)\n- [Commits](https://github.com/rust-netlink/netlink-packet-core/compare/v0.8.1...v0.9.0)\n\n---\nupdated-dependencies:\n- dependency-name: netlink-packet-core\n  dependency-version: 0.9.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-26T01:54:57+05:30",
          "tree_id": "9b8f5ded2882062c8626da17141c878a770cd1f8",
          "url": "https://github.com/zyvorai/netevd/commit/c0815525a49e1f9d40edf2d1d56a45c022e43afc"
        },
        "date": 1790368228877,
        "tool": "cargo",
        "benches": [
          {
            "name": "validate_interface_name",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "sanitize_env_value",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "interface_selector_allows",
            "value": 171522,
            "range": "± 1696",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_json",
            "value": 295,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_env",
            "value": 960,
            "range": "± 3",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}