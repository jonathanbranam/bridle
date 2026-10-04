---
id: tc7t
title: A landing that needs a UI install or gateway restart to show says so, and someone does it
kind: bug
opened: 2026-10-04
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [k3qx]
tasks: []
---

## The ask

Incident ui-wdp3 (2026-10-04): bridle-ui routing (ui-n6cu) landed at 11:56 AM ET, but the human, loading the site hours later, said "I'm just loaded up the site and it doesn't look any different at all to me." ~/.bridle/ui was last installed at 8:41 AM and the running gateway started at 8:55 AM, so neither carried the landed work. No role installs the UI (`npm run install-ui`) or restarts `bridle gateway` after a merge, and no landing note says that either step is needed.

Wanted: a landing report for work that only shows after a UI install or gateway restart says which step it needs; the step is either done by a role (the orchestrator or manager, at a quiet point) or put to the human as a to-do. Which, is the human's call; this ticket records the gap.

## The human's decision

The human, verbatim (2026-10-04, to the bridle-ui aide, choosing option C of: A the manager
installs the UI and a gateway restart is a human to-do, B both are human to-dos, C both are
automated):

> I want option C. All of this should be automated. Everything's fully automated with Bridal, and
> everything should be fully automated with the Bridal UI as well. So get that work done.
> There's, there's no reason for the gateway to be different. The Bridal UI restarts, or the
> Bridal daemon restarts, and agents restart. Everything should restart. And for doing the build,
> we should just follow the same pattern as the Bridal Daemon. So who does the final build that
> installs the binary for Bridal in the Bridal project? Whoever does that should be, I think, the
> same role that does it here. I don't know that that fits with the manager, but if that's the
> only role available to do that, then that's fine.

("Bridal" is bridle; the human was dictating.)

The answer to the human's question, as built today: no role installs bridle's binary. With
`[daemon] self_upgrade = true` in bridle's `.bridle/config.toml`, the daemon's CI watcher builds
the newest green `main` at a quiet point and restarts in place, resuming every agent
(`docs/design/agent-host/daemon.md`, "Automatic upgrade").
