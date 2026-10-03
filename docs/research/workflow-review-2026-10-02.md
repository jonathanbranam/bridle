# Bridle's workflow system: what it does today and what's missing

Oct 2, 2026 · Jonathan

## Summary

Bridle's workflow system today is a strict, well-tested rules engine, but almost nothing connects it to the agents bridle runs. Layer resolution, overrides, `rules explain/diff`, `sync` and `prime` are all built. What a spawned agent actually receives, though, is one role prompt file, chosen per project and not layered (one narrow exception landed today). A pack rule or project override changes what `bridle rules explain` reports, not what a worker is told.

The workflow also plays no part in an agent's lifecycle after spawn. Task states, the plan gate, the Stop hook and landing checks are hardcoded in the binary. `workflow.toml`, meant to hold lifecycle, gates and roles, contains one line per layer.

Against decision 5 ("the workflow, its rules and its guidelines live in one modifiable place"), the one place exists. It is not yet the place agents' behaviour comes from.

## What the design says the workflow is for

The workflow layer exists to fix one row of `docs/proposal/problem.md`: "Every project carries its own copy of the workflow… six CLAUDE.md files restating overlapping rules with no precedence." It serves the goal "one workflow, many projects, installed and managed the same way everywhere" and decision 5.

The design (`docs/design/workflow-layers.md`, `roles-and-lifecycle.md`, `gates.md`) asks the workflow to do six things:

1. **Layer.** L0 core (in the binary), L1 base, L2 packs, L3 project, L4 components. Later layers win, and every override is explicit, owned and explainable.
2. **Hold every kind of workflow content in the same shape per layer:** `workflow.toml` (lifecycle, gates, roles, models), `rules/`, `guides/`, `skills/`, `agents/`, `roles/`, `hooks/`, `facts.md`.
3. **Render** the resolved layers into what Claude Code reads (CLAUDE.md block, skills, subagents, hooks) with `bridle sync`, run automatically by a SessionStart hook.
4. **Deliver** most rule content at session start through `bridle prime`, sized to the role: the task, the role's rules, facts, guides, architecture invariants, the goals the task serves, and the explorations rule.
5. **Configure the lifecycle.** Models per role, and the plan, merge and accept gates (`[gates.plan]`, `[gates.merge]`, `[gates.accept]`), set per layer so a project can make its reviewer cheaper or move acceptance before merge.
6. **Improve itself.** `bridle rules propose` files a proposed rule against a layer as a task; it takes effect when accepted.

## What is built today

The resolution and rendering machinery is built and strict; the content is 20 base rules, 14 pack rules across three packs, six role prompts and two skills.

| Piece | What it does today | Where |
| --- | --- | --- |
| Layer discovery | Base, each listed pack, project; component chains (root-most first) resolved separately on top | `bridle-daemon/src/rules.rs` |
| Rule resolution | Redefining without `override` is an error; overriding nothing is an error; `locked` rules can't be touched; `disable` needs a `reason`; `replace` inherits severity, roles and locked | `rules.rs::resolve` |
| `rules explain` / `rules diff` | Shows which layer won a rule and its full history; what the project or a component changes | `commands/workflow.rs` |
| `bridle sync` | Validates rules, then writes the CLAUDE.md managed block, `.claude/skills/bridle-*` (SKILL.md appended, other files overlaid), `.claude/agents/` (replaced), hooks merged into `.claude/settings.json` with a sidecar | `bridle-daemon/src/sync.rs` |
| `bridle prime` | `worker` / `planner`: role-tagged rules, facts, guide paths, component scope, the explore paragraph. `orchestrator`, `advisor`, `prototyper`: role file plus state | `bridle/src/prime.rs`, `commands/orchestrator.rs` |
| Role prompts at spawn | Preamble + branch sentence + `roles/<role>.md` with `{{commands.*}}` and `{{branches.*}}` filled; default path `<workflow>/base/roles/<role>.md` | `config.rs::stable_system_prompt` |
| Two delivery modes | `workflow` as a path updates live; unset uses `.bridle/workflow/`, copied by `bridle init` and changed only by `bridle workflow update --to <tag>` | `bridle/src/vendor.rs` |
| Packs | `python` (5 rules + test adapter), `typescript` (5 + vitest adapter), `vim` (4) | `workflow/packs/` |
| Base content | 20 rules (4 locked), 6 role prompts (including `prototyper`), `worker` and `manager` skills, one `PreToolUse` hook (`arch-guard`) | `workflow/base/` |

