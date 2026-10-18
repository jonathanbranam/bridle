+++
id = "br-fyww"
title = "Stages: landing a designer's stage branch marked the whole v2 ticket integrated"
kind = "bug"
state = "open"
created_at = "2026-10-18T16:33:40.386Z"
updated_at = "2026-10-18T16:34:25.731774Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
priority = "urgent"
priority_at = "2026-10-18T16:33:45.986562Z"
ticket = "fyww"
+++

docs/tickets/open/stages-landing-a-designer-s-stage-branch-marked-the-whole-v2-fyww.md

## Thread

### note · external:advisor/tj7q · 2026-10-18T16:33:45.986Z
priority: normal -> urgent

### note · agent:pm-1 · 2026-10-18T16:34:25.731Z
Brief (pm-1). Model: Sonnet. Read docs/design/stages.md, the task state-transition code in crates/bridle-daemon (grep integrated, the land/integrate path used by `bridle ticket`/`bridle task` and the manager's land step), and br-upxq's Decision (the daemon will merge stage branches itself).
Build: the daemon refuses any transition to `integrated` for a version 2 ticket that is not prepared and built (a clear error naming why); the manager's land path hits the same guard so it cannot mark a proposal-only branch integrated. Then update the manager land rule/step text to say designer stage branches don't change the state (check workflow/base/ rules and roles for the land step; keep edits minimal).
Test: landing a proposal-only branch of a pending v2 ticket leaves it `pending`; a normal v1 ticket still integrates. Docs: stages.md, CHANGELOG. Acceptance: just check.
Out of scope: br-upxq's merge-by-daemon (separate ticket); a `pending` return path via reopen (br-2ae8 was fixed by hand).
Same files as br-upxq: run after it, not alongside.
