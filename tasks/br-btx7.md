+++
id = "br-btx7"
title = "A landing fails 'main moved' whenever docs are committed during its check"
kind = "bug"
state = "planned"
created_at = "2026-10-05T01:17:02.629Z"
updated_at = "2026-10-05T01:17:13.554362Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: btx7
docs/tickets/open/a-landing-fails-main-moved-whenever-docs-are-committed-durin-btx7.md

Goal: `bridle task land` doesn't fail "main moved" when, during its check, the integration branch gained only commits whose changed paths all match a project setting `[integration] check_skip_paths` (globs; bridle's own config: ["docs/**"]; default empty, so other projects behave as today). Then it merges them in and lands without re-running the check. Any other move still fails as today.
- Docs: wherever [integration] is documented (docs/design/agent-host/), operating-model.md ("Merging completed work"); bridle's .bridle/config.toml.
Acceptance: just check green; tests: main moved by a docs/ commit during the check -> lands; by a crates/ commit -> fails "main moved"; empty setting -> fails as today.
Model: sonnet.
Out of scope: holding other roles' commits; the doc-review watcher's commit cadence.
