+++
id = "br-awh4"
title = "Specs: step text can't quote code or markup"
kind = "feature"
state = "pending"
created_at = "2026-10-05T02:56:15.099Z"
updated_at = "2026-10-09T11:04:38.475721Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "awh4"
+++

docs/tickets/open/specs-step-text-can-t-quote-code-or-markup-awh4.md

submitted by external:orchestrator@nuc

bridle spec check rejects markup in step text (backticks, <code>, HTML), so a rendering spec can't quote the raw markdown it renders or the HTML it expects. The worker rephrased steps and normalised HTML quotes in step code instead, which makes scenarios less exact. Wanted: a way to carry literal text in a step (a quoted string or a docstring/table argument that isn't parsed as markup).

From the meta-notes-ui project (orchestrator, 2026-10-05), spec 3 (mu-pfgp, design/specs/rendering.md). Local log: meta-notes-ui ticket myeg (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:56:15.100Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:56:33.970Z
Triage (pm-1): accept. Real friction from meta-notes-ui, on the onboarding goal. Ticket minted: docs/tickets/open/specs-step-text-can-t-quote-code-or-markup-awh4.md (uncommitted; needs committing on main). Stays pending: the human or orchestrator approves with `bridle task ready br-awh4`, then I plan it (Sonnet, crates/bridle-spec).

### note · external:advisor/product-manager · 2026-10-09T11:04:38.475Z
watching the task
