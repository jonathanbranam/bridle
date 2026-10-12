+++
id = "br-d6h6"
title = "K9: return trip: back up the old database in the workspace and carry the outbox counter"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T16:16:46.256Z"
updated_at = "2026-10-12T18:07:51.121148Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wd6h6",
]
branch = "bridle/wd6h6"
commit = "da60c5b12766d26d459b63f3778a4815d4a9f3bc"
summary = "Return trip (K9). `project stop` reads the outbox AUTOINCREMENT high-water mark (Store::outbox_high_water) and commits it as `outbox-seq` beside `schedules.json` with the released mark (state_branch::release_project takes a new outbox_seq arg). `project take-over` (on Claimed) calls project_start::fresh_database: moves any old bridle.db (+ -wal/-shm) to <workspace>/.bridle/db-backups/<date>-<sha>/ (never overwrites; not in git) and seeds the fresh db's outbox counter (Store::outbox_seed_counter) to max(old db's counter, outbox-seq). Decision: seed from BOTH because on OLD->NEW->OLD the stop file holds NEW's counter, not OLD's own last id. An old release without `outbox-seq` still works (seeds from old db only). Tests: project_start::tests (return trip: backup kept, next id o-0004 after 3 sent; first visit seeds from the stop file). Docs: cli.md, storage.md, CHANGELOG. Caveat: the test is at function level with temp dirs, not an end-to-end two-home run; ids are what receivers dedup on, so the id assertion is the receiver check."
ticket = "d6h6"
+++

docs/tickets/open/k9-return-trip-back-up-the-old-database-in-the-workspace-and-d6h6.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:40.469Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec; read `project stop` (n4ug) for the `released` commit and `project start`/take-over (86qp) for where the fresh database is built, and the outbox id counter in the store (docs/design/storage.md). Test the OLD -> NEW -> OLD round trip with two test homes. Files also: docs/design/storage.md, docs/design/cli.md, CHANGELOG.md. If this changes what is kept in a project's files (released commit format), say in the change how existing projects are handled (an old released commit without the mark must still work). Out of scope: deleting databases, git storage of backups.

### note · agent:wd6h6 · 2026-10-12T17:58:01.171Z
built: 90009ceb61c44780a0b97a86b0314d82dd4e55ff (branch bridle/wd6h6)

### note · agent:wd6h6 · 2026-10-12T17:58:01.210Z
done: return trip backs up old db to db-backups/ and carries outbox counter; just check exit 0, 1615 tests passed, main merged (incl. mzy4); 90009ceb61c44780a0b97a86b0314d82dd4e55ff

### note · agent:manager-2 · 2026-10-12T17:58:05.403Z
main moved (br-kefh landed). Merge main, re-run just check, report exit status, test count and sha.

### note · agent:wd6h6 · 2026-10-12T18:07:00.949Z
done: merged main (br-kefh); just check exit 0, 1617 tests passed; d7aed5c0cddfb4dba43483471e2b34b11363d67f

### note · agent:manager-2 · 2026-10-12T18:07:07.121Z
integrated: da60c5b12766d26d459b63f3778a4815d4a9f3bc (branch bridle/wd6h6)

### note · agent:manager-2 · 2026-10-12T18:07:36.709Z
cleanup: removed agent wd6h6, branch bridle/wd6h6

### note · external:advisor/product-manager · 2026-10-12T18:07:51.121Z
advisor (product-manager): verified on origin/main (da60c5b1): stop commits the outbox high-water mark as outbox-seq with the released mark; take-over moves a left-over database (with -wal/-shm) to <workspace>/.bridle/db-backups/<date>-<sha>/ and seeds the fresh one above the mark. Matches the human's Q4 answers. Resolving.
