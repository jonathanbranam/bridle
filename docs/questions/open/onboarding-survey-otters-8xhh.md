---
id: 8xhh
title: Onboarding survey: otters (otter-life and otters-back)
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [d9nu, ajqa, u8sm, a8fk]
---

## The ask

The human, verbatim (2026-09-28):

> similar to what orchestrator did already: evaluate @~/work/file-db/ and @~/work/meta-notes/
> for bridle adoption.
>
> A much more ambitious project would be @~/work/pi both harness and track-web; There is some
> cross-repo coordination in those projects; track-web could onboard first as it also has
> separate deliverables independent of harness.
>
> the otter project was started in @~/work/otters/otter-life/ and then partially migrated to
> otters-back with a newer, better architecture, but not a complete migration
>
> I have some tooling that is shared between gaming projects, e.g. using pixel lab generation
> in traack-web for dungeon tactics and mimlings proposal and also for the otter project.

Filed by the advisor: a read-only survey by a subagent, modelled on
[[docs/context/onboarding-data-contracts|the data-contracts survey]]. The survey below is
its report, unedited; nothing in the surveyed repos was changed.

## Notes

- Related surveys from the same request: [[onboarding-survey-file-db-d9nu|d9nu]], [[onboarding-survey-meta-notes-ajqa|ajqa]], [[onboarding-survey-track-web-and-harness-u8sm|u8sm]], [[shared-pixellab-tooling-for-game-projects-a8fk|a8fk]].
- The open questions at the end are for the human; none is answered yet.

## The survey

### Summary

Read-only survey, 2026-09-28. Nothing under `/Volumes/Data/work/otters/` was changed and no `br` command was run. `OL` = `/Volumes/Data/work/otters/otter-life`, `OB` = `/Volumes/Data/work/otters/otters-back`.

- **Neither repo is live.**
  - OL's last commit is `008191a`, 2026-02-19 (64 commits, Jan 17 to Feb 19).
  - OB's last commit is `2402790`, 2026-03-08. It has 3 commits in total, all made over two days.
  - Both are clean, on `main` and level with `origin/main`. Neither has local branches, extra worktrees or stashes.
- **The "migration" is a scaffold, not a port.**
  - OB is a Yarn 4 monorepo with 5 packages (`simulation`, `protocol`, `server`, `client`, `cli`) and a better architecture: a shared command vocabulary, one dispatcher behind CLI/REST/WS, and client adapters for single-player and hosted play.
  - All of it came from one Claude-generated commit ("Base project structure from claude.", 57 files).
  - Its simulation was **rewritten generically, not moved.** OL's actual game (11 tile types, a river that winds from southwest to north, a side-scrolling river scene, overworld↔river mapping, resources, occupancy, cheat commands, fixtures) is **not in OB**.
  - OB has **zero tests**, and its CLI doesn't persist state.
- **OL is the more complete game; OB has the better skeleton.** Roughly 3,400 lines of TS in OL (about 680 of them tests), against about 1,370 in OB (no tests).
- **Beads is nearly empty.** `OL/.beads/issues.jsonl` holds 7 records: 3 closed (refactors and a CLI tweak) and 4 tombstones (a "verify br works" test epic). **Nothing is open.** There's nothing to import.
- **Claude Code memory.** OB keeps `memory/MEMORY.md` and `memory/architecture.md`, git-tracked, and its CLAUDE.md tells agents to read them. That's a repo-local "memory" convention, not Claude Code auto-memory, but it conflicts with the spirit of `workflow/base/rules/memory.none.md`, and the name has to go. The content is good architecture documentation.
- **Recommendation:**
  - Onboard **OB** (otters-back) as the one otters project.
  - Finish the port from OL into OB as bridle's first otters workload: well-bounded, testable by CLI, parallel across packages.
  - Freeze OL once the port's done, then archive it.
  - Don't import Beads. Drop `br`.

### 1. What each repo has today

