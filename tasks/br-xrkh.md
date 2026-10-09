+++
id = "br-xrkh"
title = "systemd uninstall, and an owner refusal never crash-loops a launchd or systemd unit after a project moves"
kind = "bug"
state = "planned"
created_at = "2026-10-09T14:10:29.993Z"
updated_at = "2026-10-09T19:30:09.976070Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-09T14:10:40.438796Z"
summary = "Added `bridle systemd uninstall --project P` (removes bridle-P.service, prints disable/daemon-reload; named by --project alone since a moved project is no longer placed here). Owner refusal: OwnerConflict was already a typed error; serve.rs now downcasts it to CliError::OwnerRefused, exit 78, one log line. systemd unit gets RestartPreventExitStatus=78. launchd has no per-code rule, so the plist runs serve via /bin/sh -c mapping 78 to 0 (SuccessfulExit=false then does not restart). Gateway and mail already have uninstall (gateway/mail units cover systemd); nothing added. Older units/plists need install --force. Docs: cli.md, nuc-host.md, CHANGELOG. Survey claims verified from code: no systemd uninstall existed; serve refused with exit 1 which both supervisors restart. Not tested on a real machine/launchd."
ticket = "xrkh"
+++

Ticket: docs/tickets/open/systemd-uninstall-and-an-owner-refusal-never-crash-loops-a-l-xrkh.md (read it; its claims are from a survey and NOT verified on a machine: verify each first and say on the thread what you found). Human priority 2026-10-09 (machine setup, moving a project to the Windows PC).

Do:
1. `bridle systemd uninstall`: mirror `bridle launchd uninstall` (crates/bridle/src/launchd.rs:185 `uninstall`, with its test at :267) in crates/bridle/src/systemd.rs (install is at :151): print/run (follow what launchd uninstall does: it removes the plist; for systemd remove the unit files that `install` writes and print the `systemctl --user disable --now <unit>` and `daemon-reload` commands the user must run, or run them when `install` runs them, whichever install does). Add the clap subcommand next to the install one in crates/bridle/src/cli.rs. Also `bridle gateway uninstall` and `bridle mail uninstall` exist? (mail_install.rs:103 has one; gateway: check) -- only add missing ones for systemd units; say what you found.
2. Owner refusal must be final for the supervisor. Today `bridle serve` refuses to start when the project's owner (state branch owner.toml, crates/bridle-daemon/src/state_branch.rs `fetch_and_check_owner`, called from lib.rs:474) is another machine; it returns an error and exits non-zero, so systemd `Restart=on-failure` (systemd.rs:140) and launchd `KeepAlive/SuccessfulExit=false` (launchd.rs:90) restart it in a loop. Fix: give the owner refusal a distinct exit code (EX_CONFIG 78 is conventional) from crates/bridle/src/serve.rs / main, log one clear line ("this project is owned by <host>; refusing to start; run `bridle serve --take-over` here to take it over, or uninstall this unit"), and in the generated systemd unit add `RestartPreventExitStatus=78`; for launchd there is no per-code rule, so exit 0 for this one refusal (launchd's SuccessfulExit=false then does not restart) -- implement as: serve exits 78 and launchd's plist wrapper is unchanged only if launchd treats it as non-restarting; otherwise make the plist run through a tiny `sh -c 'bridle serve; rc=$?; [ $rc -eq 78 ] && exit 0; exit $rc'`. Choose the simplest that is correct on each OS, and say which on the thread. The owner error needs a typed error (thiserror) so main can map it to the exit code without string matching.
3. Docs: docs/design/cli.md (systemd uninstall; the exit code; the take-over-then-enable order: a unit cannot pass --take-over, so on a new owner run `bridle serve --take-over` by hand once, stop it, then enable the unit) and docs/context/nuc-host.md runbook (the two move traps) if it lacks them; CHANGELOG.md.
4. Tests: unit text of the generated unit contains RestartPreventExitStatus=78; uninstall removes the files (temp dir); the owner refusal maps to exit 78 (error-type test).

Acceptance: just check passes. Model: Sonnet. Migration: units written by older `bridle systemd install` lack RestartPreventExitStatus: `bridle systemd install --force` rewrites them (say so in cli.md and CHANGELOG; the existing NUC unit is re-installed by the orchestrator, not by this task). Out of scope: `serve --take-over` from a unit, automated project transfer.

## Thread

### note · external:advisor/product-manager · 2026-10-09T14:10:40.438Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T14:10:41.211Z
PdM (advisor product-manager): machine setup is the human's priority workstream (2026-10-09 ~10:15 AM ET: "I want that as a priority so that I can use it to set up the new Windows machine and add it to the network"; quoted in hua2/xrkh/kt25). Priority high; roadmap docs/notes/roadmap.md.

### note · agent:xrkh · 2026-10-09T19:30:07.805Z
done: systemd uninstall + owner refusal exit 78 (systemd RestartPreventExitStatus, launchd sh wrapper); just check exit 0, 1395 tests, checked on 8f54a541; tip 1eeeef65 (merge of main, docs/rules only)

### note · agent:xrkh · 2026-10-09T19:30:09.976Z
Verified in code: no systemd uninstall existed; owner refusal exited 1 (restarted by both supervisors). Chose systemd RestartPreventExitStatus=78 and a sh wrapper in the launchd plist (exit 78 -> 0). Gateway/mail uninstall already exist.
