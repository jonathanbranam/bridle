+++
id = "br-ghys"
title = "A project's aide messages an orchestrator that isn't watching that project's daemon; the roles don't say the orchestrator is per machine"
kind = "bug"
state = "integrated"
created_at = "2026-10-08T01:05:40.485Z"
updated_at = "2026-10-08T05:07:58.201665Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/roleclarify"
commit = "40322d67046e930187da063ebdc2cf95c2191679"
summary = """
Clarified the aide and orchestrator role texts (workflow/base/roles/) for the ghys bug: the aide is one per project; the orchestrator is one per machine, running in one project's session and watching other projects' daemons only while it runs a waiter for each.

aide.md: first screen states per-project aide and per-machine orchestrator, and that a message to external:orchestrator is accepted even when nobody listens. New "Relaying to the orchestrator" step: check `bridle status --json` `waiter_open` before relying on the orchestrator; if false, send and tell the human at once; if true but no reply in 30 minutes, tell the human. `waiter_open` counts any principal's open waiter on that daemon, so it is not proof the orchestrator is the one waiting (a code gap, not fixed here). Also the ui-wdp3 rule: before saying work wasn't acted on, check `bridle task list`, `git log` and the orchestrator's message trail.

orchestrator.md: the one-per-machine statement in the opening paragraph; a first-after-restart check in "At every start"; the per-project waiter duty restated at the top of Watch as a duty nothing enforces, with the 15-minute incident (waiter_grace) and `bridle status --project <name>`. Also replaced a non-ASCII >= sign that was already in the file.

Migration: role text reaches projects only through `bridle workflow sync`; no project files change. CHANGELOG Unreleased/Changed has a line telling people to run it.

Not done (ticket ask 2, a code/design choice for the human): making a missed route visible to the aide. docs/design/roles-and-config.md named in the brief does not exist; no design doc states the machine-vs-project rule.

Check: just check exit 0 on commit a6786314 (1321 run, 1321 passed). The first run had one flaky failure, upgrade_test a_long_drain_wakes_the_orchestrator_once; it passed alone on rerun and the full rerun was green. Not related to this text-only change."""
ticket = "ghys"
+++

Ticket: docs/tickets/open/a-project-s-aide-messages-an-orchestrator-that-isn-t-watchin-ghys.md (read it all: what happened, what the rules say today). The human, verbatim: "the bridle-ui aide has been messaging a non-existent bridle-ui orchestrator; file this as a bug (not incident) and investigate the role rules; this again is a project vs. machine level issue; but the roles need to be clarified."
This task = the ticket's ask 1 only: clarify the role files. Docs/text only, no code.
Files: workflow/base/roles/aide.md, workflow/base/roles/orchestrator.md (the "Watch" section), and wherever the machine-vs-project roles are summarized (grep `one orchestrator per box`, `per project` in workflow/base/ and docs/design/roles-and-config.md; ticket ma8e is the human's 2026-09-30 decision: one orchestrator per box).
Write, plainly and briefly:
- Aide: one per project. Orchestrator: one per MACHINE; it lives in one project's session (today bridle's) and watches OTHER projects' daemons only while it runs a waiter for them. So `external:orchestrator` on a project's daemon is accepted even when nobody is listening.
- What the aide does about it: when relaying to the orchestrator, check `bridle status` / the daemon's wake state for an orchestrator waiter on THIS project's daemon (find the exact command or field that shows a waiter; if none exists, say so and use: send, then if no reply or read mark within a stated time, tell the human in the aide's own session). Do not invent a command: name only ones that exist (verify with --help).
- What the orchestrator does: its existing duty (one waiter per daemon it holds a token for, listed by project) restated at the top of the Watch section as a duty that nothing enforces, and the first thing to check after a restart or handover.
Do not change the rule itself; clarify it. Keep ASCII. Keep each role file's density.
Follow-up for the orchestrator/human (NOT this task): ask 2 (make a missed route visible: the daemon's "no waiter for 15 minutes" note to the project's aide, or a send-time warning) is code and a design choice; ask the human whether to file it, and note kuw2 (machine daemon) may remove the need.
Migration: role text reaches projects through `bridle workflow sync`; no project files change. Say so in the done note.
Acceptance: just check passes; the two role files state per-project aide and per-machine orchestrator in the first screen and the aide's check/escalation step. Model: Haiku.

## Thread

### note · external:orchestrator · 2026-10-08T01:06:06.405Z
Readied by orchestrator on the human's go, via aide (~10:10 PM ET): "file this as a bug (not incident) and investigate the role rules; this again is a project vs. machine level issue; but the roles need to be clarified."

### note · external:orchestrator · 2026-10-08T01:06:10.331Z
From orchestrator: br-ghys ready (the human's bug, via aide). Role-rules clarification (aide/orchestrator per machine vs per project); place it after br-grdg, which is the code half of the same symptom. Likely Haiku/Sonnet docs-only.

### note · external:orchestrator · 2026-10-08T02:21:18.485Z
orchestrator: one more aide-role change for this ticket, from bridle-ui incident ui-wdp3 (postmortem in ui-mbhk's summary): before telling the human that work hasn't started or wasn't acted on, the aide checks 'bridle task list', 'git log' and the orchestrator's message trail, not only its inbox, the human's to-dos and pending_tasks. On 2026-10-04 the bridle-ui aide told the human twice that k3qx hadn't been acted on; it had landed in 10 minutes as ui-n6cu.

### note · agent:roleclarify · 2026-10-08T05:07:41.087Z
done: role text clarified on bridle/roleclarify a6786314; just check exit 0 (1321 passed). First run had one flaky upgrade_test failure (a_long_drain_wakes_the_orchestrator_once), green on rerun.

### note · agent:roleclarify · 2026-10-08T05:07:48.077Z
done: role text clarified (aide per project, orchestrator per machine, waiter_open check); a6786314; just check exit 0, 1321 passed; see thread

### note · agent:manager-2 · 2026-10-08T05:07:58.201Z
integrated: 40322d67046e930187da063ebdc2cf95c2191679 (branch bridle/roleclarify)
