---
id: y3sd
title: Components as scoped work contexts (track-web clients and games)
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [u8sm, 63rv, rxe8, hv8e]
---

## The ask

The human, verbatim (2026-09-28):

> before onboarding track-web, this project is unique and will probably require some specific
> handling and extension: it features a single backend that is shared among many clients. Rules
> for backend db and API apply to every projects; however, eaach project (client-*) has it's own
> set of local conventions, roadmap, planning, and tickets that only apply to that particular
> client. Work should be coordinated and managed together in a single bridle, but needs to be
> clear which project is being worked on so any project-local rules can be applied.
>
> E.g. client-games is a Phaser game projects with sub-projects for different games;
> client-trips/watch are family organization and planning; client-play is for group board games.
> Each has its own peculiarities; also since client-games has sub-games; that may not need to be
> a primary thing modeled in bridle, but it is something that hsould be reflected in some way with
> a different design for each game. dungeon tactics is turn-based game my son is driving the
> requirements; but space golf is a casual game I'm building to attempt monetization in the
> future; ball merge was something my daughter asked me for.
>
> I'm not sure how to resolve all of these, but when we are scoping or doing work, these project
> and sub-project local rules need to be available and used.
>
> In particular is restricting and guiding context for an agent working with me on planning
> tickets and writing proposals. the agent should only need to know about the context for: the
> server, their client-*, and their sub-project.

Follows the track-web survey, [[onboarding-survey-track-web-and-harness-u8sm|u8sm]]. Blocks
track-web onboarding.

## What exists

[[docs/design/workflow-layers|Workflow layers]] @ 27b4ad4 already has the shape:

> ```
> L3  project     <repo>/.bridle/                 this project's overrides + additions
> L4  component   <repo>/.bridle/components/<n>/  path-scoped, e.g. client-watch/**
> ```

and, from the same doc:

> - Path-scoped (L4 component) rule rendering is still out of scope, per the table above.

Mapped onto track-web:

```
track-web (L3)       server, db, api, packages/*       → every agent
  client-games (L4)  Phaser conventions, build limits
    dungeon-tactics  son drives requirements; engine shared with harness
    space-golf       casual; monetization later
    ball-merge       daughter's request
  client-trips, client-watch, client-play …
```

## Gaps (advisor's reading)

1. **Scope by the task, not the cwd.** [[docs/spikes/06-path-scoped-rules-findings|Spike 06]]
   shows nested `CLAUDE.md` loads by cwd. That fits a worker in one client, not a planning
   or proposal agent working with the human. The task/spawn names its component(s), and
   `bridle prime` hands the agent L3 + the client + the game.
2. **Nested components.** A game is a component whose paths sit under its client's, and
   inherits the client's rules.
3. **Components carry context, not only rules:** roadmap, planning, tickets, and who the
   thing is for and why. Task lists filter by component.
4. Spike 06's suggested render target (`.bridle/components/<n>/CLAUDE.md`) would not be
   loaded by cwd, since it isn't under the component's path. Claude Code's
   `.claude/rules/*.md` with `paths:` frontmatter was not tested by the spike; unverified.
5. Anything rendered into track-web itself falls under
   [[trial-adoption-on-a-bridle-branch-63rv|63rv]] (the `bridle-adopt` branch, the human's review).

## The human's answers

2026-09-28, to the advisor's four questions:

1. Games as components nested in client-games:

   > makes sense, good, games as components

2. Hard or soft restriction of an agent's context:

   > restrictions are soft, yes, not hard; e.g. some clients share approaches and web
   > components; games learn from other implementations

3. Where per-client and per-game roadmaps, planning and tickets live:

   > I'm not sure where roadmaps live; in track-web they live in docs/ but are incomplete
   > likely; long term not sure if they live in or out of bridle; but I want to be able to
   > review them myself fairly easily as markdown files

4. Every track-web task names its component:

   > right, makes sense; soft rule but yes; some tasks could span components as well, so
   > that should be a list and/or allowed to be repo-wide in some cases.

2026-09-28, a follow-up on gap 3:

> IDK if components match 1:1 with roadmap, planning, tickets or not; I don't want to add too
> much admin overhead for that.

2026-09-28, on roadmaps:

> generally, i do like having a roadmap of where I'm planning to go with something that helps me
> pick up a project that I have set aside for a time period. Mostly, that is per-repo, but
> track-web is unique.

2026-09-28, on where component docs live:

> I like having planning docs inside of docs, yes; generally I have a parallel docs structure to
> the client/games. look at track-web

What the advisor found in track-web (`dev` branch, read-only, 2026-09-28):

| Component | Code | Docs | Roadmap today |
|---|---|---|---|
| client-games | `client-games/` | `docs/games/` | `docs/games/planning.md` (54 lines, mixes all games) |
| space-golf | `client-games/src/games/space-golf/` | `docs/games/space-golf/` | none (README, ideas, open-questions, tech) |
| dungeon-tactics | `client-games/src/games/dungeon-tactics-solo/` | `docs/games/dungeon-tactics/` | none |
| orbital-dodger | `client-games/src/games/orbital-dodger/` | `docs/games/orbital-dodger/` | none |
| ball-merge | `client-games/src/games/ball-merge/` | none | items in `docs/games/planning.md` |
| mimlings | none yet | `docs/games/mimlings/` | none |
| client-watch, -play, -time | `client-<x>/` | `docs/<x>/` | `docs/<x>/planning.md` |
| client-trips, -talks | `client-<x>/` | `docs/<x>/` | none |
| server/shared | `src/`, `packages/` | `docs/arch/`, `docs/app/` | `docs/app/planning.md` |

So the parallel structure is a convention, not exact: `docs/<client without the client- prefix>/`,
and `docs/games/<game>/` where the game's code folder name can differ
(`dungeon-tactics-solo` ↔ `dungeon-tactics`). Some docs folders map to no client
(`food`, `life-mgmt`, `pixellab`, `other`), and a game can have docs before code (mimlings).

## Still open

- Where roadmaps live (in track-web's `docs/`, or in bridle), given they must stay easy to
  review as markdown. Related: [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|hv8e]].
- Whether a component has its own roadmap, planning and tickets (not necessarily 1:1), kept
  low-overhead per the follow-up above.

## Design

[[docs/design/components|Components: scoped work contexts]] answers gaps 1–5 and the two
open items above, and names the follow-up implementation task. Not resolved (moved) yet:
that waits on the implementation.
