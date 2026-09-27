# Bridle — design proposal

*Started 2026-09-26. The successor to the markdown driver workflow in
[`../../workflow-instructions/`](../../workflow-instructions/), the research in
[`../../workflow/research/`](../../workflow/research/), and the Gherkin tooling
in [`../../workflow-tools/`](../../workflow-tools/).*

> **Status: proposal.** The decisions in §1.2 are the designer's. Everything
> else is a recommendation to be argued with. §15 lists what is still open.

---

## 1. What this is for

### 1.1 The problem

The current system is a good *workflow* on a poor *substrate*:

| Hurts today | Why |
|---|---|
| An agent cannot wait on another agent | There is no dependency edge and no wake-up; the driver holds "wait for the test-config fix" in its head |
| Agents cannot talk to each other | A subagent's only channel is its final report, to its parent |
| Two tasks cannot touch the same spec at once | OpenSpec folds deltas only at archive, matches requirements by heading text, and so refuses the second change (the *same-spec pile-up*, research 10 §2b) |
| The workflow is sequential with many human checks | Plan approval, blocking questions, archive, and the pile-up rule all wait on one person |
| Every project carries its own copy of the workflow | Six repos, six sets of ~10 OpenSpec skills, six CLAUDE.md files restating overlapping rules with no precedence |
| Gherkin generation is slow and brittle | Generated `.feature` files are committed, so they need a staleness check, two validation layers, `gherkin-official` and `uv run` |

The workflow's *shape* is right and has independent confirmation: plan with a
strong model, implement with a cheaper one, review with a strong one that is not
the implementer (Wheelhouse converged on the same three steps; research 06 §2).
What changes is the machinery under it and how much of it waits on the human.

### 1.2 Decided

1. **One machine drives.** All agents run on one machine and coordinate through
   it. Moving to a VPS or another machine must remain possible.
2. **Git is the source of truth.** Anything durable can be rebuilt from git
   remotes. Live coordination state may sit in a local database, but losing that
   database must lose nothing that matters.
3. **Rust.** Single static binary, `bridle`.
4. **OpenSpec is up for replacement, and so are all the skills.** Keep the
   workflow's shape; lose its sequencing and most of its human checks.
5. **The workflow, its rules and its guidelines live in one modifiable place**,
   as a shared base with layers on top: technology and solution packs, then per-project
   overrides and additions.
6. **Project knowledge is tiered** (added 2026-09-27): long-term goals with
   firmness and priority, a project-wide architecture that only the human can
   revise, detailed specs that trace up to it, and explorations that are allowed
   to diverge from all of it (§7).
7. **Budget: one $100/month subscription, no overage** (added 2026-09-27).
   Token efficiency is a design constraint, and bridle manages usage limits:
   it picks models, pauses at limits, resumes at reset, and tracks token use
   over time (§11).

### 1.3 Goals

- **Send agents off and let them coordinate.** Dependencies, waiting, messages,
  questions — without the driver relaying everything.
- **Parallel by default, serialised only by a real conflict**, and let agents
  find and negotiate those conflicts themselves.
- **One workflow, many projects**, installed and managed the same way everywhere.
- **Human attention spent on decisions and acceptance only.**

### 1.4 Non-goals

- Fleet scale. This is one person with a driver and a handful of workers, not
  Gas Town's 20–30 agents or Wheelhouse's 12,000 commits a day. Every trade that
  Beads v1.0 made for throughput (Dolt, daemon, memory decay) is out of scope.
- Autonomous merge-to-production. Research 03 §4 is the reason.
- A general orchestrator for other people. It is opinionated for these six repos
  and this workflow; the layering exists so *this designer's* projects can
  differ, not so strangers can configure it.

---

## 2. The projects it has to serve

The layering (§4) is justified by how different these are:

| Project | Stack | What makes it distinct | Tracking today |
|---|---|---|---|
| **track-web** | TS monorepo, 10 `client-*` apps, `packages/`, SQLite | Many unrelated clients in one repo (time, watch, games, trips, family…) — rules differ **per client**; dev ports in `packages/config/dev-ports.json`; browser verification on a disposable second instance | OpenSpec, 112 specs |
| **pi/harness** | TS, pi-based | Consumes track-web's `dungeon-engine` via a relative `file:` symlink → worktrees must be **paired siblings** (research 09 §2); never kill dev servers; reserved ports 4100–4300 / 5175–5177 | OpenSpec, 25 specs |
| **otters** | TS game (`otter-life` + others) | Simulation/rendering separation; CLI-driven testing with no browser; world generation | `br` (beads_rust) |
| **file-db** | Python now, TS planned | GitHub-backed store; two language bindings must behave the same | OpenSpec, 0 specs |
| **data-contracts** | Python, uv | Spec→Gherkin→pytest-bdd pipeline; "no assistant memory — everything durable is git-tracked" | OpenSpec, 6 specs |
| **meta-notes** | Vimscript + Python | Vim test runner; bare-function pytest style; a plugin *and* a CLI | OpenSpec (Beads before that), 17 specs |

Two of the six have already tried a Beads tracker and one moved off it. That is
evidence the task-graph idea is wanted and that a tracker alone is not enough.

---

## 3. Architecture in one picture

