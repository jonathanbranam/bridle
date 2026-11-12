+++
id = "br-mm4z"
title = "Migration 0005-a-ticket-for-every-active-task aborts on dead task rows instead of skipping them"
kind = "bug"
state = "integrated"
created_at = "2026-11-11T22:45:26.330Z"
updated_at = "2026-11-12T01:17:31.760118Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:mm4z",
]
branch = "bridle/mm4z"
commit = "6e1a4c19e2afe5c5ae0e8258af7e564674aea7bb"
summary = "0005 now checks each planned task with 'bridle task show' (dry run too); not_found/no such task rows are skipped and named in the report summary, others still abort. Logic moved to stub_tickets with an injected exec; test covers live+dead mix, dry and real agree."
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

### note · external:orchestrator · 2026-11-11T22:46:26.253Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-11T22:46:26.902Z
orchestrator: readied (kind bug, one clear fix; rule ready-without-asking). Related to dc-swuz (data-contracts adoption), whose step 1 runs migration 0005.

### note · agent:mm4z · 2026-11-12T01:07:38.033Z
Cause of dead rows (guess, not fixed): the state-branch rows under tracking/ and tasks/ outlive the daemon's db for those ids; the migration reads rows with git while the daemon is a separate store, so they can disagree.

### note · agent:mm4z · 2026-11-12T01:07:38.097Z
built: f87188f4 (branch bridle/mm4z)

### note · agent:mm4z · 2026-11-12T01:07:38.138Z
done: 0005 skips dead tasks (not_found), dry run lists the same live set; just check env -i exit 0, 2000 tests passed; f87188f4

### note · agent:mm4z · 2026-11-12T01:17:01.577Z
done: main (f99c3012) merged into bridle/mm4z; just check under env -i exit 0, 2004 tests passed (6 skipped); sha 875b7862

### note · agent:manager-2 · 2026-11-12T01:17:09.564Z
integrated: 6e1a4c19e2afe5c5ae0e8258af7e564674aea7bb (branch bridle/mm4z)

### note · agent:manager-2 · 2026-11-12T01:17:31.760Z
cleanup: removed agent mm4z, origin branch bridle/mm4z, branch bridle/mm4z