#### otter-life (OL)

**Stack.** Phaser 3.90 + TS 5.4 + Webpack 5, built from the Phaser `template-webpack` (the `package.json` name is still `template-webpack`). It uses npm, and vitest 4 for tests. The CLI runs through `tsx` and `commander`.

**Agent setup.**
- `OL/CLAUDE.md` (5.2 KB) covers:
  - Task tracking: "read `br-guide.md`".
  - The dev commands.
  - Testing: vitest, `createTestSim()` fixtures, named position constants.
  - CLI testing with no browser (`npm run cli -- -d ./session new --fixture`, `n/s/e/w`, `dive`, `surface`, and `--cheat move-to/dive-to/surface-to`). State persists as `state.json`.
  - Architecture: simulation/rendering separation, scene sleep/wake, river-path mapping.
  - Five "Important patterns".
  - "Ignore `public.archive/`".
- `OL/br-guide.md` (4.5 KB): the `br` agent workflow (claim, close with `-r`, `sync --flush-only`, and a "needs-details" label plus defer when a task is unclear).
- `OL/.claude/settings.json`: a SessionStart hook, `.claude/hooks/session-start.sh`. It runs only when `CLAUDE_CODE_REMOTE=true`, then runs `npm install` and **curl-pipes the beads_rust installer into bash**.
- `OL/.claude/settings.local.json` is **git-tracked**. It allows git add/status/diff/commit, `br:*`, `npm run cli`, `npm test` and `npm run build`.
- There's no memory directory and no agents or skills.
- Remote branches `origin/claude/decouple-simulation-phaser-1KaI6` and `origin/claude/update-docs-MPgIG` are fully merged (0 commits ahead of main). Six PRs came in through Claude web sessions.

**Code.**
- `src/game/simulation/`: `GameSimulation` (298 lines), `Serialization` (190), `PlayerState`.
- `src/game/world/`: `TileType` (11 types with property table), `Tile` (resources, items, occupancy), `World`, `WorldGenerator` (199), `River`, `RiverGenerator`.
- `src/game/scenes/`: Boot, Preloader, MainMenu, WorldScene (381), RiverScene (203), GameOver.
- `src/game/rendering/TileRenderer.ts`, `src/game/entities/Player.ts` (the otter drawn with Phaser Graphics).
- `src/cli/main.ts` (262) and `renderer.ts` (156).
- `src/game/testing/fixtures.ts`.

**Tests.** `GameSimulation.test.ts` (381 lines), `Serialization.test.ts` (187) and `Tile.test.ts` (109). Not run.

**Other files.**
- `game-spec/world.md` and `world-requirements.md`: short prose specs.
- `public.archive/`: the original single-HTML version (dialogue, virtual controls, otter textures).
- `dist/` is committed (a production bundle).
- `screenshot.png`.
- `test-session/state.json` (probably a stray CLI session; tracking not checked).

#### otters-back (OB)

**Stack.** Yarn 4.13 workspaces (`nodeLinker: node-modules`, which Phaser needs) and TS 5.9 strict with `exactOptionalPropertyTypes` and `noUncheckedIndexedAccess`.
- `server`: Koa 2, `@koa/router` 14, ws 8.
- `client`: Phaser 3.90 and Vite.
- `cli`: commander.
- There's no test runner at all.

**Agent setup.**
- `OB/CLAUDE.md` (1 KB) has two parts:
  - A "Project Memory" section telling agents to read `memory/MEMORY.md` and `memory/architecture.md` at the start of every session.
  - Key rules:
    - No `Math.random()` in simulation code: use the seeded `makeRng()`.
    - No Node or browser APIs in `packages/simulation`.
    - How to satisfy `exactOptionalPropertyTypes`.
    - Keep `nodeLinker: node-modules`.
    - Build order: simulation → protocol → server/cli/client.
