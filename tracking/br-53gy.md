+++
id = "br-53gy"
title = "Take over is a one-shot command, not 'serve --take-over'; plus stop and start a project's persistent daemon on any machine"
kind = "feature"
state = "integrated"
created_at = "2026-10-11T22:41:02.993Z"
updated_at = "2026-10-17T02:33:29.621496Z"
created_by = "external:aide"
watchers = ["external:aide"]
commit = "6cf522b5"
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

### note · agent:pm-1 · 2026-10-12T00:29:23.663Z
Brief amended (pm-1), folding in the human's four transfer steps. These supersede the matching lines of step 2 above:
- `bridle project stop`: stop the unit and disable it (no restart on boot, no auto-restart), leave the unit file installed. Stopping the daemon pushes its state; there is no separate give-up-ownership step (daemon.md:91-95, cli.md:692-708). It must not leave a launchd/systemd job that restarts it.
- `bridle project take-over`: one-shot, as in step 1, on the new machine.
- `bridle project start`: install the unit if missing, enable it (starts at boot, restarts on failure) and start it. This replaces "if no unit exists, exit non-zero".
- The old machine's unit, if left enabled, exits 78 once ownership moves; `stop` on the old machine first avoids that. Say so in the help text.
- Help text for each command and the transfer section of cli.md say what happens to ownership at each step, in the human's order: 1 stop here, 2 (nothing to give up: stop pushes state), 3 take over there, 4 start there.

### note · external:aide · 2026-10-12T00:34:41.342Z
From the human, via aide (~8:34 PM ET), verbatim: "make bridle safely stop the daemon and push the state and everything. don't make it best effort". Context (aide, checked in code): today stopping pushes the state branch best-effort with a 10 s limit (crates/bridle-daemon/src/lib.rs:1229, state_branch.rs:272 push_on_shutdown) and says nothing if it fails; take-over checks only that the new machine is level with origin (state_branch.rs:918 require_state_synced), not that the old machine pushed everything, so unpushed state would be left behind. Required for 53gy's 'bridle project stop' (step 1 of the human's four): stop the daemon, push the state branch and anything else the move needs (the integration branch's direct commits too), and confirm origin has it; if any push fails, say so plainly and exit non-zero, never report success. Take-over must not proceed on a project whose old owner stopped without a confirmed push.

### note · external:advisor/product-manager · 2026-10-12T00:34:56.917Z
PdM: required, not optional. The human (via aide m-10890, ~8:34 PM ET), verbatim: "make bridle safely stop the daemon and push the state and everything. don't make it best effort". Today shutdown pushes the state branch best-effort, 10 s, silent on failure (bridle-daemon lib.rs:1229, state_branch.rs:272), and take-over checks only that the new machine is level with origin (state_branch.rs:918). Required: the stop command stops the daemon, pushes the state branch and the integration branch's direct commits, confirms origin has them, and on any failure says so plainly and exits non-zero. Take-over refuses a project whose old owner has not confirmed a clean stop. PdM suggestion for how take-over can tell (planner's call, keep it small): the stop writes a 'released' mark in owner.toml as its last pushed commit; take-over requires it (with an explicit --force for a dead machine). That mark is the human's step 2, 'give up ownership here'.

### note · agent:pm-1 · 2026-10-12T00:35:05.764Z
Brief acknowledged (pm-1), per the PdM's last comment (required by the human): `project stop` must confirm the state push reached origin (fail non-zero and leave the unit as it was if the push cannot be confirmed), not best effort; `project take-over` must refuse unless origin's bridle/state shows the old owner's final push (fast-forward clean), saying what to run on the old machine. Tests: stop with a failing push exits non-zero and does not disable; take-over refuses on a diverged or unpushed state and succeeds after. These add to step 1 and step 2 above; the worker should read the PdM's comment for the exact wording.

### note · external:aide · 2026-10-12T00:35:36.646Z
From the human, via aide (~8:35 PM ET), verbatim: "should be a consistent command that stops everything and then pushes the state branch". So: one command, the same on every machine and OS, that stops everything for the project (its agents, the daemon, and its boot/restart service) and then pushes the state branch, confirming origin has it (per the earlier no-best-effort requirement).

### note · external:advisor/product-manager · 2026-10-12T00:35:49.529Z
The human (via aide m-10898, ~8:35 PM ET), verbatim: "should be a consistent command that stops everything and then pushes the state branch". So the stop is one command, the same on every machine and OS (launchd, systemd): it stops the project's agents, the daemon and its boot/restart service, then pushes the state branch and confirms origin has it (no best effort, as above).

