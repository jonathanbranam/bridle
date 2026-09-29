+++
id = "br-3bb4"
title = "Manager learns of new tasks without a PM: wake on task.created or poll queue on idle"
kind = "bug"
state = "planned"
created_at = "2026-09-29T04:34:59.147Z"
updated_at = "2026-09-29T04:35:08.445214Z"
+++

Problem: a project with no product-manager doesn't learn of tasks the human files. track-web's manager sat idle with five open tasks until told (open = not yet planned, so they never reach the queue; the manager is the one who should notice).

Goal: the development manager is woken when a task is created (event task.created, crates/bridle-api/src/types.rs ~486) or, if that is hard, when it goes idle it is told of unplanned open tasks. Recommended least-invasive design: when task.created fires and the project has no agent in role product-manager, the daemon delivers a short message to the running manager: 'task <id> filed: <title>; open tasks: N. Plan it or queue it.' (sender bridle, like other daemon-originated messages; look at how the governor or the delegate-reply flow (h5qd) sends daemon messages in crates/bridle-daemon/src/server.rs, governor.rs and supervisor.rs). Do nothing when a PM exists (the PM triages). Coalesce: at most one such message per manager per minute, listing all tasks filed since. No message if no manager is running.

Also make sure an idle manager is not left with a queue it doesn't look at: if the manager role prompt (workflow/base/roles/manager*.md) lacks 'when idle or woken, run bridle queue and bridle task list --state open', add it.

Acceptance: `just check` passes; daemon test with a fake manager agent: task.created with no PM -> one message; with a PM -> none; three creations within the minute -> one message. Update docs/design/agent-host or coordination.md (whichever documents daemon messages). Model: Sonnet. Out of scope: waking on other events, PM changes.
