---
id: ajqa
title: Onboarding survey: meta-notes
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [d9nu, u8sm, 8xhh, a8fk, b5zh]
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

- Related surveys from the same request: [[onboarding-survey-file-db-d9nu|d9nu]], [[onboarding-survey-track-web-and-harness-u8sm|u8sm]], [[onboarding-survey-otters-8xhh|8xhh]], [[shared-pixellab-tooling-for-game-projects-a8fk|a8fk]].
- Bears on [[what-happens-to-meta-notes-planning-cli-b5zh|b5zh]] (see its section below).
- The open questions at the end are for the human; none is answered yet.

## The human's answers

2026-09-28, on question 6 (b5zh):

> yes, close b5zh; the meta-notes CLI is part of the plugin, not related to bridle.

b5zh is resolved as unrelated. On the survey's framing and priority:

> meta-notes is a project I spent hours on this weekend and is active; it is software - it is
> a vim plugin; I have plenty of tasks to work on there and would be great to use bridle to
> get that done.

On question 8 (a Vim pack):

> yes, it's an example of a different language again, different test framework, etc.

So meta-notes is active and wanted on bridle, and a Vim/vader pack is worth building as
another language and test framework, not kept project-local.

## The survey

### Summary

Read-only survey, 2026-09-28. Nothing in meta-notes was changed. `MN` = `/Volumes/Data/work/meta-notes` (main, clean, at `da4d402`, in sync with `origin/main`).

