---
id: br-q8cs
title: "User-facing rename: bridle ticket replaces bridle task (hidden alias), roles, rules, skills, docs, gateway routes"
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-22ab]
parent: br-22ab
resolved: 2026-10-11T11:04:16Z
---

## The ask

Parent: br-22ab (step 7 of its plan, approved by the human 2026-10-09 ~4:40 PM ET: "approve; looks great"). Its gate, "after steps 1-4", cleared on 2026-10-11 when br-72t9 landed (steps 1-4: br-syqn, br-epvy, br-wrmu, br-bpku, br-3v75, br-72t9). The design is 22ab section 9 ("The rename", user-facing first pass); read it, and sections 2, 3 and 10.

The human, 2026-10-11 ~12:30 AM ET, to advisor product-manager: "I'd love the everything is a ticket epic to land or get close by morning b/c I think it will improve the design and fix a lot of minor things; I don't want epics to drag on for too long."

Scope (user-facing only; the code keeps its names until step 9):

- CLI: every `bridle task <x>` has a `bridle ticket <x>` form (`list`, `show`, `claim`, `plan`, `drop`, `done`, `priority`, `kind`, `search`, `dep`/`link` as already built, ...). Where a `ticket` form already exists (`new`, `ready`, `link`, `comment`, `ask`, `answer`, `show`, `resolve`, `check`, `set`), keep it and fold in what the `task` form did. `bridle task` stays as a hidden alias for one release (step 10 removes it); it must keep working unchanged, including `--json` output, because scripts and the PdM's roadmap generator read it.
- Help text, CLI output, messages and event names people read say "ticket", not "task".
- Roles, rules and skills in `workflow/base/` (`bridle-worker`: "claim a task" becomes "claim a ticket"; the manager, project-manager, orchestrator, advisor and aide roles), and `docs/` (`docs/README.md`, `docs/design/cli.md`, `docs/design/agent-host/`, `storage.md` where it describes what users see). Rename once, against the commands as they now are.
- Gateway API (bridle-gateway): expose tickets under ticket names, and keep the old routes working for one release; bridle-ui reads the gateway, and its own change is step 8 in the bridle-ui project (not this ticket).

Out of scope: the code rename (types, `tasks` table, state branch `tasks/` folder: step 9); removing the alias (step 10); states (stx8); change parts, gates and approvals (95mu, moved to the gates epic).

Size L. If it is too big for one worker, split it into slices (CLI first, then roles/rules/skills/docs, then gateway) as child tickets with `parent: br-<this>`.

Acceptance: `just check`; `bridle task ...` still works as before (tests for a few aliased commands, `--json` unchanged); `bridle ticket check` reports no new problems.
