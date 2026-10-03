---
id: ntca
title: "docs/design: much of 'not built' is built but unwired, and CLAUDE.md says otherwise"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34bw]
tasks: [br-7e32]
---

## The ask

The human, verbatim: "there should be a clear delineation between what we're planning to build
and what works today ... so that when the agents are reading, they can understand the difference
between what's been planned and what the vision is and what actually exists today."

The trigger: an agent insisted language packs (`workflow/packs/`) be written before adopting
bridle on a project, at a time when resolved rules didn't reach spawned agents
([[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]]).

A status check of every `docs/design/` file outside `agent-host/`, `cli.md` and `storage.md`,
against `main` at `94a5c60` (2026-10-03). Each doc now opens with a `> **Status (checked
2026-10-03):**` line (built and in use · built, not wired in · planned), and sentences that
stated planned things as present were fixed. Three things need the human.

## 1. CLAUDE.md and docs/README.md say the rest of design/ isn't built

`CLAUDE.md:9-10`: "The rest of `docs/design/` (tasks, workflow layers, specs) is future work".
`docs/README.md:5-6`: "the rest of `design/` is designed and not yet built". `docs/README.md:102`:
"human web UI (design, not built)".

All three are wrong today. Tasks, edges, claims, the queue and the state branch are built and in
daily use. Workflow rule resolution is built, and since `9561950` (br-2242, 34bw step 1) every
spawned, resumed or renewed agent gets its role's resolved rules (base, packs, project) in its
system prompt (`crates/bridle-daemon/src/config.rs:2615` `stable_system_prompt`, `:2660`
`role_rules_text`; called from `supervisor.rs:1044,2506,2706`). The gateway is built through
task 8 of 10 (`crates/bridle-gateway`). The spec tooling is built (below). An agent reading
CLAUDE.md first is told the opposite of what the code does. CLAUDE.md is config this check may
not edit.

**Recommendation:** replace the CLAUDE.md sentence with "The rest of `docs/design/` mixes built
and planned; each doc opens with a Status line saying which", and the same in `docs/README.md:5-6`;
drop "(design, not built)" from `docs/README.md:102`.

## 2. A large body of built machinery that nothing uses

Built and tested, with no role, rule or skill telling an agent to use it, and no onboarded project
(bridle, track-web, bridle-ui; data-contracts' clone has only `openspec/`) with the inputs it needs:

| Built | Why it never fires |
|---|---|
| `bridle spec check\|id\|import\|export\|coverage`, `goals`, `arch`, `explore`, `trace` (`crates/bridle-spec`) | no project has a `design/` tree; `git grep` finds no use in track-web or data-contracts |
| pytest and vitest adapters (`workflow/packs/{python,typescript}/adapters/`) | vendored by hand; no project has |
| `bridle task impact set\|check`, `task conflict`, `spec changed under you` | `grep impact workflow/base/roles` finds nothing; 0 of 459 tasks have a declared impact (`bridle task list --json`) |
| L4 components (`--component`, `BRIDLE_COMPONENTS`, `rules explain --component`) | `role_rules_text` resolves with no component (`rules_section(&res, role_name, None)`, `config.rs:2670`); no project declares one |
| `bridle prime worker\|planner` (facts, guides, component chains, the explore paragraph) | workers never run it; only `bridle session` tells orchestrator and advisor to prime (`crates/bridle/src/session.rs:44`) |
| `bridle sync` (skills, agents, hooks) and the `arch-guard` hook | run by hand only; bridle's `.claude/settings.json` has no `hooks`; rendered skills are gitignored (`.gitignore:13`), so worktrees have none, and the manager's `allowed_tools` lack `Skill` |
| `bridle cost audit --check` | not in `justfile`'s `check` or CI |
| port registry (`bridle port alloc`) | no role or rule mentions it |

This is the same shape as 34bw: commands exist and are tested, so docs (and agents) describe them
as the mechanism, but nothing in an agent's lifecycle invokes them. The docs now say "built, not
wired in" for each; the question is what to do with them.

**Recommendation:** leave them, under `yagni`, until a project trial needs one; don't extend them
in the meantime. When a trial does (63rv, track-web components), wire the one it needs end to end
(a role line or a spawn-time delivery) in the same task. The likeliest first is component rules:
pass the agent's `BRIDLE_COMPONENTS` to `role_rules_text` so component chains go into the system
prompt the way L1-L3 now do.

## 3. 34bw reads as current but is half done

34bw's verdict table says "The spawn prompt is the preamble, a branch sentence and one role file;
no resolved rules". Steps 1, 2 and 4 have since landed (`9561950` br-2242, `5a0593f` br-899d,
`f55537d` br-66a0); step 3 (layer hooks at spawn) hasn't. Packs now do reach agents: a project's
`packs = ["typescript"]` rules are in its agents' system prompts. An agent reading 34bw alone
would conclude packs are still documentation.

**Recommendation:** add a dated "Landed" line to 34bw naming the three commits and leaving step 3
open. Separately, `workflow/base/rules/plan-discipline.md:19-20` still says "Until `bridle workflow
sync` renders rules into agents, the role prompts ... carry this", which is now false (rules reach
agents at spawn, not through sync); that file is out of this check's scope.
