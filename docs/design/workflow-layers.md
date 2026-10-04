# The workflow as layered, modifiable data

> **Status (checked 2026-10-03):** Built and in use: rule resolution over base, packs and project (`rules/<id>.md`, overrides, `locked`), and its delivery: every spawned, resumed or renewed agent gets its role's resolved rules in its system prompt (`config.rs` `stable_system_prompt`); `bridle rules explain|diff`; `bridle prime orchestrator|advisor` at session start · Built, not wired in: `bridle sync` (run by hand; no hook runs it, and rendered skills and hooks don't reach worktrees); `bridle prime worker|planner` (workers never run it, so facts, guides and L4 component rules don't reach them); L4 components (`rules explain|diff --component`, `BRIDLE_COMPONENTS`); layer `workflow.toml` (only `layer = ...`; nothing parses roles, gates or models from it) · Planned: layer hooks passed at spawn (34bw step 3), `bridle rules propose`, the sync changelog

## The layers

```
L0  core        built into the binary: task states, edge types, command semantics
L1  base        workflow/base/                  shared by every project
L2  packs       workflow/packs/<name>/           opt-in: typescript, python, vim,
                                                web-ui, game, monorepo, …
L3  project     <repo>/.bridle/                 this project's overrides + additions
L4  component   <repo>/.bridle/components/<n>/  scoped by task/spawn; nests (client → game)
```

L4 is built for rule resolution: `[components.<id>]` in config declares the nesting
(`parent`, validated), each component's chain is layered after L3 and resolved on its own
(never several components in one list), and `bridle rules explain|diff --component <id>`
shows it. Tasks and spawns carry the component list (`--component`, `BRIDLE_COMPONENTS`) and
`bridle prime worker|planner` prints the chains, but a spawned agent's system prompt carries
only L1–L3 and workers don't run prime, so component rules don't reach agents yet; see
[[docs/design/components|components]].

Later layers win. A project lists its packs in `.bridle/config.toml`, and
points `workflow` at wherever `base/` and `packs/` live — a directory inside
the project's own repo (bridle's own choice, decision r2uq: a separate repo
was "too much hassle"), a path to a sibling checkout, or a git url:

```toml
project  = "track-web"
prefix   = "tw"
packs    = ["typescript", "web-ui", "monorepo"]
workflow = "workflow"                    # path or git url

[components.client-watch]
paths = ["client-watch/**"]
docs  = "docs/watch"                     # see docs/design/components.md
[components.dungeon-engine]
paths = ["packages/dungeon-engine/**"]
consumers = ["harness"]                  # a cross-project edge the tool knows about
```

**The base layer is edited in one place.** Whether `workflow` is a directory
in this repo or a separate git repo shared across projects, the same rule
applies: change a base rule, commit, and every project's agents pick it up at
their next spawn, resume or renew, when the daemon resolves the rules into the
system prompt. Skills and hooks change only when someone runs `bridle sync`;
no hook runs it. Nothing is copied into projects, so nothing goes stale.

**Two modes** (2026-09-29, ticket mrhe). A project whose `workflow` is a path to a local
bridle clone updates automatically, as below. A project on an installed binary has the base
workflow vendored into `.bridle/workflow/` by `bridle init` (used whenever `workflow` is
unset) and it changes only when the human runs `bridle workflow update [--to <tag>]`: opt-in,
so the workflow never shifts under them. The vendored copy is committed by the human.

**Updates apply automatically by default** (the path mode). The human's words, 2026-09-28:
there's no rev pinning for the common case — a project just gets whatever
`workflow` currently has next time it syncs. A project that wants to know
what changed reads a changelog (planned: there is no `workflow/CHANGELOG.md`
yet, and `bridle sync` doesn't print what's new since the last sync). A project that disagrees with a
specific base rule doesn't pin or fork — it opts out with a local
`override: disable` and a `reason` (below), which stays visible instead of
silently drifting behind.

## What a layer contains

Every layer has the same shape, so an override is a file at the same path:

```
workflow.toml        lifecycle, gates, roles, models (see roles-and-lifecycle, gates)
rules/<id>.md        must/should statements, one per file, with ids
guides/<id>.md       longer how-to prose (testing, architecture, verification)
skills/<name>/       skill sources (SKILL.md + scripts)
agents/<role>.md     Claude Code subagent definitions (rendered to .claude/agents/)
roles/<role>.md      driver-facing role prompts (bridle's own worker/manager/etc,
                      referenced by system_prompt in .bridle/config.toml)
hooks/               hook scripts, if any beyond bridle's own
facts.md             short operational facts, loaded every session (the bd prime idea)
```

