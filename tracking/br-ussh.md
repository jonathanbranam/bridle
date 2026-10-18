+++
id = "br-ussh"
title = "Stages MVP 3: stage-done, approve (signed and traced), changes, restage; ready means prepared"
kind = "feature"
state = "integrated"
created_at = "2026-10-17T21:41:28.904Z"
updated_at = "2026-10-18T03:09:21.650151Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wussh",
]
priority = "high"
priority_at = "2026-10-17T22:04:05.754869Z"
branch = "bridle/wussh"
commit = "fafb82dfd459526284a9c12dd6ed4b797a7b6af4"
summary = """Stages MVP 3. New crates/bridle/src/stage_cmds.rs: ticket stage-done / approve / changes / restage. Marks live in the ticket frontmatter (written, signed, prepared, claimed_by); stages::states now reads the marks, not files (stages.rs; stage-done checks the file or proposal section). Each command adds a thread note (tells watchers); approve records "signed by human [via <caller>] at <time>". Daemon: ticket_links.rs sync opens a pending version 2 row with prepared: true as actor bridle (tested in tasks.rs); ticket ready refuses version 2 until prepared. Version 1 untouched (tested). Caveats: (1) "claimed by human" is a claimed_by frontmatter line, not a row claim, because row claims need state planned and the ticket is pending; the same limit means step 1's ticket new --claim (claim_task on a pending row) likely returns a conflict, not fixed here. (2) approve is not restricted to the human's token; a non-human caller is traced as "human via <principal>". (3) states() signature changed to (stages, &Marks); the quvh worker's ticket next must use it. Docs: stages.md, cli.md, storage.md, CHANGELOG."""
ticket = "ussh"
+++

docs/tickets/open/stages-mvp-3-stage-done-approve-signed-and-traced-changes-re-ussh.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:57.014Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.754Z
priority: normal -> high

### note · agent:wussh · 2026-10-18T02:29:59.731Z
built: dd5a6f8f (branch bridle/wussh)

### note · agent:wussh · 2026-10-18T02:30:04.326Z
done: stage-done/approve/changes/restage + ready-means-prepared (daemon opens prepared v2 tickets as bridle); just check exit 0, 1847 tests passed; dd5a6f8f. Caveats in the summary: 'claimed by human' is a claimed_by frontmatter line (row claims need planned, ticket is pending; step 1's ticket new --claim likely conflicts for the same reason); stages::states signature is now (stages, &Marks) for quvh.

### note · agent:manager-2 · 2026-10-18T03:09:21.650Z
integrated: fafb82dfd459526284a9c12dd6ed4b797a7b6af4 (branch bridle/wussh)
