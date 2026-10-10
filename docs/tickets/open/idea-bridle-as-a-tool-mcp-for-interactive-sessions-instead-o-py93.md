---
id: py93
title: "Idea: bridle as a tool (MCP) for interactive sessions instead of the CLI, so waiting and message receipt are consistent"
kind: explore
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [9aj2, z485, k4wq]
tasks: [br-py93]
---

**A future idea, not committed and not scheduled.** For the PdM's reviews of themes and epics.

## The ask

The human, 2026-10-10 ~10:15 AM ET, verbatim (to the aide):

> I'm also interested in this. This was passed on, I think: agents sometimes run their waiter in
> the background with an ampersand. Someone probably answered me, but I didn't see the answer. How
> does that work, and how can we prevent that?

> One of the considerations here is whether the way we're using the Bridle CLI is the best way to
> do it, or whether we can use a tool call. Can we add a tool for Bridle? This is a future idea
> question: would that make things more consistent, and would that fix this waiter problem?

## Facts (2026-10-10)

- Interactive sessions (orchestrator, aides, advisors, the PdM) wait for messages by running
  `bridle agent wake` as a Claude Code background Bash command; Claude Code re-invokes the session
  when it exits. Run with a shell `&` (or its output discarded), the command still runs and marks
  what it receives read, but the session never sees the output and is never re-invoked. The role
  files forbid it; nothing enforces it (ticket 9aj2 item 4; 9aj2 item 1 makes it lose nothing).
- Background agents (workers, managers) don't wait at all: the daemon writes each message to their
  stdin and confirms delivery from the echoed message (`docs/design/agent-host/messages.md`,
  "Delivery").
- A human surface with an MCP server is already mentioned in k4wq and `docs/proposal/build-order.md`.

## Questions to explore

1. Would a bridle MCP server (tools like `send`, `inbox`, `task_comment`) in interactive sessions
   make calls more consistent (typed arguments, no shell quoting, no hook false positives)?
2. Can a tool fix waiting? An MCP tool call blocks the turn, so a long wait would freeze the
   session; a wait is better as Claude Code's own background mechanism or, at the root, by making
   the session a background agent (z485 and the human's "move interactive agents to the background"
   epic). Say which, with the trade-offs.
3. Cost: a server per session, its tokens and auth, and keeping it in step with the CLI.
