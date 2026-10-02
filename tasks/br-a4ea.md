+++
id = "br-a4ea"
title = "A prototyper role in the base workflow: build only from the prototype prompt's constraints"
kind = "feature"
state = "planned"
created_at = "2026-10-02T12:30:27.911Z"
updated_at = "2026-10-02T12:31:28.673943Z"
+++

original id: 6yb4
Ticket: docs/tickets/open/a-prototyper-role-in-the-base-workflow-build-only-from-the-p-6yb4.md (read it all: the human's words verbatim and 'The ask' 1-4). Code and docs to read first: workflow/base/roles/ (worker.md, advisor.md as examples of a role file), how roles are discovered, validated and given defaults (docs/design/agent-host/roles-and-config.md, crates/bridle-daemon config and role loading, 'bridle prime <role>'), how 'bridle spawn --role' picks a role, .bridle/roles/ project overrides (append to base).

Goal: a 'prototyper' role in the base workflow, separate from the worker, that every project gets.
1. workflow/base/roles/prototyper.md. Strong guidance, written for an agent that tends to explore: the prototype prompt is the WHOLE brief. Read only what the prompt's constraints name: 'don't worry about the current implementation' means do not look at it; 'must work with our existing database schema' means read the schema and only that. Do not read the codebase or design docs to 'understand the system'. Don't worry about the database, API or other layers the prompt doesn't name. Rethink the design when asked. When asked for more than one prototype, each takes a genuinely different approach (state the approach in one line first, check it differs from the others', not variations on one design). Where prototypes live is the project's choice: the role says to use the location the prompt names, or to ask (one question) if none is named; it does not decide one. Keep it short and strongly worded, in the style of the other role files.
2. Wire it as a first-class role where roles are listed (defaults, 'bridle prime prototyper', spawnable by role name, any role table or docs list), with a sensible default permission/model profile like the worker's. Instructions only for now (YAGNI): do NOT build a sparse/empty worktree or read-restricted sandbox; note that idea as a follow-up in the ticket.
3. Docs: roles-and-config.md and the workflow role list, CHANGELOG, a 'Built' note in the ticket.
SAFETY (the human is away): adding a role must not be able to stop the daemon starting. Add a test that loads the REAL workflow/base roles directory (including the new file) through the daemon's role-loading and validation path and succeeds, and a test that a project's .bridle/roles/prototyper.md append works. If any role validation can fail on the new file, it must degrade to a warning, not a startup failure. Do not touch daemon start-up, restart/upgrade or the orchestrator path. Lands normally.
Tests: role loads; 'bridle prime prototyper' prints the text containing the key rules (prompt-is-the-brief, differ, project decides location); spawn by role name resolves it; project append works.
Acceptance: just check passes. Model: Sonnet. Migration plan: none needed: a new base role file; projects inherit it from the workflow and nothing in their files changes. Out of scope: sandboxing, per-project prototype locations, the web UI (br-1665).
