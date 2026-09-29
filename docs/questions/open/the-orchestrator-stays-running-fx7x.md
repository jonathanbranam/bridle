---
id: fx7x
title: The orchestrator stays running (a watcher for the watchman)
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-humans-to-do-list-and-restart-checklist-ex9q, where-the-single-orchestrator-lives-hj4g]
---

## The ask

The human, verbatim (2026-09-29, via the advisor), on finding the orchestrator gone from
Remote Control:

> Also, I think we need a watcher for the watchman, and we need a cron or system process to be
> sure the orchestrator is running. Having to restart it isn't scalable.

## What happened (advisor, 2026-09-29)

- The orchestrator (`scripts/claude-orchestrator`, session `61113e23-...`, Remote Control name
  `bridle-orch`) was gone: no `claude` process for it at 08:41 ET.
- Its transcript's last real turn is 12:32 UTC (08:32 ET): it sent m-1718/m-1719 to the
  managers (pausing two builds for the incident/open-requests design) and replied to the human.
  The transcript ends at 12:36 UTC with its background task `b71x7i5bz` reported `killed`. No
  error in the transcript and no crash report in `~/Library/Logs/DiagnosticReports`.
- The cause is unknown. Nothing noticed it had gone; the human found out when they went to
  send it a message.
- It's the orchestrator running outside bridle (an `external` principal in a terminal), so the
  daemon's `resume_on_restart` doesn't cover it. The advisor is in the same position.

## Shape (for the orchestrator to design; KISS)

- Something outside the session (launchd or cron, like the launchd plan for the daemons in
  `docs/context/launchd-restart-plan.md`) checks the orchestrator is running and, if not,
  restarts it, or at least tells the human.
- Find out why it exited this time, if the logs can say.
