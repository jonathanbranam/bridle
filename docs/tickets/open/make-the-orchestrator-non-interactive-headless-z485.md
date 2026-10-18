---
id: br-z485
title: Make the orchestrator non-interactive (headless)
kind: feature
created: 2026-10-03
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r8kv, br-jttf, br-ae9f, br-gcvj, br-gb97, br-pdmd]
theme: agents-and-cli
parent: br-p7jf
---

## The ask

Now wanted, as the first step of epic br-p7jf (agents as their number grows), before the machine
admin role (gb97). The human, 2026-10-18 ~8:45 AM ET, to advisor (product-manager), verbatim:

> That is the right idea for the admin, but I don't want more interactive sessions on RC. Let's
> move the orch to background before pursuing the admin. Sequence that work.
>
> I think we would be ok to move the orch to background as a first step.

(RC: Claude Code's Remote Control, how the human reaches interactive sessions from the phone.)

Earlier, 2026-10-03 (via the advisor, a list headed "Ideas to record not build"):

> Make orch non interactive

## Proposal

- **Problem:** the orchestrator is an interactive `claude` session in a tmux pane, reached by RC.
  It hears messages only through its own `bridle agent wake` waiter, and every interactive session
  adds another RC session for the human to keep. The admin role (gb97) would add one more.
- **Approach:** design pass first (designer), the human reviews before any build. Run the
  orchestrator headless under the daemon, like managers and workers, so the daemon delivers its
  messages as turns. The design must answer: how the human talks to it without RC (gateway / web
  UI, mail, an advisor relay; research gcvj found RC cannot reach a headless agent); what happens to
  its supervisor, handover and restart path (orchestrator-supervision.md); its permissions and
  auto mode as a spawned agent; one per machine in `<host>-home` still; the cut-over and fallback.
- **Scope:** the orchestrator only. Other interactive roles (aide, PdM, advisors) are the epic's
  later design. Not the admin role (gb97 waits on this).

## Notes

- Today the orchestrator is an interactive `claude` session the human talks to
  (`bridle session orchestrator`). Goes with the split in
  [[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]]: if a triage role
  takes over the human's conversations, the orchestrator could run headless under the daemon.
