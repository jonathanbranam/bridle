---
id: br-qfut
title: "ticket new --from-task and ticket ready re-link a row to the wrong ticket: br-cah9 couldn't be readied by anyone"
kind: bug
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Found 2026-10-16 ~10:40 PM ET while readying br-cah9 (the human had approved it). Several agents
filed and readied the same bug at once, and the ticket-to-row link broke:

- `ticket new --from-task` (pm-1) minted br-ne7n, an empty duplicate; the duplicate file
  `...-zzt7.md` (manager-2) was later deleted.
- Row br-cah9 then belonged to "ticket zzt7": `bridle ticket ready br-cah9` failed with
  "conflict: task br-ne7n is dropped, not pending", `bridle task ready br-cah9` with "task br-cah9
  belongs to ticket zzt7: use `bridle ticket ready zzt7`", and `ticket ready zzt7` with "no ticket
  with id zzt7". Nobody could ready it; it was refiled as br-j86p.
- aide's `bridle ticket ready br-cah9` printed br-ne7n and readied that instead.

Fix: `ticket new --from-task` and `ticket ready` never re-link an existing row to another
ticket id; `ticket ready <id>` readies the row with that id (or refuses naming the real
problem); a row whose ticket file is gone is reported by `bridle ticket check` with the repair.

Done when: the sequence above can't leave a row unready-able; tests; `ticket check` names a
row linked to a missing ticket.
