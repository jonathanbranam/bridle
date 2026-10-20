---
id: br-fpqy
title: "bridle session stop: hand over, then stop an advisor, aide or orchestrator session until it is started again"
kind: feature
created: 2026-10-20
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-19 ~7:50 PM ET (to advisor usage), after the week's usage analysis showed that interactive Opus sessions used about 42 of the week's 60 points:

> Suggestions to reduce spend? What about having the unnecessary opus agents write a handover and then shut down. Then they can read the handover next time I start them. Is that supported now?

and, after the advisor said it takes two steps today:

> Yes let's make a command for session stop that includes writing a handover and stopping, to be restarted later.

Today: the session runs `bridle handover write --no-restart --file -` and then the human closes it by hand. The next `bridle session advisor <name>` (or `aide`) already points its opening prompt at the newest handover. `bridle session restart advisor/<name>` asks for a handover and waits for it, but only to restart.

The ask: `bridle session stop <role>[/<name>]` (for example, `advisor/tj7q`, `aide`) asks the running session to write its handover note, waits for it (as `restart --handover` does), then ends the session and leaves it stopped. Starting it again later is the existing `bridle session advisor <name>` / `bridle session aide`, which reads that note. Notes:

- Its waiter (`bridle agent wake`) is ended through the daemon, not by a signal or a kill by name (rule `no-kill-by-name`). The tmux window, a shell window (rule `tmux-shell-first`), stays open so the human can start the session again in it. Say what was stopped and how to start it again.
- The orchestrator: the same, then `bridle orchestrator hold` so the supervisor doesn't relaunch it (human only, as the hold is today), and `release` starts it with its handover.
- Messages that arrive while it is stopped wait in its inbox, as they do now.
- Update docs/design/cli.md and the role texts that describe handovers.
