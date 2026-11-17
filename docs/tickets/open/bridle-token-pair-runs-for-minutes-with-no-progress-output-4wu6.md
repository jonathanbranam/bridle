---
id: br-4wu6
title: bridle token pair runs for minutes with no progress output
kind: bug
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle token pair` prints progress, and stops wasting time on a machine that is down.

The human, 2026-11-16, via orchestrator@remus, verbatim: "bridle token pair should output something. It just sits there doing absolutely nothing, with no indication that it's working. If a command takes more than 30 seconds, it needs to tell the user that it's still working somehow."

Seen on remus, bridle 0.8.0: `bridle token pair` printed the three "machine X (peer tokens): minted 0, kept 13" lines, then nothing for 11+ minutes. It was working: one ssh per token per project per machine (e.g. `ssh dalek ... bridle --project meta-notes-ui token pair-store orchestrator --machine vesta`), each waiting up to ConnectTimeout=10 while dalek was asleep. 26 separate 10 s timeouts for dalek in one run.

## Scope (this ticket)

1. In crates/bridle/src/token_pair.rs: print a line per step as it starts or ends (project, machine, step), to stderr if stdout is parsed anywhere (check callers and tests/token_pair_test.rs).
2. When a machine fails to connect once (ssh timeout or refused), skip its remaining steps in that run, say so once ("machine dalek unreachable: skipping its remaining N steps"), and finish with a summary that names the skipped machines and exits non-zero (or with the code the command uses for partial success; keep what it does today for a real failure).
3. No wire or project-file change, no migration. Docs: docs/design/cli.md where token pair is described.

Not in this ticket (not yet: wants a design for how every command reports progress, and the human stated it as a general rule): the same "still working" line for every bridle command that can run past ~30 s. The PM files that as a separate ticket after this lands, so the pattern from token pair can be reused.

## Acceptance

Tests with a fake ssh (as token_pair_test.rs does): progress lines appear per step; one unreachable machine is skipped after the first failure and named in the summary; reachable machines are unaffected. `just check` with BRIDLE_* env unset. Model: Sonnet.

Same file as br-u974's pairing question (token_pair.rs): run after it, or together by one worker if the manager prefers.
