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

## The human, more (2026-09-30)

> the daemon is probably not the concern, but I don't need so many agents running across small
> projects; the catch is that project rules and workflow may vary; so; not sure I think the rule
> would be: one workflow for a project grouping. [...] claude agents are expensive in compute and
> tokens; small projects could easily share agents; and even one agent per small project is too
> much overhead. Our minimum is three agents per proejct, I believe. That is too much and
> particularly too much if there is no work happening. Maybe the daemon is cheap but the agents
> are not.

## Research spike (advisor's subagent, 2026-09-30; read-only)

**What runs.** Not three per project. The default is **one manager** per daemon (autostart and
resume_on_restart, `crates/bridle-daemon/src/config.rs` ~431); workers and the orchestrator role
don't autostart. Bridle's own repo also autostarts a product-manager. The orchestrator and
advisors are external sessions, one per machine. Live: bridle manager-2 and pm-1, one manager each
on meta-notes and track-web.

**What idle costs.** An idle `claude` is 280-390 MB RSS and ~0% CPU; a daemon 14-57 MB. A manager
has no timer; it takes a turn only when something sends it one:

- **resume after a daemon restart** writes a continuation note so the process has a turn to start
  (`docs/design/agent-host/agents.md` ~144; `lib.rs` ~883). The cache has expired, so it re-writes
  the whole context: $0.32-0.38 a turn against $0.02-0.13 for a normal one;
- orchestrator messages (21 of 43 to the meta-notes manager), task-filed notes (when no PM runs,
  at most one a minute), main-moved notes, CI failures, governor resume nudges, chunked messages
  (r5hc).

meta-notes' manager: 42 turns, $4.02 in 1.5 days, about 8 minutes busy; 10 resumes, and 8 whose
cost could be seen came to $2.08. Managers are ~60% of the small projects' spend (meta-notes $4.02
of $6.98, track-web $2.14 of $3.43). Unverified: why meta-notes restarted ~10 times (upgrades or
restarts; d3wq, q7rx); the per-role cost counter looks off (PM 24h at -$0.22).

**What's per project.** Everything: "Nothing is shared between daemons"
(`docs/design/agent-host/operating-model.md` ~244). One store, workspace, config and project per
supervisor; the manager works in one checkout; queue, worktrees, integration branch, check,
workflow layers and tokens are per repo.

**Options, best value first:**

1. A resume doesn't start a turn unless a message is waiting (lazy resume). Most of an idle
   manager's spend, little code.
2. `autostart = false` for small projects' managers (config only, today); the orchestrator spawns
   one when there's work, or merges small projects itself (operating-model.md ~177 allows it).
3. Stop a manager or PM after N minutes idle; start it on demand when a task is filed or ready or a
   message comes for it. Saves ~300 MB a process.
4. No restarts for docs-only commits (d3wq).
5. Batch orchestrator messages (r5hc).
6. Deferred (YAGNI): a manager across daemons (medium), a daemon over a group of repos with one
   workflow (large; against the isolation rule).

Recommendation: keep daemons per project; cut agent turns and processes with 1, 2, then 3. No
existing ticket covers stopping idle agents or lazy resume.

## The human on the findings (2026-09-30)

> keep this as research results but don't act on it yet.

Options 1 and 3 wait. Option 2 is taken up in
[[small-projects-start-without-a-manager-w2hj|small projects start without a manager]]; the
"box manager" idea is filed as research in
[[a-box-manager-for-many-projects-m6qs|a box manager for many projects]].
