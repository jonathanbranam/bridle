# Bridle

Rust workspace for bridle: a daemon + CLI that runs and coordinates headless
Claude Code agents. Read these before changing behaviour:

- `docs/design/agent-host/`: the daemon, API and agent host, as built. **Source of truth for
  the code**, with `docs/design/cli.md` and `docs/design/storage.md` (the database). Keep them
  in step when behaviour changes.
- `docs/README.md`: the docs index and reading order. The rest of `docs/design/` (tasks,
  workflow layers, specs) is future work; `docs/proposal/build-order.md` says what's next.
- `docs/questions/open/` and `docs/spikes/open/`: open questions and spikes, one ticket each.
  File new ones by the conventions in `docs/README.md`. Known v1 bugs and gaps:
  `docs/questions/open/v1-follow-ups-from-the-build-9c6e.md`.
- `docs/spikes/01-stream-json-findings.md`: verified Claude Code stream-json behaviour.
  Cite it rather than assuming how `claude` behaves.

## Layout

```
crates/bridle-claude   stream-json client (no daemon knowledge)
crates/bridle-api      wire types (types.rs is the contract) + HTTP/SSE client + discovery
crates/bridle-daemon   store (SQLite), supervisor, containment, worktrees, config, axum server
crates/bridle          the `bridle` binary: clap CLI; `serve` runs the daemon
spikes/stream-json     spike 01 (standalone, excluded from the workspace; don't modify)
```

## Commands

```
just check        # fmt-check + clippy -D warnings + nextest: must pass before you're done
just fmt
just test         # cargo nextest run --workspace
just test-live    # ignored tests that run real `claude` (Haiku; costs tokens) — only when asked
just test-contract  # the Claude Code contract suite (live, ~$0.10), after Claude Code updates — only when asked
cargo test -p <crate>   # when working on one crate
```

## Conventions

- Edition 2024, stable toolchain. `unsafe` is forbidden (workspace lint).
- Errors: `anyhow` in binaries and the daemon's glue, `thiserror` for library error types
  that callers match on. No `unwrap()` outside tests. Use `expect("why")` only for true invariants.
- Async: tokio. Never block the runtime. Wrap SQLite and other blocking work in
  `spawn_blocking`, or keep it short under a mutex.
- Logging: `tracing`. No `println!` in libraries.
- Changing the wire format means changing `bridle-api/src/types.rs`, and all clients
  and the daemon together.
- Tests don't spawn real `claude` unless marked `#[ignore]` and gated on
  `BRIDLE_LIVE_TESTS=1`. Use the fake at `crates/bridle-claude/tests/fake-claude.py`.
- Comments explain why, not what. Match the density of the surrounding code.
- Don't commit unless asked.
- **No Claude Code memory**, ever: it's off in `.claude/settings.json`, and bridle turns it off for
  every agent it spawns. Record anything worth keeping in the repo (docs, tickets, or
  `.bridle/rules/`). See `.bridle/rules/memory.none.md`.
