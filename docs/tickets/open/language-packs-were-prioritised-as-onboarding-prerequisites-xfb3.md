---
id: xfb3
title: Language packs were prioritised as onboarding prerequisites on a delivery path that never ran
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34bw, 5u9d, u8sm, 7r2c, sk52]
tasks: [br-e839]
---

## The ask

The human, verbatim (2026-10-03, relayed by the orchestrator): "there should be a clear
delineation between what we're planning to build and what works today ... so that when the
agents are reading, they can understand the difference between what's been planned and what the
vision is and what actually exists today."

The trigger: an agent (probably the orchestrator) held that language packs (`workflow/packs/`)
had to be written before adopting bridle on track-web. Resolved rules never reached spawned
agents
([[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]]), so no pack rule
has ever affected an agent. Filed by a doc status check (2026-10-03, `main` at `4c3f5b6`).

## Where the requirement came from

Every onboarding plan written on 2026-09-28 put pack work on bridle's side before the project:

- The track-web survey,
  [[onboarding-survey-track-web-and-harness-u8sm|u8sm]], section 8 "What bridle must build or
  fix first", stage 1, item 3: "**Minimal TS pack** (`workflow/packs/typescript/`): npm
  workspaces, vitest, `tsc` builds, no lint. **S–M.**" Task br-d16e cites it as its source
  ("Track-web onboarding (the human's second-priority project) needs an L2 TypeScript pack").
  It landed at `809ee92` (2026-09-29 01:35 UTC), and track-web onboarded that session with
  `packs = ["typescript"]`.
- The meta-notes survey, [[onboarding-survey-meta-notes-ajqa|ajqa]], line 90: "It needs a
  **Vimscript/vader pack** (or project rules) and a **Python pack** set up for pipenv/stdlib,
  not uv. Size estimate: **small**, about 0.5–1 day of setup, most of it pack work on the
  bridle side."
- The data-contracts plan, `docs/context/orchestrator-history.md:154-156`, step 2: "Bridle side
  (bridle's workers): move roles and role prompts into `workflow/base` ..., and build the Python
  pack."
- The otters and file-db surveys do the same
  ([[onboarding-survey-otters-8xhh|8xhh]] step "Build `workflow/packs/typescript/`";
  [[onboarding-survey-file-db-d9nu|d9nu]] "The python pack").

The specific exchange where an agent insisted on packs before track-web adoption isn't in the
repo, the handover notes or the message log; the surveys above are the recorded basis.

## What it was based on

The design said packs reach agents through two mechanisms. Neither was ever wired in.

`docs/design/workflow-layers.md` at `c6345a5` (2026-09-28, when `workflow/` was scaffolded):

> every project picks it up on its next `bridle sync` (which the SessionStart hook runs).
> (line 35)

> Most rule content is not rendered into a file at all. It is delivered by `bridle prime` at
> session start, sized to the role (lines 115-116)

There was no SessionStart hook (`workflow/base/hooks/` holds only `PreToolUse.json`), workers
never ran `bridle prime` (spike 08; `workflow/base/roles/worker.md`), and the spawn prompt was
the preamble, a branch sentence and one role file (`crates/bridle-daemon/src/config.rs`
`stable_system_prompt`, before `9561950`). Then `docs/proposal/build-order.md` at `64dcb6b`
(2026-09-28) marked P2 "**as far as it goes** (packs, rules explain/diff, sync built ...)",
which the product manager reads to triage (`workflow/base/roles/product-manager.md`, "What you
do"). The commands existed and were tested, so "built" was true; nobody checked that anything
called them.

## What it cost

- **Rule content that changed nothing.** 14 pack rules (`python` 5, `typescript` 5, `vim` 4) and
  every project's `.bridle/rules/` (meta-notes 7, track-web 3) never reached a spawned agent.
  track-web's workers were never given `typescript.dev-servers` ("Never kill or restart another
  agent's dev server") or its own `dev-servers.md`; only role prose and the project's
  `CLAUDE.md` reached them.
- **Work and sequencing.** Three pack tasks (br-7678 python, the vim pack `e6effb8`, br-d16e
  typescript) ran on 2026-09-28/29 ahead of the onboardings, and the plans sized onboarding as
  "most of it pack work". The python-pack worker also sat in three incidents (p4ks, k7nr, k3wp in
  `docs/context/incidents.md`), though those were daemon bugs, not pack bugs. Direct agent
  spend was small (a few dollars at API rates, from the `turns` table); the cost is the ordering
  and the false confidence. meta-notes' "fast path (project-layer rules, no packs)"
  (`orchestrator-history.md:197`) was no better: project rules didn't reach agents either.
- **Not wasted:** the spec adapters under `workflow/packs/*/adapters/` are code a project
  vendors. meta-notes uses the pytest one today. They never depended on rule delivery.

## Where it stands

`9561950` (br-2242, 2026-10-03) appends each role's resolved rules (base, packs, project) to the
spawn prompt. It changes agents only once a daemon runs a binary built from it: at this check
bridle's daemon last started at 01:59 UTC, before the commit (03:16 UTC), track-web's at
2026-10-02 10:11 UTC, and the installed binary is 0.4.0. Layer hooks at spawn (34bw step 3) are
approved, not built.

## Recommendation

1. **A base rule: verify "built" before ranking on it.** Before an agent makes something a
   prerequisite, or ranks work on the premise that X exists or works, it checks in the code that
   the normal flow invokes X (a role prompt, the daemon, a project's config), and cites the
   file:line. A design doc or `build-order.md` saying "built" is not evidence. This belongs with
   the PM/project-manager's ownership of the roadmap
   ([[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]]).
2. **Status words on the roadmap**: `build-order.md` now marks each step "built and in use",
   "built, not wired in" or "planned" (this status check); keep it that way
   ([[build-order-md-marked-phases-built-that-nothing-uses-the-roa-ma2x|ma2x]]).
3. **A delivery check on each project**: after a daemon restart, confirm one pack or project rule
   is in a spawned worker's system prompt (`bridle cost audit` measures those bytes). Until a
   project passes it, don't treat its packs or rules as live.
4. **Don't write more packs ahead of an onboarding** until (3) passes on track-web. How rules
   reach an agent at each step is still open in
   [[when-instructions-reach-an-agent-at-start-or-at-each-step-op-sk52|sk52]].

## Next steps (advisor workflow, retiring, 2026-10-04)

Waiting on the human's review of the three recommendations (a base rule that agents verify "built" against the code before ranking work on it; a post-restart check that a rule reaches a spawned worker's prompt; no new packs until that check passes on track-web). Task br-e839 was dropped in pm-1's k7tm sort, so nothing moves until the human decides. Steps 1-4 of 34bw have since landed, so packs reach spawned agents once daemons restart onto c4cd9af or later.
