+++
id = "br-ukpm"
title = "A designer role: a background agent that reads a problem ticket, analyses the system and writes design options into the ticket"
kind = "feature"
state = "planned"
created_at = "2026-10-06T01:21:56.084Z"
updated_at = "2026-10-06T02:54:25.042509Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = """
Added the built-in `designer` role, modelled on prototyper. Files: workflow/base/roles/designer.md (new; the role), workflow/base/rules/design-principles.md (new; KISS/YAGNI by reference, modularity, one name per action, user's side first, and an empty section "The human's own principles" for the human to fill), `designer` added to the `roles:` of 20 existing rules (kiss, yagni, tickets, ticket-references, talk-on-the-task, record-decisions, work-flow, shell-zsh, no-kill-by-name, ascii, etc.; reused by reference, not copied), config.rs (default role entry on worker defaults, preamble sentence, project append `.bridle/roles/designer.md`), cli.rs and commands/orchestrator.rs (`bridle prime designer`), tests (designer_role_test.rs, prime_designer_test.rs), docs (README, cli.md, workflow-layers, roles-and-config, cli/roles, cli/priming-and-rules) and CHANGELOG.
Decisions for review: (1) ONE role with two focus sections (Interface; Internal architecture), per the PM's YAGNI recommendation; rejected: two roles now. Splitting later is a copy to designer-interface.md/designer-architecture.md plus a role entry each (the role doc says so). (2) Output goes under `## Design options` in the ticket body; no design doc, nothing built; task ends when the proposal is committed. (3) Differs from project-manager (plans/sizes, no design) and prototyper (throwaway build, ignores current design; designer reads it closely, builds nothing). (4) Doing nothing is allowed as an option. (5) design-principles.md is severity must, roles [designer] only.
A project gets it with `bridle workflow sync`/the base workflow; no per-project file changes (an optional .bridle/roles/designer.md is appended). Not run on any ticket. just check: exit 0, 1262 tests passed. Merged main (tickets only)."""
ticket = "ukpm"
+++

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

## Thread

### note · agent:designer-role · 2026-10-06T02:54:25.042Z
done: designer role (one role, two focus sections) + design-principles rule, wiring, tests, docs. just check exit 0, 1262 tests passed; commit 9b730ed2. Summary is on the task. Awaiting the human's review; do not land yet.
