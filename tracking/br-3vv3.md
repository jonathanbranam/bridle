+++
id = "br-3vv3"
title = "K5: after a stop, the old machine can never serve the project (tools-only entry, registry, upgrade tree)"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T16:16:46.072Z"
updated_at = "2026-10-12T18:30:50.098533Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3vv3",
]
branch = "bridle/w3vv3"
commit = "969f4a27c2547b51cfad6be5bc061617f4bf0c4b"
summary = "project stop retires the old machine (project_stop.rs retire(), tools_only.rs add_entry()): tools_only entry, registry entry, .bridle/upgrade-* trees removed, bridle.db kept; idempotent. project take-over removes the entry again (tools_only::remove_entry) so a return trip serves; unit-tested."
ticket = "3vv3"
+++

docs/tickets/open/k5-after-a-stop-the-old-machine-can-never-serve-the-project-3vv3.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:36.880Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec. Blocked by br-mzy4 (K3) only because both edit `project stop`; start after it merges. Files: crates/bridle/src (project stop), crates/bridle-daemon tools_only.rs and the registry/upgrade worktree code, docs/design/cli.md, CHANGELOG.md. Idempotent: running twice changes nothing. Out of scope: deleting bridle.db (K9 handles the return trip).

### note · agent:w3vv3 · 2026-10-12T18:25:35.175Z
built: aa3d030c (branch bridle/w3vv3)

### note · agent:w3vv3 · 2026-10-12T18:25:35.211Z
done: project stop retires old machine (tools_only entry, registry, upgrade-* tree, db kept, idempotent); just check exit 0, 1619 tests passed, on aa3d030c; main merged after (cd4d43ec, docs-only). Caveat: take-over on a return trip does not remove the tools_only entry, so serve would refuse there: needs a follow-up.

### note · agent:manager-2 · 2026-10-12T18:25:39.028Z
Please fix the return-trip caveat in this ticket rather than leave it: take-over (or project start) on a machine that held the project before must remove P's tools_only entry, so serve works again; add a test (stop, then take-over, then serve accepted). Then merge main, run just check on the final tip, report exit status, test count and that sha.

### note · agent:w3vv3 · 2026-10-12T18:30:44.012Z
done: take-over now removes the tools_only entry (return trip); just check exit 0, 1620 tests passed, on 55a929a1 (main already merged). Test is at helper level (add_entry then remove_entry then serve accepted), not a full take-over run.

### note · agent:manager-2 · 2026-10-12T18:30:50.098Z
integrated: 969f4a27c2547b51cfad6be5bc061617f4bf0c4b (branch bridle/w3vv3)
