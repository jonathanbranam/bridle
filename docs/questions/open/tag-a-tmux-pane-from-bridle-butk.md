---
id: butk
title: Tag a tmux pane from bridle (bridle pane tag / untag)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-orchestrator-stays-running-fx7x, orchestrator-pane-full-of-escape-codes-csfe]
---

## The ask

The human, 2026-09-30 (about 23:30 ET), after csfe forced them to close the whole tmux terminal:

> I had to close that entire tmux terminal, so I had to start a new one and find the command to
> give it the tag. let's add a ticket for bridle to do that, could we have a command that tags
> the current window as the orch window? and one that unsets it? might just allow any tags so I
> can tag advisor windows in the future.

Today the daemon finds the orchestrator's pane by the tmux pane option `@bridle=orchestrator`
(`orchestrator-supervision.md`, "Finding the pane"). The only way to set it is to remember
`tmux set -p @bridle orchestrator`.

## Proposed

- `bridle pane tag <name>` sets `@bridle=<name>` on the current pane (`$TMUX_PANE`). Any name is
  allowed; `orchestrator` is the one the daemon reads today, and advisors can use their own later.
- `bridle pane untag` clears the current pane's tag (`tmux set -p -u @bridle`).
- A tag names one pane. Tagging a pane clears the same tag from any other pane, so the daemon
  never has two candidates. It says so when it does ("moved orchestrator from %39").
- Outside tmux (no `$TMUX_PANE`), or with tmux missing: a clear error and a non-zero exit.
- The daemon's "no pane tagged @bridle=orchestrator" incident text names the new command.
- Docs: `cli.md`, and `orchestrator-supervision.md` where it gives the tmux command.

The command goes under `pane` so it fits the a67t CLI grouping whichever way that lands. It
doesn't need the daemon: it's a local tmux call.

## Open (for the human)

- Should `scripts/claude-orchestrator` (and `claude-advisor <name>`) tag their own pane at start?
  Then a new terminal needs no step at all, and the command is for fixing things by hand.
  Recommendation: yes. The launcher already knows its role, and a pane running the orchestrator
  is the one the daemon should type into.
- A `bridle pane list` (tagged panes)? Not asked for; leave it out until it's needed.
