+++
id = "br-2wnn"
title = "Compare the [projects] list (machine, port) across machines and update every copy to match: a command of its own"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T00:19:52.348Z"
updated_at = "2026-10-17T11:37:09.979845Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:w2wnn",
]
branch = "bridle/w2wnn"
commit = "6d4875171be1f4dd13d7d7fc34df669c984edff4"
summary = "New `bridle projects sync [--apply]` (crates/bridle/src/project_sync.rs; no 'project' group existed). Compares [machines] (name->host) and [projects] (machine, port) of ~/.bridle/config.toml here and on every [machines] entry over plain ssh; --apply makes the others match this machine (adds missing, fixes differing, never deletes: 'only on X' kept), editing via toml_edit (new workspace dep, already in the lock) so comments and other tables survive; remote write is temp file then mv. Also warns when a machine's [machine] name is not a key of its [machines] (PdM comment). Exit 1 only if a machine can't be read/written. Registered in project.rs SCOPES and project_resolution_test CLASSES. Docs: cli.md, add-a-machine.md, CHANGELOG. Tests use a fake Files transport. Only reused token_pair's Where type, not Exec (it runs 'bridle' remotely; this needs cat/mv)."
ticket = "2wnn"
+++

docs/tickets/open/compare-the-projects-list-machine-port-across-machines-and-u-2wnn.md

## Thread

### note · external:aide · 2026-10-12T00:25:14.040Z
From the human, via aide (~8:25 PM ET), verbatim: "want to use these commands to set up the remus wsl2 and vesta nuc with new name". I read that as the go for br-eurm (pair records homes, re-mints no-home tokens) and br-2wnn (compare and sync [projects] across machines), both small, so the human can set up remus (v7ug) and the NUC->vesta rename (r2nr) with them instead of by hand. The human waits for both before doing remus and the rename.

### note · agent:pm-1 · 2026-10-12T00:26:08.937Z
Build brief (pm-1). Model: Sonnet. Small; the human said no big changes. Independent of br-eurm (different files), but eurm goes first.

Goal: a command of its own that compares the [projects] list (project, machine, port) in ~/.bridle/config.toml across this machine and every [machines] entry, over SSH the way `bridle token pair` reaches machines (reuse its SSH helper in crates/bridle/src/token_pair.rs), prints the differences, and can update every copy to match.

Shape (decided, keep it simple):
- Name: `bridle project sync` if a `project` command group exists, else `bridle projects sync`; check crates/bridle/src/cli.rs and pick the neighbour that fits; one name, no aliases.
- No flags: read-only compare. Table of project, machine, port per machine with each difference marked (missing on X, port differs). Exit 0 either way.
- `--apply`: this machine's list is the source; add missing entries and fix differing ports on the others, never delete an entry the source lacks (report it as "only on X"). Show what was changed. Idempotent.
- Edit config.toml without losing comments or other tables (it is hand-edited): use toml_edit if the workspace does not already have a round-trip editor; check for an existing config-writing helper first (e.g. what `bridle project add` or machine config writes use) and reuse it.
- Remote edit: write via ssh to a temp file then rename on the remote; do not touch anything else in the remote file.

Docs: docs/design/cli.md, the setup guide where it says to copy [projects] by hand (docs for adding a machine, br-hua2), CHANGELOG.
Acceptance: just check passes; tests with a fake transport (no real SSH): identical lists show no differences, missing/port-differs shown, --apply adds and fixes but never deletes, comments in the target file survive, second apply is a no-op.
Migration: none.
Out of scope: bridle-ui, evy6 (one placement record), rjd5's wider sync (clones, tokens, services), conflict resolution beyond "this machine wins", deleting entries.

### note · external:aide · 2026-10-12T00:39:28.064Z
Related, from the human via aide: the project move in br-53gy must update [projects] on every machine; it should use this command's sync. From the human, via aide (~8:39 PM ET), verbatim: "ok and the config and ports need updated on both as well right? do that". So moving a project also updates ~/.bridle/config.toml [projects] (the project's machine, and its port if it must change on the new machine) on both machines, and every other machine in [machines], as part of the move: no hand edits. Reuse 2wnn's compare-and-sync for the other machines rather than a second way to write the list. The aide's suggestion for where: in take-over, before it exits, so 'project start' on the new machine finds the right port.

### note · external:advisor/product-manager · 2026-10-12T00:42:30.772Z
PdM: the human's list for the move and rename includes machines ("config, projects, tokens, everything, config, machines, ports, projects", ~9:00 PM ET). Sync [machines] (name -> host) as well as [projects], so the NUC->vesta rename and adding remus update every machine's [machines] in the same run; and check each machine's own [machine] name is a key there.

### note · external:advisor/product-manager · 2026-10-12T00:42:44.941Z
Correction: the human said that at ~8:40 PM ET, not ~9:00.

### note · agent:manager-2 · 2026-10-12T00:43:07.122Z
The PdM added a comment (sync [machines] too). Read it with bridle task show br-2wnn; it is part of the brief.

### note · agent:w2wnn · 2026-10-12T02:03:42.390Z
built: 04aa24cc (branch bridle/w2wnn)

### note · agent:w2wnn · 2026-10-12T02:03:42.446Z
done: bridle projects sync [--apply] (compares and syncs [machines] and [projects], PdM comment included); just check exit 0, 1558 tests passed, main merged; 04aa24cc

### note · external:orchestrator · 2026-10-12T02:35:44.665Z
From orchestrator: please land br-2wnn (04aa24cc) now and push: the human wants it in v0.6.3, which I'm cutting right after its CI is green.

### note · external:orchestrator · 2026-10-12T03:13:34.751Z
From orchestrator: daemon restarted (up5r drain cleared). Land br-2wnn (bridle/w2wnn, 04aa24cc) now, ahead of anything else: it goes into v0.6.3, which the human is waiting on. Tell me when it's pushed.

### note · agent:manager-2 · 2026-10-12T03:35:51.633Z
integrated: 6d4875171be1f4dd13d7d7fc34df669c984edff4 (branch bridle/w2wnn)

### note · agent:manager-2 · 2026-10-12T03:36:20.661Z
cleanup: removed agent w2wnn, branch bridle/w2wnn

### note · external:advisor/product-manager · 2026-10-17T11:37:09.979Z
Verified by advisor (product-manager) 2026-10-17 ~8 AM ET: task integrated, landed on origin/main, every item of the ask done (or confirmed done by the human). Resolving.
