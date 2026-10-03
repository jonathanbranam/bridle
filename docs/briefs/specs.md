# Brief: bridle specs

> **Status (checked 2026-10-03):** Built and in use: spec check, ids, export, import and the
> pytest adapter, on meta-notes (imported in `15bfb81`; `spec check --require-ids` in its check
> command), which answers decision 1 below · Built, not wired in: the vitest adapter, impact and
> conflicts (no role declares impact), traceability and `re-evaluate` (no project has goals or
> architecture files) · Planned: `trace coverage`, the protected-requirement gate.

As of 2026-09-29. Status words: **built** (in the code and CHANGELOG), **partly built**,
**planned** (design docs only).

## What specs are for

A spec is a markdown file that says what a part of your project must do, in a form a
program can read: a *capability* (a file) holds *requirements* ("The system SHALL ...")
and each requirement holds *scenarios* (GIVEN / WHEN / THEN). Scenarios marked
*executable* become tests; the rest are prose. Every requirement and scenario gets a
stable id (`r-7fa2`, `s-b310`) so tests, tasks and links can point at it and it never
changes when the wording does. This is OpenSpec's model with its change workflow
(proposals, deltas, archive) removed: a task edits the spec in place on its branch, and
merging the branch updates the spec.

## The flow, end to end

1. **Write** `design/specs/<capability>.md`. Format: [[docs/design/specs|specs]]. **Built.**
2. **Check**: `bridle spec check [--require-ids]` prints `file:line:col: message` and exits 1
   on an error. Local, no daemon. Put it in the project's check recipe. **Built.**
3. **Ids**: `bridle spec id` writes an id into each heading that lacks one, touching only
   those lines. Ids are never reused (a committed ledger, `design/specs/.ids`). `--dry-run`
   previews. **Built.**
4. **Tests**: `bridle spec export --format json` (or `gherkin`, one `.feature` per
   capability) hands scenarios to a test runner at collection time; nothing generated is
   committed. Two adapters read the json: a **pytest** plugin
   (`workflow/packs/python/adapters/bridle_specs.py`, pytest-bdd) and a **vitest** one
   (`workflow/packs/typescript/adapters/vitest-bridle/`). Test names carry the scenario id.
   Export can narrow to `--scenario ID` or `--task ID`. **Built**; both adapters are new
   and only tested against bridle's own fixtures, not a real project.
5. **Coverage**: `bridle spec coverage [--require-all]` lists executable scenarios whose id
   appears in no test file (a text search under `tests/` or `test/`). It checks that a test
   *mentions* the id, not that it passes or asserts the right thing. **Built.**
6. **Tasks**: `bridle impact set <task> --modify s-.. --files 'glob'` declares which spec
   ids and files a task will touch (stored on the task). `bridle impact check` compares
   in-flight tasks: two tasks changing one scenario is a *conflict* (exit 1); the same
   requirement is a warning; file globs are matched coarsely by prefix. Each conflict is
   recorded (`bridle conflict list|resolve`) and both workers are told. `bridle probe`
   dry-runs a branch's merge into the integration branch. On landing, workers whose declared
   impact was touched get "spec changed under you". **Built** (declared impact is trusted;
   it isn't checked against what the task actually changed, and ids aren't checked to exist).
7. **Traceability**: requirements can link up to architecture and goals
   (`traces=a-12cd@3f9e`, a short hash of the upstream text). `bridle trace down|up|orphans`
   walk the links; `trace suspect` lists links whose upstream text changed since; `trace confirm`
   accepts them. Landing an architecture-revision task opens a "re-evaluate" task per spec file
   with suspect requirements. **Built.** `trace coverage` is **planned**. Goals, architecture
   and `bridle explore` findings docs sit beside specs (`bridle goals|arch|explore`); you can
   ignore them for a first migration.
8. **Protected requirements**: `{#r-7fa2 protected}` is parsed and exported. Nothing yet sends
   a task touching one to a human plan gate. **Partly built** (marker only).

## Migrating a project from OpenSpec (e.g. meta-notes)

`bridle spec check --root openspec/specs`, then `bridle spec import openspec [--dry-run]`.
It `git mv`s each `openspec/specs/<cap>/spec.md` to `design/specs/<cap>.md`, assigns ids
and writes the ledger. It changes nothing if any spec has an error. Scenario wording and the
`*Verification*` markers carry over unchanged. **Built.**

Left for a person or a task: `openspec/changes/**`, config, generated `.feature` files and
OpenSpec's skills stay in place; active (unarchived) changes are **not** converted to tasks
(**planned**); the project needs to add the adapter to its test setup and add
`spec check --require-ids` to its checks. meta-notes is Vimscript plus Python: the pytest
adapter fits the Python side, but nothing exists for vader.vim tests, so its Vim scenarios
would stay unbound.

**Cost to the project:** moved spec files, an id on every heading, a `.ids` file, a
`.bridle/` directory and a config, a test-adapter hook, and dropping the OpenSpec CLI habit
(edit specs directly, in a task). Bridle's spec commands are all local; only impact and
conflicts need the daemon.

**What you'd review:** per `workflow/base/rules/existing-projects.md`, all of it happens on
a trial branch (meta-notes already has `bridle-adopt`); `main` is untouched until you
approve. Review the import commit (renames plus id edits; `git diff -M` shows it is
mechanical), the check and adapter wiring, and one real task run through the flow.

**What could go wrong:** the import's parser is strict, so a spec OpenSpec tolerated may
need fixing first (the dry run tells you). The adapters are young. Coverage is a text match,
so it can overstate. Protection and the plan gate aren't enforced, so nothing stops a task
editing a governing requirement except the manager's review. The project's `.feature`
generation scripts would run alongside the new export until removed.

## Where design docs disagree with the code

- `docs/design/spec-flow.md` "Not yet" lists `spec coverage` and the pytest/vitest adapters
  as unbuilt; both exist (CHANGELOG, br-b1e2, br-3b72, br-a54d). The same section's
  `bridle test --task` is not in `bridle --help`.
- `docs/design/specs-to-tests.md` and `docs/design/gates.md` describe the protected gate
  (`impact.protected`); nothing in the code reads `protected` beyond parsing and export.
- `docs/design/traceability.md` lists `trace coverage` as unbuilt; correct.

## Decisions for you

1. Migrate meta-notes on its `bridle-adopt` branch as the trial, or wait for the adapter to
   be proven on another project?
2. Accept that meta-notes' Vim scenarios stay unbound (no vader adapter), or make one a
   precondition?
3. Do you want the protected-requirement gate built before a real project relies on specs?
4. Convert meta-notes' active OpenSpec changes by hand, or finish them in OpenSpec first?
