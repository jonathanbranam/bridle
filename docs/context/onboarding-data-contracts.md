# Onboarding data-contracts onto bridle: survey

Read-only survey, 2026-09-28. Nothing in data-contracts was changed. `DC` =
`/Volumes/Data/work/data-contracts-workspace/data-contracts` (main checkout, clean, at
`7b77ab1`); `BR` = the sibling worktree
`/Volumes/Data/work/data-contracts-workspace/adopt-branch-per-change-workflow`.

## 1. Summary

- data-contracts runs its own hand-built workflow: a driver / apply-agent / human process
  (`DC/docs/workflow-instructions/`), a markdown ticket pipeline (`DC/docs/tickets/`), and
  OpenSpec with a custom schema for proposals, specs, design and tasks. Bridle already replaces
  most of this, or plans to.
- The part that matters, and doesn't depend on OpenSpec, is the spec→Gherkin→pytest-bdd
  pipeline. It's plain Python that reads `openspec/specs/**/spec.md` by path and never calls the
  `openspec` CLI. So the CLI and the 10 skills can go now, and the spec files can stay where they
  are until P3.
- 9 open tickets are about the workflow, not the library. 6 of them (vf32, sjkw, zm6n, 6tps,
  mv9p, 992c) are resolved or made obsolete by bridle. The rest are P3 input (22s2 and the
  spec-to-feature usability cluster) or Python-pack concerns (jydf).
- The in-flight branch plans "one branch and worktree per change, the apply agent commits". That
  is bridle's worker model. Recommend dropping the branch: keep its verified git findings and don't
  merge it.
- The untracked `.claude/settings.json` **doesn't exist** in either checkout today. On the branch
  it was planned as `permissions.additionalDirectories: [".."]` (D11). Bridle's workers don't need
  it.

## 2. What data-contracts has today

**Agent setup.**
- `DC/CLAUDE.md` (183 lines) has these sections:
  - Orientation: a doc map. `docs/plan-of-record.md` is "read first".
  - No assistant memory: the same rule as bridle's `workflow/base/rules/memory.none.md`, plus a list of where durable things go.
  - How work is planned and run: file tickets, don't schedule; the plan is committed first; only the human lands work; run `check-tickets.py` after every move.
  - Cross-document links: written from the repo root, wiki form preferred, no leading `/`, few `../`.
  - Tooling: Python ≥3.12 is the supported floor, developed on 3.14; uv; pytest; ruff.
  - Change process: use OpenSpec. It has a skill table and the `*Verification*` marker grammar.
  - Driver workflow: three commit checkpoints (plan, unreviewed implementation, land).
- `DC/.claude/agents/`:
  - `apply-agent.md` (sonnet): implements one approved change. Hard rule: never commit, leave the tree dirty.
  - `acceptance-verifier.md` (sonnet): runs a checklist from a scratch consumer and reports raw observations without judging them.
  - Templates of both sit in `docs/workflow-instructions/agents/`, deliberately not byte-identical.
- `DC/.claude/skills/`: 10 stock OpenSpec skills (`generatedBy: 1.12.0`, `allowed-tools: Bash(openspec:*)`): `opsx-explore`, `new-change`, `continue-change`, `ff-change`, `propose-change`, `apply-change`, `verify-change`, `sync-specs`, `archive-change`, `bulk-archive-change`.
- Other contents of `DC/.claude/`:
  - `worktrees/adopt-branch-per-change-workflow/` is an **orphaned** pre-move copy. It's ignored by git, and its `.git` points at `/Volumes/Data/work/data-contracts/.git/...`, which no longer exists.
  - There is no `settings.json` or `settings.local.json`. The `.claude/` directory's mtime is 2026-09-28 12:27, so something there was probably removed today.

**Ticket pipeline** (`DC/docs/workflow-instructions/README.md`, `ticket-conventions.md`,
`maintenance.md`).
- One file per ticket, named `<slug>-<4-char id>.md`, with frontmatter fields `id`, `title`, `opened`, `repos`, `changes`, `specs`, `needs`, `see`.
- The folders are the states:
  - `intake/{bugs,usability,questions,research,features}`
  - `backlog/`
  - `active/` (means a change exists)
  - `archive/{landed,dropped}`
