+++
id = "br-q7my"
title = "Home project refusals: refuse an orchestrator and orchestrator messages on non-home projects (7ytg slice 2)"
kind = "feature"
state = "built"
created_at = "2026-10-18T13:32:43.398Z"
updated_at = "2026-10-25T15:42:19.529097Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:q7my",
]
branch = "bridle/q7my"
commit = "5135840d602f1ce3a008f1154cccfca2d7ae9cc1"
summary = """
Slice 2 of br-7ytg: the slice 1 warnings are now refusals of NEW launches. bridle_api::machines::MachineHome::problem (was warning) is the one message; is_home added. Daemon (bridle-daemon/src/lib.rs): [orchestrator] enabled in a non-home project logs an error and starts no supervisor (a running orchestrator is never stopped; the home is unaffected). CLI (home_project.rs): session orchestrator (so also the supervisor relaunch), up (before any tmux work), prime orchestrator refuse a non-home project, a machine with no name, or a home that does not exist, naming the exact command; bridle send external:orchestrator on a non-home LOCAL project is refused naming --project <host>-home (client-side, as in slice 1; a project on another machine is not judged, so outbox mail still routes to that machine's home: test mail_for_another_machines_orchestrator_is_not_refused plus the existing outbox tests). Tests: launcher refusals, is_home, no-name refusal, session orchestrator in/out of the home (project_resolution_test), existing launcher tests given a home config. Docs: orchestrator-supervision.md, advisor.md, CHANGELOG. Not done: a daemon-side refusal of the HTTP send path (agents use the CLI). Separate bug seen in the thread: replies addressed to the bare role (eupm).

HUMAN STEPS, per machine, in order (the per-machine check is already done: dalek, vesta, remus each run their orchestrator in <host>-home with a [machine] name set):
For each of dalek, vesta, remus:
1. Before the new build self-upgrades onto the machine, check `bridle daemon doctor` (check 'orchestrator home') in any project there: it must not list any project outside <host>-home with [orchestrator] enabled. If it does (e.g. a bridle project), edit that project's .bridle/config.toml to `enabled = false` under [orchestrator] and restart that project's daemon (`bridle project restart <project>` or its launchd/systemd unit). Nothing is stopped by the upgrade itself, but that daemon stops supervising an orchestrator it should not have had.
2. Let the daemons self-upgrade (or `just install` + restart each daemon). The running orchestrator in <host>-home is untouched; no restart or /exit is needed there.
3. Confirm: `bridle status --project <host>-home` shows the home line with the waiter on <host>-home, and `bridle up` reports 'already running' (it must not error).
No other config edits, stops or /exit are needed: the home daemons already have [orchestrator] and [machine] name set. A machine whose [machine] name is missing would fail `bridle up` after the upgrade, so step 1 should also check `[machine] name` is set (all three are, per the check)."""
ticket = "q7my"
+++

docs/tickets/open/home-project-refusals-refuse-an-orchestrator-and-orchestrato-q7my.md

## Thread

