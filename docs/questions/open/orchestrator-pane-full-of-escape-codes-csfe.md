---
id: csfe
title: The orchestrator's pane filled with escape codes after a restart
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [fx7x]
---

## The ask

The human, verbatim (2026-09-29, about 23:13 ET):

> yeah something went bad with bridle restarting orch in tmux. I'm getting a bunch of control
> characters whenever my cursor enters that pane now. I when I etner and "O" when I exit the
> pane. Looks like this: "❯ ^[[O^[[I^[[O^[[I^[[O^[[I^[[O^[[I^[[O^[" also - hitting enter didn't
> want to send the message.

Then: "File this for investigation."

## What the advisor saw (read-only)

- Pane `%39` (`pi:5.4`, `/dev/ttys041`, tagged `@bridle=orchestrator`). Claude Code (pid 16232,
  under `scripts/claude-orchestrator` pid 16216, started 23:10:04 ET, session `e177fc3a-...`)
  was running and answering: it replied to the human at 23:13. The focus reports (`ESC[I` when
  the pane gains focus, `ESC[O` when it loses it) landed as literal text in its input box (vim
  mode, `-- INSERT --`), and Enter didn't submit.
- tmux: `focus-events on` globally; the pane's `#{pane_key_mode}` was `Ext 2`. That alone proves
  nothing: the healthy relaunched pane (`%68`) shows `Ext 2` too.
- `stty -a -f /dev/ttys041` showed `icanon echo echoctl` while Claude Code was the foreground
  process, which is unexpected for a TUI in raw mode. Not followed up.
- How the previous session (`35c01381-...`, about 145K context) ended is unknown: no
  `orchestrator.*` stop or relaunch event, and no line in `~/.bridle/orchestrator.exits` for it.
  Who started the 23:10 session (the daemon, the orchestrator's handover, or by hand) isn't
  established. The orchestrator may know.
- A fix attempt, run by the human from another pane:
  `printf '\e[?1004l\e[>4;0m\e[<u' > /dev/ttys041` (focus reporting off, modifyOtherKeys off,
  pop one kitty keyboard level). The human: "it did get slightly better, but it still printed
  out a lot of escape codes". The human killed the session (exit logged 23:15:24 ET); the daemon
  found it dead at 23:15:28 and relaunched (session `56dad04b-...`, new pane `%68`), which came up
  fine.

## How the 23:10 session started (the human, 2026-09-30)

> I asked orch to show me how the handoff works. And context was around 150k, so that orch
> session wrote the handoff and I guess bridle killed the process. then bridle restarted orch to
> read the new handoff

So `35c01381` wrote handover h-0003 (03:09 UTC, 23:09 ET) and ran `bridle handover done`. The
daemon stopped it and started the 23:10 session. The escape codes came after that relaunch.
Two consequences:

- Reproduce first with `bridle handover done`: the daemon's stop-and-relaunch path.
- That path left no `orchestrator.*` event and no `orchestrator.exits` line. That's a gap of
  its own: a handover relaunch should be recorded like any other.

## Working theory (unproven)

A terminal mode left on in the pane. Either the previous Claude Code ended without restoring its
modes, or the new one enabled a mode (focus reporting, extended keys) it then didn't decode. The
partial improvement from turning modes off supports this. What remained could be more pushed
keyboard levels, or other modes (mouse, bracketed paste).

## To investigate

- Reproduce: start the orchestrator in a tagged pane, end it the ways it can end (clean `/exit`,
  SIGTERM from the daemon's forced stop, SIGKILL, the human's kill) and relaunch. Does the new
  session get the escape codes?
- Find out how the 23:10 session was started and how the one before it ended.
- If a mode leak is confirmed: reset the pane before relaunching (e.g. the mode resets, or
  `tmux respawn-pane`/`clear-history` + `reset`), in the daemon's relaunch and/or
  `scripts/claude-orchestrator`.
