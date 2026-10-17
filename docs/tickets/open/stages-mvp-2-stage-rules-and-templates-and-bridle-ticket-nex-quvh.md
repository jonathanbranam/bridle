---
id: br-quvh
title: "Stages MVP 2: stage rules and templates, and bridle ticket next"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-9mqh]
related: []
parent: br-tj7q
theme: product-process
---

## The ask

Step 2 of 7 of the MVP of epic br-tj7q. The human, 2026-10-17: "When we tell an agent it is time to
write the proposal, we give it all the instructions for writing a proposal and the template right
there, every single time." Instructions are layered bridle rules; the template ships with bridle.

---

## Proposal

### Why

The writer of each stage must get that stage's instructions and template at the moment it writes,
the same for a background or an interactive agent, not from its role prompt or CLAUDE.md.

### What changes

- Four stage rules in `workflow/base/rules/` (`stage-proposal`, `stage-design`, `stage-specs`,
  `stage-tasks`), each holding the instructions, and a template file per stage beside them,
  layered like rules (base, packs, project). First drafts, from OpenSpec's spec-driven schema
  (`/Volumes/Data/work/bridle/OpenSpec/schemas/spec-driven/`) and the epic's design section:
  - proposal: Why; What changes; What does not change; Risk and impact; Out of scope; Open
    questions. Written into the body between `---` lines (step 1's delimiters).
  - design: Problem; What exists today; Options (user view, internals, cost, shortfall);
    Recommendation; Rejected.
  - specs: per capability, a `## design/specs/<capability>.md` heading and requirement blocks in
    bridle spec syntax (docs/design/specs.md), existing ids kept on altered requirements.
  - tasks: an ordered checklist, each item one step with its check; the first item is always
    "merge specs.md into design/specs, run `bridle spec id` and `bridle spec check`".
- `bridle ticket next <id>`: prints the next unblocked stage of a version 2 ticket with its
  instructions (resolved rule), template, output path and the files it depends on (as OpenSpec's
  `openspec instructions`). With `--comments`, also the human's comments since the stage was last
  written (for a redo). Refuses when the next stage waits on an unsigned gate; says "prepared"
  when none is left. `--json` for bridle's own use in step 4.

### What does not change

Rules delivered to roles today; version 1 tickets.

### Risk and impact

Low: new files and a read-only command.

### Out of scope

Checking a written file against its template (V1).

### Open questions

None. The template wording is reviewed as rules in the PR.

---
