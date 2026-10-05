+++
id = "br-2d6x"
title = "Specs: vitest-bridle setup leaves step files untypechecked"
kind = "chore"
state = "pending"
created_at = "2026-10-05T02:51:25.805Z"
updated_at = "2026-10-05T02:51:25.821672Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

In meta-notes-ui the spec entry (specs.test.ts) and its step definitions aren't covered by tsc (vitest only transpiles), so a typo in a step's types passes until it runs. The vitest-bridle README's setup could include the tsconfig and a typecheck in the check script, so every adopting project gets it.

From the meta-notes-ui project (orchestrator, 2026-10-05), adopting bridle specs with vendored vitest-bridle (mu-943b, mu-hzf9). Local log: meta-notes-ui docs/tickets/open (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.821Z
submitted by external:orchestrator@nuc
