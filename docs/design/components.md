# Components: scoped work contexts

Design for the L4 layer of [[docs/design/workflow-layers|workflow layers]], from
[[components-as-scoped-work-contexts-y3sd|y3sd]] (the human's ask and settled
answers are quoted there; not re-argued here). Designed, not built. It supersedes
the render target suggested in [[docs/spikes/06-path-scoped-rules-findings|spike 06]].

The case is track-web: one server shared by many clients (`client-games`,
`client-trips`, `client-watch`, `client-play`, …), each with its own conventions,
roadmap and docs, and `client-games` has games (`dungeon-tactics`, `space-golf`,
`ball-merge`) nested under it. One bridle coordinates all of it. An agent planning
with the human should get the server's context, its client's, and its game's, and
not the rest.

## The model

A **component** is a named scope inside one project. It has an id, optional code
paths, an optional parent, and optional docs. Everything but the id is optional, so
adding a component costs one table in `.bridle/config.toml`:

```toml
[components.client-games]
paths = ["client-games/**"]
docs  = "docs/games"

[components.dungeon-tactics]
parent = "client-games"
paths  = ["client-games/src/games/dungeon-tactics-solo/**"]
docs   = "docs/games/dungeon-tactics"

[components.client-watch]
paths = ["client-watch/**"]
docs  = "docs/watch"
```

- **Nesting is declared with `parent`, not inferred from paths.** track-web's
  parallel docs tree is a convention with exceptions (`dungeon-tactics-solo` code ↔
  `dungeon-tactics` docs; `mimlings` has docs and no code yet; `ball-merge` has code
  and no docs). Explicit `parent` and `docs` absorb all of that. Ids are unique in
  the repo. A parent cycle or unknown parent is a config error.
- **A component's *chain*** is the root-most ancestor down to itself:
  `client-games → dungeon-tactics`. The project (L3) is above every chain; a
  component with no parent hangs directly off it.
- `paths` say where a component's code is. They're metadata for tooling (finding
  which components a diff touches, spotting a task whose diff strays), not something
  Claude Code reads. Repo-wide code (server, `packages/*`) is just L3: it needs no
  component.
- Component **content** sits in `<repo>/.bridle/components/<id>/`, the same shape
  as any layer (`rules/`, `guides/`, `facts.md`; see workflow-layers). A component
  with no directory is valid: it scopes docs and tasks only.

## Scope by the task or spawn, not the cwd (gap 1)

Bridle workers run in a worktree at the repo root and planning agents run wherever
the human is, so cwd says nothing about which client is meant. The scope is named:

- **A task carries `components`**, an optional list of component ids. Empty means
  repo-wide (server work, cross-cutting work). A task may name several
  (`components: [client-games, client-play]` for a shared web component). Naming a
  child implies its ancestors; there's no need to list both.
