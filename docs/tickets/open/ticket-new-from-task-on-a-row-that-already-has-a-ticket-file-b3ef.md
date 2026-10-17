---
id: br-b3ef
title: ticket new --from-task on a row that already has a ticket file re-links the row to the new id
kind: bug
created: 2026-10-17
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Found with br-cah9 (2026-10-17): running 'bridle ticket new --from-task <row>' on a row whose ticket file already existed minted a duplicate file (zzt7) and re-linked the row to it. Deleting the duplicate left the row pointing at a missing ticket, so 'ticket ready' failed and the row needed a refile (br-j86p). Expected: refuse when the row already has a ticket file. Related: a refused mint should write no file (br-j86p step 2).
