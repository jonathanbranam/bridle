---
id: d4mz
title: One-command orchestrator handover
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [htp6, hj4g]
---

## The question

Handing the orchestrator over to a fresh Claude Code session takes too much
work. On 2026-09-28 the first handover meant rewriting
`docs/context/orchestrator-state.md` by hand, the human copying a long opening
prompt into a new session, and the old session stopping its watcher and
heartbeat. How should a handover work so the human only agrees to it and runs
one command?

The human's words, 2026-09-28:

> so, this handoff takes too much work; it should be something that we can do
> together more easily; when we decide to start a new session, you should
> write out anything necessary in bridle, then we agree, and i'll start the new
> session - but it should be easy for me, no copy/paste, just a command if
> possible; like claude-orchestrator or whatever and that should be like a
> simple bash script that starts claude as orchestrator using some kind of
> bridle prime orchestrator or something along those lines; file that as a
> ticket for future work.

The shape the human describes:

- **The outgoing orchestrator writes its state into bridle**, not only into a
  hand-edited doc: what's in flight, the queue, open items, the decisions made
  this session.
- **They agree to hand over**, and the old session shuts down its own
  background jobs.
- **One command starts the new session**, e.g. `claude-orchestrator`, a small
  script that runs `claude` as the orchestrator, primed by something like
  `bridle prime orchestrator`, which prints the role, the current state and
  the startup steps (token, watcher, heartbeat).

## Why it matters

The orchestrator should renew itself as readily as bridle's own agents will
(htp6), and every handover costs the human effort and risks losing state.
`prime` is already planned for P1 ([build order](docs/proposal/build-order.md)).

## Progress

Owned by the orchestrator (the human, 2026-09-28: "this ticket d4mz is yours
now").

- **Step 1, done 2026-09-28:** `scripts/claude-orchestrator` starts `claude`
  with Remote Control on and the standing opening prompt, and the role's
  "Handing over" section is the outgoing checklist. State still lives in
  `docs/context/orchestrator-state.md`.
- **Step 2, open:** `bridle prime orchestrator` (P1) prints the role, the state
  and the startup steps, and replaces the script's fixed prompt. After P0-6,
  the handover state moves into bridle's task records instead of the doc.
