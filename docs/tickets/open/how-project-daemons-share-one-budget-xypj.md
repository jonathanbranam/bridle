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
