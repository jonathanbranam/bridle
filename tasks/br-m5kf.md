+++
id = "br-m5kf"
title = "Specs: say what happens to unit tests a scenario now covers"
kind = "chore"
state = "pending"
created_at = "2026-10-05T02:51:25.940Z"
updated_at = "2026-10-09T11:04:38.439965Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
+++

From meta-notes-ui's orchestrator (2026-10-05): when a project adopts specs, existing unit tests duplicate the new scenarios; one worker left them, the next removed them (keeping internals like tokenMatches). Add a short guidance paragraph, in the doc that tells workers how specs become tests (docs/design/specs-to-tests.md; and the specs workflow rule under workflow/base/rules/ if one covers writing tests against specs, read docs/design/spec-flow.md too): once an executable scenario covers a behaviour, remove the unit tests that only restate it; keep tests of internals and of edge cases below the spec's level; when unsure, keep the test and say so in the done summary. Short, matching the doc's voice, ASCII, no new rule file unless an existing rule is the natural home. Acceptance: just check passes. Model: Haiku. Out of scope: any code, tooling, or other specs questions (br-pakx, br-2d6x).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.943Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:52:28.873Z
Accepted as a small docs chore (no ticket needed): brief written. Pending until the orchestrator or the human readies it; then I plan it.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.439Z
watching the task
