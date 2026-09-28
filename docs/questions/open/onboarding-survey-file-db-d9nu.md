---
id: d9nu
title: Onboarding survey: file-db
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [ajqa, u8sm, 8xhh, a8fk]
---

## The ask

The human, verbatim (2026-09-28):

> similar to what orchestrator did already: evaluate @~/work/file-db/ and @~/work/meta-notes/
> for bridle adoption.
>
> A much more ambitious project would be @~/work/pi both harness and track-web; There is some
> cross-repo coordination in those projects; track-web could onboard first as it also has
> separate deliverables independent of harness.
>
> the otter project was started in @~/work/otters/otter-life/ and then partially migrated to
> otters-back with a newer, better architecture, but not a complete migration
>
> I have some tooling that is shared between gaming projects, e.g. using pixel lab generation
> in traack-web for dungeon tactics and mimlings proposal and also for the otter project.

Filed by the advisor: a read-only survey by a subagent, modelled on
[[docs/context/onboarding-data-contracts|the data-contracts survey]]. The survey below is
its report, unedited; nothing in the surveyed repos was changed.

## Notes

- Related surveys from the same request: [[onboarding-survey-meta-notes-ajqa|ajqa]], [[onboarding-survey-track-web-and-harness-u8sm|u8sm]], [[onboarding-survey-otters-8xhh|8xhh]], [[shared-pixellab-tooling-for-game-projects-a8fk|a8fk]].
- The open questions at the end are for the human; none is answered yet.

## The survey

### Summary

Read-only survey, 2026-09-28. Nothing in file-db was changed. `FD` = `/Volumes/Data/work/file-db` (main checkout, clean, at `8511642`, level with `origin/main`).

- file-db is a **design-stage project with no implementation**. It has 5 commits between 2026-08-28 and 2026-09-03 and nothing since, so it has been idle for about 3.5 weeks. The code is a version stub (`FD/src/file_db/__init__.py`), and the only test is `FD/tests/test_version.py`.
- The real content is design:
  - About 1,900 lines across four docs in `FD/docs/`: principles, data-model, workflows and open-questions.
  - 8 Gherkin `.feature` files in `FD/specs/`, with about 68 scenarios. `FD/specs/README.md` declares them the normative requirements.
- **OpenSpec is installed but has never been used.**
  - `FD/openspec/config.yaml` is the stock `schema: spec-driven` template with nothing filled in.
  - `openspec/specs/` and `openspec/changes/archive/` are both empty.
  - There are 10 stock skills (`generatedBy: 1.11.0`).
  - The requirements were written as raw `.feature` files outside OpenSpec, so `bridle import openspec` would find nothing to import.
- There is no agent setup beyond those skills: no `CLAUDE.md`, no `AGENTS.md`, no `.claude/settings.json`, no agents. There is no ticket system either. Open questions are Q1–Q20 in `FD/docs/open-questions.md`.
- **Nothing is in flight**: one branch (`main`), one worktree, no stash.
- **The "TS planned / two bindings must behave the same" claim in `projects.md` is not in the repo.**
  - The docs describe two *clients*: a checkout client (W1) and a GitHub Pages browser editor (W2, `FD/docs/workflows.md:88`). Both must produce the same operation format.
  - Byte-identical replay (P6, `FD/docs/principles.md:97`) is the cross-implementation property that would matter.
  - Neither "TypeScript" nor "binding" appears in any doc.
- **Readiness: easy to onboard, low risk. Size: small.** There is almost no workflow to retire. The real work is deciding:
  - how the `.feature`-first specs relate to bridle's markdown specs (P3), and
  - what the first implementation tasks are. The open questions currently block parts of the design.

### What file-db has today