```
                         ┌──────────────────────────────────────────┐
                         │  bridle-workflow repo  (the one place)    │
                         │   base/   packs/<stack>/   (git)          │
                         └──────────────┬───────────────────────────┘
                                        │ resolved per project
  ┌─────────────────────────────────────▼─────────────────────────────────────┐
  │  bridle (Rust CLI)                                                         │
  │                                                                            │
  │  ~/.bridle/bridle.db  (SQLite, WAL)  ── live index + ephemeral state       │
  │     tasks · edges · claims · messages · impact · agents · waits            │
  │                                                                            │
  │  per project:                                                              │
  │    <repo>/.bridle/            project layer: config, rules, overrides (git)│
  │    <repo>/design/             goals, architecture, specs, explorations (git) │
  │    state branch `bridle`      task records + event log (git, own worktree) │
  │    .claude/…  CLAUDE.md block rendered outputs (gitignored or managed)     │
  └──────────┬───────────────────────────────┬─────────────────────────────────┘
             │ hooks: prime / inbox / heartbeat │ bridle wait (background Bash)
     ┌───────▼───────┐   ┌──────────────┐   ┌─▼────────────┐
     │ driver session │   │ worker (wt A) │   │ worker (wt B) │  … any project
     └───────────────┘   └──────────────┘   └──────────────┘
```

Three kinds of state, each in the place its lifetime demands:

| State | Lives in | Lifetime | Survives a lost DB? |
|---|---|---|---|
| Workflow, rules, guidelines, skill sources | `bridle-workflow` repo + `<repo>/.bridle/` | months–years | yes (git) |
| Goals, architecture, specs, exploration findings | `<repo>/design/`, edited on task branches (§7) | months–system lifetime | yes (git) |
| Tasks, edges, decisions, answered questions, impact | state branch `bridle` in each repo | until closed, then history | yes (git) |
| Claims, leases, heartbeats, unread flags, waits, agent chatter | `~/.bridle/bridle.db` only | minutes–days | **no, by design** |

`bridle rebuild` recreates the database from the state branches of every
registered project. That is the migration story: clone the repos on the new
machine, `bridle project add` each, `bridle rebuild`.

---

## 4. The workflow as layered, modifiable data

### 4.1 The layers

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

### 4.2 What a layer contains

Every layer has the same shape, so an override is a file at the same path:

```
workflow.toml        lifecycle, gates, roles, models (§5)
rules/<id>.md        must/should statements, one per file, with ids
guides/<id>.md       longer how-to prose (testing, architecture, verification)
skills/<name>/       skill sources (SKILL.md + scripts)
agents/<role>.md     subagent definitions
hooks/               hook scripts, if any beyond bridle's own
facts.md             short operational facts, loaded every session (the bd prime idea)
```

### 4.3 Rules have ids, and overrides are explicit

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

### 4.4 Rendering into what the agent harness reads

Claude Code reads `CLAUDE.md`, `.claude/skills/`, `.claude/agents/`,
`.claude/settings.json`. `bridle sync` renders the resolved layers into them:

| Output | Content | Committed? |
|---|---|---|
| `CLAUDE.md` | a small **managed block** (between markers) pointing at `bridle prime`; the human-written rest of the file is untouched | yes |
| `.claude/skills/bridle-*/` | rendered skills, with project addenda appended | **no** — gitignored, regenerated |
| `.claude/agents/*.md` | rendered role definitions | no |
| `.claude/settings.json` hooks | bridle's hook entries, merged into existing settings | yes (small, stable) |
| path-scoped rules | L4 component rules, rendered as nested/path-scoped rule files so they load only when the agent works in that path (**verify** the exact Claude Code mechanism) | no |

The principle is the Gherkin lesson (§9) applied to configuration: **do not
commit generated output.** Keep the sources in git, make regeneration fast, and
there's no stale copy to check for. Everything but the managed CLAUDE.md block
and hook entries can be rebuilt with `bridle sync`.

Most rule content is not rendered into a file at all. It is delivered by
`bridle prime` at session start, sized to the role: a worker gets its task,
the rules tagged for `worker`, the facts, the guides its task's components
point to, the architecture invariants, the goals its task serves, and the
standing rule about explorations (§7.5). It does not get every rule in the
tree, and it does not get the full goals document.

### 4.5 Rules improve through the workflow itself

When an agent hits a gotcha it can file a proposed rule against a layer:

```
bridle rules propose --layer project --id tests.fixture-world \
  "The 20×20 fixture world is the only one fast enough for unit tests"
```

This creates a task on the right repo (the project, or `bridle-workflow` for
base or pack changes), and the proposed rule takes effect once that task is
accepted. It's `bd remember` with review attached, so operational memory can't
go stale unnoticed (research 06 §8).

---

## 5. The workflow itself

### 5.1 Roles

Split by what each role may decide (research 09 §4):

| Role | Default model | Decides | Never |
|---|---|---|---|
| **Human** | — | priorities, product questions, **acceptance** | watch agents type |
| **Driver** | Opus/Fable-class, long-lived | decomposition, plans, ordering, conflict arbitration, what to ask the human | implement bulk code; accept |
| **Worker** | Sonnet-class, per task | how to implement a planned task; negotiating conflicts with peers | change design silently; accept; talk to the human directly |
| **Reviewer** | strong model, never the task's implementer | whether a diff matches its plan and specs | fix what it reviews |
| **Integrator** | *not an agent* — bridle itself | merge order, conflict probes, rebase notices | resolve a semantic conflict |

Models and roles are set in `workflow.toml` per layer, so a project can make its
reviewer cheaper or its worker stronger.

### 5.2 Task lifecycle

```
          ┌────────── question posted ─────────┐
          ▼                                     │
 open ─► planned ─► ready ─► claimed ─► in_review ─► integrated ─► accepted
   │                  ▲         │           │             │
   │                  └─ lease lapses / released           └─► reopened
   └──────────────────────────────► dropped (reason required)

 blocked        derived: an open `blocks` edge, or an unanswered question
 needs-input    derived: a question addressed to the human
```

- **ready** is computed: planned, no open blockers, no unanswered questions.
  `bridle ready` is the dispatch primitive.
