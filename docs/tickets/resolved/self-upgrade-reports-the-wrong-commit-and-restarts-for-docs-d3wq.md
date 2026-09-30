---
id: d3wq
title: Self-upgrade reports the wrong commit, and restarts for docs-only commits
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [bridle-restarts-itself-q7rx, unattended-while-the-human-travels-tv8r]
closed: 2026-09-30T12:48:28Z
---

## What happened (2026-09-30, orchestrator)

1. **The wrong commit in the restart message.** The upgrade built `e99d7b9` (event "building
   e99d7b9"), then the restart said "daemon restarted at cbaf6c0" and told every agent "now at
   cbaf6c0". `cbaf6c0` had landed during the build and was not in the binary. `restart.rs:22`
   reads `refs/heads/<integration>` at restart time instead of the commit that was built (the
   upgrade's `BUILT_KEY`). The next wake, "building 6e9fbf4", then looked like a downgrade from
   `cbaf6c0`. Fix: report the built commit.
2. **A restart for a docs-only commit.** `6e9fbf4` changed only a ticket under `docs/`, yet the
   daemon rebuilt and restarted, stopping and resuming every agent. Each restart is a small risk
   (tv8r: nobody is home to recover a dead daemon), and three of today's four were for nothing
   the binary uses. Fix: skip the upgrade (but record the commit as built) when
   `git diff --name-only <built>..<candidate>` touches nothing the binary is built from: for this
   repo, nothing under `crates/`, `Cargo.toml`, `Cargo.lock` or the role and rule files the daemon
   embeds at build time, if any. Check what `include_str!`/`include_dir!` pull in before
   choosing the paths; don't guess.

## State

Open, small. Wanted before the human's trip (Thu 2026-10-01 afternoon), since it cuts restarts.
