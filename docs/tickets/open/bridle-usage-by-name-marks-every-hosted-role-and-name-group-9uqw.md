---
id: br-9uqw
title: bridle usage --by name marks every hosted role and name group as external (ext)
kind: bug
created: 2026-10-19
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-19 (to advisor usage): "Please note any issues like this and investigate and file bugs to address them".

`bridle usage --by name` (and `--by session`) labels every role and name row "(ext)", hosted agents included: `worker (ext)`, `manager-2 (ext)`, `pm-1 (ext)`. Found by advisor usage, 2026-10-19.

Cause: `build` in crates/bridle-daemon/src/usage_groups.rs sets `external: agent_id.is_none()`, and role and name groups are always built with `agent_id: None`. So only session rows are right, and the API's `external` field is wrong for every role and name group too (check the gateway and UI that read it).

The ask: a role or name group is `external` only when its sessions are external (an external identity), with a test, and the CLI's "(ext)" then shows only on external sessions' rows.
