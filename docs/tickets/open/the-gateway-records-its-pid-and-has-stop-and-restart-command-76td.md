---
id: 76td
title: The gateway records its pid and has stop and restart commands
kind: feature
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: [bek3, tc7t]
tasks: [br-76td]
---

## The ask


The human, verbatim (2026-10-06 ~10:00 PM ET, to the bridle-ui aide). They asked "I have the
gateway detached. How do I restart it?"; the aide answered that there is no restart command,
found the pid with `ps ... | grep "bridle gateway"` and `lsof -iTCP:7878`, and gave
`kill 68010 && bridle gateway --detach`:

> That's a problem. The gateway needs to be restartable with a command. I don't want to find a pid, and how did you find the pid? `ls -f`? Okay, that's not going to work. We need to track this. It should be tracked in a file. The pid should be written down. We should be paying attention to these things. We don't need auto restart yet, but it should be stoppable and startable. All that stuff should be done. I'll do it now.

The ask:
- **The gateway writes its pid to a file** when it starts (detached or foreground), e.g.
  `~/.bridle/gateway.pid` beside `~/.bridle/gateway.log`, and removes it on a clean exit.
- **Commands to stop, start and restart it** without the human finding a pid:
  `bridle gateway stop`, `bridle gateway restart` (stop, then `--detach`), and a status that
  says whether it runs, its pid, its URL and which build it runs (stale binary or not).
  Stop uses the recorded pid (rule `no-kill-by-name`: never a pattern match), and checks the
  pid is still a gateway before signalling it.
- **Not now:** auto-restart (the human: "We don't need auto restart yet"); that stays with
  [[the-gateway-runs-detached-and-keeps-itself-current-like-the-bek3|bek3]].

## Context

- `bridle gateway --detach` exists (bek3): it re-executes in a new process group, logs to
  `~/.bridle/gateway.log`, waits for health and is refused if a gateway already answers. It
  writes no pid file and there is no stop or restart.
- On dalek on 2026-10-06 the gateway (pid 68010) had run since Oct 5 7:40 PM ET while
  `~/.cargo/bin/bridle` was rebuilt at 9:55 PM ET on Oct 6, so it ran a day-old build with no
  command to swap it (same pattern as incident ui-wdp3 and
  [[a-landing-that-needs-a-ui-install-or-gateway-restart-to-show-tc7t|tc7t]]).
- The daemon already has `bridle daemon stop` and `bridle daemon restart` (no pid hunting;
  `docs/design/agent-host/daemon.md`); give the gateway the same, as the human asked in bek3.