### note · agent:pm-1 · 2026-10-12T00:35:57.353Z
Brief acknowledged (pm-1), per the PdM's latest comment: `project stop` is one consistent command that stops, in order, the project's agents, its daemon and its service unit, then confirms the state push. The worker follows the PdM's comment for the exact semantics; where it conflicts with step 2 above, the PdM comment wins. If this makes step 2 larger than a modest diff, land step 1 (take-over) first and comment so stop/start can be split.

### note · external:aide · 2026-10-12T00:36:56.820Z
From the human, via aide (~8:36 PM ET), verbatim: "and the take-over is not starting a server, right? that needs to be just a command to take it over, then start the daemon; remove 'bridle serve --take-over' or make it exit afater take over is complete. don't start a server AND take over". So: take-over is its own command that claims ownership (owner.toml committed and confirmed pushed to origin) and exits, never serving; starting the daemon is a separate step. 'bridle serve --take-over' is removed (or, if kept, exits once the take-over is complete and never serves).

### note · external:advisor/product-manager · 2026-10-12T00:37:10.496Z
The human (via aide m-10905, ~8:36 PM ET), verbatim: "and the take-over is not starting a server, right? that needs to be just a command to take it over, then start the daemon; remove 'bridle serve --take-over' or make it exit afater take over is complete. don't start a server AND take over". So: take-over is its own command; it commits owner.toml, confirms origin has it, and exits; it never serves. Starting the daemon is a separate step. PdM: remove 'serve --take-over' (simpler than keeping a flag that exits), with a clear error pointing at the new command.

### note · external:aide · 2026-10-12T00:39:24.021Z
From the human, via aide (~8:39 PM ET), verbatim: "ok and the config and ports need updated on both as well right? do that". So moving a project also updates ~/.bridle/config.toml [projects] (the project's machine, and its port if it must change on the new machine) on both machines, and every other machine in [machines], as part of the move: no hand edits. Reuse 2wnn's compare-and-sync for the other machines rather than a second way to write the list. The aide's suggestion for where: in take-over, before it exits, so 'project start' on the new machine finds the right port.

### note · external:aide · 2026-10-12T00:39:41.991Z
From the human, via aide (~8:39 PM ET), verbatim: "and on any other connected machines; so, this is a request just recently". Confirms: the move updates [projects] on every connected machine, not just the two in the move, using the sync the human asked for earlier tonight (br-2wnn).

### note · external:advisor/product-manager · 2026-10-12T00:39:42.991Z
The human (via aide m-10912, ~8:39 PM ET), verbatim: "ok and the config and ports need updated on both as well right? do that". So the move updates ~/.bridle/config.toml [projects] (machine, and port if it must change) on both machines and every machine in [machines], no hand edits, by reusing 2wnn's compare-and-sync (no second writer of the list). Where: in take-over, before it exits, so starting the daemon on the new machine finds the right port. Order: 53gy builds after 2wnn lands.

### note · agent:pm-1 · 2026-10-12T00:40:12.101Z
Brief acknowledged (pm-1), final form after the PdM's comments. Planner decisions:
- Order: br-53gy is blocked-by br-2wnn (edge set). It builds after 2wnn lands.
- take-over, before it exits: commits owner.toml, confirms origin has it, then updates [projects] (machine, and port if it must change) on this machine and every [machines] entry by calling 2wnn's compare-and-sync code (no second writer of the list). It never serves.
- Release mark: adopt the PdM's suggestion. `project stop` writes a 'released' mark in owner.toml as its last pushed commit, after confirming origin has all pushes (state branch and the integration branch's direct commits); take-over requires the mark, with an explicit --force for a dead machine. Keep it small.
- If the whole is more than one branch's worth, land in this order: (1) take-over, serve flag removed, released-mark check, [projects] sync; (2) stop/start. Comment on the ticket before splitting.

### note · agent:pm-1 · 2026-10-12T00:40:25.101Z
Correction (pm-1): the released-mark check in take-over depends on `project stop` writing it, so the split order above does not work as written. Land stop (with the mark) and take-over together; split off only `project start`, if the branch is too big.

### note · external:advisor/product-manager · 2026-10-12T00:42:25.491Z
PdM check of the whole move (the human asked to confirm config, projects, tokens, machines, ports all work). Gap found: a project's database (<workspace>/.bridle/bridle.db, storage.md) does not move, and only tickets, edges, claims, open questions and handovers rebuild from the state branch. Left behind on the old machine: every token minted by that project's daemon (local role tokens, visitor tokens other machines hold, peer tokens others forward with), inboxes and message history, the outbox (undelivered mail), scheduled messages, agent and cost history. So after a move every machine's credentials for the project stop working. Options for the human: (1) take-over copies bridle.db from the stopped old machine over SSH (one file after a clean stop; everything moves, tokens stay valid); (2) leave it, and re-run 'bridle token pair' after a move (needs eurm to re-mint tokens the daemon rejects, not only no-home ones); inbox and schedules are lost. PdM recommends (1). Also not in 53gy: retargeting the mail bridge and gateway (kt25). Not an issue for the NUC->vesta rename or remus setup: no project changes machine there, so the database stays put.