**Agent setup.**
- `FD/.claude/skills/`: 10 stock OpenSpec skills, all with `allowed-tools: Bash(openspec:*)`: `openspec-apply-change`, `-archive-change`, `-bulk-archive-change`, `-continue-change`, `-explore`, `-ff-change`, `-new-change`, `-propose`, `-sync-specs`, `-verify-change`. They were committed in `f903af9` ("Installs openspec").
- There is nothing else under `.claude/`: no `settings.json`, `settings.local.json` or `agents/`.
- There is no `CLAUDE.md` or `AGENTS.md` anywhere in the repo, and no memory rule.
- Unrelated to the dev workflow, but a likely source of confusion: the *product* design says migrations (`migrations/<nnnn>_<slug>.py`) are "authored by a coding agent, reviewed by a human, run once by CI" (`FD/docs/principles.md:22,85`, `FD/docs/workflows.md:397`). That is a file-db user's agent, not a bridle worker.

**Tracking.**
- There are no tickets and no tracker (no Beads, no `docs/tickets/`), and GitHub issues were not checked.
- `FD/docs/open-questions.md` (557 lines) has stable-numbered questions Q1–Q20:
  - Q1, Q6 and Q14 are closed; the other 17 are open.
  - It ends with a "Revised on direction" change record.
- 3 scenarios carry `@blocked-by-Qn` tags: 1 in `change-log-retention.feature`, 2 in `referential-integrity.feature`.

**Design and specs.**
- `FD/docs/README.md` gives the reading order: principles → data-model → workflows → open-questions.
  - `principles.md` has 8 principles, P1–P8, and 12 checkable invariants, I1–I12 (`FD/docs/principles.md:211`).
  - The pre-split draft is kept in git at `42f992a:docs/schema-and-conflict-design.md`.
- `FD/specs/README.md` (143 lines) sets the conventions:
  - Short aliases (`r1`, `C1`, `L1`, `op-1`) and fixed example tables (`people`/`notes`).
  - Every scenario states a base, and byte-level assertions are deliberate.
  - A **closed step vocabulary**: "keep to these".
  - The tags `@blocked-by-Qn` and `@disaster-recovery`.
  - A coverage table, and a "not yet written" list.
- The runner is meant to be pytest-bdd with steps in `tests/steps/`. Neither is installed or written: "the implementation surface … does not exist."
- The 8 features, by scenario count: replay (6), content-conflict (7), column-liveness (8), migrations (16), ordering (7), referential-integrity (10), ci-fold (8), change-log-retention (6).

**Commands and tooling.**
- There is no Makefile or justfile. The commands are in `FD/README.md`:
  - `uv sync`
  - `uv run pytest` (`--cov`)
  - `uv run ruff check --fix .`
  - `uv run ruff format .`
  - `uv run pre-commit run --all-files`
- `FD/pyproject.toml`:
  - Builds with hatchling and has no runtime dependencies. The dev group is pytest, pytest-cov, pre-commit and ruff.
  - `requires-python >=3.10` with `target-version py310`, but `.python-version` is `3.13`.
  - pytest runs with `--strict-markers --strict-config --import-mode=importlib` and `filterwarnings=error`.
  - The project URLs point at `github.com/jbranam/file-db`, but the remote is `jonathanbranam/file-db`. That is a small inconsistency.
- `FD/.pre-commit-config.yaml` runs pre-commit-hooks basics, ruff-format, `ruff-check --fix --exit-non-zero-on-fix` and `uv-lock` on the pre-commit and pre-push stages.
  - The hooks are installed in `FD/.git/hooks` (`pre-commit`, `pre-push`, plus `.legacy` copies).
  - Other hooks are present too: `commit-msg`, `post-checkout`, `post-commit`, `post-merge`, `post-rewrite`, `prepare-commit-msg`, `ctags`. They look like a global template; their contents were not checked.
- **Dependencies on other repos**: none in `pyproject.toml` or the docs. GitHub is the product's substrate (Pages, CI, logins), not a build dependency.
- The global `merge.ff` is `only`, the same as for data-contracts. Bridle ticket m2fq applies here too.

### Classification

