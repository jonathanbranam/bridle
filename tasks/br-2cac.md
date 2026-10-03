+++
id = "br-2cac"
title = "bridle spec import openspec: move OpenSpec specs to design/specs and assign ids"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T04:01:04.813Z"
updated_at = "2026-09-29T04:12:02.975942Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/spec-import"
commit = "1f645fa"
summary = "Added 'bridle spec import openspec [--from] [--to] [--dry-run]' (crates/bridle/src/spec_import.rs): parses all <from>/<cap>/spec.md first, refuses on parse error or existing target, git mv (rename if untracked) to <to>/<cap>.md, then reuses specid::assign with <to>/.ids ledger. .feature and other openspec files left, with a note. Idempotent. Tests use a tempdir tree; docs in specs.md and cli.md updated."
+++

Goal: `bridle spec import openspec [--from openspec/specs] [--to design/specs] [--dry-run]` (P3; docs/design/specs.md 'Migration from OpenSpec'). Local, no daemon call, like `spec check`/`spec id`.

Behaviour: for each <from>/<capability>/spec.md, git mv it to <to>/<capability>.md (plain rename outside a git repo; refuse if the target exists), then run the existing id assignment (crates/bridle/src/specid.rs, reuse its function, ledger <to>/.ids included) over the moved files. Leave generated <cap>.feature files, openspec/changes/**, config.yaml, schemas and skills alone (print a one-line note of what was left). Parse the source specs with bridle-spec first; on any parse error change nothing. Idempotent: a second run with nothing to move is a no-op. The *Verification* marker grammar is unchanged.

Files: crates/bridle/src/cli.rs (new `spec import openspec` subcommand), commands.rs or a new spec_import.rs beside spec_export.rs/specid.rs, docs/design/specs.md (mark built, document flags), docs/design/cli.md.

Acceptance: `just check` passes; tests use a tempdir with a fake openspec tree (2 capabilities, one already with ids) and assert moved paths, ids assigned, .feature left in place, dry-run writes nothing, parse error changes nothing. Model: Sonnet.

Out of scope: converting active changes to tasks (data-contracts has none; YAGNI), archived changes, touching any real project's repo, deleting openspec skills/CLI.

## Thread

### note · agent:manager-2 · 2026-09-29T04:12:02.975Z
integrated: 1f645fa (branch bridle/spec-import)
