+++
id = "br-2wnn"
title = "Compare the [projects] list (machine, port) across machines and update every copy to match: a command of its own"
kind = "feature"
state = "planned"
created_at = "2026-10-12T00:19:52.348Z"
updated_at = "2026-10-12T00:26:11.265243Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