Key: **Base** = bridle L1 (`workflow/base/`), **Py** = python pack (L2), **TS** = typescript pack (L2, planned), **Proj** = file-db `.bridle/` (L3), **Super** = superseded by bridle, **OS** = OpenSpec-dependent (needs P3).

| Item (source) | Class | Note |
|---|---|---|
| 10 OpenSpec skills (`.claude/skills/openspec-*`) | OS → Super | They have never been used. Delete them at onboarding; nothing depends on them. |
| `openspec/config.yaml` (stock, empty) and the empty `openspec/specs/` and `changes/archive/` | OS → Super | Delete them. There is nothing to import. |
| uv, `uv run pytest`, ruff check and format, pre-commit (`README.md`, `pyproject.toml`) | Py | Standard pack commands. file-db has no `check` wrapper, so a project binding is needed (see steps). |
| pre-commit hooks that modify files (ruff-format, `--fix`, `uv-lock`) | Py | A pack rule: a worker re-stages and recommits after the hooks rewrite files. Worth checking that the hooks behave in `wt/<agent>` worktrees. |
| `filterwarnings=error`, `--strict-markers/--strict-config` | Proj | Project pytest policy. Registering BDD tags as markers will be required. |
| Python floor 3.10 vs dev 3.13 (`pyproject.toml`, `.python-version`) | Proj | The floor value is per project. "Test at the floor" is a Py rule. |
| Principles P1–P8 and invariants I1–I12 (`docs/principles.md`) | Proj | The governing rules. P6 (deterministic replay) and I1 (only CI writes `data/**`/`schema/**`) are `must` rules and candidates for `{protected}` in P3. |
| "Feature file beats design doc when they disagree" (`specs/README.md`) | Proj | A spec-authority rule. It maps onto bridle's "spec is the only artifact that outlives the task". |
| Closed step vocabulary; aliases; every scenario states a base; byte-level asserts (`specs/README.md`) | Proj | A `.bridle/rules/specs.*` rule. |
| `@blocked-by-Qn`: don't implement until the question closes | Proj (P3: Base?) | A good general pattern: a scenario gated on an open question. Possibly a bridle spec-marker idea for P3. |
| Open questions with stable numbering; closed ones keep their number (`docs/open-questions.md`) | Proj | Keep it as-is, or map it to bridle questions later. Don't renumber. |
| Doc reading order (`docs/README.md`) | Proj | The orientation for a new CLAUDE.md. |
| Runner: pytest-bdd with steps in `tests/steps/` (planned, `specs/README.md`) | Py / OS-adjacent | Bridle's python pack adapter (`docs/design/specs-to-tests.md`) is the eventual binder. In the interim, plain pytest-bdd over `specs/*.feature` works. |
| Two clients/bindings behaving the same (projects.md; W1/W2 and P6 in the docs) | TS + Proj (P6) | Not started. Bridle's plan is TS adapter plus migration at P6 (`docs/proposal/build-order.md:16`, `docs/design/specs-to-tests.md:19`). One `.feature` suite run by both adapters is the natural parity check. |

### Workflow tickets and their disposition

There are none: file-db has no ticket system. The 17 open design questions (Q2–Q5, Q7–Q13, Q15–Q20) are **library questions** and stay with the project. Some block specs and features, and would become PM-owned questions or blocked tasks once imported:
- Q3 anonymous namespace
- Q5 null encoding
- Q10 generations
- Q20 compaction

### In-flight branches and worktrees

- Branches: `main` only (`refs/heads/main`, `refs/remotes/origin/main`).
- Worktrees: only the primary. There is no stash, and the tree is clean.
- The workspace layout is still the plain `FD/`. There is no workspace folder like data-contracts' `data-contracts-workspace/`, so the bridle workspace (and `wt/`) location is a decision (Q4 below).

### OpenSpec resolution

