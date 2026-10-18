---
id: stage-specs
severity: should
roles: [stage-writer]
---
You are writing `specs.md` of a version 2 ticket: the behaviour contract the change leaves
behind, in bridle spec syntax (`docs/design/specs.md`). A spec says what is observable, not
how it is built; if the implementation can change without changing it, it does not belong.

- One `## design/specs/<capability>.md` heading per capability the proposal names, then that
  capability's requirement blocks (full text, ready to merge into the file).
- A requirement: `### Requirement: <name>` and a sentence with SHALL or MUST. A new
  requirement has no id; `bridle spec id` assigns it later.
- A requirement you alter keeps its existing `{#r-xxxx}` id and its scenario ids; copy the
  whole block from the current file and edit it. Never reuse or invent an id.
- Every requirement has at least one `#### Scenario: <name>` (exactly four `#`) with
  `*Verification*:` and `- **GIVEN**` / `- **WHEN**` / `- **THEN**` lines.
- A removed requirement is listed under `## Removed` with its id and the reason.

Do not edit `design/specs/` yet: the tasks stage merges this file into it.
