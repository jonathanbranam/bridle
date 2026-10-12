+++
id = "br-yt5v"
title = "K4 follow-up: the project drain's hold survives a daemon restart"
kind = "fix"
state = "integrated"
created_at = "2026-10-12T18:52:06.972Z"
updated_at = "2026-10-12T19:19:09.243247Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wyt5v",
]
branch = "bridle/wyt5v"
commit = "557f9a55568d4b8cd4bfd57d0ac49bcfaf9ddf91"
summary = "The project drain's hold is stored in the meta table (key project.held, written by POST /v1/project/hold) and restored in bridle-daemon start; only drain --undo lifts it. Status already showed held. Test: project_hold_test restart case. Docs: cli.md, storage.md, CHANGELOG."
ticket = "yt5v"
+++

docs/tickets/open/k4-follow-up-the-project-drain-s-hold-survives-a-daemon-rest-yt5v.md

## Thread

### note · agent:pm-1 · 2026-10-12T18:52:27.815Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec. Find the hold in the K4 code (project drain, bridle-daemon claim/spawn gates) and persist it in the database meta table (docs/design/storage.md: a new meta key needs no migration; absent = no hold, so existing projects need nothing). Restore at daemon start; lift only on `project drain --undo` or start on the new machine (86qp's start path). Show it in `bridle status` (wire type in bridle-api/src/types.rs: change clients and daemon together, additive field with a default). Update docs/design/storage.md, docs/design/cli.md, CHANGELOG.md. Test with the restart harness. Out of scope: hold expiry, other drain bugs.

### note · agent:wyt5v · 2026-10-12T19:11:34.742Z
wyt5v: implemented (meta key project.held, restore in lib.rs start, test in project_hold_test.rs, docs+CHANGELOG); main merged; just check running, will commit and mark built when green.

### note · agent:wyt5v · 2026-10-12T19:19:00.867Z
built: 839328c6 (branch bridle/wyt5v)

### note · agent:wyt5v · 2026-10-12T19:19:00.915Z
done: drain hold persisted in meta project.held, restored at start; just check exit 0, 1621 tests passed, main already merged; 839328c6

### note · agent:manager-2 · 2026-10-12T19:19:06.324Z
integrated: 557f9a55568d4b8cd4bfd57d0ac49bcfaf9ddf91 (branch bridle/wyt5v)

### note · agent:manager-2 · 2026-10-12T19:19:09.243Z
cleanup: removed nothing; kept agent wyt5v (background job pid 44061)
