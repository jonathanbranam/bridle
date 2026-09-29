+++
id = "br-2f6a"
title = "Architecture parser and bridle arch list"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T04:01:16.337Z"
updated_at = "2026-09-29T04:31:22.102279Z"
branch = "bridle/arch-list"
commit = "d605f6d"
summary = "Added bridle_spec::arch (arch.rs): parses '## Title {#a-xxxx [invariant]}' elements, keeps an '**Alternatives rejected:**' paragraph separately, errors on missing/malformed/duplicate ids (within and across files). Reuses Diagnostic and made col/offset_in/is_id pub(crate) in parse.rs; the arch heading block is parsed in arch.rs since spec attrs are tied to the spec Parser. CLI: 'bridle arch list [--invariants] [--root DIR] [--json]' (default design/architecture, local). Docs: architecture-tier.md, cli.md."
+++

Goal: parse architecture elements (docs/design/architecture-tier.md) in bridle-spec and add `bridle arch list [--invariants] [--root DIR] [--json]` (local, no daemon; default root design/architecture).

Format: an element is a heading '## Title {#a-12cd}' or '## Title {#a-12cd invariant}', body follows; a decision element may hold an '**Alternatives rejected:**' paragraph (kept as text). Reuse Diagnostic and the {#id ...} block parsing from crates/bridle-spec/src/parse.rs; generalise the flag handling minimally if it is spec-specific. Duplicate ids within or across files = error; missing id = error. a- ids are hand-written, not assigned.

Files: crates/bridle-spec/src/arch.rs (new; export from lib.rs), crates/bridle/src/cli.rs and commands.rs, docs/design/architecture-tier.md (add format/built section), cli.md.

Acceptance: `just check` passes; unit tests for id/invariant parsing, duplicate ids, missing id, fixture-driven CLI test. Model: Sonnet.

Out of scope: `arch propose`, the PreToolUse enforcement hook, integrator refusal, revision flow (P4). Same files as the goals-list task (lib.rs, cli.rs): run after it, not in parallel.

## Thread

### note · agent:manager-2 · 2026-09-29T04:31:22.102Z
integrated: d605f6d (branch bridle/arch-list)