- meta-notes is a Vim plugin (Vimscript) plus a Python CLI (`bin/meta-notes`, which runs `scripts/meta_notes/`, v0.13.0). Its development process is **plain OpenSpec with the stock `spec-driven` schema**. It has 10 stock OpenSpec skills (`generatedBy: "1.4.1"`), no subagents, no ticket system and no custom workflow docs. There is much less to supersede than in data-contracts.
- Beads was used from February to June 2026 and removed in `7e8b6d4` (2026-06-14). When it was removed it had 103 closed issues and 2 deferred ones. Nothing Beads-related is left except one line in `MN/.gitattributes` and four merged `origin/claude/*beads*` branches.
- **b5zh (meta-notes' planning CLI):** the "planning CLI" is a *personal life-planning* tool. It covers PPARA notes, markdown tasks, daily and weekly ceremonies, calendar and time logging. It does not plan software work. It overlaps bridle only in vocabulary ("task", "project", "review"). Recommendation: **unrelated**, not a client of bridle and not an intake source. `MN/docs/planning-system.md` also rules out scripted agent automation at work (see "Relation to b5zh").
- `projects.md` is out of date. There are **21** spec capabilities, not 17: 167 requirements and 459 scenarios. There is 1 active change, `task-age`, which is explicitly deferred.
- The project is ready to onboard. The tree is clean, there are no in-flight branches and no worktrees. Test commands are simple (`./run_tests.sh`, `pipenv run pytest test/unit/`). It needs a **Vimscript/vader pack** (or project rules) and a **Python pack** set up for pipenv/stdlib, not uv. Size estimate: **small**, about 0.5–1 day of setup, most of it pack work on the bridle side.

### What meta-notes has today

**Agent setup.**
- `MN/CLAUDE.md` (11 lines) says "Read @README.md and @AGENTS.md", then gives one convention: all key mappings live in the plugin, use `<localleader>`, are buffer-local through `autocmd FileType` in `plugin/meta_notes.vim`, and never go in `after/ftplugin/` (with the reason).
- `MN/AGENTS.md` (about 200 lines) covers:
  - Project: Vimscript first for compatibility, Python for larger work. Python ≥3.11, standard library only, except commands whose spec needs more. Those commands pin their libraries in `requirements.txt`, install them only into the notes root's `.venv` (via `meta-notes init`) and import them lazily.
  - Issue tracking: "Beads has been replaced by openspec."
  - Vader testing: install, `run_tests.sh` flags, and a temp-dir setup/cleanup pattern for filesystem tests. The prose mixes up `g:test_dir` and `g:test_root`, which is a minor doc bug.
  - Python testing: bare functions only, no classes, `test_<module>_<function>_<scenario>`, `tmp_path`, comment headers to group tests.
  - Versioning: `__version__` in `scripts/meta_notes/__init__.py`. Bump PATCH, MINOR or MAJOR in the same commit that archives a behaviour-changing change, then tag `v<version>` and push the tag. Changes to docs, tests or openspec only don't bump it.
- `MN/.claude/`:
  - `settings.local.json` is **tracked in git**. It allows `git add`, `git commit`, `git push`, `git pull`, `git checkout`, `openspec *`, `./run_tests.sh`, `pipenv run pytest/python`, `vim` and a few read-only tools.
  - 10 OpenSpec skills (`openspec-{explore,new-change,continue-change,ff-change,propose,apply-change,verify-change,sync-specs,archive-change,bulk-archive-change}`).
  - No `settings.json`, no agents, no hooks.
- `MN/skills/` (7 skills) is **product, not dev tooling**: `daily-plan`, `daily-shutdown`, `weekly-plan`, `weekly-review`, `task-cleanup`, `project-review`, `calendar`. `meta-notes init` links them into a *notes root's* `.claude/skills`. `MN/docs/planning-system-conversation.md:465` notes they are deliberately kept out of `.claude/skills/`.
- `meta-notes prime` and `templates/suggested-CLAUDE.md` are likewise product features for agents working in a notes root.

**Tracking.**
- There is no ticket system. Work is proposed as OpenSpec changes, and design notes live in `MN/docs/`:
  - `planning-system.md`: the 428-line requirements/roadmap, with a Sequence and Open questions.
  - `planning-system-conversation.md`
  - two feature design notes
  - `gherkin-compiler-testing.md`
- `MN/requirements.md` (525 lines) is the original product requirements. It was last touched 2026-09-25.
- `MN/tasks.md` is a **tracked sample file of task syntax** from 2026-02-13, not a backlog. The tests don't reference it; they build their own `tasks.md` under `tmp_path`.
- Beads history: `.beads/` was committed until `7e8b6d4`. The 2 deferred issues at removal were `meta-notes-17y.3` (YAML frontmatter support) and `meta-notes-17y.4` (`encrypted: true` handling). Neither was carried anywhere I could find (not checked exhaustively).

**OpenSpec usage.**
- `MN/openspec/config.yaml` (21 lines) sets `schema: spec-driven` (stock) and has only a `context:` block: the domain, the languages, the architecture map and the testing conventions. There are no `rules:`.
- `MN/openspec/specs/`: 21 capabilities (agent-prime, archive, calendar-agenda, calendar-skill, ceremony-skills, ceremony-status, change-summary, cli, conventions, date-period, init, note-create, notes-config, project-brief, project-fields, project-list, task-query, task-update, template, time-log, time-report). `archive` is the note-archiving capability, not an archive folder.
  - Plain `### Requirement:` / `#### Scenario:` with WHEN/THEN bullets.
  - No `*Verification*` markers and no `.feature` files.
- `MN/openspec/changes/archive/`: 21 archived changes (2026-06-19 to 2026-09-26). Active: `changes/task-age/`, which has only `proposal.md`, marked "Deferred (2026-09-25)".
- The cadence shows in commit subjects: "Proposes X" → "Implements X and bumps version to N" → "Archives X and syncs its specs". Tags run from `v0.1.0` to `v0.13.0`.
- `MN/docs/gherkin-compiler-testing.md` proposes writing specs as `.feature` files and compiling them to pytest **and vader**. It was never built (no `compile_features.py` exists). It is direct P3 input: a vader adapter for specs-to-tests.

**Commands.**
- Vimscript: `./run_tests.sh [file] [--quiet|--debug|--interactive]`.
  - Runs `vim -es` with a clean runtimepath and vader.vim from `$VADER_PATH`, default `~/.vim/pack/testing/start/vader.vim`, which is installed on this machine.
  - 11 `test/*.vader` files.
- Python: `pipenv run pytest test/unit/`, set up by `MN/pytest.ini`:
  - `testpaths = test/unit`
  - `-v --strict-markers --tb=short`
  - markers: slow, integration, unit
  - 26 test files, about 1014 `def test_` functions, no `class Test`.
- There is no `check` target, no Makefile or justfile, no linter or formatter config, and no CI (not checked for GitHub Actions beyond the absence of `.github/` in `ls -la`).
- Dependencies:
  - `MN/Pipfile` has `python_version = "3.14"`, pytest, `icalendar==7.3.0` and `recurring-ical-events==3.8.2`. `requirements.txt` holds the two pins. The documented floor is 3.11, but development runs on 3.14.
  - External repos: vader.vim (test only) and the `openspec` CLI (sibling checkout `/Volumes/Data/work/OpenSpec` exists; not checked whether that is what's installed). No other repo dependency.
  - `/Volumes/Data/work/notes-test` is a notes root used for manual testing (not a git repo, has `.meta-notes`, `.venv` and `.claude/skills`).
- Git activity: 206 commits. By month: 2026-02 had 132, 2026-03 had 1, 2026-06 had 15 and 2026-09 had 58 (last on 2026-09-26). Commit authors are Jonathan Branam (198) and Claude (8).

### Classification

Key: **Base** = bridle L1 (`workflow/base/`), **Py** = python pack (L2), **Vim** = a vimscript/vader pack (L2, new), **Proj** = meta-notes `.bridle/` (L3), **Super** = superseded by bridle, **OS** = OpenSpec-dependent (needs P3).

| Item (source) | Class | Note |
|---|---|---|
| Key mappings in the plugin: `<localleader>`, buffer-local through `autocmd FileType`, never `after/ftplugin/` (`CLAUDE.md`) | Proj | Could become a Vim pack default ("don't rely on `after/` being in rtp"), but the mapping style is project-specific. |
| Vimscript first for compatibility, Python for larger work (`AGENTS.md`) | Proj | |
| Python ≥3.11, stdlib only, lazy imports for pinned extras in the notes-root `.venv` (`AGENTS.md`, `config.yaml`) | Proj | A must rule. The floor value is per-project; "test at the floor" is a Py rule, and today they develop on 3.14 only. |
| vader.vim tests through `./run_tests.sh`, with the temp-dir setup/cleanup pattern (`AGENTS.md`) | Vim | The core of a Vim pack. The runner script stays in the project. |
| Bare-function pytest, `test_<module>_<function>_<scenario>`, `tmp_path` (`AGENTS.md`) | Py (style option) / Proj | data-contracts doesn't mandate this. Make it a pack option or keep it Proj. |
| `pipenv run pytest` (`AGENTS.md`, `settings.local.json`) | Py | The pack must support pipenv as well as uv (data-contracts). This is a tool binding. |
| Definition of done: `./run_tests.sh && pipenv run pytest test/unit/` | Proj | There's no single check command. The worker skill's hardcoded `just check` needs a binding (the same finding as data-contracts). |
| Version bump in the same commit as the archive, then tag `v<ver>` and push the tag (`AGENTS.md` Versioning) | Proj | Needs a decision: under bridle, who bumps (worker) and who tags and pushes (manager, after merge)? |
| `.claude/settings.local.json`, tracked, allowing `git push`, `git checkout`, `openspec *` | Super | Bridle controls worker permissions. Recommend untracking it (a local file shouldn't be in git) and not granting push to workers. |
| 10 OpenSpec skills (`.claude/skills/openspec-*`) | OS | Replaced by `bridle-plan`/`-worker`/`-review`. |
| `openspec/config.yaml` `context:` (architecture map, testing) | Proj | Move it into CLAUDE.md or `.bridle/facts.md`. It mostly duplicates `AGENTS.md`. |
| `openspec/changes/` lifecycle ("Proposes / Implements / Archives and syncs") | OS | See "OpenSpec resolution". |
| `openspec/specs/**/spec.md` (21 capabilities, stock grammar) | OS | Keep the files. Grammar migration is simpler than data-contracts' (no markers). |
| Beads leftovers: `.gitattributes` `merge=beads` line, 4 merged `origin/claude/*beads*` branches | Super | Delete them (step 1). |
| "Issue Tracking" section of `AGENTS.md` | Super | Rewrite it: `bridle task`. |
| `tasks.md` (sample task file), `feature-demo.ipynb`, `requirements.md` | Proj | Product artifacts, not workflow. Leave them. The human might move `tasks.md` into `test/fixtures/` (optional). |
| `skills/` (7 planning skills), `meta-notes prime`, `templates/suggested-CLAUDE.md` | Proj (product) | These are meta-notes' own deliverables, not dev workflow. Bridle must not treat them as workflow skills. |
| `docs/gherkin-compiler-testing.md` (specs compiled to pytest and vader) | OS → bridle P3 input | Argues for a **vader adapter** in specs-to-tests. |
| Deferred Beads issues 17y.3/17y.4 (frontmatter, encryption) | Proj | Import them as bridle tasks if still wanted (question 5). |

### Workflow tickets and their disposition

meta-notes has **no ticket system**. There's no `docs/tickets/` or equivalent, so no workflow tickets need a disposition. The nearest items:

| Item | Disposition |
|---|---|
| `openspec/changes/task-age/` (proposal only, deferred 2026-09-25) | Convert it to a parked bridle task that links the proposal, then delete the change dir, or leave it as is until P3 import. Recommend converting it. |
| `docs/planning-system.md` "Sequence" items 7–8 (dashboard, archive tiers) and "Open questions" | Product backlog. Import them as bridle tasks when the human wants them scheduled. |
| `docs/gherkin-compiler-testing.md` | Link it from bridle's specs-to-tests design, or from a P3 ticket, as the vader-adapter requirement. |
| Beads 17y.3 / 17y.4 (deferred) | Ask the human (question 5). |

### In-flight branches and worktrees

- `git worktree list` shows only the main checkout. The only local branch is `main`.
- Remote branches are all from February 2026 and are **fully merged into main**, so it's safe to delete them:
  - `origin/claude/beads-01`
  - `origin/claude/beads-02`
  - `origin/claude/dev-work-6shtZ`
  - `origin/claude/install-beads-cli-6shtZ`
- Nothing is in flight. There's no workspace folder yet. The repo sits directly at `/Volumes/Data/work/meta-notes`, so `<workspace>/wt/` needs a decision (question 3).

### OpenSpec resolution (interim, until P3)

What meta-notes depends on OpenSpec for:
1. The spec store: `openspec/specs/<cap>/spec.md` in the stock grammar.
2. The change lifecycle (`openspec/changes/`, `openspec archive` to sync deltas).
3. `config.yaml` context through `openspec instructions`.
4. The 10 skills.

No project tooling reads the specs. There's no spec→test pipeline at all, so nothing breaks if the CLI goes away.

- **A. Keep OpenSpec whole until P3.** Two process models. Not recommended.
- **B. Keep the spec files, drop the tool (recommended).** This is the same as data-contracts option B:
  - The worker edits `openspec/specs/<cap>/spec.md` directly on its `bridle/<agent>` branch as part of the task. The branch diff is the delta, and the manager's merge is the archive.
  - Delete the 10 skills. Fold the `config.yaml` context into CLAUDE.md or facts.
  - Convert `task-age` to a task.
  - It's cheaper here than in data-contracts because there are no generated `.feature` files and no custom schema.
- **C. Move to the P3 layout now.** Not recommended, for the same reason as data-contracts: it's churn before the parser exists.

### Relation to b5zh (`docs/questions/resolved/what-happens-to-meta-notes-planning-cli-b5zh.md`)

Findings bearing on the three options in the ticket:
- **Client of bridle?** No. meta-notes plans a person's workday: ceremonies, calendar, time logs, PPARA projects (`MN/docs/planning-system.md`, `MN/skills/`). Bridle plans software work for agents. Their "task" models differ: markdown checkbox lines with 📅/🛫 dates, compared with bridle tasks on a state branch. Their "project" models also differ: PPARA folders with home notes, compared with a repo.
- **Source of intake?** Weak at best. A human could jot "bridle: X" tasks in a daily note, and a skill could file them with `bridle task`. But:
  - `planning-system.md` Constraints (Work) says "scripted or scheduled agent automation is not [acceptable]" at work, and "work data never leaves the work machine". A daemon reading notes roots would violate that for the work root.
  - Personal-root intake would be a new meta-notes feature, not something that exists today.
- **Unrelated?** Recommended answer. Bridle's only relationship to meta-notes is managing its *development*, as one of the six projects.
- One real overlap worth noting in the ticket: meta-notes already does "agent primes itself from a CLI" (`meta-notes prime`, `meta-notes conventions`: skills call the CLI instead of copying rules). That's the same pattern as bridle's layered guidance. It's worth citing as prior art, not as shared code.
- Proposed resolution: close b5zh as "unrelated; meta-notes is a personal planner. Revisit only if the human wants dev-task capture from daily notes (would be a meta-notes feature calling `bridle task`)."

### Onboarding steps

1. **meta-notes, on main** (human approval needed):
   - Delete the 4 merged `origin/claude/*` branches.
   - Drop the `.beads` line from `.gitattributes`.
   - Untrack `.claude/settings.local.json` and add it to `.gitignore`.
2. **Bridle side:**
   - Python pack: support pipenv as well as uv, and make bare-function test style an option.
   - Vim pack: vader.vim, a clean-rtp `vim -es` runner, the temp-dir test pattern, and "don't depend on `after/` in rtp".
   - Worker-skill check-command binding (shared with data-contracts).
   - Record in `docs/context/projects.md` that there are 21 specs, not 17.
   - Note the vader-adapter need for P3 (`MN/docs/gherkin-compiler-testing.md`).
3. **meta-notes, on a branch:**
   - `.bridle/config.toml` with project, prefix, `workflow`, `packs = ["python", "vim"]` and `max_workers = 1`.
   - `.bridle/rules/`:
     - the Python floor and stdlib-only rule
     - the key-mapping convention
     - versioning and tagging, with who tags decided
     - "`skills/` are product, not workflow"
   - `.bridle/skills/worker/SKILL.md` addendum: the check is `./run_tests.sh && pipenv run pytest test/unit/`, and edit `openspec/specs` directly.
   - Trim `AGENTS.md`/`CLAUDE.md` (the Issue Tracking section, the OpenSpec references), then run `bridle sync`.
   - Remove the 10 OpenSpec skills.
   - Convert `task-age` to a parked task.
4. Choose a workspace folder (see question 3) and start a daemon for it.
5. First task with one worker. `task-age` is deferred, so pick something small from the `planning-system.md` open questions, or the `g:test_dir`/`g:test_root` doc fix in `AGENTS.md`.

Size: **small**. On the project side it's about 10 file edits and deletions, with no in-flight work to reconcile and no ticket migration. Most of the effort is the Vim pack, which is new, and pipenv support in the Python pack.

### Open questions for the human

1. OpenSpec interim option B (keep `openspec/specs/`, retire the CLI, skills and change dirs): OK? Convert `task-age` to a parked bridle task?
2. `.claude/settings.local.json` is tracked and grants `git push`. May we untrack it? Bridle workers shouldn't push.
3. Workspace layout: move meta-notes into a `meta-notes-workspace/` folder (like data-contracts) so worktrees go to `wt/`, or put `wt/` somewhere else? It's a Vim plugin that may be loaded from this path (`README.md` "For Development": not checked how it's installed), so moving it could break the human's Vim setup.
4. Versioning: the worker bumps `__version__` in its commit. Does the manager then tag `v<ver>` and push the tag after merge, or does the human keep tagging?
5. Beads 17y.3 (frontmatter) and 17y.4 (encrypted notes) were deferred when Beads was removed. Import them as tasks or drop them?
6. b5zh: agree to close it as "unrelated", with dev-task intake from daily notes as a possible future meta-notes feature?
7. Python floor: AGENTS.md says 3.11, but the Pipfile pins 3.14. Should workers test at 3.11, and is a 3.11 interpreter available?
8. Is a Vim pack wanted in bridle's `workflow/packs/`, or should the vader rules stay project-local, since meta-notes is the only Vimscript project of the six (per `projects.md`)?
