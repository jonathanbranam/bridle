+++
id = "br-mm4z"
title = "Migration 0005-a-ticket-for-every-active-task aborts on dead task rows instead of skipping them"
kind = "bug"
state = "pending"
created_at = "2026-11-11T22:45:26.330Z"
updated_at = "2026-11-11T22:45:43.395173Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "mm4z"
+++

docs/tickets/open/migration-0005-a-ticket-for-every-active-task-aborts-on-dead-mm4z.md

submitted by external:aide

Seen on data-contracts, 2026-11-11 (ticket dc-swuz, worker agent:adopt).

`bridle migrate --only 0005-a-ticket-for-every-active-task --dry-run` listed 7 stub tickets,
for dc-vueh, dc-ne9h, dc-7qe9, dc-p4ef, dc-hwju, dc-xrfv and dc-s5jm. A real run aborted with
"no such task" on dc-vueh: those six rows are dead (`bridle task show` gives not_found, and
`bridle task list` shows only dc-s5jm). The worker made the one live ticket by hand with
`bridle ticket new --from-task dc-s5jm`.

Expected: the migration skips (and reports) task ids the daemon doesn't have, and the dry run
lists only the tasks it would really stub, so the dry run and the real run agree.

## Thread

### note · external:aide · 2026-11-11T22:45:26.332Z
submitted by external:aide
