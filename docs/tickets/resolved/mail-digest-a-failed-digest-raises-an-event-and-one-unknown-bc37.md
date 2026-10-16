---
id: br-bc37
title: "Mail digest: a failed digest raises an event, and one unknown ticket kind/state no longer empties the task section (follow-up to tcae)"
kind: fix
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-16T10:02:58Z
---

## The ask

Follow-up to br-tcae item 2, which landed short. A failed digest is only a tracing::warn (crates/bridle-mail/src/bridge.rs ~244-255, failures.rs), not raised as an event or message the human sees. And the wire enums lack serde(other), so one unknown ticket kind or state empties the whole task section of the digest instead of skipping that item.
