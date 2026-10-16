---
id: br-zwup
title: "Small gaps: stale reopened line in roles-and-lifecycle, mail watchdog outside S3 calls"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-16T21:08:11Z
---

## The ask

From the PdM's 2026-10-16 verification.

1. docs/design/roles-and-lifecycle.md lines 62-64 still say a reopened ticket is a conflict for `task plan`; br-r66n landed and changed that. Fix the doc to the built behaviour.
2. br-mfcj's watchdog sits inside the bridge loop, so a hang outside an S3 call still wedges the mail bridge. Put the watchdog around the whole loop iteration (or a separate task) so any hang is caught; add a test with a fake hang.

Acceptance: just check. Model: Sonnet. Out of scope: other mail bridge redesign.
