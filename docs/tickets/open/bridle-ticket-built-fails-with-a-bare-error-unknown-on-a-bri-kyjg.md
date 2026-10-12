---
id: br-kyjg
title: "'bridle ticket built' fails with a bare 'error: unknown: ' on a bridle-ui ticket"
kind: bug
created: 2026-10-12
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

On bridle-ui, worker usage-page (2026-10-12 23:14Z) ran `bridle ticket built u6uz --branch bridle/usage-page --commit 717186a` and `bridle task built ui-u6uz ...`; both failed with `error: unknown: ` and no detail. The work was committed and checked green; manager-2 landed it with `bridle task land` instead (5690c8b), so nothing was lost. Recorded on the ui-u6uz thread.

Fix: find why `built` fails for this ticket and make the error say what went wrong (an empty `unknown:` message is itself a bug). Possibly related: shortly before, `bridle --project bridle-ui ticket edit ui-u6uz --body-file ...` printed the row and exited 0, but the new body appears neither in the ticket file nor in `ticket show`.

Check: `ticket built` on a planned, claimed bridle-ui ticket succeeds or fails with a named cause; `ticket edit --body-file` changes the body `ticket show` prints (or says where it went).

Model: Sonnet (diagnosis).
