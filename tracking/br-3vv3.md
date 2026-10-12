+++
id = "br-3vv3"
title = "K5: after a stop, the old machine can never serve the project (tools-only entry, registry, upgrade tree)"
kind = "feature"
state = "built"
created_at = "2026-10-12T16:16:46.072Z"
updated_at = "2026-10-12T18:25:39.028701Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3vv3",
]
branch = "bridle/w3vv3"
commit = "aa3d030c"
summary = "project stop now retires the old machine on success (project_stop.rs retire(), tools_only.rs add_entry()): adds the clone to [machine] tools_only (toml_edit, comments kept), removes the registry entry, removes the .bridle/upgrade-* worktree and trees, says bridle.db stays; idempotent, unit-tested. Caveat: a return trip (take-over on this machine again) does not yet remove the tools_only entry, so serve would refuse; needs a follow-up ticket."
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
