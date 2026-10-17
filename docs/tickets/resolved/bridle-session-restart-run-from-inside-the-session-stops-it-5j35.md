---
id: br-5j35
title: "bridle session restart run from inside the session stops it and never relaunches: the detached restarter dies with the session"
kind: fix
created: 2026-10-06
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T20:19:00Z
---

## The ask

Seen 2026-10-06 ~7:43 PM ET. The human: "the aide tried to restart but just failed".

Aide (bridle, dalek) was at 200K context and ran `bridle session restart aide` from inside its own session, as its role says to. The command re-ran itself detached (`detach_restart`, `crates/bridle/src/session.rs`: its own process group, output to `~/.bridle/restart-aide.log`). It asked for a handover note and waited. The note aide had written just before didn't count, because the restart waits for one newer than its request, so the human saw "that command is hanging" until aide wrote it again. Then the aide session ended and its pane (%86) was left at the shell. Nothing relaunched it. The log's only line is `asked aide to write a handover note; waiting`: no "restarted", no error. No restart process was left running. The orchestrator typed `bridle session aide` into the pane by hand; the new session read h-0052 and carried on.

Likely cause (not verified): `stop_session` sends SIGTERM to the launcher's children (claude). The restarter is still claude's descendant: a new process group doesn't reparent it. When claude exits, it takes its descendant processes with it, the restarter included, before `tmux send-keys` runs. So a restart started from inside the session can never finish, which is exactly the case the detach was built for.

Fix options: fully daemonize the restarter (double fork, or setsid plus reparenting to launchd), so it isn't claude's descendant; or have the daemon carry out the restart, as the orchestrator supervisor does. Also, a note written within a minute or so before the request should count, so the session isn't asked twice.

Check: a test (or a live trial with the human) of an in-session restart that relaunches in the pane. `just check`.

## Seen again, 2026-10-11 (advisor/wsl2): the pane itself was gone

advisor/wsl2 hit 201k at 15:30:50Z, wrote handover h-0110 at 15:31:10Z, and its session ended at
15:31:11Z (`session.ended`). Its tmux window (`1_bridle:wsl2-install`) closed with it, and
`~/.bridle/restart-advisor-wsl2.log` says `can't find pane: %22` / `pane %22 is gone`, then
`no handover note after 600s; nothing restarted` (likely a second restart attempt that didn't
count h-0110). The same day, advisor/product-manager restarted fine into pane %9.

The human, 2026-10-11 ~11:50 AM ET, verbatim:

> this is likely because orchestrator launched that session and keeps using a tmux command that
> binds the pane to the claude process

That fits the code: the restart relaunches by `tmux send-keys` into the pane
(`crates/bridle/src/session.rs`, `restart`), which needs a shell left in the pane. A window opened
as `tmux new-window '<bridle session ...>'` runs the session as the pane's own command, so the pane
closes when the session exits and there is nothing to type into. Fix options: whoever opens a
session window opens a shell and types the command into it (new-window, then send-keys), or sets
`remain-on-exit` / uses `respawn-pane`; the restart could also fall back to a new window when the
pane is gone.
