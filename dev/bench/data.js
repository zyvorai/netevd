window.BENCHMARK_DATA = {
  "lastUpdate": 1789424068724,
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
          "id": "16c378bf0625cdbb0e6f40e5b30f8e632b02007b",
          "message": "build(deps): Bump futures from 0.3.32 to 0.3.34 (#92)\n\nBumps [futures](https://github.com/rust-lang/futures-rs) from 0.3.32 to 0.3.34.\n- [Release notes](https://github.com/rust-lang/futures-rs/releases)\n- [Changelog](https://github.com/rust-lang/futures-rs/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/rust-lang/futures-rs/compare/0.3.32...0.3.34)\n\n---\nupdated-dependencies:\n- dependency-name: futures\n  dependency-version: 0.3.34\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-15T01:41:44+05:30",
          "tree_id": "ed0f18c9663ba8d393e29058bb8d8800ba8b430f",
          "url": "https://github.com/zyvorai/netevd/commit/16c378bf0625cdbb0e6f40e5b30f8e632b02007b"
        },
        "date": 1789424067265,
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "interface_selector_allows",
            "value": 191602,
            "range": "± 7810",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_json",
            "value": 317,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_env",
            "value": 895,
            "range": "± 3",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}