- `OB/memory/` (git-tracked):
  - `MEMORY.md`: the package list, config facts, dependency gotchas (`@types/koa__router@^12`), build order, and verification commands.
  - `architecture.md`: the `GameCommand` union, the `IGameAdapter` contract, server routes, and file locations.
  - These are hand-committed docs, not Claude Code auto-memory (`~/.claude/projects/...`), but the name and the "read me each session" instruction mirror it.
- `OB/.claude/settings.local.json` is **git-tracked**. It allows `yarn install`, `yarn info`, `npm:*` and `yarn workspace:*`.
- There's no `settings.json`, so memory isn't explicitly turned off.
- `README.md` states the goals: single-player and hosted modes, and REST/WS protocols that are "nearly identical to the CLI protocol" so the whole game can be simulated and tested from the CLI.

**The new architecture** (per `OB/memory/architecture.md`, checked against the source):
- `@otters/simulation`: pure TS with zero I/O. It has `GameSimulation` (multi-player map, `advanceTick` with hunger/energy decay), `World`, a mulberry32 `makeRng`, and `snapshotOf()`.
- `@otters/protocol`: `GameCommand` = `new | move | dive | surface | look | status`, plus `GameEvent` and HTTP types.
- `@otters/server`:
  - `CommandDispatcher.executeCommand(sim, playerId, cmd) → GameEvent[]`, shared by REST (`POST /api/sessions`, `GET …/state`, `POST …/commands`) and WS (`/ws/sessions/:id`, one connection per player).
  - An in-memory `SessionStore`.
- `@otters/client`: Phaser scenes (MainMenu and WorldScene only) that talk to an `IGameAdapter` from `AdapterRegistry`, implemented by `SinglePlayerAdapter` (local sim) and `HostedAdapter` (REST + WS).
- `@otters/cli`: commander plus `AsciiRenderer`, with the same command set.

**Weak spots in OB** (from reading the code, not from running it):
- `client/src/engine/localDispatcher.ts` duplicates the server's `CommandDispatcher` (a 42-line diff) instead of sharing a package.
- The CLI has no state persistence (no fs or session code in `packages/cli/src`) and there's no deserialization, so a multi-step CLI test isn't possible yet.
- There are no tests.

#### Other folders in `/Volumes/Data/work/otters/`

- **`koa-app/`**: a 2019 Koa boilerplate ("koa-template") renamed `otters-back.koa`. It has 1 commit (2026-03-06), **no remote**, and uncommitted changes (`package.json`, `server.js`, a new `yarn.lock`). It looks like a discarded starting point for OB's server. Not part of the project.
- **`ai-in-life/`**: a separate repo (`github.com/jonathanbranam/ai-in-life`), 13 commits, last 2026-04-27. It's an HTML slide deck for a 5-minute company talk on personal AI use. It mentions the otter game's origin (built on Claude Mobile) and the "agent can't see the Phaser canvas" problem, which is the reason for the CLI. Not a code dependency.
- **`pixellab-notes.md`** (2026-09-27, the newest file here): prompt notes for a PixelLab sprite animation ("Running, Attempt 3": a 9-step otter lope, and the result "more of a hop than a run").

#### PixelLab

- Neither repo references PixelLab: no MCP config, script, asset pipeline or sprite sheet.
- A grep for "sprite" hits only CLAUDE.md, the README and `public.archive/`. OL's otter is drawn with Phaser Graphics (`src/game/entities/Player.ts`), and OB's client has no sprite code.
- `~/.claude.json` has no pixellab entry.
- So PixelLab is manual, human-driven prompting for now. Bringing sprites in is future work, not migration work.

### 2. Classification

Key: **Base** = bridle L1, **TS** = TypeScript pack (L2, planned), **Proj** = otters `.bridle/` (L3), **Super** = superseded by bridle, **Drop** = remove.