- Each move is a `git mv`, made by these verbs: file, promote (only at a planning session), schedule, land (only on the human's approval), drop.
- Tooling: `tools/new-ticket.sh` creates a ticket. `tools/check-tickets.py` (384 lines) checks ids and links. Both are vendored twice: `docs/workflow-instructions/scripts/` holds a byte-identical copy.
- Counts:
  - intake: 18 tickets. That's 17 open plus 992c, which is done but was never moved.
  - backlog: 4 tickets.
  - active: none.
  - archive: 17 landed, 2 dropped.
- There has never been a triage session (`DC/docs/plan-of-record.md` §2).
- `DC/docs/plan-of-record.md` (791 lines) holds orientation, state, next steps and an append-only list of dated decisions.

**OpenSpec usage.**
- `DC/openspec/config.yaml`:
  - `schema: spec-driven-gherkin`.
  - A `context:` block: what the library is, the governing invariant, principles, the 3.12 floor, and "openspec/specs/ is authoritative and normative". It has a no-state rule, recorded in the plan-of-record decisions on 2026-09-07.
  - `rules.proposal`: Impact names every file the change may touch; deferrals are stated as "not yet" with a reason.
  - `rules.specs`: the marker grammar and tags registered in pyproject; regenerate the `.feature` after editing; reuse steps (`gherkin-steps.py --list`).
  - `rules.design`: record rejected alternatives.
  - `rules.tasks`: each task says how to verify it; the final task presents the work and stops.
  - `operations.apply`: don't commit; file anything out of scope; never weaken a test; the durable feature is expected to go red mid-change.
  - `operations.archive`: regenerate features after folding deltas.
- `DC/openspec/schemas/spec-driven-gherkin/`: `schema.yaml` plus templates. The flow is proposal → specs → design → tasks, and `skip_specs: true` marks a tooling-only change.
- `DC/openspec/specs/`: 6 capabilities: contract-model, verification, verification-engine, derivation-model, derivation-engine, derivation. That's 56 requirements, each `spec.md` with a generated `<cap>.feature` beside it.
- `DC/openspec/changes/archive/`: 5 archived changes. There are no active changes on main. BR adds one.

**Spec format and pipeline.**
- The spec format:
  - Each `### Requirement:` has one or more `#### Scenario:`.
  - The first line of every scenario is `*Verification*: **executable** [@tags]` or `**non-executable**`.
  - An executable body is `- **GIVEN**/**WHEN**/**THEN**` bullets. A markdown table under `*Examples*:` turns it into a Scenario Outline.
  - Placeholders are written `\<x\>`.
  - (`DC/docs/workflow/openspec-guide.md`.)
- The tools:
  - `tools/spec-to-feature.py` (1039 lines) validates the markdown strictly, emits one `Rule:` per requirement, and re-parses its output with `gherkin-official`.
  - `tools/check-specs.py` checks that features are fresh.
  - `tools/run-specs.py --change X` runs a change's features plus the durable ones it doesn't touch, using the `DATA_CONTRACTS_FEATURES` env var.
  - `tools/gherkin-steps.py --list/--missing` lists existing steps or reports missing ones.
  - `tests/test_specs_bdd.py` collects `openspec/specs/**/*.feature`, with steps in `tests/steps/`.
- None of these call the `openspec` CLI. They depend only on the directory layout.
- Bridle's replacement is described in bridle's `docs/design/specs-to-tests.md` (a Rust parser, nothing generated is committed, a pytest adapter in the python pack) and `docs/design/specs.md` "Migration from OpenSpec" (the marker grammar is kept).

**Commands.**
- The `DC/Makefile` targets:
  - `test`, which runs `uv run pytest`
  - `test-generic`, which runs with `-m "not engine"`
  - `test-polars`, which runs with `-m polars`
  - `lint`, which runs `ruff check .`
  - `format`, which runs `ruff format .`
  - `specs`, which regenerates every `.feature`
  - `run-specs ARGS=…`
  - `check`, which runs lint, then `check-specs.py`, then pytest. It doesn't run `ruff format --check` (ticket jydf).
- `DC/pyproject.toml`:
  - Builds with `uv_build`.
  - Has a `polars` extra.
  - Its dev group is gherkin-official ≥29, polars, pytest ≥9.1, pytest-bdd ≥8.1 and ruff.
  - `ruff extend-exclude = ["*.md"]`.
  - pytest runs with `--strict-markers`, `bdd_features_base_dir="."`, and the markers `engine` and `polars`.
- The suite has about 227 tests.

## 3. Classification

Key: **Base** = bridle L1 (`workflow/base/`), **Py** = python pack (L2), **Proj** =
data-contracts `.bridle/` (L3), **Super** = superseded by bridle, **OS** = OpenSpec-dependent
(needs P3).

| Item (source) | Class | Note |
|---|---|---|
| No assistant memory (`CLAUDE.md`) | Super | Same as `workflow/base/rules/memory.none.md`. Drop it from the project. |
| "Where durable things go" list (`CLAUDE.md`) | Proj | Rewrite it for bridle: tasks instead of tickets, plus rules in `.bridle/rules/`. |
| Orientation / doc map (`CLAUDE.md`) | Proj | Keep it as the human-written part of CLAUDE.md, or as `.bridle/facts.md`. |
| Governing invariant: never silently rewrite a contract or a source (`config.yaml` context, plan-of-record §0) | Proj | A `must` rule, and a candidate for `locked` or `{protected}` in P3. |
| Python ≥3.12 floor, dev 3.14, no 3.13+ syntax (`CLAUDE.md`, `config.yaml`) | Proj | The floor value is per-project. "Test at the floor" could be a Py rule. |
| uv / `uv run pytest` / ruff check and format (`CLAUDE.md` Tooling) | Py | These are the pack's commands. |
| `ruff extend-exclude = ["*.md"]`: don't reformat quoted code in prose (`pyproject.toml`, tickets at7m/yhgu) | Py | Good pack default guidance. |
| Format check is repo-wide, not diff-scoped (jydf) | Py | A pack concern: which files the check covers. |
| Engine/polars pytest markers, `--strict-markers` (`pyproject.toml`) | Proj | |
| `make check` as the definition of done | Proj | A binding. Bridle's `bridle-worker` skill **hardcodes `just check`** (`workflow/base/skills/worker/SKILL.md`), which needs a pack or project hook. |
| Cross-doc links from the repo root, wiki form, no leading `/`, few `../` (`CLAUDE.md`, ticket-conventions "References") | Base | Bridle's docs already follow it. The checker is an open question (Q7). |
| Filing doesn't schedule; only the human lands work; the plan comes before code; a report is a claim, so verify it (`workflow-instructions/README.md` rules 1–5) | Base | Rules 1 and 3 are already bridle's task states and plan gate. "Only the human accepts" is the example locked rule in `workflow-layers.md`. |
| Ask only blocking questions, about 2 a round, with a recommendation (`driver-guide.md` §2) | Base | Manager and orchestrator guidance. |
| Record decisions where they outlive the change (`driver-guide.md` §2, rule 5) | Base | |
| Proposal names every file in Impact; "not yet" with a reason; rejected alternatives in design; each task says how to verify it (`config.yaml` rules) | Base | They map onto `bridle-plan` and the impact declaration (P4). |
| File out-of-scope findings, don't fix them; never weaken a test to pass (`config.yaml` operations.apply) | Base | `bridle-worker` already says "note it rather than fixing it". |
| Apply agent: don't commit, leave the tree dirty (`apply-agent.md`, `apply-agent-guide.md`) | Super | Bridle workers commit on `bridle/<name>`. |
| `apply-agent` subagent (`.claude/agents/apply-agent.md`) | Super | Replaced by the bridle `worker` role and the `bridle-worker` skill. Its "Project facts" section becomes a Proj addendum. |
| `acceptance-verifier` subagent | Super (later) + Proj | `bridle-review` isn't built yet. For a library, verification means a scratch consumer that also exercises the error and refusal paths. That's a Proj guide (`driver-workflow.md`). |
| Driver role (`driver-guide.md`) | Super | Replaced by the bridle manager, PM and orchestrator roles and the `bridle-manager` skill. |
| Three commit checkpoints (`docs/workflow/driver-workflow.md`) | Super | Replaced by worker commits on its branch, then the manager's `git merge --no-ff`. An unmerged branch is the "unreviewed" state. |
| Ticket pipeline, `new-ticket.sh`, `check-tickets.py` + vendored copies (`docs/tickets/`, `tools/`, `workflow-instructions/scripts/`) | Super | Replaced by `bridle task` / `bridle queue`, with tasks on the state branch. The link-check half is Q7. |
| Planning session, promote/drop (`maintenance.md`) | Super | Done by the PM role's triage (`bridle-triage`, planned). |
| `docs/workflow-instructions/` (the whole governing guide) and the shared-submodule idea | Super | Bridle's L1 base plus layers is exactly 6tps and mv9p. |
| `plan-of-record.md` §0 orientation and §3 decisions | Proj | Keep them. §1/§2 (state, next) become the queue plus `bridle status`, and eventually knowledge tiers (ew97). |
| "Regeneration comparison is a reading aid, not pass/fail; don't automate it with a cold agent" (`driver-workflow.md`) | OS / Proj | Only relevant to spec-tooling changes. |
| `openspec/config.yaml` `context:` | Proj | Move it into facts or CLAUDE.md. Its "specs are the only normative record" line stays true. |
| `openspec/config.yaml` `rules.specs` (marker grammar, registered tags, regenerate, reuse steps) | OS → Proj in the interim | A `.bridle/rules/specs.*` rule until P3's parser owns the grammar. |
| `spec-driven-gherkin` schema and templates | OS | |
| The 10 OpenSpec skills | OS | Replaced by `bridle-plan`/`-worker`/`-review` (`docs/design/skills.md`). |
| `openspec/changes/` delta lifecycle, `openspec archive`/`validate` | OS | See §6. |
| `tools/spec-to-feature.py`, `check-specs.py`, `gherkin-steps.py`, `tests/test_specs_bdd.py`, `make specs/check` | OS (paths only) | These stay until P3's Rust parser and Python adapter replace them. The adapter goes to Py. |
| `tools/run-specs.py --change` | OS | Loses its purpose once there are no change directories. `--spec` and the default mode still work. |
| Design decisions numbered D1 per change, cited from code (22s2) | OS | Resolved by P3 ids and traceability (`docs/design/traceability.md`). |
| Worktrees as siblings in a workspace folder, visible to Obsidian and vim (992c) | Super | Bridle uses `<workspace>/wt/<agent>`, which is the same idea. |
| `.claude/settings.json` `additionalDirectories: [".."]` (BR design D11) | Super | Bridle workers run inside their own worktree. See §5. |

## 4. Workflow tickets in `DC/docs/tickets/`

The paths are relative to `DC/docs/tickets/`.

| Ticket | One line | Disposition |
|---|---|---|
| `backlog/apply-agent-should-implement-in-a-worktree-branch-not-the-main-tree-vf32.md` | The apply agent works on its own branch and worktree and commits there | **Obsoleted by bridle.** Workers get `wt/<agent>` on `bridle/<agent>` and commit. Drop it, with that reason. |
| `backlog/driver-session-should-also-work-from-the-ticket-s-branch-worktree-duri-sjkw.md` | The driver moves onto the change's branch so work landing mid-flight doesn't derail it | **Obsoleted.** Bridle tasks live on the state branch, not in the tree, so filing never shows up in `git status`. Workers fork at a known commit, and the manager is read-only. Drop it. Its verified git findings are worth one line in bridle (see §5). |
| `backlog/split-implementation-verification-into-a-separate-validator-agent-zm6n.md` | A separate validator agent, with the driver coordinating the feedback loop | **Covered by planned bridle work** (`bridle-review` and the reviewer role, `docs/design/skills.md`). Move it to bridle only if nothing there tracks the "send it back to the implementer" loop. |
| `intake/features/promote-the-driver-workflow-instructions-to-a-shared-git-submodule-acr-6tps.md` | Share the workflow across projects instead of copying it | **Resolved by bridle**: L1 base plus the `workflow` path or git url (`docs/design/workflow-layers.md`). Drop it. |
| `intake/research/how-should-project-specific-guidance-overlay-a-shared-workflow-submodu-mv9p.md` | How project guidance overlays a shared base | **Resolved by bridle**: rule ids, `override: replace/append/disable` with a reason, SKILL.md addenda, `rules explain/diff`. Drop it. Its "binding vs deviation" distinction is still a good test of the design. |
| `intake/features/restructure-into-a-workspace-folder-with-sibling-worktrees-992c.md` | Move to a workspace folder with sibling worktrees | **Done** (the move landed; commit `7b77ab1` says so) but never landed. Bridle's layout is compatible. Land it. |
| `intake/questions/how-should-design-decisions-be-identified-and-referenced-across-change-22s2.md` | Positional D-numbers are fragile; decisions need global ids | **Move to bridle** as P3 input (ids and traceability), or link it from bridle's `traceability-in-older-specs-5bw7`. |
| `intake/questions/why-were-spec-md-s-per-scenario-html-comments-consolidated-into-one-to-d35t.md` | Why the spec template's comments were merged | **Obsoleted by P3.** The OpenSpec template goes away. |
| `intake/usability/ruff-format-check-fails-on-pre-existing-files-unrelated-to-any-in-flig-jydf.md` | Format checks run repo-wide, not scoped to the diff | Stays in the **project**, or becomes a python-pack rule. Most of its practical impact is gone (plan-of-record §2). |
| Spec-tool cluster: `intake/bugs/…-67eh`, `intake/usability/…-ghze`, `…-q53z`, `…-bqqs`, `…-2x2e` | Error-message and edge-case gaps in `spec-to-feature.py` | **P3 input.** File one bridle ticket that cites these five as requirements for the Rust spec parser's diagnostics. Don't fix them in `spec-to-feature.py` unless they bite before P3. |

Library tickets stay with the project: 5hwh, vpaw, y3cr, 8tz3, ptwp, wdbz, wh6r. So do the 20
questions in `docs/open-questions.md`. The archive (19 tickets) is history, and nothing there
needs action.

## 5. The in-flight branch, and `.claude/settings.json`

**What `adopt-branch-per-change-workflow` is.**
- It's an OpenSpec change that is still only at the planning stage (`skip_specs`, no `tasks.md`, no implementation).
- 7 commits ahead of main (`git log main..adopt-branch-per-change-workflow`):
  - They schedule vf32 and sjkw into `active/`.
  - They add sjkw's verified merge mechanics and what the dogfooding turned up.
  - They add `openspec/changes/adopt-branch-per-change-workflow/proposal.md`.
- Uncommitted in BR: edits to `proposal.md` (they add `.claude/settings.json` and setup.md content to Impact) and an untracked 399-line `design.md` (D1–D11).
- The proposal:
  - One branch and one sibling worktree per change, created when the change is created. Branch name = change name.
  - The driver session moves into it with `EnterWorktree path:`, and subagents inherit it.
  - The apply agent commits on the branch.
  - Landing: `git merge --no-ff main`, archive on the branch, then fast-forward main.
  - Ticket moves happen on the branch.
  - "Untracked isn't filed."
  - It amends `docs/workflow-instructions/` in place, for all projects.

**How it relates to bridle.**
- Bridle already provides the substance:
  - Each worker gets its own worktree and branch and commits there (`docs/design/agent-host/operating-model.md`).
  - The worker merges the local main into its branch.
  - The manager checks `merge-base --is-ancestor`, then `git merge --no-ff` and pushes (`.bridle/roles/manager.md`).
  - Tasks and planning live on the state branch, so they never create noise in the tree.
- The branch's design conflicts with bridle in three places:
  - Per-change branches driven by an interactive driver using `EnterWorktree`, where bridle has per-agent branches under `bridle/`.
  - Planning artifacts on the branch, where bridle keeps the plan on the task.
  - It amends a guide that bridle supersedes.
- **Recommend: drop it, don't merge it.** Leave the branch ref in git as a record (or commit the untracked `design.md` to it first so its reasoning survives). Mark vf32 and sjkw dropped on main as "superseded by bridle". Then remove the BR sibling worktree.
- Its verified findings are worth one bridle ticket, or a check that bridle already handles them:
  - The human's global `merge.ff = only` means bringing main into a branch needs an explicit `--no-ff`. Bridle's worker skill says `git merge main`, which **would be refused under that config**, so this needs checking.
  - `git mv` stages the rename but not the edited body.
  - A worktree-isolated interactive session rejects compound git commands.
  - A hook that uses a relative `.git/hooks/` path breaks in a worktree.

**`.claude/settings.json`.**
- Neither DC nor BR has one; checked on 2026-09-28.
- The intended content (BR design D11) was `{"permissions": {"additionalDirectories": [".."]}}`. It granted the whole workspace folder, so one interactive session could edit both the primary tree and sibling worktrees without prompts. It was safe only because `..` is a folder that holds nothing but this repo's trees.
- Under bridle it isn't needed: each worker runs with its own worktree as its cwd. Bridle's `sync` will write hook entries into `.claude/settings.json` anyway, so the file will be created then.
- Recommend not re-adding the grant (Q2).

## 6. OpenSpec resolution (interim, until P3)

**What data-contracts depends on OpenSpec for:**
1. The spec **store and format**: `openspec/specs/<cap>/spec.md`, with the `### Requirement` / `#### Scenario` heading grammar. The `*Verification*` grammar is data-contracts' own.
2. The change lifecycle: proposal, delta specs, design and tasks under `openspec/changes/`, delta folding with `openspec archive`, and `openspec validate`.
3. `openspec instructions`, which serves `config.yaml` context and rules to agents.
4. The 10 skills and the custom schema.

The test pipeline depends only on (1)'s **paths**.

**Options.**
- **A. Keep OpenSpec whole until P3.** Workers would run the OpenSpec skills inside bridle tasks. That means two process models (change directories and tasks) and the same-spec pile-up, and it keeps alive the thing that was dropped. Not recommended.
- **B. Keep the files, drop the tool (recommended).**
  - `openspec/specs/**/spec.md` stays where it is, in the same grammar, as the normative record, and the spec→feature→pytest-bdd pipeline stays as it is.
  - Stop using `openspec/changes/`. A bridle task's plan names the requirements it adds or changes. The worker edits `openspec/specs/<cap>/spec.md` directly on its branch, runs `make specs`, and passes `make check`. The branch diff is the delta, and the manager's merge is the archive.
  - Delete the 10 skills and the schema, or leave the schema in place unused.
  - Move `config.yaml`'s `context:` into the project layer and `rules.specs` into a project rule.
  - What's lost: `openspec validate`, which `spec-to-feature.py`'s own strict validation mostly covers, and parallel edits to one spec. `max_workers = 1` makes the second moot for now.
  - P3's `bridle import openspec` then has only files to move and no active changes to convert.
- **C. Pull the P3 layout forward**: move the specs to `design/specs/<cap>.md` now and repoint the tools. That's churn before the parser exists, and it's a second migration. Not recommended.

## 7. Onboarding steps and open questions

**Steps** (after the human answers the questions below):
1. **data-contracts, on main**:
   - Drop vf32, sjkw, 6tps, mv9p and d35t with the reason "superseded by bridle".
   - Land 992c.
   - Park or drop BR and remove its worktree.
   - Delete the orphaned `DC/.claude/worktrees/`, which is ignored and untracked.
2. **Bridle side**:
   - Move the roles into `workflow/base`.
   - Build `workflow/packs/python/`: rules for uv, pytest and ruff, "test at the supported floor", and the ruff `*.md` exclude.
   - Make the worker skill's check command a pack or project binding instead of `just check`.
   - Check the `git merge main` versus `merge.ff=only` issue.
   - Harvest the §3 "Base" rows from `workflow-instructions/` into `workflow/base/rules/`.
3. **data-contracts, on a branch**:
   - `.bridle/config.toml`: project, prefix, `workflow` pointing at bridle's `workflow/`, `packs = ["python"]`, roles, `max_workers = 1`.
   - `.bridle/rules/`: the governing invariant as a must, the 3.12 floor, the interim spec grammar, acceptance verification for a library, and links.
   - `.bridle/skills/worker/SKILL.md` addendum: `make check`, `make specs` after spec edits, and `gherkin-steps.py --list`.
   - Trim CLAUDE.md to orientation, then `bridle sync`.
   - Remove the 10 skills and `apply-agent`. Keep `acceptance-verifier` until `bridle-review` exists.
   - Mark `docs/workflow-instructions/` and `docs/tickets/` as frozen history.
4. Start a daemon with the workspace set to `data-contracts-workspace/`, so worktrees go to `data-contracts-workspace/wt/`.
5. Import the open library tickets as bridle tasks, each linking its file.
6. First real task with one worker. vpaw is small and well understood.

**Open questions for the human:**
1. May we drop `adopt-branch-per-change-workflow`, commit its uncommitted `design.md` to the branch for the record first, and drop vf32 and sjkw as superseded?
2. `.claude/settings.json` isn't there. Was it removed on purpose? We recommend not re-adding the `..` grant.
3. OpenSpec interim option B: keep `openspec/specs/` and the pipeline, retire the CLI, the skills and change directories. OK?
4. Gates: data-contracts requires the human's approval of every plan and every land. Bridle's manager merges on its own after checks pass. Should data-contracts keep a human plan gate and land gate at first?
5. `docs/tickets/`: freeze it and move the open tickets into bridle tasks (recommended), or keep filing library tickets there until P3?
6. `plan-of-record.md`: keep §0 and §3 (orientation, decisions) and retire §1 and §2 (state, next) in favour of the queue?
7. Does bridle take over link checking (a `bridle` check, or a base-layer hook), or does data-contracts keep a trimmed `check-tickets.py` for links only?
8. How does data-contracts reference bridle's `workflow/`: an absolute path, a path relative to the workspace, or a git url?