- **claimed** carries a lease renewed by the heartbeat hook (§6.4). If the lease
  expires, the task returns to ready, and whatever the worker wrote on it goes
  to the next claimant (research 10 §5).
- **Kinds** change the gates and the prime: `feature`, `bug`, `chore`,
  `question`, `research`, `explore` (§7.5), `arch-revision` (§7.3) and
  `re-evaluate` (§7.4).
- **integrated** means merged to the integration branch with tests green.
  **accepted** means the human said yes. They are separate states, so specs fold
  continuously while acceptance stays with the human (research 10 §3.2).

### 5.3 Gates: where the human is and isn't

Each gate is configured, not hardcoded:

```toml
[gates.plan]
default  = "driver"                              # driver reviews plans
human_when = ["impact.protected", "new_capability", "kind == 'arch-revision'"]  # arch-revision is locked
skip_when  = ["kind == 'explore'"]            # explorations are not plan-gated

[gates.merge]
default  = "reviewer+tests"

[gates.accept]
default  = "human"                               # locked in base
batch    = true                                  # human reviews a queue, not each task live
when     = "after-merge"                         # or "before-merge" per project
```

Where each current human check goes:

| Current check | New home |
|---|---|
| Approve every plan | **driver**, unless the plan touches a spec requirement marked `protected`, creates a capability, or the driver chooses to escalate |
| Blocking question stops the driver | **async**: the question goes on the task, the task blocks, and the agent claims other work (§6.3) |
| Archive after "land the work" | **gone**. Specs fold on merge; acceptance is a state change |
| Finish one same-capability change before starting the next | **gone**. The impact registry decides (§8.3) |
| Accept finished work | **kept**, as a batched queue: `bridle review` |
| *(new)* Revise the architecture | **human, always**, via an `arch-revision` task (§7.3) |
| *(new)* Change a goal's firmness, priority or stance | **human**; agents may propose (§7.2) |
| *(new)* Divergence introduced by an exploration | **not a human check**. The human hears about it when the exploration concludes (§7.5) |

`when = "before-merge"` is for projects where an unaccepted change on the
integration branch is itself a hazard; harness is the likely candidate. The
default is after-merge, with `reopened` for rejected work.

### 5.4 The loop, from the driver's seat

```
bridle ready --all                      # what can move, across projects
bridle task new "Fix vitest config" -k chore            → tw-c0f1
bridle task new "Watch: ratings filter" -k feature      → tw-7fa2
bridle dep add tw-7fa2 --blocked-by tw-c0f1
bridle plan tw-7fa2                     # driver writes plan + spec edits + impact
bridle spawn worker tw-c0f1             # worktree + session, claims the task
bridle wait tw-c0f1 --until integrated  # run as background Bash; driver is woken
…
bridle spawn worker tw-7fa2             # now ready
bridle review                           # human's batch: diffs, scenarios, verification notes
```

The driver does not have to stay alive for this. The edge, the wait and the
plan are all in the store, so a new driver session can run `bridle prime` and
pick up where the last one stopped.

---

## 6. Coordination and communication

### 6.1 Edges

| Edge | Meaning | Affects ready? |
|---|---|---|
| `blocks` | B cannot start until A is integrated (or a named state) | yes |
| `parent` | decomposition | parent closes when children close |
| `discovered-from` | found while working on another task | no |
| `related`, `supersedes`, `duplicates` | provenance | no |

Edges can cross projects (`hx-19ab blocked-by tw-7fa2`), which turns "engine
first, host second" into something the tool enforces.

### 6.2 Messages

A message goes to an agent, a task (all current and future claimants), a role
(`driver`) or `human`. Kinds:

| Kind | Effect |
|---|---|
| `note` | informational; appended to the task's thread if addressed to a task |
| `question` | **blocks** the task until answered; to `human` it shows in `bridle inbox --human` |
| `answer` | unblocks; the answer is written durably to the task record |
| `handoff` | "here is where I left it" when releasing a claim |
| `conflict` | opened by the impact registry between two claimants (§8.3) |
| `system` | from bridle: rebase needed, blocker cleared, lease lapsing |

Messages addressed to a task are durable: they go to the state branch as part of
the task's thread, which gives later claimants the context Gas Town calls
*seance*. Agent-to-agent chatter is kept only in the database and expires.

### 6.3 Questions do not stop work

```
worker hits a question ─► bridle ask tw-7fa2 --to driver "…"
   task → blocked; worker writes a handoff note, releases or keeps claim
   worker ─► bridle ready ─► claims something else
driver answers, or forwards to human ─► answer lands on the task ─► task ready
   ANY worker may claim it — not necessarily the one that asked
```

### 6.4 How agents actually hear things (Claude Code integration)

| Mechanism | Used for |
|---|---|
| **SessionStart hook** → `bridle prime` | identity, claimed task, unread messages, facts, role-scoped rules |
| **PostToolUse / UserPromptSubmit hook** → `bridle inbox --inject` | new messages injected into the running agent's context between tool calls; also renews the claim's heartbeat. Must be fast (<20 ms) and silent when there's nothing new |
| **Stop hook** → `bridle stop-check` | refuses to let a worker stop with an unreleased claim and no handoff note |
| **`bridle wait` as background Bash** | the driver or a worker is re-invoked when a task reaches a state or a message arrives |
| **PreToolUse hooks** | enforce locked rules mechanically, e.g. workers can't run `git push` or `bridle accept` |

> **Update 2026-09-27:** [`research/01-agent-runtime.md`](research/01-agent-runtime.md)
> answers most of this. Hooks do fire in subagents and carry `agent_id`. The
> recommendation is headless `claude -p` stream-json workers that bridle owns
> directly: stdin to wake an agent, hooks for mid-turn injection and status,
> and a bridle MCP server for tools.
>
> **Update 2026-09-27 (later):** spike 01 verified the stream-json host, and
> [`agent-host.md`](agent-host.md) designs the daemon, API and agent host built
> on it. Mid-turn delivery there uses stdin rather than hooks.

