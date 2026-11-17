+++
id = "br-4wu6"
title = "bridle token pair runs for minutes with no progress output"
kind = "bug"
state = "built"
created_at = "2026-11-17T03:58:44.645Z"
updated_at = "2026-11-17T13:32:52.240148Z"
created_by = "external:orchestrator@remus"
watchers = [
    "external:orchestrator@remus",
    "agent:4wu6",
]
branch = "bridle/4wu6"
commit = "3a75fac7e52e7bc416c4f14c6e3957a6e14b09bf"
summary = "token_pair.rs: pair() wraps the runner in Tracked: stderr line per ssh/local step (start, ok/failed), and after one unreachable-type failure (unreachable, no answer, refused, timed out) on a host, its remaining steps fail at once with no ssh; said once on stderr and named in a closing stdout line 'machine H unreachable: skipped N steps'; exit non-zero. Integration test with fake ssh; cli.md updated. Caveat: N is counted at the end, not announced up front."
ticket = "4wu6"
+++

docs/tickets/open/bridle-token-pair-runs-for-minutes-with-no-progress-output-4wu6.md

submitted by external:orchestrator@remus

Reported by the human via orchestrator@remus, 2026-11-16 (verbatim): "bridle token pair should output something. It just sits there doing absolutely nothing, with no indication that it's working. If a command takes more than 30 seconds, it needs to tell the user that it's still working somehow."

Seen on remus, bridle 0.8.0: `bridle token pair` printed the three "machine X (peer tokens): minted 0, kept 13" lines, then nothing for 11+ minutes. It was working: one ssh per token per project per machine (e.g. `ssh dalek ... bridle --project meta-notes-ui token pair-store orchestrator --machine vesta`), each waiting up to ConnectTimeout=10 while dalek was asleep and unreachable. Nothing on screen said which project, machine or step it was on, so the human took it for hung.

Ask: any bridle command that can run past ~30 s prints progress (the project/machine/step it is on, or a periodic still-working line). The human states it as a general rule for every bridle command, not only token pair. Also worth considering: when a machine is unreachable once, skip its remaining steps instead of timing out on each one (26 separate 10 s timeouts for dalek in one run).

## Thread

### note · external:orchestrator@remus · 2026-11-17T03:58:44.648Z
submitted by external:orchestrator@remus

### note · external:orchestrator · 2026-11-17T11:00:32.502Z
readied by external:orchestrator

### note · agent:4wu6 · 2026-11-17T13:32:52.188Z
built: 3a75fac7e52e7bc416c4f14c6e3957a6e14b09bf (branch bridle/4wu6)

### note · agent:4wu6 · 2026-11-17T13:32:52.240Z
done: token pair progress per step + skip unreachable machine; 3a75fac7e52e7bc416c4f14c6e3957a6e14b09bf; just check exit 0 (BRIDLE_* unset), 2011 tests run, 2011 passed
