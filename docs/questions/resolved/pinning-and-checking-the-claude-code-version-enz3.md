---
id: enz3
title: How does bridle pin and check the Claude Code version?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: [docs/design/agent-host/agents.md]
needs: []
see: []
---

## The question

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups:

> **Version pinning.** Everything here is 2.1.283. The control protocol (`interrupt`, `get_usage`, …),
> `rate_limit_event` shape, block-per-event splitting and mid-turn folding are undocumented or semi-internal.
> Bridle should pin the version, check `init.claude_code_version` and `capabilities`, and rerun these
> fixtures as a contract test on upgrade (`cargo run -- all`, then diff the catalogue).

The docs split recorded this as build work, not a spike, and didn't file it.
It's filed here so it isn't lost.

## Why it matters

Claude Code auto-updates. v1 depends on:

- a mid-turn stdin message being folded into the running turn, which bridle's
  delivery relies on;
- `--replay-user-messages` echoing text verbatim, which is how acks are
  matched ([[replay-echo-matching-by-text-6q8s|replay echo matching]]);
- the interrupt receipt;
- the exit-code semantics.

A silent change breaks delivery tracking, and nothing would say so.

## Notes

State as of the v1 build (2026-09-27):

- The daemon doesn't read `system/init.claude_code_version` or `capabilities`.
  A cheap first step is an `agent.version` event plus a warning when either
  differs from a known-good list.
- `just test-live` runs one live test, in `bridle-claude`. There's no live
  daemon test. The end-to-end run in the v1 build (a Haiku worker commits,
  messages the human, is woken, stopped, resumed and removed) was done by hand
  and would make a good `#[ignore]` live test.
- The spike harness (`spikes/stream-json`, `cargo run -- all`) is the fuller
  contract test, but it costs about $0.40 per run.

## Resolution

Don't pin. The human, verbatim:

> We need a set of automatic tests instead of pinning the Claude version.
> Claude wants to auto update and I'm not even sure if we can control that
> would we disable that behavior? Seems like we want to update Claude and run
> the test suite that is VERY LIMITED to key behaviors, then accept the upgrade
> as new. If something breaks preference to fix forward rather than rollback.

So:

- **A contract suite** of three live tests,
  `crates/bridle-claude/tests/contract_test.rs`, covering turn boundaries,
  verbatim replay echoes, the exit codes, cumulative cost across `--resume`,
  mid-turn folding, the interrupt receipt and the budget cap's result.
  `just test-contract` runs it (Haiku, about $0.10) and on success records the
  version in `crates/bridle-claude/tests/contract-verified.txt`. First passed on
  2.1.283.
- **The daemon notices upgrades**: it records the version from each agent's
  `system/init`, shows it in `bridle status`, and emits `claude.version` (and
  logs a warning) when it changes.
- Auto-update stays on. A failure is fixed forward in bridle.

Recorded in [[docs/design/agent-host/agents#Claude Code upgrades|agents]].