**To verify before building on it:** whether hooks fire inside `Agent`-tool
subagents with enough identity (session/agent id) to tell them apart, or whether
workers must be separate `claude` sessions started by `bridle spawn` (tmux +
worktree). The design supports both, since everything goes through the store.
Which is the default depends on the answer.

### 6.5 Worktrees and ports

`bridle spawn` creates the task's worktree according to a layout the project
declares, including harness's **paired** sibling layout (research 09 §2.2):

```toml
[worktree]
layout = "paired"
root   = "/Volumes/Data/work/pi/wt/{task}"
pair   = { harness = "worktree", "track-web" = "symlink|worktree" }
setup  = "npm install --prefer-offline"
```

Ports come from a registry in the database. The human's reserved ports are
excluded by config, and every allocation records task and pid, so "stop what
you start" can be checked.

---

## 7. Project knowledge: goals, architecture, specs, explorations

### 7.1 The tiers

Each project keeps its knowledge in four tiers of artifact, ordered from long
lifetime to short. They sit under a configurable root, `design/` by default:

| Tier | Where | Holds | Changes | Who approves a change |
|---|---|---|---|---|
| **Goals** | `design/goals.md` | long-term direction, some clear and some fuzzy | at planning sessions | **human** (agents may propose) |
| **Architecture** | `design/architecture/*.md` | project-wide design: principles, invariants, component boundaries, key decisions with alternatives | rarely, deliberately | **human, always** (locked rule) |
| **Specs** | `design/specs/<capability>.md` | detailed behaviour: requirements and scenarios | by tasks, continuously | driver, or human for `protected` requirements (§5.3) |
| **Explorations** | `design/explore/<task>/` | spike findings that may contradict everything above | per spike | nobody; they are not adopted by being written |

Tasks sit below all four, on the state branch (§10.1). Everything in the table
is in-tree and changes on code branches, so it is reviewed as a diff with the
work that motivated it.

Every element in every tier has a stable id (`g-`, `a-`, `r-`, `s-`), so the tiers
can link to each other (§7.4).

### 7.2 Goals: direction, not scope

Goals are the long-term direction. They are **not a backlog**: most of them are
not being built now, and some are deliberately left out of the current design
because building toward them now would be too complex. Each goal is
categorised on two axes, plus a stance that says how the current design relates
to it:

```markdown
## Offline-first clients                         {#g-03}
firmness: firm · priority: later · stance: unaddressed

Every client should work without a network connection and sync on reconnect.

**Why unaddressed:** it needs a sync engine and a conflict model we don't have.
Designing for it now would stall everything else. The current design is
online-only on purpose.
```

| Axis | Values | Meaning |
|---|---|---|
| **firmness**: how locked the requirement is | `fixed` · `firm` · `soft` · `open` | `fixed`: the project will meet this as stated. `firm`: intended, but the shape may change. `soft`: likely, details open. `open`: a direction we are curious about, which may be dropped |
| **priority**: when it matters | `now` · `next` · `later` · `someday` | ordering of attention, not a schedule |
| **stance**: how the current design relates | `build` · `keep-open` · `unaddressed` | `build`: current work may implement it. `keep-open`: don't implement it, but don't make it harder. `unaddressed`: the current design intentionally ignores it |

The stance defaults from priority (`now`→`build`, `next`→`keep-open`,
`later`/`someday`→`unaddressed`) and can be set explicitly. `unaddressed` needs a
one-line *why*.

Base rules for goals (locked):

- **A worker never implements a goal its task does not link to.** Goals are
  context, not instructions.
- **A gap between a goal and the current design is expected.** Agents don't
  report it unless the goal's stance is `build`. An `unaddressed` goal missing
  from the design is the intended state.
- **Plans cite the goals they serve** (`serves: g-03`), so the driver can see
  which goals have work behind them.
- **Changing a goal's firmness, priority or stance is the human's call.** An
  agent can propose a change with `bridle goals propose`.

### 7.3 Architecture: stable, revisable only with the human

The architecture tier holds what every task has to respect: principles, the
governing invariants (*the engine referees every rule*), component boundaries,
and major decisions recorded with the alternatives that were rejected. Each
element has an id and may be marked `invariant`.

It is **largely immutable, but open for revision**. It is not frozen, but it
never changes as a side effect of other work:

- Any edit under `design/architecture/` requires a task of kind
  `arch-revision`, and that task's plan gate is the human (§5.3). This is
  enforced, not only stated. A PreToolUse hook stops workers editing those files
  outside an `arch-revision` task, and the integrator refuses to merge a branch
  that touches them without a linked, human-approved revision.
- An agent that thinks the architecture is wrong runs `bridle arch propose`,
  which creates an `arch-revision` task containing its argument. It then either
  continues within the current architecture or blocks its own task on the
  revision. It does not work around the architecture silently.
- An accepted revision starts the downstream re-evaluation described in §7.4.

### 7.4 Traceability: from goals down to tests

Each tier links upward to the one above it:

```
goal  g-03
  └─ architecture element  a-12cd        serves: g-03
       └─ requirement  r-7fa2             traces: a-12cd@3f9e
            └─ scenario  s-b310           (belongs to r-7fa2)
                 └─ test                  bound by scenario id (§9)
```

Links are written inline in the lower element, next to its id:

```markdown
### Requirement: The engine referees every rule    {#r-7fa2 traces=a-12cd@3f9e}
```

The `@3f9e` is a short hash of the upstream element's text **as of when this
link was last confirmed**. That is what makes change tracing mechanical (the
*suspect link* idea, borrowed from requirements-management tools):

