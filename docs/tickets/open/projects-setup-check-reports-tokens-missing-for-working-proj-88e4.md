---
id: br-88e4
title: projects setup --check reports tokens missing for working projects
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

`bridle projects setup <p> --check` must pass when the tokens work, and when it reports missing tokens it must name the principal, project and machine. Following its `next: bridle token pair --projects <p>` advice must clear it.

Seen on remus, bridle 0.8.0, 2026-11-17 ~04:15 UTC, right after a clean human-run `bridle token pair` from remus-home (dalek minted 26/kept 34, remus 10/52, vesta 15/45, no failed lines). For flume (new that night), glassbox and remus-home alike, `--check` prints:
  tokens ... todo: tokens missing on some machine
  aide token ... todo: tokens missing on some machine
  next: bridle token pair --projects flume
But glassbox and remus-home have run for weeks with working aide, advisor and orchestrator tokens, and flume's aide and advisor start fine and its mail forwards. So the check is wrong, or too vague.

Lead (from the submitter, unverified): an earlier pair run printed check lines such as "check remus peer -> remus/glassbox: refused: daemon: error: forbidden: this endpoint is human-only" and "check remus orchestrator -> remus/flume: refused: error: no entry". If the setup check counts a human-only refusal as missing, that would explain it.

## Scope

crates/bridle/src/project_setup.rs (the message is at line ~218; read how it decides a token is missing and what it probes) and, if it shares the check with token pair, crates/bridle/src/token_pair.rs. Fix the decision: a refusal that proves the token was accepted and the endpoint is human-only is not "missing"; only an absent or rejected token is. Make the message name principal, project and machine. Docs: docs/design/cli.md where setup --check is described. No project-file change, no migration.

## Acceptance

Tests: tokens present and working (including a human-only refusal on a probe) -> passes; one genuinely missing token -> the todo names which; after `token pair` the check clears. `just check` with BRIDLE_* env unset. Model: Sonnet.

May touch token_pair.rs, as br-4wu6 and br-u974 may: do not run beside them; the manager orders them.
