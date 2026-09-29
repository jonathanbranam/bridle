---
id: 9mxw
title: What's per project, per machine, and per account
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [incident-notices-that-retract-themselves-nc7r, how-project-daemons-share-one-budget-xypj, one-orchestrator-and-advisor-or-one-per-project-ma8e, finding-remote-daemons-from-the-laptop-xqvg]
---

## The ask

The human, verbatim (2026-09-29, via the advisor), for future follow-up:

> Something else to note for future follow-up is the question of project-local and global
> bridle issues, incidents, and tasks and management. There are some things that are global,
> such as our token budget, and other things that are project-local, such as the roadmap and
> priorities. There are also potentially global issues for each system where we might overtax
> the box that we're running on.
>
> I don't know how to resolve any of these, but it's clear that we need to consider some of
> them. An incident with SSH being down, or the internet or something being down, affects
> everybody on the same box, but not necessarily different boxes. Token budget would affect
> every box, even if I'm running separate projects, separate venvs, separate repos, separate
> boxes. They still all share a token budget, and there could be coordination problems there.
> Essentially, it's not a big deal, but it's something to consider.

Prompted by this morning: after the laptop reboot, the SSH agent had no keys and Tailscale was
down, which blocked pushes for every project on the laptop.

## Three scopes (advisor's reading)

- **Project**: roadmap, priorities, the task queue, project incidents.
- **Machine**: SSH keys, network, Tailscale, disk, CPU and memory load. An incident here hits
  every project on that machine and none on another one.
- **Account**: the Claude token budget, shared by every project on every machine.

Tasks, incidents and management each need to say which scope they belong to, and who handles
each scope.

## Related, already open

- [[incident-notices-that-retract-themselves-nc7r|incident notices]]: incidents, today per
  daemon, i.e. per project.
- [[how-project-daemons-share-one-budget-xypj|how project daemons share one budget]]: the account
  scope, on one machine.
- [[one-orchestrator-and-advisor-or-one-per-project-ma8e|one orchestrator or one per
  project]]: who manages across projects.
- [[finding-remote-daemons-from-the-laptop-xqvg|finding remote daemons]]: more than one machine
  (the NUC).

## The orchestrator's supervisor is per project in code, machine-wide in the design (2026-09-29)

`orchestrator-supervision.md` section 2 puts `[orchestrator]` in the machine config beside
`[budget]`, but the daemon reads it from the project's `<repo>/.bridle/config.toml`
(`Config::load`; only `[budget]` has a machine-wide loader), so a line in `~/.bridle/config.toml`
does nothing. Only a daemon whose project enables it supervises; the pid, session and exits files
are machine-wide under `$BRIDLE_HOME`, so enabling it in two projects would have two daemons
relaunching one orchestrator. `bridle wait-for-wake` talks to one daemon, so wakes cover only
that project. Today the orchestrator lives in the bridle clone and its host is the bridle
daemon, which must be running even when no bridle work is. The human asked (2026-09-29) how this
works when they aren't working on bridle itself; decide with this ticket: one machine-level
orchestrator host (a machine daemon or a designated project) versus per project, and make the
doc and code agree.
