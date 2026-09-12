window.BENCHMARK_DATA = {
  "lastUpdate": 1789242940839,
  "repoUrl": "https://github.com/zyvorai/netevd",
  "entries": {
    "Rust Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "ssahani@gmail.com",
            "name": "ssahani",
            "username": "ssahani"
          },
          "committer": {
            "email": "ssahani@gmail.com",
            "name": "ssahani",
            "username": "ssahani"
          },
          "distinct": true,
          "id": "7aa32dc14e5d81b2e13c09fef10dbd1cf9d9b0b6",
          "message": "Ignore __pycache__/ (docs/overrides/hooks.py bytecode cache)\n\nCo-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>\nClaude-Session: https://claude.ai/code/session_014Z4m6JGKATrFMBiwJiJgad",
          "timestamp": "2026-09-13T01:17:47+05:30",
          "tree_id": "8d10db02dcb99d0a468f30f877cfeb707fe9fe5a",
          "url": "https://github.com/zyvorai/netevd/commit/7aa32dc14e5d81b2e13c09fef10dbd1cf9d9b0b6"
        },
        "date": 1789242939307,
        "tool": "cargo",
        "benches": [
          {
            "name": "validate_interface_name",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "sanitize_env_value",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "interface_selector_allows",
            "value": 182665,
            "range": "± 2151",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_json",
            "value": 286,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "hook_event_to_env",
            "value": 889,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}