- When an upstream element's text changes on merge, every link pointing at it
  with an old hash becomes **suspect**.
- `bridle trace suspect` lists suspect links. When an `arch-revision` is
  accepted, bridle opens one `re-evaluate` task per affected capability, listing
  its suspect requirements.
- For each requirement, the worker on that task either **confirms** it
  (`bridle trace confirm r-7fa2`, which rewrites the hash and records that it is
  still valid) or **edits** it, which may in turn make its scenarios suspect.
- An `arch-revision` task's impact (§8.3) is the whole downstream set, so the
  conflict check warns every in-flight task on an affected spec before the
  revision merges, not afterwards.

Queries:

```
bridle trace down a-12cd     # everything that depends on this element
bridle trace up s-b310       # why this scenario exists, up to goals
bridle trace orphans         # requirements tracing to nothing (warning, configurable)
bridle trace coverage g-03   # how much of this goal has design, specs, tests
```

### 7.5 Explorations: deliberately divergent, and known to be

A spike or exploration is a task of kind `explore`. It is **meant** to
contradict the accepted goals, architecture or specs, because its purpose is to
try something different. The recurring failure is that other agents come across
its artifacts, are surprised, and report the divergence as a defect. The design
deals with this up front:

- **The exploring agent is told at prime.** `bridle prime` for an `explore` task
  says: you are exploring an alternative; contradicting the architecture, goals
  and specs is intended; don't bring the code back into line and don't flag the
  divergence.
- **Every other agent is told too.** A locked base rule, delivered at every
  prime and as a path-scoped rule on `design/explore/**`, says: *material under
  `design/explore/`, or marked `exploratory: true`, deliberately contradicts the
  current design. It is not a defect, not stale, and not something to reconcile
  or report.*
- **Each exploration labels itself.** The findings doc starts with frontmatter
  bridle checks:

  ```yaml
  exploratory: true
  task: tw-e41a
  diverges-from: [a-12cd, r-7fa2]
  status: open            # open | concluded | adopted | abandoned
  ```

- **Explorations are exempt from checks built for mainline work**: the
  architecture gate, spec protection, impact conflicts against mainline tasks
  and trace validation. Locked safety rules (ports, dev servers, no pushing)
  **still apply**.
- **Exploration code never merges to main.** It stays on `explore/<id>`. Only
  the findings doc may merge, into `design/explore/<id>/`.
- **The human is not told about the divergence.** They hear about an exploration
  when it concludes with a recommendation. Adopting it goes through the normal
  path: an `arch-revision` and/or a goals change (human gates), then ordinary
  tasks on mainline. Until that happens, the exploration has no authority.

---

## 8. Specs: keep the model, replace the lifecycle

### 8.1 What stays

Capability → requirement (`SHALL`) → scenario (`GIVEN/WHEN/THEN`), with the
verification marker from the current Gherkin tooling. Research 10 §2a covers why
this part works.

### 8.2 What changes

1. **Stable ids** on requirements and scenarios, assigned by `bridle spec id`
   and never reused:

   ```markdown
   ### Requirement: The engine referees every rule            {#r-7fa2}
   #### Scenario: Reach comes from the engine                  {#s-b310}
   *Verification*: **executable** @engine
   - **GIVEN** a unit with reach 2
   - **WHEN** the bench asks for legal targets
   - **THEN** the engine's answer is used unchanged
   ```

2. **No deltas.** A task edits `design/specs/<capability>.md` **in place on its
   branch**. The plan commit's spec diff is the proposal. Merging the branch
   updates the spec, and there is no separate archive step.
3. **`proposal.md` / `design.md` / `tasks.md` become the task record.** The
   task body holds why and what, the plan section holds decisions and the
   alternatives considered, and checklist items are subtasks or a checklist in
   the body. The spec is the only artifact that outlives the task.
4. **Protection is per requirement.** `{#r-7fa2 protected}` means any task that
   modifies it goes through the human plan gate (§5.3). The governing invariants
   get this marker; most requirements don't.

### 8.3 The impact registry

Every planned task declares its impact:

```
bridle impact set tw-7fa2 \
  --modify s-b310 --add-under r-7fa2 --remove s-11c0 \
  --files 'client-watch/**' 'packages/ratings/**'
```

`bridle impact check` compares every in-flight task and reports:

| Overlap | Level | Action |
|---|---|---|
| Same scenario modified/removed by two tasks | **conflict** | open a `conflict` thread between the claimants |
| Same requirement, different scenarios | warn | notify both; usually fine |
| Same capability, different requirements | info | nothing |
| Same files | warn | early warning (research 09 tier 1) |
| Real textual conflict from `git merge-tree` | **conflict** | same as the first row (research 09 tier 2) |

**Declared impact is checked against actual impact.** When a branch changes,
bridle diffs the spec ids and file paths it touched and flags anything the task
didn't declare. An agent can't avoid a conflict by under-declaring.

### 8.4 The conflict protocol

1. Bridle opens a `conflict` thread between the two claimants (or the driver,
   for unclaimed tasks) and injects it into both sessions.
2. They decide between them, and one records the outcome:
   - `bridle conflict resolve C12 --compatible "…"`: not a real conflict; the
     reason is recorded.
   - `bridle conflict resolve C12 --order tw-7fa2,tw-a9d0`: adds a `blocks`
     edge. The blocked task's claimant writes a handoff note, releases or
     parks the claim, and takes other ready work.
   - `bridle conflict resolve C12 --merge-into tw-7fa2`: one task absorbs the
     other's scenario change.
3. If they disagree, or the conflict is a product question rather than an
   ordering question, it escalates to the driver, and to the human only if it's
   a product question.
