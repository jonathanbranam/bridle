---
id: ma2x
title: build-order.md marked phases built that nothing uses; the roadmap the PM ranks from didn't match reality
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34bw, 7r2c, 8awb]
tasks: [br-dcf2]
---

## The ask

The human, verbatim (2026-10-03, relayed by the orchestrator), on the roadmap: a product manager
should "own the roadmap and understand exactly this. What exists today, what's been built,
what's the future plan, so that they can make a good decision on what work gets into the queue".

Filed by a doc status check of `docs/proposal/`, `docs/context/` and `docs/briefs/` against the
code (`main` at `4c3f5b6`).

## What build-order.md said, and what the code shows

The product manager reads `docs/proposal/build-order.md` to triage
(`workflow/base/roles/product-manager.md`, "What you do": "Read the open tickets ...,
`docs/proposal/build-order.md` and what the human and the orchestrator send you"). Before this
check its State column read (at `4c3f5b6`):

| Phase | It said | The code |
|---|---|---|
| P1 | "wait and prime for non-orchestrator roles not" | both built: `bridle wait`, `bridle prime <role>`. Neither is used by a spawned role |
| P2 | "**as far as it goes** (packs, rules explain/diff, sync built; ...)" | built, but resolved rules reached no spawned agent before `9561950` and `sync` is run by nothing ([34bw](docs/tickets/open/the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw.md)) |
| P3 | "Python adapter built (br-3b72); data-contracts not yet switched" | meta-notes uses bridle specs and the adapter on `main` (`15bfb81`); data-contracts has no daemon |
| P4 | (blank) | `task impact`, `task conflict`, `trace`, `arch`, re-evaluate tasks all built; none used (no impact declared, no project has architecture files) |
| P5 | "plain per-agent worktrees built in v1" | also built: `task land` (the integrator), merge probe, port registry, paired worktrees; the last two unused |
| P6 | "migrations not started" | meta-notes on bridle; track-web on a `bridle-adopt` trial since 2026-09-29 |
| P7 | "**planned** (br-a4ea)" | the prototyper role landed (`cacab9c`); never spawned |
| P8 | "**planned** (br-1665)" | the gateway API is 8 of 10 tasks in (`crates/bridle-gateway`); not running |

So it was wrong both ways: it called unwired things built (P1 prime, P2), and left out built
things (P4, P5, P7, P8). One word, "built", stood for both "in the code" and "doing something",
which is how packs looked done
([[language-packs-were-prioritised-as-onboarding-prerequisites-xfb3|xfb3]]).

The same check found two more ways a reader can be misled:

- **Merged isn't live.** A change reaches agents only when each project's daemon restarts on a
  binary built from it. At this check bridle's daemon (started 01:59 UTC) predates `9561950`
  (03:16 UTC), and the installed `bridle` is 0.4.0, which still offers `init --stack rust`
  (dropped in `f55537d`).
- **The briefs said "built" too.** `docs/briefs/` (8awb) marked `bridle sync`, the `arch-guard`
  hook and the port registry **Built.** without saying nothing uses them.

## Done in this check

`build-order.md` now marks every step "built and in use", "built, not wired in" or "planned",
with a status line under its H1, and defines the words. The briefs, `docs/context/projects.md`,
`nuc-host.md` and `adding-a-project.md` have the same status line.

## Recommendation

1. **The PM (project manager, [[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]])
   owns `build-order.md`'s State column** and uses only the three status words. Moving a step to
   "built and in use" needs a citation of what in the normal flow invokes it.
2. **Re-check it at each handover or release**, the way `docs/context/orchestrator-state.md`'s
   handover is kept: a short pass of each "built" row against `bridle --help`, the role prompts
   and the daemon.
3. **Report what's live, not just merged**: `bridle status` could show when the daemon's binary
   is older than `main`'s tip. Not filed as a task; the human's call.

## Next steps (advisor workflow, retiring, 2026-10-04)

Waiting on the human: who owns build-order.md's status column (the project manager, 7r2c, once renamed) and whether bridle status should flag a daemon binary older than main. Task br-dcf2 was dropped in pm-1's k7tm sort. build-order.md itself was already corrected (5aca429).
