---
id: npj2
title: "Research: why Rust builds take ~10 minutes and what would speed them up"
kind: research
opened: 2026-10-07
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [[b7cz, z7y5]]
tasks: []
---

## The ask

The human, 2026-10-06 ~10:40 PM ET, verbatim:

> can we do some research on any way to speed up Rust builds, or is this just actually how long they take? ... 10 minutes feels like a long time.

They say they've asked before (b7cz covers the cold-worktree part: warm `target/`, `check-affected`, measured 2026-09-29).

## Ask

Research, no build: where the ~10 minutes goes today and what would cut it. Measure first (the human, 2026-09-29: "measure it when possible before committing to more work").

- Which builds take ~10 min: the self-upgrade's release build (`cargo install`, NUC measured 10m53s cold), a worker's first build, `just check`, CI?
- `cargo build --timings` on the release and debug builds: the slowest crates, the critical path, link time.
- Candidates to weigh with numbers: release profile settings (codegen-units, lto, incremental for the self-upgrade build, `debug = 0` / split-debuginfo for dev), a faster linker (lld / ld-prime on macOS), trimming heavy deps or features, splitting the `bridle` / `bridle-daemon` crates for parallelism, sccache, cranelift for dev builds, a persistent target dir for self-upgrade.
- macOS specifics: syspolicyd / Gatekeeper scanning new binaries (z7y5), Spotlight indexing `target/`.

Output: findings on this ticket, a ranked list of changes with the measured or expected saving, and a recommendation. Follow-up builds get their own tickets.