What exists in `workflow/` today: `rules/` (base and every pack), `roles/` and `skills/` (base),
`hooks/PreToolUse.json` (base), and a `workflow.toml` per layer that holds only `layer = ...`.
No layer has `guides/`, `agents/` or `facts.md` yet; `bridle prime worker|planner` prints facts
and guide paths when they exist, but the spawn-time system prompt carries rules only.

`agents/<role>.md` and `roles/<role>.md` are easy to confuse but not the same thing:
`agents/` is Claude Code's own subagent mechanism (the `Agent` tool, `.claude/agents/`);
`roles/` is bridle's driver-facing role prompt, appended after bridle's own preamble to
the `claude` process's system prompt for a whole bridle role (worker, manager,
project-manager, orchestrator, advisor, prototyper) — see
[[docs/design/agent-host/roles-and-config|roles and config]]. `bridle sync` renders
`agents/` into `.claude/agents/*.md` (below); it does nothing with `roles/`. Instead, a
role with no `system_prompt` in `.bridle/config.toml` defaults to
`<workflow>/base/roles/<role>.md` when that file exists, and a project's own `system_prompt`
wins. There is no layer overlay of role prompts.

## Rules have ids, and overrides are explicit

```markdown
---
id: servers.never-restart
severity: must            # must | should | may
roles: [manager, worker]
---
Never kill or restart a dev server the human is running. …
```

A project overrides by id:

```markdown
---
id: verify.browser
override: replace          # replace | append | disable
reason: otters has no browser path; the CLI harness is the verification surface
---
Verify with `npm run cli -- …` against the fixture world, not playwright.
```

- `disable` requires a `reason`. Silent drops are how the current system lost
  rules.
- A base rule may be `locked: true` (e.g. *only the human accepts work*). A
  project can't override a locked rule; it has to be changed in base.
- `bridle rules explain <id>` shows which layer won and what it shadowed.
  `bridle rules diff --project-layer` (run in the project) shows everything that
  project does differently. This fixes the current state where CLAUDE.md, the plan of
  record and a skill all restate a rule with no stated precedence.

SwarmForge's layering (research 14 §1.1) forbade shadowing shared files. Bridle
allows it deliberately, because the designer wants project overrides. What it
keeps from SwarmForge is that each override is visible and has an owner.

## Rendering into what the agent harness reads

Claude Code reads `CLAUDE.md`, `.claude/skills/`, `.claude/agents/`,
`.claude/settings.json`. `bridle sync` renders the resolved layers into them, when someone runs
it by hand in a clone. The rendered skills are gitignored, so a worker's fresh worktree has none,
and the hook entries go into `.claude/settings.json` only where sync ran and the result was
committed (bridle's own committed settings have no `hooks`, so the base `arch-guard` hook isn't
live here):

| Output | Content | Committed? |
|---|---|---|
| `CLAUDE.md` | a small **managed block** (between markers) pointing at `bridle prime`; the human-written rest of the file is untouched | yes |
| `.claude/skills/bridle-*/` | rendered skills, with project addenda appended | **no** — gitignored, regenerated |
| `.claude/agents/*.md` | rendered role definitions | no |
| `.claude/settings.json` hooks | bridle's hook entries, merged into existing settings | yes (small, stable) |
| path-scoped rules | none: L4 component rules are delivered by `bridle prime` from the task's or spawn's named components, not rendered ([[docs/design/components|components]]) | n/a |

The principle is the Gherkin lesson ([[docs/design/specs-to-tests|specs to tests]]) applied to configuration: **do not
commit generated output.** Keep the sources in git, make regeneration fast, and
there's no stale copy to check for. Everything but the managed CLAUDE.md block
and hook entries can be rebuilt with `bridle sync`.

`bridle sync` is built (P2-3), local like `rules explain`/`diff` (no daemon call — see
[[docs/design/cli|cli.md]]). A few things the table above left open turned out to need a
concrete answer to implement, so — layer-content conventions this command owns, not
written down anywhere else yet:

- **A layer's skill/agent/hook sources sit next to its `rules/` directory**, not inside
  it: `<layer root>/skills/<name>/`, `<layer root>/agents/<role>.md`,
  `<layer root>/hooks/<event>.json` (e.g. `workflow/base/skills/verify/`,
  `<repo>/.bridle/agents/worker.md`).