| Item (source) | Class | Note |
|---|---|---|
| `OB/memory/*.md` plus "read at session start" (`OB/CLAUDE.md`) | Super + Proj | The content moves to `OB/docs/architecture.md` or `.bridle/facts.md`. Remove the `memory/` name and the instruction; `memory.none` is locked. |
| No memory setting in OB/OL `.claude/settings.json` | Base | Bridle's `--settings` covers spawned agents. The project still needs `autoMemoryEnabled: false` for human-run sessions. |
| Simulation is pure: no Node or browser APIs, no `Math.random()`, seeded RNG (`OB/CLAUDE.md`) | Proj | `must` rules. They're the core invariant of the game. |
| All game logic lives in the simulation; scenes call `sim`/adapter and only render (`OL/CLAUDE.md` pattern 2, OB adapter rule) | Proj | A `must`. The same rule in both repos. |
| Build order simulation → protocol → rest; `nodeLinker: node-modules` | Proj | |
| `exactOptionalPropertyTypes` conditional-spread idiom | TS | A good pack rule for any strict TS project. |
| Yarn 4 workspace commands and `tsc` build/typecheck | TS | Pack commands. The check command binding is Proj (OB has no single `check` script yet). |
| Tests with vitest, deterministic fixtures, named position constants instead of magic numbers (`OL/CLAUDE.md` Testing) | TS + Proj | The vitest runner goes in TS. "Fixture world plus named constants" is Proj. |
| Verify with the CLI against the fixture world, not a browser | Proj | Already the example `verify.browser` override in bridle `docs/design/workflow-layers.md:79`. It cites `npm run cli`, which is OL's command; OB has no equivalent yet. |
| Cheat commands (`--cheat move-to/dive-to/surface-to`) for test setup | Proj | Port them to OB's CLI. |
| Scene sleep/wake, viewport culling, occupancy bookkeeping (`OL/CLAUDE.md` patterns 1, 3, 4) | Proj | Phaser-specific. Keep as rules scoped by path to `packages/client`. |
| "Ignore `public.archive/`" | Proj | Or delete it after the port. |
| `br` + `br-guide.md` + `.beads/` | Super | Replaced by bridle tasks. "Needs-details label + defer when unclear" corresponds to bridle's blocked or question flow. |
| Close with a one-sentence reason | Base | Probably already covered by bridle's task close or report. Not checked. |
| SessionStart hook that installs `br` via `curl \| bash` (`OL/.claude/hooks/session-start.sh`) | Drop | Web-session only. It becomes unnecessary once `br` is gone. |
| Tracked `settings.local.json` (both repos) | Proj | `.local` files shouldn't be committed. Move the allowlists to `settings.json` or bridle's permission config. |
| Committed `dist/` (OL) | Drop | OB already gitignores `packages/*/dist/`. |
| `koa-app/` | Drop | Leftover. Delete it or leave it outside the project; it isn't in any workspace. |

### 3. Tracking: Beads

- `OL/.beads/`:
  - `config.yaml` sets `issue_prefix: ol`.
  - `issues.jsonl` holds 7 rows. `interactions.jsonl` is empty.
  - `beads.db` is gitignored. It couldn't be opened read-only (the sandbox refused), but its mtime (Feb 18) matches the JSONL's.
- The issues:
  - `ol-1d2` closed: "Refactor generation out of World.ts"
  - `ol-3d6` closed: "Refactor river generation out of River.ts"
  - `ol-2nx` closed: "Add tens-digit to overworld CLI map"
  - `ol-3ff`, `ol-3ff.1`, `.2`, `.3` tombstone: a test epic, "Verify br task tracker is working"
- **0 open.** All the history is in git commits anyway.
- **Recommendation:** don't import. Freeze `.beads/` with OL and don't bring it to OB. Bridle's `br`-avoidance (`docs/questions/resolved/command-name-and-short-alias-sqt6.md`) stays prudent, because the binary may still be installed.

### 4. Migration map: OL → OB

