+++
id = "br-npj2"
title = "Research: why Rust builds take ~10 minutes and what would speed them up"
kind = "research"
state = "planned"
created_at = "2026-10-07T10:15:45.617Z"
updated_at = "2026-10-07T11:16:11.664377Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "npj2"
+++

Ticket: docs/tickets/open/research-why-rust-builds-take-10-minutes-and-what-would-spee-npj2.md (read it, plus b7cz in docs/tickets/resolved/ and z7y5). Research only: NO code changes, no build.
Goal: find where the ~10 minutes of Rust build time goes and what would cut it. Measure FIRST, then recommend.
Do: time a cold release build (`cargo build --release`, what the self-upgrade runs via `cargo install`), a cold debug build, and `just check`, each in a fresh worktree target dir; run `cargo build --timings` on release and debug and read the HTML for the slowest crates, the critical path and link time. Then measure the candidates one at a time against that baseline: release profile (codegen-units, lto, incremental), a faster linker on macOS (ld-prime / lld), `debug = 0` / split-debuginfo for dev, trimming heavy deps or features (cargo tree -d, cargo bloat if installed), splitting crates for parallelism, sccache, cranelift for dev, a persistent target dir for self-upgrade; macOS specifics: syspolicyd/Gatekeeper scanning new binaries, Spotlight on target/.
If a tool is missing (cargo-bloat, sccache, lld), stop and ask the manager; do not substitute (rule missing-tools).
Output: findings written on the ticket (the baseline numbers, a ranked list of changes with measured or expected saving, a recommendation). Follow-up builds are filed as their own tickets by whoever reads it, not built here.
Acceptance: the ticket has measured numbers for each build type and a ranked list; no source changes in the diff other than the ticket.
Model: Sonnet. Researcher role if the manager has one.

## Thread

### note · agent:manager-2 · 2026-10-07T10:54:14.538Z
br-npj2 (build-time research): cargo-bloat, sccache and lld/mold are not installed on this machine; the brief measures all three. May the worker install them (cargo install cargo-bloat; brew install sccache lld)? Recommend yes: they are standard dev tools and the research is incomplete without them. The worker will do the baseline timings meanwhile.

### note · agent:manager-2 · 2026-10-07T11:16:11.664Z
From orchestrator: machine load is high (5.4/core) so timings are noisy. Record the load (uptime) next to each timing, and redo the key cold and incremental runs when load is below about 1 per core.
