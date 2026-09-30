---
id: 3nkk
title: One daemon for several small projects?
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [dotfiles-local-as-a-bridle-project-35mw, how-project-daemons-share-one-budget-xypj, one-orchestrator-and-advisor-or-one-per-project-ma8e]
---

## The question

The human, verbatim (2026-09-30, via the advisor, filing dotfiles-local):

> side note: a full bridle daemon for some of my repos feels like overkill - I could easily see
> grouping a set of projects under a single daemon; IDK, maybe the daemon is light; something to
> consider anyway

Is a daemon per project heavy enough to matter (memory, CPU, ports, one manager and PM each, one
orchestrator each)? If so, should one daemon serve a group of small projects?

## To find out first

Measure an idle daemon (RSS, CPU, wakeups) on the NUC. The cost may be the agents a daemon
autostarts (a manager, maybe a PM), not the daemon: those could be off for small projects.
Low priority; something to consider.