4. **After a merge**, bridle sends `system: spec changed under you` to every
   in-flight task whose impact overlaps what just merged, so those agents rebase
   and re-read before building on stale text.

This replaces the same-spec pile-up rule. Serialisation happens only when two
tasks actually collide, and the agents involved decide the order.

### 8.5 Migration from OpenSpec

`bridle import openspec` handles it once per repo: move `openspec/specs/*/spec.md`
to `design/specs/<capability>.md`, assign ids, convert active changes to tasks (their
delta specs are applied on a task branch), and leave archived changes in git
history without converting them. The current `*Verification*` marker grammar is
kept, so data-contracts' scenarios carry over unchanged.

---

## 9. Specs to tests: making Gherkin disappear

The current pipeline commits a generated `.feature` beside every `spec.md`, so
it needs a staleness check (`check-specs.py`), strict input validation, output
re-parsing through `gherkin-official`, and `uv run`. Most of the brittleness
comes from the committed generated file.

- **Parse in Rust, in the binary.** One parser, used by `bridle spec check`,
  the impact registry, id assignment and export. Target: all six repos' specs
  in well under a second.
- **Nothing generated is committed.** Test runners get scenarios at collection
  time:
  - `bridle spec export --format gherkin --out .bridle/cache/features/`
    (gitignored) for runners that need files;
  - `bridle spec export --format json` for adapters that register scenarios
    directly.
- **Adapters live in stack packs.** `python` ships a small pytest plugin that
  gets scenarios from bridle and binds them to pytest-bdd steps. `typescript`
  ships a vitest equivalent, needed by track-web, harness, otters and file-db's
  TS side.