| Area | OL | OB | Status |
|---|---|---|---|
| Monorepo, strict TS, package boundaries | single package, webpack | 5 Yarn workspaces | **OB only** (new) |
| Protocol, command vocabulary, one dispatcher for CLI/REST/WS | no (the CLI calls the sim directly) | yes | **OB only** |
| Server (REST + WS sessions, multi-player) | none | Koa + ws, in-memory sessions | **OB only** |
| Client adapter pattern (single-player and hosted) | the sim sits in the Phaser registry | `IGameAdapter` | **OB only** |
| Tile model | 11 types, property table, resources, items, occupancy | 6 types, `{type, elevation, moisture}` | **Not migrated**. OB's is simpler and different. |
| World gen | 500×500, one river from the SW corner to the north edge, shoreline/mud transitions, rock border (`game-spec/world.md`) | 32×32, random rocks and trees, N random-walk rivers | **Not migrated**. Rewritten generically. OB does have the seeded RNG, which OL may lack (not checked). |
| River side-scroller (`River` depth profile, `RiverScene`, `tryEnterRiver`/`tryExitRiver`, `riverPath` mapping) | yes | `dive` just flips `status` | **Not migrated** |
| Player | position, direction, swimming | id, x, y, surface/diving, hunger/energy/health, tick decay | Diverged. OB adds needs stats. |
| Serialization, save/load without regenerating | `Serialization.ts` (190 lines) plus tests | snapshot out only, no load | **Not migrated** |
| CLI: persisted session dir, fixture world, cheats, map renderer with tens-digit ruler, river view | yes | `new/move/dive/surface/look/status`, ASCII map, no persistence | **Partly**. The command set exists; persistence, fixture, cheats and river view don't. |
| Test suite and fixtures | about 680 lines of vitest | none | **Not migrated** |
| Phaser rendering: TileRenderer, Player, RiverScene, GameOver, debug grid (G) | yes | WorldScene (98 lines) and MainMenu only | **Mostly not migrated** |
| Specs and prose (`game-spec/world.md`, `world-requirements.md`) | yes | README goals only | Not migrated |

**What's left to migrate**, in dependency order:
1. Add vitest to OB, and turn `@otters/simulation` into a real test target.
2. Port OL's tile model and world gen into `@otters/simulation`. Keep OB's seeded RNG and the purity rule. Decide the size and the river shape.
3. Port the river model, entering and exiting the river, and the river movement rules. Extend `GameCommand` to match.
4. Port serialization with load (restore without regenerating) and the fixture world.
5. Port OL's tests onto the new API.
6. CLI: session persistence, `--fixture`, the cheat commands, and the river view in `AsciiRenderer`.
7. Client: `RiverScene`, the tile and player renderers, and the debug overlay, all behind the adapter.
8. Share the dispatcher between server and client: move it into `protocol` or a new `engine` package to remove the `localDispatcher` duplication.
9. A root `check` script (typecheck + test + build), docs moved out of `memory/`, and then OL is retired.

**Is this a good first bridle workload?** Yes, with caveats.
- Why it fits:
  - Each step is small and has a clear definition of done: port X plus its OL tests, then pass `yarn check`.
  - Steps 2–4 depend on each other in order, while 6, 7 and 8 can run in parallel after 4. That gives the manager a real task graph.
  - It's verifiable headless through the CLI, which is exactly the verification path bridle's layer design already cites.
  - The stakes are low: a hobby game with no users and no in-flight work.
- Caveats:
  - Several steps need **design decisions from the human** first (see the questions). OB's generic rewrite diverged on purpose or by accident, and a worker shouldn't pick which one wins.
  - OB's toolchain has never been built in this survey (Yarn 4 install and build weren't run). Task 0 should confirm `yarn install && yarn build && yarn typecheck` passes.
  - The Phaser client steps (7) can't be verified headless beyond `tsc`, so the human reviews them by eye.

