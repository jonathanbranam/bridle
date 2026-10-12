+++
id = "br-86qp"
title = "53gy C: project start (systemd --project), token re-pair in take-over (eurm's code), scheduled messages carried over"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T01:04:04.447Z"
updated_at = "2026-10-12T06:57:24.734478Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
    "agent:w86qp",
]
branch = "bridle/w86qp"
commit = "6cf522b52adfa40c20e40f869947183e20692a8b"
summary = "`bridle project start` (crates/bridle/src/project_start.rs): refuses unless this machine owns the project; installs the launchd/systemd unit if missing (existing launchd/systemd install code, systemd with --project), enables and starts the daemon and any installed mail unit (launchd enable + bootstrap-if-unloaded, systemd daemon-reload + enable --now; idempotent), waits for the daemon, then runs the carry-in. --machine runs it over ssh. Decision: tokens and schedules cannot be made in take-over because minting needs a running daemon and take-over never serves; so take-over writes <workspace>/.bridle/moved-in.json and start does the re-pair (token_pair::pair narrowed to the project: no second minting path) and re-adds schedules, deleting the note only when both succeed. Schedules travel on bridle/state: project stop now commits schedules.json (store::active_schedules, read-only, release_project takes the JSON) with the released mark; take-over reads it. A one-shot already past is reported, not added. Stops by an older bridle carry none. Tests: start_units for both OSes (idempotent, no re-bootstrap), failing unit, schedule conversion/carry, moved-in note, systemd install. Docs: cli.md, add-a-machine.md, CHANGELOG. Not done: the pre-stop listing and mail-count prompt were already in br-n4ug."
ticket = "86qp"
parent = "br-53gy"
+++

docs/tickets/open/53gy-c-project-start-systemd-project-token-re-pair-in-take-o-86qp.md

Slice of br-53gy (pm-1's split, agreed by advisor product-manager 2026-10-11 ~9:10 PM ET). The brief is br-53gy's ask and thread, plus br-3pc2's 'What 53gy must add' and the human's move decisions (comment on br-53gy). Order A -> B -> C, behind br-2wnn.

## Thread

### note · agent:pm-1 · 2026-10-12T01:04:50.956Z
Build brief (pm-1). Model: Sonnet. Slice C of br-53gy (A br-ga6s -> B br-n4ug -> C); read br-53gy's thread and br-3pc2 "What 53gy must add". Blocked by br-n4ug.

Do: `bridle project start`: install the unit if missing, enable (boot, restart on failure) and start it; local, `--machine <m>` over SSH. systemd --project support per br-3pc2 (units for a project on Linux). In take-over (slice A): re-pair tokens by calling br-eurm's pair code (no second minting path); carry schedules to the new machine (per the human's decision on br-53gy); count undelivered mail and offer stop/continue (--yes continues). Help text for the whole transfer: 1 stop here, 2 nothing to give up (stop pushes state), 3 take over there, 4 start there.
Docs: cli.md, the add-a-machine guide, CHANGELOG. Acceptance: just check passes; tests with launchd/systemd/SSH seams: start installs when missing and is idempotent, schedules carried, mail count shown, re-pair called.
Migration: none. Out of scope: bridle.db copy (decided no), kt25 (one-command move, built from these), evy6.

### note · agent:w86qp · 2026-10-12T06:52:41.168Z
built: b1c8c78a212693a3ee28b205f4fb1258a96f51e0 (branch bridle/w86qp)

### note · agent:w86qp · 2026-10-12T06:52:41.223Z
done: project start (unit install/enable/start), re-pair + schedule carry-over via start (needs a running daemon; take-over writes moved-in.json), schedules.json committed by stop; just check exit 0, 1589 tests passed; b1c8c78a212693a3ee28b205f4fb1258a96f51e0

### note · agent:manager-2 · 2026-10-12T06:53:21.143Z
br-wayb just landed on main. Please merge main, re-run just check, and message me the sha with exit status and test count.

### note · agent:w86qp · 2026-10-12T06:56:59.726Z
merged main (001a6c5b): just check exit 0, 1590 tests passed; 0675b0f2ac0ed4340c790b95165040141113b207

### note · agent:manager-2 · 2026-10-12T06:57:04.609Z
integrated: 6cf522b52adfa40c20e40f869947183e20692a8b (branch bridle/w86qp)

### note · agent:manager-2 · 2026-10-12T06:57:24.734Z
cleanup: removed agent w86qp, branch bridle/w86qp
