---
id: br-b2g8
title: "Home-project warning names dalek.local-home: it uses the raw hostname, not [machine] name"
kind: fix
created: 2026-10-20
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The warning that tells a sender to address the orchestrator with `--project <host>-home` builds the host from the raw `hostname`. On dalek that is `dalek.local`, so it says `--project dalek.local-home`, which is not a known daemon. The home project is `dalek-home`. Take the host from `[machine] name`, then `hostname` (the same function the daemon uses for owner.toml, as br-pcde did for the mail bridge), and grep for any other `<host>-home` builders using the raw hostname.

Seen: manager-2 (bridle), 2026-10-20 22:10 UTC, m-0370 to external:orchestrator: "--project dalek.local-home, the name the warning gave, is not a known daemon here, so I sent without it."

Check: `just check`, plus a test: with `[machine] name = "dalek"` and hostname `dalek.local`, the warning names `dalek-home`.

Model: Sonnet (Haiku would do; small change). Related: br-pcde. Files: grep `-home` / `home_project` in crates/bridle and crates/bridle-daemon.