- **Scenario ids in test results.** Results report by `s-b310`, which lets
  `bridle test --task tw-7fa2` run only the scenarios in that task's impact,
  and `bridle spec coverage` list executable scenarios with no bound test
  (research 11's "147 scenarios with zero links to code").
- **Validation stays strict but gets cheaper.** Rejecting near-misses is the
  valuable part of today's tool, and that moves into the Rust parser. Re-parsing
  the output is no longer needed because the emitter and the validator share
  one AST.

---

## 10. Storage details

### 10.1 The state branch

Each project repo gets a `bridle` branch, checked out by bridle into
`~/.bridle/state/<project>/` (a normal git worktree, not visible in the working
checkout):

```
tasks/tw-7fa2.md          one file per task: TOML frontmatter + markdown body + thread
events/2026-09.jsonl      append-only transitions, for history and rebuild
questions/…               (or inline in the task thread — open, §15)
```

- **One file per task** merges cleanly, can be read on GitHub, and is the file
  design from research 13 carried over.
- **Bridle commits it**, batching writes (e.g. at most one commit every 30 s,
  plus one on every accept), and pushes on a configurable schedule.
- **Code branches never contain task state.** Task chatter can't cause a merge
  conflict with code, and main isn't committed to on every status change.

The alternative, task files in-tree under `.bridle/tasks/` on the main line, is
easier to browse next to code but brings back the worktree-visibility and
churn problems. It's listed in §15 as open.

### 10.2 The database

`~/.bridle/bridle.db`, SQLite in WAL mode, one writer per transaction. It
indexes every registered project's tasks and holds the ephemeral tables:
`claims`, `agents`, `messages`, `waits`, `ports`, `impact_cache`. Every
durable write goes to the database and the state branch in the same logical
operation. The database is the read path because it's fast, and git is the
recovery path.

### 10.3 Project registry

`~/.bridle/projects.toml` lists each registered repo with its path, prefix and
remote. `bridle status --all` and `bridle ready --all` work across all of them,
which gives one view of work over all projects.

---

## 11. Usage limits and token efficiency

### 11.1 The constraint

**The ceiling is one $100/month subscription (Max 5x), with no API overage.**
That's the opposite end of the scale from Yegge's ~$2,800/month across 13
accounts (research 06 §5). Every design choice in bridle has to be judged by its
token cost as well as its function.

In practice this means:

- A subscription has **rolling limits, not a bill**: a 5-hour window, a 7-day
  window, and per-model weekly windows (`seven_day_opus`, `seven_day_sonnet`).
  Hitting one stops all work on the account, **including the human's own
  sessions**.
- Parallelism is not free. Two workers use the window twice as fast. The number
  of concurrent workers is a budget setting, not an architectural one.
- Tokens spent on coordination (priming, injected rules, tool schemas, agents
  maintaining the tracker) buy no work. Wheelhouse's *"agents burn tokens
  invisibly keeping beads synchronized"* is the failure this section exists to
  prevent.

### 11.2 Where bridle can see usage

| Source | Gives | Available in |
|---|---|---|
| stream-json `rate_limit_event` | `status` (`allowed` / `allowed_warning` / `rejected`), `resetsAt`, `utilization` (0–1), and the window type (`five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`). Emitted **when the status changes** | headless workers |
| stream-json `result` | `usage` (input, output, cache-creation and cache-read tokens), per-model `modelUsage`, `total_cost_usd` (list-price equivalent, **cumulative across turns** in streaming mode) | headless workers |
| status line JSON | `rate_limits.five_hour` / `.seven_day`: `used_percentage`, `resets_at`, plus session cost and context use | the interactive driver: bridle ships `bridle statusline` as the status line command, which shows the numbers **and** records them |
| assistant error `rate_limit` | a turn failed on a limit | both |
| OpenTelemetry | tokens, cost, per request | later, optional |

The status line matters more than it looks. It's how bridle sees the account's
windows while no workers are running, which means it can **measure today's
workflow before bridle changes anything** (§11.5).

Caveats:

- `total_cost_usd` is a list-price estimate. For a subscription it's only a
  relative unit, useful for comparing things with each other.
- How many tokens a percentage point of a window corresponds to is not
  published and may change. Bridle **learns it** by relating the ledger to the
  window percentages over time, and treats that as an estimate.

### 11.3 The budget governor

Bridle checks the budget before every dispatch:

```toml
[budget]
plan                 = "max-5x"
max_workers          = 2            # concurrent headless workers
reserve.five_hour    = 25           # % of the window kept free for the human
pause_at.five_hour   = 80           # stop dispatching
pause_at.seven_day   = 85
pause_at.seven_day_opus = 70

[models]                            # defaults by role; the governor may step down
driver   = ["opus", "sonnet"]
planner  = ["opus", "sonnet"]
reviewer = ["sonnet", "opus"]       # opus for protected/arch-revision reviews only
worker   = ["sonnet", "haiku"]
chore    = ["haiku"]
explore  = ["sonnet"]
```

- **Dispatch** checks headroom first. If a window is past its `pause_at`, bridle
  doesn't spawn. The ready queue waits in priority order, and explorations and
  `someday` work go last.
- **Model choice** starts from the role's list and steps down as a window gets
  tight, e.g. Sonnet instead of Opus when `seven_day_opus` is high. A task can
  pin a model (`model = "opus"`) when its plan says the step-down would be a
  false economy.
- **Pausing when a limit is hit**:
  1. On `allowed_warning`, bridle stops dispatching and lets running turns
     finish.
  2. On `rejected`, it interrupts cleanly at the next turn boundary.
  3. It records each paused agent's session id and moves its task to
     `paused:limit`. The claim, worktree and branch are all kept.
  4. It sets a timer for `resetsAt`.
- **Resuming** when the window resets: bridle restarts paused agents with
  `--resume <session-id>`, in priority order, up to `max_workers`, and tells the
  driver.
- **The human's reserve** is respected even when work is queued. The human's
  interactive sessions share the account, and running out mid-conversation is
  the worst outcome.

### 11.4 Designing for fewer tokens

Rules the design follows, and that the build is reviewed against:

1. **Bookkeeping is done by the tool, not the model.** Conflict detection,
   merges, trace links, spec parsing, rendering, state transitions and git
   commits are Rust code. No agent ever hand-edits bridle's state or reconciles
   it.
2. **The system prompt is stable, and the task is in the first message.**
   Everything role-specific and slow-changing goes in
   `--append-system-prompt-file`, identical for every agent in that role and
   project. The task-specific content goes in the first user message. That
   keeps a long shared prefix cached across agents.
3. **Prime is role-scoped and has a size budget.** `bridle prime` has a token
   budget per role, e.g. a worker's prime ≤ 3k tokens. Guides are pointed to,
   not included, unless the task's components need them.
4. **Injections are silent by default.** Hooks print nothing when there's
   nothing new. Messages are summarised with a pointer, not pasted in full,
   when they're long.
5. **Tool schemas cost tokens on every turn.** The bridle MCP server exposes a
   small set of tools, and workers launch with `--strict-mcp-config` so they
   don't inherit every MCP server the human has configured.
6. **Output is terse.** Bridle's `--json` output is compact, and skills tell
   agents to use quiet or summary flags on test runners and linters.
7. **Cheapest adequate model per role** (§11.3), with Opus reserved for
   planning, architecture work and protected reviews.
8. **Fresh context versus resuming.** A fresh worker re-reads its context; a
   resumed one carries its history forward. Bridle measures both (§11.5)
   instead of assuming one is cheaper.

### 11.5 Tracking token use over time

Every `result` and status-line snapshot goes into a **usage ledger** in the
database. Each row records: time, project, task, task kind, role, agent,
model, the four token counts, the cost equivalent, turn count, and the
**workflow revision**, i.e. the bridle version plus the `bridle-workflow` git
revision in effect.

```
bridle usage                      # today / this window / this week vs limits
bridle usage --by role|project|kind|model --since 30d
bridle usage task tw-7fa2         # what one task cost, per agent and phase
bridle usage trend --per kind     # tokens per task kind over time
bridle usage compare --workflow <rev-a> <rev-b>   # did a workflow change cost more?
bridle cost audit [--check]       # static: size of everything bridle injects
```

The last two answer *"are new systems increasing the token budget?"* directly:

- **`bridle cost audit`** counts the tokens bridle adds to each role's context
  with no work done: prime, the rendered system-prompt file, skill
  descriptions, MCP tool schemas and hook boilerplate. It compares the result
  with a committed baseline (`.bridle/cost-baseline.json`, a record, not
  generated output). `--check` fails when a change to rules, skills or tools
  grows any role's fixed overhead by more than a set percentage. The growth
  then shows up in review, as a number, before it costs anything.
- **`bridle usage compare --workflow`** compares tokens per task kind before
  and after a workflow revision. The comparison is rough, because tasks
  differ, but a clear increase in "tokens per chore" after a rules change is
  exactly the signal needed.
- **Cache hit ratio** (cache-read ÷ total input) is reported per role. A drop
  means something broke the stable prefix in rule 2 above.

**Measure the baseline first.** `bridle statusline` and the ledger are
deliberately early in the build order (§14), so the current OpenSpec workflow's
usage gets recorded before bridle replaces it. Without that baseline there's
nothing to compare against.

---

## 12. The CLI surface (first cut)

```
bridle init | sync | prime | doctor              project setup, render, session start, health
bridle project add|list|remove
bridle task new|show|edit|list|drop|reopen
bridle dep add|rm            bridle ready [--all] [--role]
bridle claim|release|handoff bridle plan <id>     bridle accept <id> (human only)
bridle send|ask|answer|inbox [--human] [--inject]
bridle wait <id> [--until <state>] [--or-message] [--timeout]
bridle spawn <role> <task>   bridle agents        bridle review
bridle impact set|show|check bridle conflict list|resolve
bridle spec check|id|export|coverage|import
bridle rules show|explain|diff|propose
bridle goals list|propose       bridle arch propose
bridle trace up|down|suspect|confirm|orphans|coverage
bridle explore new|conclude|adopt|abandon
bridle rebuild
```

Every command takes `--json`. Agents always use it, and humans get tables.

---

## 13. Skills: rewrite as a small set

The ~10 OpenSpec skills per repo are replaced by a handful in the base layer,
with project addenda:

| Skill | For | Replaces |
|---|---|---|
| `bridle-driver` | orient, decompose, plan, spawn, wait, arbitrate | driver-guide, propose/new/continue/ff-change |
| `bridle-worker` | claim, read plan, implement, report on the task as it goes, handoff | apply-agent-guide, apply-change |
| `bridle-plan` | writing the task plan, spec edits with ids, impact declaration | propose-change, design.md conventions |
| `bridle-review` | reviewing a diff against plan + specs + impact | verify-change, acceptance-verifier |
| `bridle-triage` | intake → planned, dropping, dedup | ticket-conventions, maintenance |
| `bridle-conflict` | the §8.4 protocol from a worker's side | the same-spec pile-up rule |

Each skill is short because the procedure lives in bridle's commands. The
skill says when to run which command and what judgement applies. Rules and
guides are delivered by `bridle prime` and aren't repeated in skills.

---

## 14. Build order

Each phase is usable on its own. Bridle's own development is the first test
project.

| Phase | Delivers | Proves |
|---|---|---|
| **P0a** | `bridle statusline` + usage ledger + `bridle usage` (read-only, no other bridle features) | a usage baseline for the **current** workflow, before anything changes (§11.5) |
| **P0** | workspace, `project add`, tasks, edges, `ready`, claims with leases, state branch persistence, `rebuild` | the store and the git story |
| **P1** | `send/ask/answer/inbox`, `wait`, hooks (`prime`, `inject`, heartbeat, stop-check) | the test-config case: a driver waits for a worker, which asks a question, gets an answer and finishes |
| **P2** | layers: `bridle-workflow` repo, packs, project overrides, `rules explain/diff`, `sync` rendering, the six skills | one workflow across two real projects (suggest otters + data-contracts: most different, least OpenSpec risk) |
| **P3** | parser for goals, architecture and specs; ids; `spec check/export`; `import openspec`; the explore task kind and its prime rules | data-contracts on bridle specs, with the Python adapter replacing `spec-to-feature.py` |
| **P4** | impact registry, conflict protocol, post-merge rebase notices; trace links with suspect tracking; `arch-revision` → `re-evaluate` flow | two workers on one capability at once; an architecture change traced to the specs it affects |
| **P5** | `spawn` with worktree layouts (incl. paired), port registry, merge-tree probes, integration branch | harness + track-web in parallel |
| **P6** | TS test adapter; migrate track-web, harness, meta-notes, file-db | everything on bridle |

Don't start at P5. Research 09 §7 still applies.

---

## 15. Open questions

1. **Task records on a state branch or in-tree?** (§10.1) The recommendation is
   a state branch. In-tree is easier to browse and review with code.
2. **Workers as `Agent` subagents or separate `claude` sessions?** *Mostly
   answered* by [`research/01`](research/01-agent-runtime.md): separate headless
   `claude -p` sessions that bridle spawns, with 7 spikes (§8 there) to run
   before committing.
3. **Integration branch per project, or merge straight to main?** With
   after-merge acceptance, rejected work on main needs a revert. An integration
   branch that the human fast-forwards to main on acceptance is safer and adds
   a step.
4. **Where does `bridle-workflow` live?** It could be its own repo or a
   `workflow/` directory inside this repo. The recommendation is its own repo:
   it changes on a different cadence from the binary, and projects pin it.
5. **A human surface beyond the CLI?** `bridle review` and `inbox --human` in
   the terminal first. A TUI or local web board (like SwarmForge's cockpit)
   could come later, with push notifications for questions if the harness
   supports them.
6. **What happens to meta-notes' own planning CLI?** It's a planning tool too.
   It may be a client of bridle, a source of intake, or unrelated.
7. **Command name.** `bridle` in full. `br` belongs to beads_rust, which otters
   still uses. The short alias is still undecided.
8. **Cross-project specs.** harness consumes engine behaviour that is specified
   in track-web. The question is whether a harness scenario can reference a
   track-web requirement id (`tw:r-7fa2`), and whether impact checks follow that
   reference.
9. **Where the knowledge root lives, and migrating to it.** `design/` is only a
   default. The existing material has to be sorted into the tiers once per
   repo: track-web's `docs/`, harness's plan of record and `docs/arch/`,
   data-contracts' orientation docs. This is judgement work, not a script. It
   is probably a driver task per repo, reviewed by the human.
10. **The goal vocabulary.** Are firmness × priority × stance (§7.2) the right
    three axes, and are four values each too many? This should be tested
    against a real goals list, e.g. track-web's, before the parser fixes it.
11. **Traceability in older specs.** A newly imported spec has no `traces=`
    links. Either orphans are allowed to be the norm and links added as
    requirements are touched, or the import requires a linking pass. The first
    is cheaper; the second makes §7.4 useful sooner.
12. **Is the driver Opus by default?** Max 5x has a separate weekly Opus
    window. A long-lived Opus driver may be the largest single consumer. An
    alternative is a Sonnet driver that escalates to Opus for planning and
    architecture turns only. The usage baseline (§11.5) should decide this.
13. **The human's reserve.** 25% of the 5-hour window is a guess. The right
    number depends on how much the human works interactively while workers run.
