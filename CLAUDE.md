# Bridle

Rust workspace for bridle: a daemon + CLI that runs and coordinates headless
Claude Code agents. Read these before changing behaviour:

- `docs/design/agent-host/`: the daemon, API and agent host, as built. **Source of truth for
  the code**, with `docs/design/cli.md` and `docs/design/storage.md` (the database). Keep them
  in step when behaviour changes.
- `docs/README.md`: the docs index and reading order. The rest of `docs/design/` mixes built
  and planned; each doc opens with a Status line saying which (built and in use, built but not
  wired in, planned). Don't treat a designed thing as working, or a planned one as a
  prerequisite, without checking that line or the code. `docs/proposal/build-order.md` says what's next.
- `docs/tickets/open/` and `docs/spikes/open/`: open tickets and spikes, one ticket each.
  File new ones by the conventions in `docs/README.md`. Known v1 bugs and gaps:
  `docs/tickets/open/v1-follow-ups-from-the-build-9c6e.md`.
- `docs/spikes/01-stream-json-findings.md`: verified Claude Code stream-json behaviour.
  Cite it rather than assuming how `claude` behaves.
- **If you are the human's orchestrator** (directing bridle's workforce on this repo):
  `bridle prime orchestrator` (the role, the newest handover note and the startup steps).

## Layout

```
crates/bridle-claude   stream-json client (no daemon knowledge)
crates/bridle-api      wire types (types.rs is the contract) + HTTP/SSE client + discovery
crates/bridle-daemon   store (SQLite), supervisor, containment, worktrees, config, axum server
crates/bridle-tui      `bridle tui` (bridle-api only)
crates/bridle-gateway  `bridle gateway`: the human web UI's API
crates/bridle-mail     `bridle mail run`: the email bridge
crates/bridle-spec     spec-file parser behind `bridle workflow spec`
crates/bridle          the `bridle` binary: clap CLI; `serve` runs the daemon
spikes/stream-json     spike 01 (standalone, excluded from the workspace; don't modify)
```

## Commands

```
just check        # fmt-check + clippy -D warnings + nextest: must pass before you're done
just fmt
just test         # cargo nextest run --workspace
just check-affected [base]  # local-only fast path: nextest for changed crates + their
                  # reverse-dep closure (vs. base, default merge-base with main); falls
                  # back to the full suite whenever it can't be sure. Never replaces
                  # `just check`, which always runs everything.
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
  `workflow/base/rules/`). See `workflow/base/rules/memory.none.md`.

## Build configuration

On Intel Macs (x86_64), binaries must be ad-hoc code-signed at link time to avoid crashes in macOS's
system policy daemon. `.cargo/config.toml` adds `rustflags = ["-C", "link-arg=-Wl,-adhoc_codesign"]`
for the x86_64-apple-darwin target. This applies to `cargo install`, test binaries, and worker builds.
The flag is a no-op on arm64 (which signs automatically). See `docs/tickets/resolved/sign-binaries-on-intel-macs-cs7x.md`.

Firewall "Allow" survival (p88z): an ad-hoc signature is a new identity on every build, so the
macOS application firewall re-asks after each rebuild. Run `just sign-setup` once per Mac (works over
SSH, asks for the login keychain password once) to create the self-signed "bridle local signing"
identity (override the name with `BRIDLE_SIGNING_IDENTITY`). Then `just install` and the daemon's
self-upgrade re-sign the installed binary with it; with no such identity (CI, other machines) the
ad-hoc signature stays. `bridle sign binary [path]` does the signing by hand. Plain `cargo build`
in a worktree stays ad-hoc: only the installed binary needs a stable identity.
