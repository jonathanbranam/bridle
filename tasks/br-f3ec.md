+++
id = "br-f3ec"
title = "hw6c 1: state branch records its owner host; serve refuses another host's project; serve --take-over"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T03:06:13.247Z"
updated_at = "2026-09-30T03:41:27.812730Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/host-owner"
commit = "440a4b0d66193b5b0e5bf1c67c00da46acc16f88"
summary = "The state branch now carries owner.toml (host, since; written via StateBranch::claim_owner, no commit when the host is unchanged). serve (state push on) calls fetch_and_check_owner before opening the branch, on every start: fresh clones adopt origin's state (fetch also no longer hard-resets the project checkout when the branch isn't checked out there; it moves the ref) and another host's owner.toml gives an OwnerConflict naming host and since. serve --take-over (Overrides.take_over; bridle_daemon::run now takes take_over) claims it. Origin unreachable or lacking the branch does not block. Host from 'hostname' (Overrides.host for tests). Tests with bare-repo remotes in state_branch.rs; docs storage/cli/daemon and CHANGELOG updated. 795 tests pass."
+++

Ticket: docs/questions/open/one-machine-owns-a-project-hw6c.md (Shape 1; read it, plus docs/design/storage.md state branch and crates/bridle-daemon/src/state_branch.rs, we2r push). Do: the state branch carries one file (e.g. owner.toml: host name, daemon start time) written by the daemon that serves the project. 'bridle serve' fetches origin/bridle/state first; if another host owns it, the daemon refuses to start, saying which host and since when. 'bridle serve --take-over' on the new machine claims it (writes itself as owner, pushed with the normal we2r push) after the old daemon stopped and pushed. Also make the fetch-before-first-flush ordering safe: serve's initial fetch must make the NUC runbook's manual 'git fetch origin bridle/state:bridle/state' unnecessary (state_branch.rs try_fetch only accepts an empty seed; fix so a fresh clone adopts origin's state instead of reporting Diverged). No origin configured / same host / first ever serve must keep working unchanged. Tests with local bare-repo remotes: owned by other host refuses; --take-over succeeds; same host starts; fresh clone adopts origin state. Docs (storage.md, cli.md, daemon.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: tools-only clones (next task), GitHub branch protection.

## Thread

### note · agent:host-owner · 2026-09-30T03:41:14.857Z
done: owner.toml on bridle/state, serve fetches first and refuses another host, serve --take-over, fresh clone adopts origin; just check passes (795 tests); ec5bdef

### note · agent:manager-2 · 2026-09-30T03:41:20.163Z
integrated: 440a4b0d66193b5b0e5bf1c67c00da46acc16f88 (branch bridle/host-owner)

### note · agent:manager-2 · 2026-09-30T03:41:27.812Z
cleanup: removed agent host-owner, branch bridle/host-owner
