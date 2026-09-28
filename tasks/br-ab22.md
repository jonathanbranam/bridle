+++
id = "br-ab22"
title = "Components 2/3: components on tasks and spawns (y3sd)"
kind = "feature"
state = "planned"
created_at = "2026-09-28T22:30:03.573Z"
updated_at = "2026-09-28T22:30:10.195405Z"
+++

design: docs/design/components.md, section "Scope by the task or spawn, not the cwd";
ticket docs/questions/open/components-as-scoped-work-contexts-y3sd.md
part 2 of 3. Needs part 1 (the `[components]` config and ids) merged first.

Implement item (3) of the design's "Build now" list:
- An optional `components` list (component ids) on tasks: wire types `Task` and
  `NewTaskRequest` in crates/bridle-api/src/types.rs (all clients and the daemon together),
  the task store, and the state-branch task format (docs/design/storage.md; old records
  without the field must still load, and a rebuild from the state branch must round-trip it).
- `bridle task new --component <id>` (repeatable) and `bridle task edit` to set it; reject an
  unknown id; in a project that has components, print a one-line reminder when none is given.
  Never refuse (soft rule). Empty list means repo-wide.
- `bridle task list --component <id>` matches tasks naming that component or any descendant;
  `task show` displays the list. Naming a child implies its ancestors (no need to list both).
- `bridle spawn ... --component <id>` (repeatable), defaulting to the claimed task's list; the
  daemon records the list on the agent and passes `BRIDLE_COMPONENTS` (comma-separated) in the
  agent process env.
Docs: cli.md, storage.md, and the agent-host doc that lists the env vars.

Acceptance: `just check` passes; tests for round-trip through the store and state branch,
unknown-id rejection, descendant matching in list, spawn env var, and unchanged behaviour in
a project with no components.

Out of scope: prime output (part 3), any enforcement/blocking on missing components, a lint
for docs folders, per-component ticket lists.

Model: Sonnet (wire type + store + state branch + CLI).
