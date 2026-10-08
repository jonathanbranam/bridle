---
id: xypj
title: How do separate project daemons share one budget?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [xcpy, 3nqf]
---

## The question

From `docs/agent-host.md` §13 @ c192bfc, item 1:

> **How do separate project daemons share one budget?** Projects are
> deliberately isolated, one daemon each (§2.1). They share one account,
> though, and don't know about each other's usage. Options for the budget
> governor (§12.6):
> - each daemon reads a shared `~/.bridle/budget/` ledger and lease file;
> - a tiny machine-level governor that daemons ask before dispatching;
> - static per-project `max_workers` shares.
>
> The first keeps daemons independent.

## Why it matters

Hitting a window stops all work on the account, including the human's own
sessions ([[docs/design/usage-and-budget|usage and budget]]).

## Notes

- Machine capacity has the same shape. On the [[docs/context/nuc-host|NUC]]
  (2 cores / 4 threads, 16 GB), each `claude` process is a few hundred MB and a
  Rust build per worktree is the real limit. A per-daemon `max_workers` lets
  several projects each fill the machine. So a machine-level cap is needed for
  CPU and memory as well as tokens.
- The account is also shared with the human's laptop sessions, so a
  machine-local ledger doesn't see everything.
- **Pausing no longer needs this.** The governor reads account-wide
  utilisation (`get_usage`) and uses machine-wide thresholds, so every daemon
  winds down at the same point without coordinating
  ([[docs/design/usage-and-budget#Across projects|across projects]]). What's
  left is sharing `max_workers` and the dispatch rate below `hold_at`.

## Machine load (the human, 2026-09-30)

Load can't be scheduled in advance; the machine's one orchestrator coordinates it across projects
at run time, winding down agents by the projects' relative priority when the machine is bogged
down. Quoted in [[one-orchestrator-and-advisor-or-one-per-project-ma8e|ma8e]].

## Per-machine and per-project worker limits (the human, 2026-10-07)

Prompted by bridle-ui's one worker leaving ui-qbbk queued while other projects were idle. The
human, verbatim (~10:15 PM ET):

> Okay, is that because there's only one worker for bridle-ui? This is what I'm talking about
> here. We really need to manage workers both per machine and per project, but if none of the
> other projects are doing anything, might as well use two workers here.
>
> I think there's a whole system of defining those limits. We could put a max for the machine
> and a max for each project, and then we need a machine-level orchestrator watching the load
> and making sure things don't overwhelm the machine. Anyway, there's a ticket for that. Add a
> note about the proper way to manage this between a bunch of workers on the same machine on
> different projects.

So: a machine max and a per-project max, with idle projects' share usable by busy ones, and
a machine-level orchestrator watching load (see kuw2, the machine daemon).