One new mechanism landed on 2 October (br-a4ea): the `prototyper` role appends the project's `.bridle/roles/prototyper.md` to the base role prompt. It is the only role prompt with a project layer, and it is special-cased by role name.

## How it touches an agent's lifecycle

The workflow is read once, at spawn, to build the role prompt; every later stage is driven by the binary and `.bridle/config.toml`.

**After spawn, the binary runs the agent; the workflow supplies only its prompt**

| Stage | From the workflow layers | From the bridle binary |
| --- | --- | --- |
| Spawn | **In use:** `roles/<role>.md` with placeholders filled — the only workflow input | Preamble, branch sentence, tool lists, memory off via `--settings` |
| Session start | Nothing: no `prime`, no `sync` | `BRIDLE_*` environment variables |
| Each edit | *Built, not live:* `arch-guard` hook, only if `sync` wrote it to committed settings | Permission mode and tool lists from `.bridle/config.toml` |
| Turn end | Nothing | Stop hook: `bridle stop-check` |
| Task transitions | Nothing: `workflow.toml` is empty | Task states, plan gate, queue, settle period, claim leases |
| Landing | Nothing | `bridle land`: integration check, architecture-path refusal |

Three consequences follow.

- **Rules reach agents by restatement.** `roles/worker.md` names rules (`kiss`, `missing-tools`, `yagni`) and summarises them. `work-flow.md` says so: "Until `bridle workflow sync` renders rules into agents, the role prompts… carry this." Spike 08 measured zero `bridle prime` calls by workers; its 12.5K-token output for `worker` exists but is never read.
- **Overrides don't change behaviour.** A project's `override: replace` of `kiss` shows up in `rules explain`, but the worker still reads the base role prompt's summary of `kiss`.
- **Skills are generated but not used.** Default role tool lists leave out `Skill`, and `.claude/skills/` is gitignored, so worktrees don't have it. `skills/worker/SKILL.md` largely repeats `roles/worker.md`.

On renewal and resume the daemon rebuilds the same prompt from the role plus the agent's stored overrides; nothing new is read from the workflow.

## Gaps against the design goals

Of 13 things the design asks of the workflow, 2 are built, 5 are partly built and 6 are not started; every partial one stops short of reaching a running agent.

| Design ask | Status | Evidence |
| --- | --- | --- |
| Layered rules with explicit, owned overrides | Built | `rules.rs::resolve` enforces every rule in the design; `rules explain` and `diff` show the history |
| `bridle sync` renders skills, agents and hooks | Built | `sync.rs`. The SessionStart hook that would run it automatically is not built |
| One workflow, many projects, managed the same way | Partly | Path and vendored modes work. A git-URL `workflow` resolves to no base layer; packs and overrides don't reach agents |
| Rules delivered at session start by `prime`, sized to the role | Partly | `prime worker` and `planner` print rules, facts, guides and component scope. The daemon never runs it; task, invariants and goals are not in it |
| L4 component rules delivered by task or spawn | Partly | Resolution and `prime --component` built; spawn sets `BRIDLE_COMPONENTS`, but nothing reads it unless `prime` is run |
| Skills from layers ("the six skills") | Partly | Two skills exist (`worker`, `manager`). Spawned roles lack the `Skill` tool and worktrees lack `.claude/skills/` |
| Hooks from layers | Partly | `hooks/PreToolUse.json` (`arch-guard`) exists; bridle's own committed `.claude/settings.json` has no hooks |
| Role prompts layered like everything else | Not built | Base file or the project's `system_prompt`, wholesale. Only `prototyper` gets a project append, special-cased by name |
| Lifecycle, models and roles in `workflow.toml` | Not built | Each `workflow.toml` holds one line, `layer = "…"`. Roles and `[models]` live in `.bridle/config.toml` |
| Configurable plan, merge and accept gates | Not built | No `[gates]` table is parsed. The plan gate and `land` checks are hardcoded; no `reviewer` role exists, though 30 of 34 rules are tagged for one |
| L0 core layer | Not built | No file-backed layer; `rules.rs` defers it until L0 has content |
| Facts and guides per layer | Not built | `prime` reads `facts.md` and `guides/`, but no layer has either file |
| `bridle rules propose` | Not built | No such subcommand; rules change by hand-edited commits |

The goal most affected is "one workflow, many projects": the projects share files, but not behaviour that can be layered per project.

## Inconsistencies and stale claims

These are small, but each is a place where the docs, the code or the config disagree.

