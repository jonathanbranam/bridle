---
id: nrbf
title: A daemon whose claude isn't logged in runs agents that silently do nothing
kind: bug
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [ex9q]
tasks: [br-5b39]
---

## The ask


Found 2026-10-03 by the orchestrator (incident log, 01:46 entry). The human started the
bridle-ui daemon with `bridle serve --detach` over SSH from the phone. On macOS, Claude Code
keeps its login in the login keychain, which an SSH session can't read. So every `claude` the
daemon spawned answered each turn with `Not logged in · Please run /login`, and the daemon
recorded each one as `turn done (success, $0.0000)`. The manager looked idle, and the only
signal was the `all_idle` wake 15 minutes later. A `daemon restart` re-executes in place and
keeps the same session, so restarting doesn't fix it.

Wanted, smallest first:

1. **Treat it as an error, not a success.** A turn whose only result is Claude Code's
   not-logged-in text (or an auth error result) should end the agent as `failed` with a clear
   reason, so it's a crash wake ("claude is not logged in in this daemon's session"), not an idle.
2. **Check at start-up.** `bridle serve` (and `bridle doctor`) runs `claude auth status` in the
   daemon's own environment and logs or warns loudly when it isn't logged in.
3. Docs: `docs/context/adding-a-project.md` says to start a dalek daemon from a local terminal or
   tmux, not over SSH (done with this ticket's filing).

The fix for the human today: stop the daemon, then start it from a pane of the laptop's own tmux
server (which can read the keychain), e.g. over SSH:
`tmux new-window -d -c <clone> 'bridle serve'`, or with `bridle launchd install`.
