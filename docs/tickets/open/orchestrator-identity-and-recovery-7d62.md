---
id: 7d62
title: Orchestrator identity and disaster recovery (the pid file, several orchestrators)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [nuc-recovery-on-boot-4r3k, one-orchestrator-and-advisor-or-one-per-project-ma8e, hold-the-orchestrator-relaunch-8fsx]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> does that pid needing cleaning up? We had an incident where a worker started orchestrators on
> its own during testing and clobbered the pid and the daemon was confused for a long time. We need
> a failure / disaster recovery mode. I think the daemon should also message orchestrator so they
> can determine if the pid is out of date or some other issue has occurred. multiple orch running
> could be a problem

The incident: `docs/tickets/resolved/worker-tests-reach-the-live-orchestrator-k6b3.md` (a worker
ran the launcher; the pid and session files were overwritten; the daemon supervised a dead test
session and told the human the orchestrator was dead, m-2319).

## Today

`$BRIDLE_HOME/orchestrator.pid`, `.session` and `.exits` are **one per machine**, not per project
(orchestrator-supervision.md section 2). Liveness is pid plus start time, so a reboot leaves a dead
pid that is read as "dead, relaunch": that part is fine and needs no cleanup. The problems are:

- anything that runs the launcher overwrites the files; nothing refuses a second orchestrator;
- when the files and reality disagree, the daemon only acts on the files.

## Shape (sketch)

- **One orchestrator per machine** (the human's working rule, 2026-09-30, recorded in ma8e): the
  per-machine files stay. Only one daemon supervises it, the one whose project sets `[orchestrator]
  enabled = true`, as on the laptop today (bridle's; meta-notes and track-web don't set it). On the
  NUC that's one of the projects it owns. Nothing new is needed; at most a warning when two daemons
  on one machine both enable it.
- **One at a time**: the launcher refuses to start when the pid file names a live orchestrator (with an explicit `--replace`), and refuses when run by a bridle agent (k6b3's
  second option).
- **Cross-check, then ask**: the daemon compares the pid file with what it sees of
  `external:orchestrator` (its wake long poll, its API calls). If they disagree (the pid is dead but
  the orchestrator is polling; two sessions polling), it doesn't relaunch. It messages the
  orchestrator to confirm its own pid, and records an incident for the human if that doesn't settle
  it.
- **Recovery mode**: a state where the supervisor stops acting (no relaunch, no handover pushes)
  until the human or the orchestrator clears it, e.g. with the hold in 8fsx.
