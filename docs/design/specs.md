# Specs: keep the model, replace the lifecycle

## What stays

Capability → requirement (`SHALL`) → scenario (`GIVEN/WHEN/THEN`), with the
verification marker from the current Gherkin tooling. Research 10 §2a covers why
this part works.

## What changes

1. **Stable ids** on requirements and scenarios, assigned by `bridle spec id`
   and never reused:

   ```markdown
   ### Requirement: The engine referees every rule            {#r-7fa2}
   #### Scenario: Reach comes from the engine                  {#s-b310}
   *Verification*: **executable** @engine
   - **GIVEN** a unit with reach 2
   - **WHEN** the bench asks for legal targets
   - **THEN** the engine's answer is used unchanged
   ```

2. **No deltas.** A task edits `design/specs/<capability>.md` **in place on its
   branch**. The plan commit's spec diff is the proposal. Merging the branch
   updates the spec, and there is no separate archive step.
3. **`proposal.md` / `design.md` / `tasks.md` become the task record.** The
   task body holds why and what, the plan section holds decisions and the
   alternatives considered, and checklist items are subtasks or a checklist in
   the body. The spec is the only artifact that outlives the task.
4. **Protection is per requirement.** `{#r-7fa2 protected}` means any task that
   modifies it goes through the human plan gate ([[docs/design/gates|gates]]). The governing invariants
   get this marker; most requirements don't.

## Migration from OpenSpec

`bridle import openspec` handles it once per repo: move `openspec/specs/*/spec.md`
to `design/specs/<capability>.md`, assign ids, convert active changes to tasks (their
delta specs are applied on a task branch), and leave archived changes in git
history without converting them. The current `*Verification*` marker grammar is
kept, so data-contracts' scenarios carry over unchanged.
