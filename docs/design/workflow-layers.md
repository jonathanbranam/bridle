# The workflow as layered, modifiable data

## The layers

```
L0  core        built into the binary: task states, edge types, command semantics
L1  base        bridle-workflow/base/          shared by every project
L2  packs       bridle-workflow/packs/<name>/   opt-in: typescript, python, vim,
                                                web-ui, game, monorepo, …
L3  project     <repo>/.bridle/                 this project's overrides + additions
L4  component   <repo>/.bridle/components/<n>/  path-scoped, e.g. client-watch/**
```

Later layers win. A project lists its packs in `.bridle/config.toml`:

```toml
project  = "track-web"
prefix   = "tw"
packs    = ["typescript", "web-ui", "monorepo"]
workflow = "~/work/bridle-workflow"      # path or git url; rev pin optional

[components.client-watch]
paths = ["client-watch/**"]
[components.dungeon-engine]
paths = ["packages/dungeon-engine/**"]
consumers = ["harness"]                  # a cross-project edge the tool knows about
```

**The base layer is edited in one place.** `bridle-workflow` is a git repo;
change a base rule, commit, and every project picks it up on its next
`bridle sync` (which the SessionStart hook runs). Nothing is copied into
projects, so nothing goes stale. A project can pin `rev = "…"` if it needs to
stay behind.

## What a layer contains

Every layer has the same shape, so an override is a file at the same path:

```
workflow.toml        lifecycle, gates, roles, models (see roles-and-lifecycle, gates)
rules/<id>.md        must/should statements, one per file, with ids
guides/<id>.md       longer how-to prose (testing, architecture, verification)
skills/<name>/       skill sources (SKILL.md + scripts)
agents/<role>.md     subagent definitions
hooks/               hook scripts, if any beyond bridle's own
facts.md             short operational facts, loaded every session (the bd prime idea)
```

## Rules have ids, and overrides are explicit

```markdown
---
id: servers.never-restart
severity: must            # must | should | may
roles: [driver, worker]
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
  `bridle rules diff --project otters` shows everything that project does
  differently. This fixes the current state where CLAUDE.md, the plan of
  record and a skill all restate a rule with no stated precedence.

SwarmForge's layering (research 14 §1.1) forbade shadowing shared files. Bridle
allows it deliberately, because the designer wants project overrides. What it
keeps from SwarmForge is that each override is visible and has an owner.

## Rendering into what the agent harness reads

Claude Code reads `CLAUDE.md`, `.claude/skills/`, `.claude/agents/`,
`.claude/settings.json`. `bridle sync` renders the resolved layers into them:

| Output | Content | Committed? |
|---|---|---|
| `CLAUDE.md` | a small **managed block** (between markers) pointing at `bridle prime`; the human-written rest of the file is untouched | yes |
| `.claude/skills/bridle-*/` | rendered skills, with project addenda appended | **no** — gitignored, regenerated |
| `.claude/agents/*.md` | rendered role definitions | no |
| `.claude/settings.json` hooks | bridle's hook entries, merged into existing settings | yes (small, stable) |
| path-scoped rules | L4 component rules, rendered as nested/path-scoped rule files so they load only when the agent works in that path (**verify** the exact Claude Code mechanism) | no |

The principle is the Gherkin lesson ([[docs/design/specs-to-tests|specs to tests]]) applied to configuration: **do not
commit generated output.** Keep the sources in git, make regeneration fast, and
there's no stale copy to check for. Everything but the managed CLAUDE.md block
and hook entries can be rebuilt with `bridle sync`.

Most rule content is not rendered into a file at all. It is delivered by
`bridle prime` at session start, sized to the role: a worker gets its task,
the rules tagged for `worker`, the facts, the guides its task's components
point to, the architecture invariants, the goals its task serves, and the
standing rule about explorations ([[docs/design/explorations|explorations]]). It does not get every rule in the
tree, and it does not get the full goals document.

## Rules improve through the workflow itself

When an agent hits a gotcha it can file a proposed rule against a layer:

```
bridle rules propose --layer project --id tests.fixture-world \
  "The 20×20 fixture world is the only one fast enough for unit tests"
```

This creates a task on the right repo (the project, or `bridle-workflow` for
base or pack changes), and the proposed rule takes effect once that task is
accepted. It's `bd remember` with review attached, so operational memory can't
go stale unnoticed (research 06 §8).
