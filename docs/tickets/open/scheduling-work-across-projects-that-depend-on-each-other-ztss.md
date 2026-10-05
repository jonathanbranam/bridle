---
id: ztss
title: Scheduling work across projects that depend on each other
kind: question
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [cross-project-specs-yghs, one-orchestrator-and-advisor-or-one-per-project-ma8e, a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy, how-project-daemons-share-one-budget-xypj]
tasks: [br-1d48]
---

## The ask


The human, verbatim (2026-10-02, via the advisor), choosing option B for the web UI (a
separate UI server in its own repo, [[a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]):

> I prefer B - but that raises a unique problem that may be interesting to solve - how is work
> scheduled between projects that have inter-dependencies?
>
> I would take a very hard line on the API - it's fine to version it, but only one previous
> version maintained or NONE. A version is good regardless in the API, but bridle is changing
> rapidly and I own all parts so we don't need to support any other clients or users.
>
> I would plan to collocate the bridle and UI projects and have the advisor and orchestrators
> collaborate to make changes nearly simultaneously.
>
> Something missing here though in a separate UI is that I want to be able to read and edit or
> comment on literally everything bridle eventually - tasks, tickets, roadmaps, config, messages,
> incidents, etc. eventually everything is visible in the UI organized by machine and project.
>
> That goes beyond what bridle serves today since some of that is stored only as files on disk.

## The question

When work in one project depends on work in another, how is it scheduled? The case at hand: a
UI change waits on a bridle API change landing (and being deployed), and the two projects'
agents should move nearly together. Others: harness waits on track-web's engine (yghs).

## Today (advisor, checked 2026-10-02)

- [[docs/design/coordination|coordination]] says "Edges can cross projects (`hx-19ab blocked-by
  tw-7fa2`)", but the edges table is per daemon, so only same-project edges are built.
- Each project has its own daemon and queue; "there's no cross-project daemon"
  ([[docs/design/agent-host/operating-model|operating model]], "Several projects at once"). The
  machine's orchestrator (one per machine, ma8e) is the only thing that spans projects, by hand.

## Options (advisor)

1. **The orchestrator coordinates by hand** (today): it holds the UI task until the bridle one
   lands. No code; relies on attention.
2. **Cross-project `blocks` edges**: `bridle task dep add ui:ui-3f2a --blocked-by bridle:br-1665`.
   The waiting daemon finds the other daemon the way `--project` does and polls the task's state;
   it isn't startable until that task is integrated. An unreachable daemon counts as still
   blocked, and the queue shows why. The orchestrator still balances priority.
3. **A cross-project coordinator** (one daemon or service owning all queues). Contradicts the
   per-project daemon design; heavy.

Recommendation: 2, with a cross-project edge needing more than "integrated" in some cases (for
the UI, the bridle change must also be released and installed on the machine), so the edge may
name a condition: integrated, or released.

## The human's direction (2026-10-04)

The human asked for option 2, with watchers notified on the dependency's state change: filed as
[[cross-project-task-dependencies-a-task-waits-on-another-proj-yug5|yug5]].