**Size estimate.** About 9–12 bridle tasks.
- Roughly 1,500–2,500 lines changed, mostly ports of existing code and tests (OL's simulation, world and CLI total about 2,100 lines including tests).
- Each simulation or CLI task is small to medium for one worker. The client rendering tasks are medium, with human review.
- A few days of wall time with `max_workers` at 1–2.

### 5. Git activity

- **OL**:
  - Active Jan 17 to Feb 19, 2026. There were bursts on Jan 27 (21 commits) and Feb 18 (17 commits).
  - PRs #1–#6 came from Claude web branches (`claude/*`).
  - Dormant for about 7 months.
- **OB**:
  - 3 commits, Mar 6–8, 2026:
    - "First commit"
    - the README goals
    - "Base project structure from claude."
  - Dormant for about 6.5 months.
  - `OB/.git/` has an mtime of today (2026-09-28 12:50), but there are no new refs, commits or `FETCH_HEAD`. Probably read-only git commands from a survey. Not investigated.
- **koa-app**: 1 local commit plus uncommitted edits, no remote.
- **ai-in-life**: last touched 2026-04-27.
- **Live now:** only `pixellab-notes.md` (2026-09-27), which says the human is working on sprite art rather than code.

### 6. Onboarding steps

1. **Human decisions first** (see the questions below).
2. **Bridle side:**
   - Build `workflow/packs/typescript/`: Yarn and npm workspace commands, `tsc --noEmit`, vitest, the `exactOptionalPropertyTypes` idiom.
   - Make the worker's check command a binding. `workflow/base/skills/worker/SKILL.md:28` hardcodes `just check`.
   - Update the `verify.browser` example in `docs/design/workflow-layers.md` to OB's command once it exists.
3. **OB, on a branch:**
   - `.claude/settings.json` with `autoMemoryEnabled: false`.
   - Move `memory/*.md` to `docs/architecture.md` and similar, and strip the "Project Memory" section from CLAUDE.md.
   - Untrack `settings.local.json`.
   - Add a root `check` script.
   - `.bridle/config.toml` (packs `["typescript"]`, `max_workers = 1`).
   - `.bridle/rules/`: simulation purity, logic in the sim, headless CLI verification, and Phaser patterns scoped to `packages/client`.
4. **OL:**
   - Freeze it: no new work.
   - Leave `.beads/` as history.
   - Add a README pointer to OB.
   - Archive the GitHub repo once port step 9 lands.
5. File the port as bridle tasks (§4 steps 1–9), each citing the OL source files.
6. Delete `koa-app/`, or move it out of the workspace (the human decides).
7. Workspace layout: this survey assumes `/Volumes/Data/work/otters/` becomes the bridle workspace, with worktrees at `otters/wt/<agent>`. It isn't a git repo itself, which matches bridle's model. Not verified against the daemon's expectations.

### 7. Open questions for the human

1. **Which is canonical for game rules:** OL's world (500×500, one SW→N river, 11 tile types, resources, side-scrolling river) or OB's generic rewrite (32×32, N rivers, 6 types, needs stats)? The port can't start until this is settled.
2. Is OB's `dive` meant to become OL's side-scrolling river scene, or was a flat "diving" status a deliberate simplification?
3. OK to rename `OB/memory/` into `docs/` and drop the "read at session start" instruction, given that `memory.none` is locked?
4. OK to retire and archive otter-life after the port, and to drop `br`/Beads without importing (0 open issues)?
5. `koa-app/`: delete it? It has uncommitted changes and no remote.
6. Should the Phaser client port be done with bridle workers now, or held until sprite assets exist? And is PixelLab going to be a tool in the pipeline (MCP or scripted generation), or stay manual?
7. Gates: should the human review client or rendering merges by eye, given they can't be verified headless?
8. Package manager: OB uses Yarn 4 and OL uses npm. Confirm Yarn 4 for the TS pack's otters binding.
9. `public.archive/` (the original HTML game with dialogue and virtual controls): is any of it wanted in OB (dialogue system, touch controls), or is it history only?
