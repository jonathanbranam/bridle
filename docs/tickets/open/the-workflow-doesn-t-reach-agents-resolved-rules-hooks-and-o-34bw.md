---
id: 34bw
title: "The workflow doesn't reach agents: resolved rules, hooks and overrides stop at the CLI"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [5u9d, mrhe, chvf]
tasks: [br-86c6]
---

## The ask


The human asked whether an external review of the workflow system
([[external-review-of-the-workflow-system-what-it-does-today-an-5u9d|5u9d]],
[[docs/research/workflow-review-2026-10-02|the review]]) misread the code or is right. The
advisor (workflow) checked its claims against `main` at `5572208`, 2026-10-02. None of the 10
commits since the review's `be2440d` touch the code it read.

## Verdict: the review is accurate

Every claim checked holds. No claim was a misreading; one attribution is incomplete (below).

| Claim | Check | Evidence |
|---|---|---|
| The spawn prompt is the preamble, a branch sentence and one role file; no resolved rules | true | `crates/bridle-daemon/src/config.rs:2609` `stable_system_prompt` |
| Only `prototyper` gets a project addendum, special-cased by name | true | `config.rs:2635` |
| Workers never run `bridle prime` | true | spike 08 (0 calls); `workflow/base/roles/worker.md` doesn't mention `prime` |
| Role prompts restate rules by summary | true | `worker.md:14,17,26` (`kiss`, `missing-tools`, `yagni`) |
| A project override changes `rules explain`, not the agent | true | follows from the two rows above |
| Layer hooks aren't live | true | bridle's committed `.claude/settings.json` has no `hooks`; the daemon passes only the Stop hook (`stop_check`, `config.rs:340`) |
| Rendered skills don't reach worktrees | true | `.claude/skills/bridle-*/` is gitignored (`.gitignore:13`); the manager's `tools` list leaves out `Skill` |
| `workflow.toml` is one line; no `[gates]` parsed; no `rules propose` | true | `workflow/base/workflow.toml`; no `[gates` in `crates/`; `bridle rules --help` |
| A git-URL `workflow` gives no base layer silently | true | `config.rs:1515` returns `None`; contradicts `roles-and-config.md:176` |
| `init --stack rust` names a pack that doesn't exist | true | `crates/bridle/src/init.rs:112`, `cli.rs:772`; `workflow/packs/` has python, typescript, vim |
| Stale "no skill/agent/hook content yet" | true | `workflow-layers.md:165`, `sync.rs:29` |
| "until `bridle workflow sync` renders rules" | true, in two files | `work-flow.md:22` and `cost-of-not-doing.md:19` |
| Counts: 20 base rules (4 locked), 14 pack rules, 6 roles, 2 skills, 1 hook | true | `workflow/` |

Why it can look like a misreading: the design docs describe delivery through `bridle prime` and
`bridle sync` as the mechanism, and both commands exist and are tested. What the docs don't say
plainly is that nothing in the agent's lifecycle invokes them. The advisor's own first summary to
the human (2026-10-02) repeated the design's "most rules go out through `bridle prime`" before
checking.

## What it means

Rules reach bridle's agents today only through the role prompt prose, edited by hand. That works
for bridle's own repo, where base is the only layer that matters. It fails the P2 test in
`build-order.md`: an override in one project doesn't change what that project's agents do. Packs
are documentation, not behaviour.

## Recommendation

Agree with the review's order, and its step 6 (hold `workflow.toml`, gates, skills and
`rules propose` under `yagni`):

1. **Rules into the spawn prompt.** `stable_system_prompt` appends the role's resolved rules (id,
   severity, body; guides as paths). Same per role and project, so the prompt cache holds. Size
   is the risk: measure it against spike 08's budget first.
2. **Then cut the restatements** from `roles/*.md`. This is what makes an override change
   behaviour.
3. **Layer hooks through `--settings` at spawn**, beside the Stop hook, not via committed settings.
4. **Small breaks, one chore:** error on a git-URL `workflow`, drop `rust` from `init` (or add the
   pack), fix the stale lines in `sync.rs`, `workflow-layers.md`, `work-flow.md` and
   `cost-of-not-doing.md`.

Open for the human: whether role prompts get a general project addendum (generalise the
prototyper case) or rules are the only per-project surface. The advisor leans to rules only: one
override mechanism, already explainable with `rules explain`.

Related: chvf (`br-751e`) changes where the base layer is read from, not how it reaches agents;
it doesn't conflict.

## Measured: what step 1 adds (2026-10-02)

The human, verbatim: "if you want to measure it, measure it. Do it right now."

`bridle prime worker` prints exactly the resolved worker rules (id, severity, layer, full body),
so its output is what step 1 would add. Tokens estimated at ~3.5 characters each.

| Project | Worker rules | Characters | ~Tokens |
|---|---|---|---|
| bridle | 18 (all base) | 15,907 | ~4.5K |
| meta-notes | 25 (18 base + 7 of its own) | 21,702 | ~6K |

Against spike 08's worker: first call 17.5K tokens (system prompt 5.0K), whole session ~55K
mean. Step 1 adds ~25-35% to the first call and ~8-11% to an average session's context.
It's the stable, cached prefix, so the cost is mostly cache reads. Step 2 takes back little:
`worker.md`'s rule summaries are ~1.1K characters (~300 tokens). Spike 08's "12.5K" for `prime
worker` doesn't match today's output; it's likely characters at an earlier commit.

Most of the size is each rule's "Why" (the human's words, history). An option if it matters:
render only the statement at spawn and leave the Why in the file, roughly halving it.

## Decided (2026-10-02)

The human, verbatim, on steps 1 and 2: "I'm pretty sure that I agree and understand one and
two", and after the measurement: "Yep, that's good. Let's get that out there and fixed."

Approved: steps 1 (resolved rules into the spawn prompt), 2 (cut the restatements from
`roles/*.md`) and 4 (the small breaks). Sent to the orchestrator to plan. Step 3 (layer hooks at
spawn) and the role-addendum question are still open.

Step 3 approved too. The human, verbatim (2026-10-02): "I don't care much about this particular
hook or rule, to, but the idea is sound and the implementation is sound and let's make it work so
we can have additional hooks." The point is the mechanism (layer hooks passed at spawn, beside the
Stop hook), not `arch-guard` itself. Sent to the orchestrator.
