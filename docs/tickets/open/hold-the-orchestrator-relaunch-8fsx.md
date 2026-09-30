---
id: 8fsx
title: Hold the orchestrator relaunch without restarting the daemon
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [orchestrator-identity-and-recovery-7d62, nuc-recovery-on-boot-4r3k]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> daemon starting orchestrator is great, but can be a little annoying if I shut orch down for a
> reason and I'm waiting to bring it back up. I think long term I need a way to override that (that
> doesn't require restarting bridle serve or anything) so I can indicate "don't auto-start orch"
> and then clear that setting.

## Today

The only switch is `[orchestrator] enabled = false` in config, which needs a daemon restart.

## Shape

`bridle orchestrator hold [--reason ...]` and `bridle orchestrator release` (the human only): a
runtime flag the supervisor checks before a relaunch. While held, no relaunch, no backoff counting,
no "not running" notes; `bridle status` shows it. It survives a daemon restart (in the store) until
released. Also the natural switch for 7d62's recovery mode and for 4r3k's boot step (it doesn't
start an orchestrator that's held).
