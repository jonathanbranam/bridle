+++
id = "br-m5kf"
title = "Specs: say what happens to unit tests a scenario now covers"
kind = "chore"
state = "planned"
created_at = "2026-10-05T02:51:25.940Z"
updated_at = "2026-10-18T10:19:06.995684Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "m5kf"
+++

docs/tickets/open/specs-say-what-happens-to-unit-tests-a-scenario-now-covers-m5kf.md

From meta-notes-ui's orchestrator (2026-10-05): when a project adopts specs, existing unit tests duplicate the new scenarios; one worker left them, the next removed them (keeping internals like tokenMatches). Add a short guidance paragraph, in the doc that tells workers how specs become tests (docs/design/specs-to-tests.md; and the specs workflow rule under workflow/base/rules/ if one covers writing tests against specs, read docs/design/spec-flow.md too): once an executable scenario covers a behaviour, remove the unit tests that only restate it; keep tests of internals and of edge cases below the spec's level; when unsure, keep the test and say so in the done summary. Short, matching the doc's voice, ASCII, no new rule file unless an existing rule is the natural home. Acceptance: just check passes. Model: Haiku. Out of scope: any code, tooling, or other specs questions (br-pakx, br-2d6x).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.943Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:52:28.873Z
Accepted as a small docs chore (no ticket needed): brief written. Pending until the orchestrator or the human readies it; then I plan it.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.439Z
watching the task

### note · external:advisor/product-manager · 2026-10-17T15:26:12.500Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:pm-1 · 2026-10-18T10:19:06.995Z
Brief (pm-1; the 10-05 brief did not survive the ticket move, so this is rewritten from the title). Model: Haiku/Sonnet, docs only.
Problem: when an executable scenario (a spec step run by vitest-bridle or another adapter) now covers behaviour that a hand-written unit test also covers, nothing says what to do with the unit test: keep, delete or reduce it. Reported by orchestrator@nuc.
Do: read docs/design/specs-to-tests.md and docs/briefs/specs.md, then add a short section (a few bullets) stating the rule. Recommended: a scenario covers the behaviour at the user-visible level, so a unit test is kept only if it covers an edge the scenario does not (a fast branch/error case, a pure function with many inputs); a unit test that only repeats the scenario's happy path is deleted in the same change that adds the scenario, and the commit says so. Check the existing docs for any stricter stance and follow it.
Acceptance: docs read cleanly, links root-relative, ASCII only; just check. CHANGELOG line (Internal). Out of scope: tooling to detect duplicate coverage.