- **`bridle init --stack rust` names a pack that doesn't exist.** It writes `packs = ["rust"]`, but `workflow/packs/` has only `python`, `typescript` and `vim`; `bridle doctor` then fails on the missing pack.
- **A git-URL `workflow` silently gives an empty base layer.** `Config::workflow_root` returns `None` for `://` and `git@` values. `roles-and-config.md` says a missing workflow is "never a silent empty base layer".
- **`sync.rs` and `workflow-layers.md` say no skill, agent or hook content exists yet.** `workflow/base/skills/` has two skills and `workflow/base/hooks/PreToolUse.json` exists.
- **The CLAUDE.md managed block tells agents to read the rule files themselves.** It lists `.bridle/rules/` and `base/rules/`, but not pack or component rules, and gives no precedence: the problem the layers were built to remove.
- **`work-flow.md` waits on a sync that will never come.** It says the role prompts carry the rule "until `bridle workflow sync` renders rules into agents". That command exists, but by design it renders no rule content.
- **`roles-and-lifecycle.md` says models and roles are set in `workflow.toml` per layer.** They are set only in `.bridle/config.toml`.
- **Rules name roles that don't run.** `reviewer` and `human` appear in rule tags; neither is a spawnable role, so those tags filter nothing today.

## Recommendations

The highest-value next step is to make the daemon put resolved rules into every agent's prompt at spawn; most other gaps matter only after that.

1. **Deliver resolved rules at spawn.** Render the role's active rules (id, severity, body) into the stable system prompt, after the role file. It stays identical per role and project, so the prompt cache holds. Leave guides as paths; spike 08's 12.5K-token `prime worker` output is too large to inject whole.
2. **Then cut the restatements out of the role prompts.** `roles/worker.md` keeps the procedure and drops its rule summaries. This is the step that makes a project override change behaviour.
3. **Pass layer hooks at spawn, not through committed settings.** The daemon already injects the Stop hook through `--settings`; adding resolved `hooks/*.json` there makes `arch-guard` live in worktrees without a commit.
4. **Generalise the prototyper append.** Let any role read `.bridle/roles/<role>.md` as a project addendum, or decide that rules, not role prompts, are the per-project surface, and record that.
5. **Fix the small breaks.** Error on a git-URL `workflow` until it's resolved, add a `rust` pack or drop it from `init`, and update the stale comments in `sync.rs`, `workflow-layers.md` and `work-flow.md`.
6. **Hold `workflow.toml`, gates, skills and `rules propose` until a second project needs them.** Your own `yagni` and `cost-of-not-doing` rules apply: `.bridle/config.toml` already carries roles and models, and nothing yet needs a different plan gate.

The test for steps 1 to 3 is the one `build-order.md` set for P2: one workflow across two real projects, where an override in one changes what its agents do.

## Sources

Read from [jonathanbranam/bridle](https://github.com/jonathanbranam/bridle) at commit `be2440d` (2 October 2026).

- Design: [goals-and-non-goals.md](https://github.com/jonathanbranam/bridle/blob/main/docs/proposal/goals-and-non-goals.md), [problem.md](https://github.com/jonathanbranam/bridle/blob/main/docs/proposal/problem.md), [decisions.md](https://github.com/jonathanbranam/bridle/blob/main/docs/proposal/decisions.md), [build-order.md](https://github.com/jonathanbranam/bridle/blob/main/docs/proposal/build-order.md), [workflow-layers.md](https://github.com/jonathanbranam/bridle/blob/main/docs/design/workflow-layers.md), [roles-and-lifecycle.md](https://github.com/jonathanbranam/bridle/blob/main/docs/design/roles-and-lifecycle.md), [gates.md](https://github.com/jonathanbranam/bridle/blob/main/docs/design/gates.md), [roles-and-config.md](https://github.com/jonathanbranam/bridle/blob/main/docs/design/agent-host/roles-and-config.md), [spike 08](https://github.com/jonathanbranam/bridle/blob/main/docs/spikes/08-lean-context-findings.md)
- Code: [rules.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle-daemon/src/rules.rs), [sync.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle-daemon/src/sync.rs), [config.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle-daemon/src/config.rs), [prime.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle/src/prime.rs), [vendor.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle/src/vendor.rs), [commands/orchestrator.rs](https://github.com/jonathanbranam/bridle/blob/main/crates/bridle/src/commands/orchestrator.rs)
- Content: [workflow/](https://github.com/jonathanbranam/bridle/tree/main/workflow), [.bridle/config.toml](https://github.com/jonathanbranam/bridle/blob/main/.bridle/config.toml)
