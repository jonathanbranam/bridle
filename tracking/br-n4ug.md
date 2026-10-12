+++
id = "br-n4ug"
title = "53gy B: project stop with confirmed pushes and released mark, pre-stop listing, old units unloaded, schema guard; take-over requires the mark"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T01:04:04.403Z"
updated_at = "2026-10-12T06:16:39.390963Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
    "agent:wn4ug",
]
branch = "bridle/wn4ug"
commit = "075e5e9565cdcd51ab851de8d03c99360683e876"
summary = "Added 'bridle project stop' (project_stop.rs): lists what stays behind (undelivered outbox mail, unread, schedules, unpushed branches, dirty worktrees), confirms (--yes, --list, --machine over ssh via the token-pair Runner), stops daemon+agents, release_project() pushes bridle/state then the integration branch with confirmed-on-origin checks, pushes a 'released' mark in owner.toml last, and only then unloads+disables daemon and mail units (launchctl bootout+disable / systemctl disable --now; files stay). Failure exits non-zero, no mark, units untouched. take-over requires the mark (--force for a dead machine); a restarted owner clears it. Schema guard: Store migrate refuses user_version newer than the binary (SchemaTooNew). Docs: cli.md, daemon.md, CHANGELOG. Caveat: unit unload is last by design, so if it fails the pushes and mark already exist (stop is rerunnable); real launchctl/systemctl untested, only via Ctl seam. Mail unit unloaded too. Not done (slice C): start, re-pair, schedule carry-over."
ticket = "n4ug"
parent = "br-53gy"
+++

docs/tickets/open/53gy-b-project-stop-with-confirmed-pushes-and-released-mark-n4ug.md

Slice of br-53gy (pm-1's split, agreed by advisor product-manager 2026-10-11 ~9:10 PM ET). The brief is br-53gy's ask and thread, plus br-3pc2's 'What 53gy must add' and the human's move decisions (comment on br-53gy). Order A -> B -> C, behind br-2wnn.

## Thread

### note · agent:pm-1 · 2026-10-12T01:04:43.500Z
Build brief (pm-1). Model: Sonnet. Slice B of br-53gy (A br-ga6s -> B -> C br-86qp); read br-53gy's thread and br-3pc2 "What 53gy must add". Blocked by br-ga6s (same files).

Do: `bridle project stop`, one command, the same on every machine and OS (launchd, systemd): stops the project's agents, its daemon and its boot/restart unit (unload, leave the file installed), then pushes the state branch and the integration branch's direct commits and confirms origin has them. Any failure: say so plainly, exit non-zero, leave the unit as it was, never report success. Pre-stop listing of what is lost or left behind (per the human's decisions on br-53gy), with a confirm and a --yes flag. Write a 'released' mark in owner.toml as stop's last pushed commit; take-over (slice A's seam) then requires it, with --force for a dead machine. Unload the old machine's units per br-3pc2. Schema guard per br-3pc2. `--machine <m>` over SSH like token pair.
Docs: cli.md transfer section (what happens to ownership at each step, in the human's order), daemon.md, CHANGELOG. Acceptance: just check passes; tests: failed push exits non-zero and changes nothing, mark written after confirmed push, take-over refuses without it and --force overrides. Use existing launchd/systemd/SSH seams, no real ones.
Migration: none to files; owner.toml gains the mark field, absent means not released (take-over refuses, --force). Out of scope: start, re-pair, schedules, bridle.db.

### note · agent:wn4ug · 2026-10-12T06:16:15.211Z
built: 3701f07b (branch bridle/wn4ug)

### note · agent:wn4ug · 2026-10-12T06:16:15.267Z
done: project stop with confirmed pushes + released mark, units unloaded, take-over requires mark (--force), schema guard; just check exit 0, 1583 tests, 3701f07b (main merged)

### note · agent:manager-2 · 2026-10-12T06:16:21.095Z
integrated: 075e5e9565cdcd51ab851de8d03c99360683e876 (branch bridle/wn4ug)

### note · agent:manager-2 · 2026-10-12T06:16:39.390Z
cleanup: removed agent wn4ug, branch bridle/wn4ug
