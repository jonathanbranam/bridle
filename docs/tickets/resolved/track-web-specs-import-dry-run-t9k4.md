---
id: t9k4
title: track-web specs import dry run and vitest adapter fit
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [u8sm]
closed: 2026-10-02T00:43:36.539963Z
---

## The question

What does migrating track-web's 115 OpenSpec specs to bridle specs take, and does
`vitest-bridle` fit? (Task br-92c0, P6b; read-only research.)

## Method

Scratch clone `/Volumes/Data/work/track-web-scratch` of `/Volumes/Data/work/track-web` (committed
`dev` HEAD `ccbbf99`, so the human's uncommitted work was not in it), throwaway branch. The checkout was
not touched (`git status` clean, same HEAD afterwards). Nothing was fixed in the project; the probes
below were mechanical edits in the scratch only.

## Findings

**1. `bridle spec check --root openspec/specs` fails: 2209 errors in 115 files. Two kinds.**

| Kind | Count | Example |
|---|---|---|
| Scenario lacks the `*Verification*: **executable**` / `**non-executable**` line | 2208 (every scenario; none has one) | `world-rendering/spec.md:18` "Named location resolves to a grid coordinate" |
| Requirement with no `#### Scenario:` | 1 | `multi-app-hosting/spec.md:47` "Watch app builds as an independent workspace" |

The checker reports the first problem per scenario, so I probed for hidden layers: inserting
`*Verification*: **non-executable**` under every scenario heading (perl, scratch only) and adding one scenario
to the empty requirement left **0 errors** (720 "no id" warnings, expected). No second layer.
Steps are all `- **WHEN**` / `- **THEN**` / 25 `- **AND**` bullets, which parse.

**2. `bridle spec import openspec --dry-run` after that probe:** would move 115 files to
`design/specs/<cap>.md` and assign 2933 ids. The real import in the scratch clone worked, and
`spec check --require-ids` then reported 0 errors, 0 warnings. `Purpose`/`App` preamble text carried over.
Left in place, as documented: `openspec/changes/` (4 active changes plus `archive/`), `config.yaml`.

**3. vitest-bridle fit.** It runs in track-web's vitest (track-web's own 836 tests still pass alongside). Numbers:

| | Count |
|---|---|
| Scenarios | 2208 |
| Executable (after the probe) | 0: nothing was executable, since no scenario had a marker |
| Trial: one capability flipped to executable (`dungeon-tactics-turn-sequencer`) | 40 executable, 40 run, 0 pass, 40 unbound (each fails "no step definition") |

Two adapter-fit facts that matter more than the counts:

- **Executable scenarios must have plain-text steps.** Export refuses (`expected plain text in scenario ... step
  (matched literally by a step definition), found markup`) on any backtick in a step of an executable scenario.
  track-web writes code in backticks heavily: 1506 of 4452 step lines, in 105 of 115 files. About half the scenarios
  (roughly 1100 of 2208) are backtick-free. Flipping `user-invites` to executable failed with 13 such diagnostics.
  And because export fails as a whole, one such scenario breaks registration for the capability being run.
- **The scenarios are prose requirements, not step scripts.** Steps like "the only tower is removed while the round is in the
  placement phase" need a step definition each; there are ~4400 step lines and no reuse to speak of. track-web already has
  836 vitest tests. Binding scenarios means writing steps or (cheaper) tagging tests with scenario ids for `spec coverage`,
  which only text-searches ids under `tests/`/`test/` (track-web tests live in `src/**`, `client-*/src/**`,
  `packages/*/src/**`, so coverage would not see them).

Small: the checkout's vitest `include` globs don't cover a `specs.test.ts` outside `src/` and the like; wiring needs a
new include line (I put it in `src/specs/`). The adapter must be copied into the repo (or a `file:` dependency).

## Recommendation

Migrating is mechanical **once every scenario has a marker**, and that is the whole cost: a one-line `*Verification*: **non-executable**` insertion under
2208 scenario headings plus one scenario for one requirement. It is a scripted edit on a trial branch, not a
judgement call. Recommend: do that as **all non-executable**, import, add `spec check --require-ids` to
track-web's check, and stop there. Do not promise executable specs for track-web yet: binding needs step
definitions or a coverage change and prose rewrites (no backticks), which is the human's call per capability. Best first
executable candidates are backtick-free pure-logic capabilities such as `dungeon-tactics-turn-sequencer` (40
scenarios) backed by the existing `packages/dungeon-engine` tests.

Follow-ups worth filing (not done; importer changes were out of scope):
- `bridle spec import openspec` could offer `--default-verification non-executable` (or insert it), since OpenSpec
  never had the marker; today the whole migration is blocked by a preparatory edit the tool could do.
- `spec check` could report the missing-marker count once as a summary instead of 2208 lines.
- `spec coverage` should take search roots (track-web tests aren't under `tests/`).
- Decide whether backticks in executable steps should be allowed (strip in matching) for TypeScript projects.

## Decisions for the human before a real trial branch

1. Trial branch in track-web (off `dev`, never `main`, which deploys to production per [[onboarding-survey-track-web-and-harness-u8sm|the onboarding survey]]), with the marker edit as its own commit?
2. Is "all non-executable to start" acceptable, or is a first executable capability (suggest the turn sequencer) a precondition?
3. Backticks in step text: rewrite specs, or ask for a bridle change first?
4. Where do spec tests live and should `spec coverage` learn track-web's layout?
5. The 4 active OpenSpec changes (`add-from-tmdb-search`, `dungeon-tactics-sprite-rendering`, `food`, `watch-ratings-filter-search-prototype`): finish in OpenSpec first, or convert by hand? (Bridle doesn't convert them.)
6. Keep the OpenSpec CLI and skills for track-web during the trial, or drop them on the branch?

## Resolution

Resolved by: br-92c0 (ff4c045)
