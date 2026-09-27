---
id: w8bz
title: Should idle-with-a-live-background-task be a distinct agent state?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

During bridle's first self-hosted run, a headless worker used the Agent tool
to run something in the background. When that background work finished, the
worker's own idle process started a new turn on its own about 40 seconds
later. This matches the idle-message behavior documented in
`docs/spikes/01-stream-json-findings.md`, but it was triggered by an
internal Claude Code mechanism (the background agent's completion), not by a
bridle message.

Bridle's stall detection and `idle` state didn't know this new turn was
coming: from the daemon's side, an agent that has gone idle with a live
background task looks identical to one that's genuinely done and waiting.

Should idle-with-a-live-background-task be tracked as a distinct state from
plain idle, so the supervisor/stall detector doesn't mistake the pending
self-resumption for either "done" or "stuck"?

## Why it matters

Stall detection and any "the agent looks idle, is it actually done" logic
(see `docs/design/agent-host/agents.md`) currently has no visibility into
in-flight background tool work, which can cause an agent to resume on its
own outside of any tracked event.
