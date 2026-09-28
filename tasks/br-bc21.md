+++
id = "br-bc21"
title = "Harvest the onboarding survey's Base rows into workflow/base/rules/"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T16:42:02.250Z"
updated_at = "2026-09-28T22:21:06.869536Z"
+++

Part of onboarding data-contracts (docs/context/onboarding-data-contracts.md §3, §7 step 2;
go-ahead from the orchestrator/human 2026-09-28). The survey's §3 classification table
marks several rows "Base" -- guidance data-contracts already follows informally that
belongs in workflow/base/rules/ if not already covered there, since it's project-agnostic.

Brief: read the source docs at
/Volumes/Data/work/data-contracts-workspace/data-contracts/docs/workflow-instructions/
(read-only -- don't edit anything there) and pull these §3 "Base" rows (see the table
in docs/context/onboarding-data-contracts.md §3 for exact sourcing) into
workflow/base/rules/<id>.md, one file per rule, matching the existing convention
(workflow/base/rules/kiss.md, yagni.md, cost-of-not-doing.md, human-timezone.md,
memory.none.md are the precedents for tone and format):
- Cross-doc links: written from the repo root, wiki form preferred, no leading `/`,
  few `../`.
- Filing doesn't schedule work; only the human/manager's merge accepts it; the plan
  comes before code; a report is a claim, so verify it. (Check this isn't already
  covered by bridle's existing task-state/plan-gate model before adding a redundant rule
  -- if it's already true by construction, a short rule documenting the invariant is
  still useful, but don't duplicate mechanism docs.)
- Ask only blocking questions, roughly 2 a round, with a recommendation attached.
- Record decisions where they outlive the change that produced them.
- A proposal/plan names every file it may touch; a deferral is stated as "not yet" with
  a reason; design records rejected alternatives; each task states how to verify it.
- File out-of-scope findings rather than fixing them; never weaken a test to make it pass.

Skip anything the survey marks Proj, Py, Super, or OS -- those are project-specific,
pack-specific, already superseded by bridle, or OpenSpec/P3-only, respectively, and
don't belong in the base layer.

Acceptance: just check passes; each new rule file has a stable id and reads as a
standalone must/should statement, consistent with the existing rules/ files;
`bridle rules explain` (crates/bridle/src/cli.rs) lists the new rules.

Out of scope: anything not explicitly marked "Base" in the survey's §3 table; editing
data-contracts itself.

Model: Sonnet (judgment calls on wording/redundancy against existing bridle rules).

## Thread

### note · agent:pm-1 · 2026-09-28T22:21:06.869Z
dropped: Merged to main (b5ec903).
