+++
id = "br-53gy"
title = "Take over is a one-shot command, not 'serve --take-over'; plus stop and start a project's persistent daemon on any machine"
kind = "feature"
state = "planned"
created_at = "2026-10-11T22:41:02.993Z"
updated_at = "2026-10-12T00:29:03.231393Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "53gy"
+++

docs/tickets/open/take-over-is-a-one-shot-command-not-serve-take-over-plus-sto-53gy.md

## Thread

### note · external:aide · 2026-10-12T00:27:24.998Z
From the human, via aide (~8:27 PM ET), verbatim: "the serve --take-over is bogus too and needs to be fixed as well. is tehre a ticket explaining that?" I read that as the go for br-53gy, alongside eurm and 2wnn, so the NUC->vesta rename (r2nr step 4, a take-over per NUC project) doesn't need the serve --take-over dance. Keep it small (the human: no big changes); if it must be sliced, the one-shot take-over comes first, stop/start after.

### note · agent:pm-1 · 2026-10-12T00:28:03.643Z
Build brief (pm-1). Model: Sonnet. The human: no big changes. One branch, two steps; if step 2 grows past a modest diff, land step 1 and comment so the rest is split off.

Step 1, the one-shot take-over (the human's priority; the NUC->vesta rename, r2nr step 4, waits on it):
- A command (name: the neighbour that fits in crates/bridle/src/cli.rs, for example `bridle project take-over`, with the usual --project) that moves owner.toml on bridle/state to this machine (the hw6c mechanism now behind `bridle serve --take-over`: see crates/bridle/src/serve.rs, crates/bridle-daemon/src/state_branch.rs, tools_only.rs) and exits. It must work with no daemon running for the project, and not start one.
- `bridle serve --take-over` is removed, not kept as an alias (one name per action); `serve` refused for a non-owner keeps its error but names the new command. Update every message and doc that says `serve --take-over` (grep docs/ and crates/ for it, including the one-pusher rule's text in workflow/base/rules and the pre-push hook message).
- Tests: take-over moves the owner and exits; a second run is a no-op; serve as non-owner still refuses and names the command. Existing owner_push_test.rs and restart_test.rs cases that use the flag move to the command.

Step 2, stop and start a project's persistent daemon:
- `bridle project stop` and `bridle project start` (same group as step 1), local by default, `--machine <m>` runs it on that machine over SSH the way `bridle token pair` does. They drive the existing launchd/systemd units (crates/bridle/src/launchd.rs, systemd.rs, the same unit names install/uninstall create): stop = unload/stop without uninstalling; start = load/start. If no unit exists, say so and exit non-zero, do not create one.
- Tests with the existing launchd/systemd test seams (no real launchctl/systemctl, no real SSH).

Docs: docs/design/cli.md, daemon.md (ownership), the add-a-machine guide, the r2nr ticket's steps if they quote the old flag (comment on r2nr rather than editing the human's ticket), CHANGELOG.
Acceptance: just check passes.
Migration: units written by older bridle that run `serve --take-over`: check `bridle service`/install output and the units on disk; if any unit or doc template contains the flag, `bridle` install regenerates it, and say in the done note how existing units on dalek and the NUC are refreshed (a rerun of the install command). Trial branches/projects' files unaffected.
Out of scope: kt25 (move in one command), evy6 (placement record), the machine daemon (kuw2), changing how ownership is stored.

### note · external:aide · 2026-10-12T00:28:50.940Z
From the human, via aide (~8:28 PM ET), verbatim: "I'm not sure exactly how it works RN - we have daemons auto-restarting. To transfer a project the steps are 1. stop boot restarting daemon here 2. ?? give up ownership here? 3. take over on the new project 4. start the boot and restarting daemon there". The aide's answer from the docs (daemon.md:91-95, cli.md:692-708): today there is no give-up step; stopping the old daemon pushes its state, and the new machine's take-over rewrites owner.toml once origin fast-forwards cleanly; the old machine's unit then exits 78 and stays stopped, and should be uninstalled. Build 53gy's commands to match the human's four steps (stop and disable here, take over there, install and start there), and say in their help and in cli.md what happens to ownership at each step.

### note · external:advisor/product-manager · 2026-10-12T00:29:03.231Z
PdM: the human's four steps (via aide m-10881, ~8:28 PM ET), verbatim: "To transfer a project the steps are 1. stop boot restarting daemon here 2. ?? give up ownership here? 3. take over on the new project 4. start the boot and restarting daemon there". Today there is no give-up step: stopping the old daemon pushes its state; the new machine's take-over rewrites owner.toml once origin fast-forwards; the old unit then exits 78 and should be uninstalled (daemon.md:91-95, cli.md:692-708). Build the commands to match the four steps (stop and disable here; take over there, one-shot; install and start there), and say in their help and in cli.md what happens to ownership at each step.
