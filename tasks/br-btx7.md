+++
id = "br-btx7"
title = "A landing fails 'main moved' whenever docs are committed during its check"
kind = "bug"
state = "integrated"
created_at = "2026-10-05T01:17:02.629Z"
updated_at = "2026-10-05T05:14:53.083909Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/land-moved"
commit = "0a8573bca3de489eadffae6bcb22c8945d081cac"
summary = "Added [integration] check_skip_paths (globs, default empty; bridle's config: docs/**). In integrator::land, if the integration branch moved during the check only by commits whose changed paths all match, the squash is cherry-picked onto the new tip and lands without re-checking (conflict or other move -> 'moved, retry' as before). Small built-in glob matcher (*, **, ?), no new dependency. Tests in land_test.rs (docs commit lands, crates commit refused, empty setting refused) plus unit tests. Docs: operating-model.md, roles-and-config.md, CHANGELOG."
+++

original id: btx7
docs/tickets/open/a-landing-fails-main-moved-whenever-docs-are-committed-durin-btx7.md

Goal: `bridle task land` doesn't fail "main moved" when, during its check, the integration branch gained only commits whose changed paths all match a project setting `[integration] check_skip_paths` (globs; bridle's own config: ["docs/**"]; default empty, so other projects behave as today). Then it merges them in and lands without re-running the check. Any other move still fails as today.
- Docs: wherever [integration] is documented (docs/design/agent-host/), operating-model.md ("Merging completed work"); bridle's .bridle/config.toml.
Acceptance: just check green; tests: main moved by a docs/ commit during the check -> lands; by a crates/ commit -> fails "main moved"; empty setting -> fails as today.
Model: sonnet.
Out of scope: holding other roles' commits; the doc-review watcher's commit cadence.

## Thread

### note · agent:land-moved · 2026-10-05T04:46:15.531Z
done: [integration] check_skip_paths lets a landing survive docs-only moves of main; just check exit 0, 1199 tests run (band ref 1194), sha c1d0e122 (main merged)

### note · agent:manager-2 · 2026-10-05T05:12:22.705Z
integrated: 0a8573bca3de489eadffae6bcb22c8945d081cac (branch bridle/land-moved)

### note · agent:manager-2 · 2026-10-05T05:14:53.083Z
cleanup: removed agent land-moved, branch bridle/land-moved
