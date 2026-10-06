+++
id = "br-6hx4"
title = "Binaries accept unknown config sections and keys with a warning; doctor warns, and fails with a strictness flag"
kind = "feature"
state = "planned"
created_at = "2026-10-06T22:20:29.388Z"
updated_at = "2026-10-06T22:21:29.556773Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 6hx4
Ticket (the human's words; read all of it): docs/tickets/open/binaries-accept-unknown-config-sections-and-keys-with-a-warn-6hx4.md . Related: jmpf, 2ax5 (a [gateway] section stopped every daemon), 95mu (exact names before building).
Goal: an older binary reads a config file that has sections or keys a newer one added, runs, and warns; `bridle daemon doctor` reports them and can fail on them.

EXACT NAMES (settled here; do not rename):
- Doctor flag: `bridle daemon doctor --strict`. Default (no flag): unknown sections/keys are WARNINGS, doctor exit code stays as today (0 if no failures). With `--strict`: any warning of this kind makes doctor exit 1. `--strict` covers only these config warnings plus whatever doctor already calls a warning; do not reclassify existing failures.
- Warning text, one line per finding, exactly:
  unknown section: `<file>: unknown section [<section>] (unknown to this build (bridle <version>); a newer build may use it)`
  unknown key: `<file>: unknown key <section>.<key> (unknown to this build (bridle <version>); a newer build may use it)`
  typo of a known key (see below): the unknown-key line followed by ` - did you mean <known_key>?` before nothing else, i.e. `...may use it) - did you mean <known_key>?`
  <file> is the path as read; <section> is the dotted table path (e.g. `budget`, `projects.bridle`); <version> is the binary's CARGO_PKG_VERSION.
- Where printed: the daemon logs each finding with tracing::warn! at start-up and at every config reload (not repeated if the reload finds the same set); the CLI prints each finding to stderr ONCE per process run, on the first config load, not per output line; doctor prints them in its report under the heading "config warnings".
- Typos: an unknown key whose name is within edit distance 2 of a known key in the SAME section is a typo candidate and gets the "did you mean" suffix (this is how `max_worker` for `max_workers` is still caught: it is reported on every run, in doctor and in the daemon log, and --strict fails on it). Unknown keys with no close match get no suffix. Typos are warnings, not load errors, like every other unknown key. Wrong TYPES and missing required keys for known keys stay hard errors as today.
- Files: config.toml (machine and project: crates/bridle-daemon/src/config.rs, 6 deny_unknown_fields), crates/bridle-api/src/machines.rs, crates/bridle-mail/src/config.rs, crates/bridle-gateway/src/config.rs (2 places), crates/bridle-daemon/src/focus.rs (1). credentials.toml is already lenient: leave it. workflow.toml: only if it currently uses deny_unknown_fields; say in the done note what you found.
Build: remove `deny_unknown_fields` from those structs and collect unknown paths instead (preferred: the `serde_ignored` crate, adding it to the workspace Cargo.toml, so the check uses the real structs and cannot drift; or, if you find a simpler way that uses the real structs, say why). One shared helper (place it where the daemon and bridle-mail can both use it, e.g. bridle-api) returns Vec of findings; the loaders call it. Document in docs/design/cli.md (doctor flag), docs/design/agent-host/daemon.md or wherever config is described (the behaviour and the warning text), and the config docs.
Migration: none needed (no stored state; behaviour only).
Acceptance: just check passes; tests: (1) an OLD-SHAPED config struct (a test-local copy of a section without the newer field, built the way the real structs are) reads a NEWER file with an extra section and an extra key, loads successfully and yields exactly the two findings with the exact text above; (2) a typo key `max_worker` yields the did-you-mean finding; (3) a known key with the wrong type still errors; (4) doctor: default exit 0 with warnings printed, `--strict` exit 1; (5) the CLI prints findings once per run.
Model: Sonnet. Out of scope: credentials.toml, changing what any known key means, a config-versioning scheme, auto-fixing typos.
