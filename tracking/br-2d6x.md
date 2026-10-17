+++
id = "br-2d6x"
title = "Specs: vitest-bridle setup leaves step files untypechecked"
kind = "chore"
state = "open"
created_at = "2026-10-05T02:51:25.805Z"
updated_at = "2026-10-17T15:26:12.295426Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "2d6x"
+++

docs/tickets/open/specs-vitest-bridle-setup-leaves-step-files-untypechecked-2d6x.md

From meta-notes-ui's orchestrator (2026-10-05), adopting bridle specs with vendored vitest-bridle. vitest only transpiles, so the spec entry (specs.test.ts) and its step definitions are not covered by tsc: a typo in a step's types passes until it runs. Fix in the vitest-bridle README (find it under tools/vitest-bridle in this repo): the setup section gives a tsconfig fragment that includes the spec entry and step files, and a typecheck line (tsc --noEmit -p <that tsconfig>) to add to the project's check script. If vitest-bridle has an example project or template in the repo, update it the same way, and make sure the example typechecks. Docs only, plus the example if present. Acceptance: just check passes; if there is an example with a package script, run its typecheck. Model: Haiku. Out of scope: distribution, CI and export format (ticket pakx, br-pakx).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.821Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:52:10.010Z
Accepted as a small docs chore (no ticket needed): brief written. Pending until the orchestrator or the human readies it; then I plan it. Related to the specs-distribution ticket pakx, but independent.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.381Z
watching the task

### note · external:advisor/product-manager · 2026-10-17T15:26:12.064Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.
