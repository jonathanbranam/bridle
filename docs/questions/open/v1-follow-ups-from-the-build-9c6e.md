---
id: 9c6e
title: Small v1 follow-ups found during the build
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [k4wq, enz3]
---

## The question

Which of these small gaps from the v1 build are worth fixing before the next
feature? None is a design question. They're collected here so they aren't
lost, and each can become its own ticket when picked up.

## Items

- **SSE clients don't reconnect.** `Client::events_stream` ends when the
  daemon drops a lagging subscriber, or when the connection breaks. A
  long-lived client (the TUI, [[a-human-surface-beyond-the-cli-k4wq|human
  surface]]) needs to resume with `since` = last seq. The server already
  supports `Last-Event-ID` and `since`.
- **`bridle daemons` shows no agent counts.** The CLI deferred it because it
  would need to probe every registered daemon. With a short timeout that's
  cheap.
- **`bridle logs` renders tool input as a raw JSON fragment**
  (`→ Write("content":"hi","file_path":…)`). Show the command or path, as
  `tool.use` events do.
- **Spawn failures after `claude` starts** (bad model name, auth expired)
  surface only as a `crashed` agent with a stderr tail. `spawn` still returns
  201. Either wait for the first `system/init`, or document that `spawn`
  returning isn't readiness.
- **The fake `claude` matches magic words on the last line of a message**,
  because bridle wraps deliveries in an envelope. Tests that send multi-line
  bodies need the magic word last.
- **The agent-host design needs the docs cutover.** `docs/README.md`
  § Cutover lists the steps. The build's changes to `agent-host.md` are §4.9,
  the status line and §2.1.
