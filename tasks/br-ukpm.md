+++
id = "br-ukpm"
title = "A designer role: a background agent that reads a problem ticket, analyses the system and writes design options into the ticket"
kind = "feature"
state = "planned"
created_at = "2026-10-06T01:21:56.084Z"
updated_at = "2026-10-06T01:22:20.399229Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: ukpm
docs/tickets/open/a-designer-role-a-background-agent-that-reads-a-problem-tick-ukpm.md
Ticket (the ask with the human's words; read all of it):
Goal: a base-workflow role `designer` (workflow/base/roles/designer.md, plus any rules it needs under workflow/base/rules) for a daemon-spawned background agent. Input: one existing ticket describing a problem. Work: read it, analyse the current system (code, CLI, docs), then write options (several), a recommendation and the trade-offs, measured against stated design principles. Output: the proposal written INTO THE TICKET (the human: "I don't want a design document. I want a ticket."), committed; no design doc, nothing built; the designer's task ends when the proposal is in the ticket.
Principles, asserted strongly in the role: KISS (kiss.md), YAGNI (yagni.md), modularity, one name per action and no near-duplicate commands (ticket fne2), design read from the user's side first, plus a clearly marked place (a section or a separate rule file, e.g. workflow/base/rules/design-principles.md) where the human adds their own. Reuse the existing rules by reference, don't copy them.
Decide, and record in the role doc with the rejected alternative: one role with a focus (API/CLI surface vs internal architecture) or two roles. PM recommendation (YAGNI): ONE role, designer, with a "focus" section for each of the two sides, so the human can later split it if the prompts diverge; the human leans toward possibly two, so make the split a cheap rename.
State how designer differs from project-manager (plans and sizes work, doesn't analyse deeply or propose designs) and prototyper (builds a throwaway to learn; designer builds nothing). Use the prototyper role (ticket 6yb4) as the model for how a new base role is defined, registered and spawned; follow whatever it needed (role list, docs/README or roles index, the workflow's role table, tests that enumerate roles). Say in the done note how a project gets the new role (workflow sync) and that no per-project file changes are needed.
THE HUMAN REVIEWS the role and its rules BEFORE it is used: finish with a done note that lists the files and the key decisions for review, and the manager must not land it until the human approves (the orchestrator holds the landing; a worker finishing is not the work landing). Do not run the designer on any ticket; its first job (fne2) comes after the review.
Files likely: workflow/base/roles/designer.md, workflow/base/rules/*, wherever roles are listed/registered (grep prototyper), docs/design/workflow-layers.md or the roles doc, CHANGELOG.
Acceptance: just check passes (role-enumerating tests included); the role reads cleanly start to finish; ASCII only in docs the human edits.
Model: Sonnet. Out of scope: running it, the fne2 design itself, changing other roles.