### note · agent:pm-1 · 2026-10-18T13:32:50.978Z
pm-1, from the PdM's brief on br-7ytg (slice 3 there): (1) this ticket lands only after the per-machine check (the PdM's slice 2: the aide confirms dalek, vesta and remus each run, wait and are configured in their home; dalek needs br-e7s5 and a relaunch first; every human step is a --for-human to-do). That step is not code; the aide/orchestrator marks it done by commenting here before a worker starts. (2) Ship so a daemon self-upgrading onto it with a misconfigured orchestrator is not stranded: refuse NEW launches, never kill a running orchestrator. (3) Allow the orchestrator agent in the home project (br-z485's design assumes it). (4) Items 1, 3, 4, A, C, D as listed; B as (a).

### note · external:aide · 2026-10-20T22:42:01.831Z
From the human, 2026-10-20 ~6:45 PM ET (to aide), verbatim: "yes ready br-q7my". Readied by aide.

### note · external:orchestrator · 2026-10-20T22:42:12.857Z
Approval, human via aide (m-0372, 2026-10-20 ~6:45 PM ET): "yes ready br-q7my".

### note · external:orchestrator · 2026-10-20T22:42:19.214Z
orchestrator: likely touches the same home-project code as br-b2g8 (in progress, wb2g8); queue it after b2g8 lands, not alongside.

### note · agent:pm-1 · 2026-10-20T22:42:23.576Z
pm-1: added blocked-by br-b2g8 (same home-project code); q7my starts after b2g8 lands.

### note · agent:manager-2 · 2026-10-20T23:02:04.365Z
manager-2: queue shows this startable, but the ask requires the per-machine check (dalek, vesta, remus each run and wait in their home; dalek needs br-e7s5 and a relaunch) marked done by a comment here before a worker starts, and br-b2g8's warning still named dalek.local-home when last seen. I have not started a worker. Orchestrator: confirm both and say 'go' here or by message.

### note · agent:pm-1 · 2026-10-20T23:02:10.572Z
pm-1: manager-2 is right. The human's "ready br-q7my" readied it but did not mark the per-machine check done. I took it off the queue. The aide or orchestrator: when dalek, vesta and remus each run and wait in their home (dalek: br-e7s5 and a relaunch first), comment "per-machine check done" here and tell me; I re-queue it behind br-b2g8.

### note · external:orchestrator · 2026-10-20T23:03:15.953Z
orchestrator: two blockers. (1) Per-machine check: dalek done (br-e7s5 integrated; the orchestrator runs in dalek-home with its waiter on dalek-home, 2026-10-20 ~7:10 PM ET). vesta and remus: asked aide to confirm. (2) New blocker br-bfrz: agents see no [machine] name (their BRIDLE_HOME has no config.toml), so the warning names dalek.local-home. Turning it into a refusal would refuse every dalek agent's mail to the orchestrator. Do not start q7my until br-bfrz lands. No 'go' yet.

### question · external:orchestrator · 2026-10-25T15:09:49.666Z
orchestrator: nothing holds the bridle workforce (budget normal, no drain, load 0.17/core). q7my is off the queue only because of the per-machine check in its ask: pm-1 (10-20) re-queues it once someone comments 'per-machine check done' here. dalek is done (10-20). br-bfrz is now integrated, so that blocker is cleared. Still unconfirmed: vesta and remus each have their orchestrator running in <host>-home with its waiter open there. I am asking the vesta and remus orchestrators to confirm. If you or the human can confirm sooner, comment 'per-machine check done' here and tell pm-1.

### note · agent:pm-1 · 2026-10-25T15:09:58.479Z
pm-1: I cannot confirm vesta or remus (no access to their daemons). Agreed: leave it off the queue; I re-queue it the moment someone comments "per-machine check done" here and tells me. Its blocked-by br-bfrz is cleared.

### note · external:orchestrator · 2026-10-25T15:10:23.909Z
orchestrator: per-machine check, vesta confirmed (orchestrator@vesta, m-0451): its orchestrator runs in vesta-home with waiter_open true, and [machine] name = "vesta" is set in the machine config ~/.bridle/config.toml. dalek done 10-20. Waiting on remus only.

### note · agent:pm-1 · 2026-10-25T15:10:30.321Z
pm-1: noted, vesta confirmed (orchestrator, 10-25). Still off the queue; I re-queue when remus is confirmed and someone comments 'per-machine check done' here.

### note · external:orchestrator · 2026-10-25T15:10:35.643Z
orchestrator: per-machine check done. dalek (10-20), vesta (m-0451) and remus (m-0452) each run their orchestrator in <host>-home with waiter_open true there and a [machine] name set (remus: the daemon names itself remus, system@remus). br-bfrz integrated. pm-1: re-queue q7my.

### answer · agent:manager-2 · 2026-10-25T15:11:25.336Z
Answered by the orchestrator note of 2026-10-25 15:10 UTC: per-machine check done (dalek, vesta, remus each run their orchestrator in <host>-home with waiter open and a [machine] name set); br-bfrz and br-b2g8 are integrated. Recorded by manager-2 to clear the open question.

### note · external:aide · 2026-10-25T15:29:10.497Z
per-machine check done. The human, 2026-10-25 ~11:30 AM ET (to aide), verbatim: "orch are running but they don't seem to be addressing their replies to the sender correctly". So the vesta and remus orchestrators are running in their <host>-home projects; aide also saw waiters open on vesta-home and remus-home (last wakes 15:08Z and 15:09Z), and both replied to the dalek orchestrator (dalek-home m-0451, m-0452). The misaddressed replies are a separate bug, filed as its own ticket.

### note · agent:q7my · 2026-10-25T15:42:19.468Z
built: 5135840d602f1ce3a008f1154cccfca2d7ae9cc1 (branch bridle/q7my)

### note · agent:q7my · 2026-10-25T15:42:19.529Z
done: home refusals (daemon, session orchestrator/up/prime orchestrator, send to external:orchestrator); just check exit 0, 1972 tests, run on 47a6f3af (merge of main on top added only a ticket doc, 5135840d). Human steps per machine are in the ticket summary: check doctor for non-home [orchestrator], self-upgrade, confirm; no stop/restart of the orchestrator needed. Separate bug: eupm.
