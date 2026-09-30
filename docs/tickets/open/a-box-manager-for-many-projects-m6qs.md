---
id: m6qs
title: A box manager for many projects (research, not to be done yet)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [one-daemon-for-several-small-projects-3nkk, small-projects-start-without-a-manager-w2hj, projects-on-other-machines-by-config-k7mw, nuc-recovery-on-boot-4r3k]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> basically, it seems like I'll have a dozen projects on the NUC; is that OK? I don't have enough
> time or attention to make or review tasks for all my projects and I work in fits and starts; it
> isn't too bad if I have to start a daemon up, but I think a better management solution would
> help. feels like turning the daemons off is the easiest approach, but that needs some design
> work: I should be able to message the orch to start a project daemon up and/or someone (bridle or
> orch) would notice pending messages for a project and start that project's daemon.
>
> I guess, I keep feeling like we need a box-manager kind of daemon that knows about the projects
> and can manage them; but KISS for now - file this as research, not to be done; for now I'll
> start/stop daemons and let's do just option (b) - a per-project config that dsiables auto-start;
> and we let the orch start them if there is work to do or if I ask.

**Research only; not to be done yet.** For now the human starts and stops daemons by hand, and
[[small-projects-start-without-a-manager-w2hj|w2hj]] keeps idle projects cheap.

## Is a dozen projects on the NUC OK? (advisor, 2026-09-30)

From the 3nkk spike: a daemon is 14-57 MB and idle; an idle `claude` agent is ~300-400 MB. Twelve
daemons are nothing on 16 GB; twelve idle managers would be ~4-5 GB plus a costly turn on every
daemon restart. With managers off (w2hj), a dozen daemons is fine. The real limit is work: the
NUC's 2 cores and 4 threads for workers' builds, and the shared budget (xypj).

## Questions for the research

- **Daemons off when idle**: who starts one when there's work: the orchestrator on the human's
  ask, or something that notices pending messages or ready tasks for a stopped project? Where do
  messages to a stopped project go meanwhile (its daemon is its mailbox)?
- **A box manager**: one small always-on process per machine that knows the machine's projects
  (k7mw's config), starts and stops their daemons (the systemd units of 4r3k), holds their mail
  while they're stopped, and is what the orchestrator and boot talk to. Compare with the
  orchestrator doing it by `bridle serve` in tmux, and with one daemon per group (3nkk option 6).
- What "idle" means for a daemon: no running agents, no ready tasks, no unread mail, for how long.