- **Skills**: each layer's `skills/<name>/` is overlaid onto the same name's output
  directory (`.claude/skills/bridle-<name>/`) in layer order, file by file — except
  `SKILL.md`, which a later layer's `SKILL.md` is *appended* to rather than replacing.
  That's "project addenda appended" made concrete: a project only needs a
  `skills/<name>/SKILL.md` with the extra paragraph, not a full copy of the skill.
- **Agents**: each layer's `agents/<role>.md` replaces the earlier layer's file for that
  role wholesale — no addenda convention, since a role definition doesn't read well as
  fragments.
- **Hooks**: each layer's `hooks/<event>.json` is a JSON array of Claude Code hook-config
  entries for that event name (e.g. `hooks/SessionStart.json`); a later layer's file for
  the same event replaces the earlier one wholesale, same as agents. `bridle sync` merges
  the result into `.claude/settings.json`'s `hooks.<event>` array without disturbing
  anything a human added by hand: a gitignored sidecar,
  `.claude/.bridle-sync-hooks.json`, records exactly which entries the previous sync
  wrote, and only those are removed before the current ones go in.
- **Layer hooks are live at spawn, without `bridle sync`**: the daemon resolves the same
  overlay (base, packs, project; `sync::discover_hooks_lossy`) and merges it into the
  `--settings` of every spawned agent, beside bridle's own hooks. An event with both keeps all
  of them: arrays concatenate, bridle's `Stop` check first, so a layer can't drop or override
  it. A malformed, unreadable or non-array hook file is skipped with a tracing warning and
  never fails a spawn (`bridle sync` still errors on it). Entries identical to ones already
  in the project's committed `.claude/settings.json` (what `bridle sync` writes) are dropped,
  so a synced project doesn't run a hook twice; sync's own output is untouched. Commands
  resolve on the agent's `PATH`, which points at the daemon's own binary, so
  `bridle arch-guard` works from any worktree.
  `bridle session orchestrator|advisor|aide` merges the same overlay into its `--settings`
  (current directory's project; bridle's own focus, reply and session-note hooks first), so the
  human's interactive sessions get layer hooks too. Projects that want to stamp prompts with
  a timestamp can add `hooks/UserPromptSubmit.json` in their own layer or in a pack.
- **Little skill/agent/hook content exists in `workflow/` yet**: `base/hooks/PreToolUse.json`
  and the `manager`/`worker` skills; no `agents/<role>.md`. The conventions above are
  sync's contract for the rest.
- **Per-project command bindings**: a skill source can reference `{{commands.check}}`
  instead of hardcoding a build tool. `bridle sync` substitutes it from
  `.bridle/config.toml`'s `[commands] check` (default `"just check"`, so bridle's own
  project needs no explicit setting, though it sets one anyway for clarity). Projects
  like data-contracts that use `make check` set `commands.check = "make check"` and get
  the same base `workflow/base/skills/worker/SKILL.md` rendered with their own command.
- **The `SessionStart` hook that would run `sync` automatically is not built yet** — a
  follow-up (P2-3 built the command itself, not the auto-invocation).
- L4 component rules aren't rendered into files: scope comes from the task or spawn, and
  `bridle prime worker|planner` delivers them ([[docs/design/components|components]]).

Most rule content is not rendered into a file at all. For agents bridle spawns, the daemon
appends the role's resolved rules (L1–L3, the rules tagged for the role) to the system prompt
under `## Workflow rules`; that is how rules reach workers, managers and the PM today. The
orchestrator and advisor sessions get theirs from `bridle prime orchestrator|advisor`, which
`bridle session` tells them to run.

The design goes further (planned): a worker is primed with its task, the rules tagged for
`worker`, the facts, the guides its task's components point to, the architecture invariants,
the goals its task serves, and the standing rule about explorations
([[docs/design/explorations|explorations]]), sized to the role. Built so far:
`prime worker|planner` prints the role's rules, facts and guide paths and the named
components' scope (see cli.md), but nothing in a worker's lifecycle runs it; the task,
invariants and goals aren't in it. The explorations rule is a base rule, so it reaches every
spawned agent with the rest. A worker does not get every rule in the tree, and it does not get
the full goals document.

## Rules improve through the workflow itself (planned)

When an agent hits a gotcha it can file a proposed rule against a layer:

```
bridle rules propose --layer project --id tests.fixture-world \
  "The 20×20 fixture world is the only one fast enough for unit tests"
```

`bridle rules propose` doesn't exist yet. It would create a task on the right repo (the project, or `bridle-workflow` for
base or pack changes), and the proposed rule would take effect once that task is
accepted. It's `bd remember` with review attached, so operational memory can't
go stale unnoticed (research 06 §8).
