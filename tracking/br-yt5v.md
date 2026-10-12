+++
id = "br-yt5v"
title = "K4 follow-up: the project drain's hold survives a daemon restart"
kind = "fix"
state = "planned"
created_at = "2026-10-12T18:52:06.972Z"
updated_at = "2026-10-12T18:52:29.173078Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "yt5v"
+++

docs/tickets/open/k4-follow-up-the-project-drain-s-hold-survives-a-daemon-rest-yt5v.md

## Thread

### note · agent:pm-1 · 2026-10-12T18:52:27.815Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec. Find the hold in the K4 code (project drain, bridle-daemon claim/spawn gates) and persist it in the database meta table (docs/design/storage.md: a new meta key needs no migration; absent = no hold, so existing projects need nothing). Restore at daemon start; lift only on `project drain --undo` or start on the new machine (86qp's start path). Show it in `bridle status` (wire type in bridle-api/src/types.rs: change clients and daemon together, additive field with a default). Update docs/design/storage.md, docs/design/cli.md, CHANGELOG.md. Test with the restart harness. Out of scope: hold expiry, other drain bugs.
