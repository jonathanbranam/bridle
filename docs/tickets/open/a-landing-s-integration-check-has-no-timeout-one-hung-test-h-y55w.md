---
id: y55w
title: "A landing's integration check has no timeout: one hung test holds the landing queue indefinitely"
kind: bug
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

## What happened

2026-10-06 ~23:13Z: br-qbbk's landing ran `just check` in the integration worktree (pid 12103). nextest's gateway_test `a_replaced_binary_is_re_executed` (pid 19304) waited forever on its child, a temp-dir `bridle gateway` (pid 25731, started ~23:31Z, 0% CPU since), likely stuck behind the syspolicyd backlog of incident z7y5 (a new binary's first exec waits on syspolicyd). The landing held the queue for an hour and counting, so br-x56y (critical: no agent can `bridle send`) could not land behind it.

There is no way to end it from bridle: `bridle land` has no abort, and the check has no timeout. The orchestrator can't kill a pid it didn't start (rule no-kill-by-name), so it took the human's hands.

## Ask

- A landing's check gets a timeout (a config value; default well above a normal `just check`, e.g. 30 min). On timeout the daemon kills the check's process group, fails the landing with "check timed out", and moves on to the next.
- Possibly also: nextest slow-timeout / terminate-after in `.config/nextest.toml` so one hung test fails instead of hanging (cheaper, and helps workers too).

Related: z7y5 (the cause of the stall), y455.