- file-db **does not depend on OpenSpec at all**. The CLI was never run against real content, there are no specs or changes, and the config is the stock template.
- Its requirements are Gherkin `.feature` files, not OpenSpec `spec.md` markdown. Recommend: **remove OpenSpec entirely at onboarding**, meaning the 10 skills and `openspec/`.
- For the interim until P3:
  - `specs/*.feature` stay as the normative record.
  - A bridle task that changes behaviour edits the `.feature` and the docs in place on its branch.
  - pytest-bdd binds them once there is code.
- **P3 gap to flag for bridle**:
  - Bridle's spec format is markdown with `### Requirement` / `#### Scenario` / `*Verification*` (`docs/design/specs.md`).
  - Its importer only handles `openspec/specs/*/spec.md`.
  - file-db needs either a `.feature` → `design/specs/<cap>.md` conversion (Feature→capability; one requirement per rule/scenario group) or bridle accepting `.feature` as a spec source.
  - Its closed step vocabulary and `@blocked-by-Qn` tags should survive the conversion.

### Onboarding steps

Size: **small**. There is no workflow to retire, no branches and no tickets. The only real design work is the spec-format question, which can wait for P3.

1. **file-db, on a branch**:
   - Delete `.claude/skills/openspec-*` and `openspec/`.
   - Add a short `CLAUDE.md`: orientation from `docs/README.md`, commands, the no-memory rule via bridle base, and "feature beats doc".
2. **`.bridle/config.toml`**:
   - project and prefix, `workflow` pointing at bridle's `workflow/`, `packs = ["python"]` (add `"typescript"` when the TS side starts), and `max_workers = 1`.
3. **`.bridle/rules/`**:
   - The P6 determinism and I1 CI-only-writer invariants as musts.
   - The spec conventions and step vocabulary.
   - `@blocked-by-Qn` means don't implement.
   - The 3.10 floor.
4. **`.bridle/skills/worker/SKILL.md` addendum** with a definition of done: `uv run ruff format --check . && uv run ruff check . && uv run pytest`, or `uv run pre-commit run --all-files && uv run pytest`. This needs the bridle-side fix that the worker skill hardcodes `just check` (`workflow/base/skills/worker/SKILL.md:28`), the same gap as data-contracts.
5. **Bridle side**:
   - The python pack (uv/pytest/ruff/pre-commit, and handling hooks that rewrite files).
   - m2fq (`git merge main` under `merge.ff=only`).
   - The `.feature` import question for P3.
6. Start a daemon with a workspace chosen per Q4.
7. First tasks, in order:
   - Wire pytest-bdd with an empty `tests/steps/` so the features collect and skip.
   - Then implement `replay` against `replay.feature`. It is the core function everything calls (`docs/workflows.md` "The replay function").

### Open questions for the human

1. May we delete the unused OpenSpec install (the 10 skills and `openspec/`) outright?
2. Where do the requirements live until P3?
   - Keep `specs/*.feature` as the normative record (recommended).
   - Or convert them now to bridle-style markdown?
   - And should bridle P3 support `.feature` input directly?
3. "TS planned / two bindings must behave the same" isn't written anywhere in the repo. Is the TS side the GitHub Pages browser client (W2), a full second implementation of replay, or both? Should parity (same bytes from both, per P6) be written down now as a principle or invariant?
4. Workspace layout: move to a `file-db-workspace/` folder like data-contracts, so worktrees land in `file-db-workspace/wt/`? Or keep `FD` and put `wt/` elsewhere?
5. Gates: plan and land approval by the human, at least while the open questions are unsettled?
6. Should the open questions in `docs/open-questions.md` stay a doc (recommended, given the stable numbering), or become bridle questions?
7. Priority: the repo has been idle since 2026-09-03 and `build-order.md` puts the file-db migration at P6. Onboard now as a Python-only pilot, or wait?
8. Minor: fix the `pyproject.toml` URLs (`jbranam` vs the remote's `jonathanbranam`) as part of onboarding, or leave them alone?

Not checked: GitHub issues and PRs on the remote, the contents of the non-pre-commit git hooks, and whether `uv sync`/pytest currently pass (running them would write files).
