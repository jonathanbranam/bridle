---
id: 9trg
title: Spike: process containment on macOS
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [2mj9]
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 5:

> **Containment**: have an agent start `npm run dev` with `nohup … &`, and a
> double-forked daemon. Confirm the scan-and-kill loop finds and kills both,
> and see what `sysinfo`'s `environ()` returns on macOS.

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Process containment** (Surprise 3) must be solved before bridle can guarantee cleanup. Check
> descendants with background Bash (`run_in_background`) too, and the "killed ~5 s after final result"
> claim, which this spike didn't test.

## Why it matters

"Stop what you start" can't be guaranteed without it.

## Notes

- `docs/agent-host.md` §4.6 implements a `ps` scan and sweep. Double-forked
  daemons that detach between two scans aren't covered in v1.
- Linux needs its own: [[process-containment-on-linux-2mj9|containment on Linux]].