- **Naming is a soft rule** (the human's answer 4). `bridle task new` takes
  `--component <id>` repeatedly, rejects an unknown id, and in a project that has
  components prints a one-line reminder when none is given. It never refuses.
  Nothing else in bridle blocks on it.
- **A spawn takes the same list** (`bridle spawn … --component <id>`), defaulting to
  the claimed task's. A planning or proposal agent has no task, so this is how it
  gets scoped. The daemon records the list on the agent and passes it to the
  process as `BRIDLE_COMPONENTS` (comma-separated), which `bridle prime` reads.
  This is a wire change: `Task`, `NewTaskRequest` and the spawn request in
  `bridle-api/src/types.rs`, and the state-branch task format
  ([[docs/design/storage|storage]]).

## What an agent is handed (gaps 1 and 4)

`bridle prime <role>` (built today for `orchestrator` only) is the delivery
mechanism, as workflow-layers already says for most rule content. Given the named
components, prime prints, sized to the role:

1. L1–L3 as it does now (resolved rules tagged for the role, facts, guides).
2. For each named component, its chain's rules, facts and guide pointers, resolved
   on top of L3.
3. For each named component (not its ancestors, whose docs are the parent's
   business): its **docs pointers**: `docs` folder, plus which of `README.md`
   ("who this is for and why") and `roadmap.md` exist there. Paths only, except
   the README, which is printed when short. The agent reads more if it needs it.
4. A one-line list of the components *not* named, with their docs folders. This is
   what makes the restriction soft: a game can look at how a sibling did it (human
   answer 2), it just isn't handed the sibling's rules.

**Claude Code's own loading isn't the mechanism.** Spike 06 showed nested
`CLAUDE.md` loads by cwd, and its suggested target,
`.bridle/components/<n>/CLAUDE.md`, isn't under any component's code path, so
nothing would ever load it. Prime doesn't need it, so nothing gets rendered for
components and `bridle sync` stays as it is. The one case prime doesn't cover is a
human running plain `claude` inside `client-watch/` with no bridle session. That's
the only place a cwd-based render helps, and the candidate is
`.claude/rules/<id>.md` with `paths:` frontmatter, which spike 06 didn't test.
**Deferred**, with a reason: no bridle-run agent needs it, and building on an
unverified mechanism is the mistake this design is avoiding. If the human wants it, a
small spike verifies the mechanism first (below).

## Nested components resolve correctly, with one rule (gap 2)

`rules::resolve` (`crates/bridle-daemon/src/rules.rs`) takes an ordered list of
layers and folds them in order; nothing in it is specific to base/pack/project. A
two-deep chain is therefore just two more layers after L3:
`base, packs…, project, client-games, dungeon-tactics`. A game rule replaces or
appends to a client rule by id with an explicit `override`, disables need a
`reason`, and a locked client rule can't be overridden by a game, exactly as for
the existing layers. (Read from the code, not yet run; the follow-up's first test
proves it.)

What it needs:

- `LayerKind::Component` and a `Layer` for `.bridle/components/<id>/rules`, and a
  helper that builds the chain for one component from config.
- **Resolve each named component's chain separately, never all named components
  in one list.** Two siblings (`client-games`, `client-play`) that each define a rule
  id would otherwise hit `MissingOverride`, since the second looks like a silent
  redefinition of the first. Siblings are independent; prime prints each chain's
  result under its own heading. If two chains disagree on the same id, both are
  shown labeled. Prime doesn't pick a winner between siblings.
- `bridle rules explain <id> --component <id>` (and `diff`) take the same flag; with
  none they show L1–L3 as today.

## Roadmap, planning and tickets (gap 3, hv8e's slice)

The human doesn't want admin overhead, doesn't know whether components map 1:1 to
roadmaps and tickets, and wants to review roadmaps as markdown. So **no new storage
format and no per-component ticket tree.**

- **Roadmap and planning are markdown in the repo's `docs/`, in the component's
  `docs` folder.** `roadmap.md` there is the human's "where I'm going, to pick this up
  after time away"; `planning.md` is the existing working doc; `README.md` is who it's
  for and why. All optional. A component gets a `roadmap.md` only when the human
  wants one; most stay repo-level (as they are now, mostly). track-web is the
  exception, since it has several products under one repo: a per-component roadmap
  is possible there and costs one file.
- **Tickets are bridle tasks, filtered by component.** `bridle task list
  --component <id>` matches tasks naming that component or any descendant. The
  component field is the whole linkage: a component doesn't own a ticket list, and
  tasks needn't match roadmap items 1:1 (a task's body can link the roadmap line it
  serves).
- **What this answers of [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|hv8e]]:**
  roadmaps and planning docs live in the project's `docs/`, next to the component's
  other docs; the task queue lives in bridle. hv8e's broader question (which
  *other* docs live in bridle's records, and link checking) stays open.
- **Deferred:** restructuring track-web's `docs/games/planning.md` (54 lines, mixes
  all games) into per-game roadmaps. That edits the human's project, so it belongs to
  the adoption trial ([[trial-adoption-on-a-bridle-branch-63rv|63rv]]) and is the
  human's call. Also deferred: a lint that a component's `docs` folder exists.

## Track-web and the trial (gap 5)

Nothing here touches track-web. Adopting it means writing its
`[components.*]` tables and `.bridle/components/<id>/` content into the
`bridle-adopt` branch for the human to review (63rv), after the branch-rules work
([[branch-rules-per-project-rxe8|rxe8]]) lands. The design only needs to be settled
first, because the onboarding's config shape depends on it.

## Build now, defer

**Follow-up implementation task** (one task; L4 layer plus prime, no renderer):

> Components as scoped work contexts. Implement docs/design/components.md:
> (1) `[components.<id>]` in config (`paths`, `parent`, `docs`, `consumers`), with
> validation of unknown parents and cycles; (2) `LayerKind::Component`, a chain
> builder, and tests that a two-deep chain resolves (replace, append, disable, locked)
> and that sibling chains are resolved separately; (3) an optional `components` list
> on tasks and spawns (wire types, store, state branch, `bridle task new/list
> --component`, `bridle spawn --component`, `BRIDLE_COMPONENTS`); (4) `bridle prime`
> for worker and planner roles printing L1–L3, the named chains, docs pointers and the
> other-components line; (5) `rules explain/diff --component`. Update
> workflow-layers, cli.md and rules.rs's header comment. Out of scope: rendering
> component rules into files, anything in track-web.

**Deferred:**

| Item | Why |
|---|---|
| Rendering component rules for plain-`claude` sessions (`.claude/rules/` `paths:`) | No bridle-run agent needs it; the mechanism is unverified. Verify first with a small spike, only if the human runs bare `claude` in client dirs and misses the rules. |
| Warning when a task's diff touches paths outside its named components | Soft rule; wants real use before deciding it's worth the noise. |
| Per-component ticket lists, roadmap schema or lint | Admin overhead the human doesn't want. |
| Splitting `docs/games/planning.md` | The human's project; goes through 63rv. |
| hv8e beyond roadmap location | Broader question, still open. |