### note · external:advisor/product-manager · 2026-10-12T01:00:07.316Z
PdM: the human's decisions on what a move does with the project's database (2026-10-11, to advisor product-manager), verbatim:

> 1. it's ok to lose tokens; we have a command that re-creates them; so, we can lose them, move the project; create them; bridle should do this as part of the take-over somewhere
> 2. undelievered messages - the command to stop/transfer or whatever should check for them and count them and offer to stop or continue
> 3. scheduled messages need to be enumerated; a simple command should transfer them to the new machine; i assume this is only scheduled messages to roles in that project
> 4. agent turns and handovers are lost; this is ok
> 5. event history is interesting; can we query it between machines? Or communicate? Seems like an ok loss
> 6. usage history wouldn't be lost, but just not transfered I guess
>
> I htink the main thing here is, when moving a project
>
> 1. check and list what will be lost and explain it before committing
> 2. move anything that can be moved (scheduled messages would be good here)
> 3. re-create things that can't be moved - tokens

So the database is NOT copied (supersedes the PdM's copy recommendation). The move:
1. Before committing: lists what stays behind and why (counts: undelivered outbox mail, unread inbox messages, agents and sessions, events, usage), and asks to continue or stop. Undelivered mail is counted and the human chooses stop (let it drain) or continue.
2. Moves what can move: scheduled messages for that project's principals (enumerated, recreated on the new daemon). Tickets, task rows, edges, claims, open questions, queue and handovers already move on the state branch (rebuild); handovers are NOT lost (correcting point 4).
3. Recreates what can't: tokens, as part of take-over (re-run pair: eurm), and config everywhere (2wnn).
Events and usage stay in the old machine's bridle.db, which the move leaves in place (not deleted), so they can still be read there; querying across machines is out of scope.

### note · agent:pm-1 · 2026-10-12T01:00:18.099Z
Brief acknowledged (pm-1): the PdM's latest comment (the human's decisions on a move: no bridle.db copy; list what is lost and confirm; carry schedules; re-pair tokens in take-over; count undelivered mail and offer stop/continue) is part of this brief. Where it adds to or conflicts with my comments above, the PdM's comment wins. Planner notes: re-pair means calling the code from br-eurm (token pair with homes), not a second minting path; the loss list and mail count belong to take-over/stop output and need a non-interactive flag (for example --yes) so scripts and agents can run them. If the total is more than one branch, split off `project start` first, then the schedule carry-over, and comment before doing so.

### note · agent:pm-1 · 2026-10-12T01:02:52.055Z
Brief acknowledged (pm-1): the section "What 53gy must add" in br-3pc2's findings (confirmed pushes, pre-stop listing, unload old units, owner.toml machine identity, systemd --project, schema guard) is part of this brief; the worker reads it with `bridle ticket show br-3pc2`.

Size: with the human's move decisions, the released mark, the [projects] sync and now these, 53gy is past one branch. Slices, each merging independently, in order (the manager claims them as they are made; I will make child tickets if the PdM agrees): (A) take-over one-shot + serve flag removed + owner.toml machine identity + [projects] sync (needs 2wnn); (B) stop with confirmed pushes, released mark, pre-stop listing, unload old units, schema guard, take-over requiring the mark (touches the same files as A: after A); (C) start, systemd --project, token re-pair in take-over, schedule carry-over. The worker lands A first and comments; do not attempt all of it in one branch.

### note · external:advisor/product-manager · 2026-10-12T01:04:04.377Z
split off br-ga6s: 53gy A: one-shot take-over (serve --take-over removed), owner.toml machine identity, [projects] sync via 2wnn

### note · external:advisor/product-manager · 2026-10-12T01:04:04.403Z
split off br-n4ug: 53gy B: project stop with confirmed pushes and released mark, pre-stop listing, old units unloaded, schema guard; take-over requires the mark

### note · external:advisor/product-manager · 2026-10-12T01:04:04.447Z
split off br-86qp: 53gy C: project start (systemd --project), token re-pair in take-over (eurm's code), scheduled messages carried over

### note · agent:manager-2 · 2026-10-17T02:33:29.621Z
integrated: 6cf522b5
