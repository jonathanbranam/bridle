---
id: u8sm
title: Onboarding survey: track-web and harness (pi)
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [d9nu, ajqa, 8xhh, a8fk]
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

- Related surveys from the same request: [[onboarding-survey-file-db-d9nu|d9nu]], [[onboarding-survey-meta-notes-ajqa|ajqa]], [[onboarding-survey-otters-8xhh|8xhh]], [[shared-pixellab-tooling-for-game-projects-a8fk|a8fk]].
- The two surveys ran minutes apart while the human was working in `~/work/track-web`, so its HEAD and dirty state differ between them.
- The open questions at the end are for the human; none is answered yet.

## The survey

### Summary

Read-only survey, 2026-09-28. Nothing in pi, harness or track-web was changed, and no deploy script was run. `TW` = `/Volumes/Data/work/track-web`, which is the **active** track-web checkout (§2). `PTW` = `/Volumes/Data/work/pi/track-web`, which is stale. `H` = `/Volumes/Data/work/pi/harness`. `PI` = `/Volumes/Data/work/pi`.

- **Two track-web checkouts, and the wrong one is wired to harness.**
  - TW is where the work happens: daily commits, 3 local commits not yet pushed, a dirty tree, and a stash.
  - PTW's last local commit was 2026-08-23. Since then it has only had pulls, most recently a fast-forward today at 12:49.
  - Harness's `file:` dependency on `@repo/dungeon-engine` resolves to **PTW**, not TW. It works only because nobody has touched the engine since 08-23.
- **The pi wrapper repo is an Obsidian vault shell, not a workspace repo.**
  - It has one commit (`d3cdcd0 Initial commit: .gitignore`), and its `.gitignore` ignores `.obsidian/`.
  - `harness/` and `track-web/` are plain untracked nested clones, not submodules.
  - Its real job is to make the two repos siblings at the same depth, which the relative `file:` path needs. H's research 12 §2.4 rejected a submodule-based workspace repo, with measurements.
- **The cross-repo coupling is narrow and has been quiet for five weeks.**
  - It is one package (`packages/dungeon-engine`), linked by npm symlink into two harness workspaces.
  - Every engine commit (11, from 08-18 to 08-23) fell on a day with harness dungeon commits.
  - Since 08-23: no engine commits, and harness has had only 2 doc commits (08-27 and 09-05).
- **Most of track-web is independent of harness.** 9 of the 10 clients, the backend, `packages/{auth,ui,config}` and the Python `scripts/pixellab` never touch the engine. Inside `client-games`, only `src/games/dungeon-tactics-solo/` does.
- **Recommendation: onboard track-web first**, as the human suggested.
  - Start with `client-games` minus Dungeon Tactics. That's where September's work is (Orbital Dodger, Space Golf), and it's the only client already able to run a disposable second instance.
  - Lock `packages/dungeon-engine/**` until paired worktrees exist.
  - Harness goes last, after P5-style paired worktrees and after the checkout question is settled.
- **Deploy hazard.** In TW, pushing to `main` deploys to production (a GitHub webhook to a t4g.micro); day-to-day work is on `dev`. Bridle's manager role and worker skill hardcode `main` and `git push origin main`. This **must** be fixed before any bridle agent touches track-web.

### 1. What the pi wrapper is

- `PI/.git` has one commit on `main` and no remote. `git status` shows `harness/` and `track-web/` as untracked.
- `PI/.obsidian/` holds the vault config (`app.json`, `workspace.json`, …). So `PI` is a notes vault spanning both repos, plus the sibling-depth anchor.
- It is also where research 09 §2.2 (`H/docs/workflow/research/09-orchestration.md`) puts paired worktrees: `PI/wt/<task>/{harness,track-web}`. Bridle's `docs/design/worktrees-and-ports.md` copies that layout (`root = "/Volumes/Data/work/pi/wt/{task}"`).
- H's research 12 (`H/docs/workflow/research/12-repo-topology-and-scoped-rules.md`) §2.4 settles the workspace-repo question: **don't** use submodules. The measured cost was a redundant pointer conflict on every change, and worktrees re-cloning submodules, which kills live engine editing.

### 2. Which track-web checkout is active

| | TW `/Volumes/Data/work/track-web` | PTW `/Volumes/Data/work/pi/track-web` |
|---|---|---|
| HEAD | `b34a8d7` on `dev`, 2026-09-27 22:11 | `f3f25cc` on `dev`, 2026-09-27 13:20 |
| vs `origin/dev` (`f3f25cc` in both) | **3 ahead, not pushed** | equal |
| Working tree | **dirty**: 5 modified files under `scripts/pixellab` and `docs/pixellab`, plus untracked `openspec/changes/asset-game-init/` | clean |
| Stash | `stash@{0}: WIP on main: 7cf5ab9 …` | none |
| Reflog | commits every day (5 on each of 09-25, 09-26 and 09-27) | last local commit 2026-08-23 (engine work); after that only `pull: Fast-forward` on 09-26 and 09-28 12:49 |
| `.mcp.json` (PixelLab MCP, gitignored, holds a bearer token) | present | absent |
| Harness `file:` link resolves here | no | **yes** (`H/node_modules/@repo/dungeon-engine -> ../../../track-web/packages/dungeon-engine`) |

**TW is the active checkout.**
- PTW was the active one during the August engine and harness work. H's `docs/workflow/driver-workflow.md` "Working across two repos" still names `/Volumes/Data/work/pi/track-web`.
- Both checkouts share a remote. A future engine edit made in TW would **not** be seen by harness until it's pushed and pulled into PTW.
- That breaks property 1 in research 12 §1 ("an uncommitted engine edit is live in the harness immediately").

### 3. track-web today (TW)

**Agent setup.**
- `TW/CLAUDE.md` (256 lines) covers:
  - Commands.
  - Local iPhone testing, which is not a secure context.
  - Verification: playwright-cli, screenshots go to `/tmp/track-verify/`, and never kill or restart the developer's servers or create users in the dev database. Use a second instance instead.
  - A 12-file "Keep in sync" list for adding a client.
  - The deploy rules (below).
  - "Phaser must always be externalized" (the t4g.micro build outage on 2026-07-03).
  - Per-app planning docs.
  - The OpenSpec archive-on-the-human's-word rule and the same-spec pile-up rule.
  - An architecture map. The `dungeon-engine` rules: Node-safe, with no `fetch`, `window` or Phaser, because harness consumes it; and it owns gameplay decisions.
- Nested `CLAUDE.md` files already exist:
  - `TW/client-proto/CLAUDE.md`: prototypes are write-once and never import `@repo/*`. This is a real per-client rule.
  - `TW/docs/CLAUDE.md`
  - 4 under `docs/talks/…`
- `TW/.claude/settings.json`:
  - Allows `git add`, `git commit`, `git log`, `git push` and `git push origin dev`.
  - Has a SessionStart hook, `.claude/hooks/session-start.sh`. It only runs when `CLAUDE_CODE_REMOTE=true`, and then runs `npm install` plus a global install of `@fission-ai/openspec@latest`.
- `settings.local.json` holds a long allowlist: npm, openspec, playwright-cli, vitest.
- `TW/.claude/skills/`:
  - 9 stock OpenSpec skills: `new-change`, `continue-change`, `ff-change`, `propose-change`, `apply-change`, `verify-change`, `sync-specs`, `archive-change`, `openspec-explore`.
  - `playwright-cli`, with 10 reference docs.
- `TW/.agents/skills/` holds 6 OpenSpec skills plus two kb pointers, `phaser-mobile-input.md` and `swipe-card.md` (the articles themselves are in `TW/kb/`).
- There are no `.claude/agents/`.

**Tracking.**
- OpenSpec (`TW/openspec/`) uses the stock `spec-driven` schema.
  - `config.yaml` has a `context:` block and a long `rules:` block: a task to build every affected client, update `openapi.yaml` and `llm-context.md`, update `TABLE_NAMES` when a table is added, the Phaser and swipe-card kb pointers, and "Add **App**: <name> to each spec".
  - **115 specs** (projects.md says 112).
  - 134 archived changes.
  - 4 active changes, all stale drafts: `food` (05-04), `add-from-tmdb-search` (05-11), `watch-ratings-filter-search-prototype` (06-21), `dungeon-tactics-sprite-rendering` (08-20, proposal only).
  - One untracked change in progress: `asset-game-init`.
- The `**App**:` tag gives a ready-made per-client split of the specs:

  | App | Specs |
  |---|---|
  | games | 15 |
  | dungeon-tactics-solo | 15 |
  | talks | 14 |
  | trips | 11 |
  | client-watch | 5 |
  | all | 5 |
  | admin | 5 |
  | me / home / proto | 2 each |
  | untagged | 34 |

- Other tracking:
  - Per-app `docs/*/planning.md`, 99 lines in total.
  - `TW/todo.md`, last touched 2026-06.
  - `docs/openspec/spec-instructions.md`.
  - There is no ticket pipeline like data-contracts or harness have.

**Commands** (`TW/package.json`, `TW/CLAUDE.md`).
- Dev:
  - `npm run dev` runs the backend with `tsx watch`, default port 3000.
  - `npm run dev -w client-<x>` runs one client.
  - `./dev-local.sh` starts tmux, every client, and Caddy.
- Build:
  - `npm run build` builds everything in parallel.
  - `npm run build:<app>` builds one client.
  - `npm run build:server` runs `tsc`.
- Test:
  - `npm test` runs vitest. The include list in `vitest.config.mts` covers `src`, `client-{watch,games,trips,play,talks}`, and `packages/{config,dungeon-engine}`. time, admin, me, home and proto have no tests.
  - `npm run test:dungeon-tactics` uses a separate config for the Gherkin features.
  - `npm run test:pixellab` runs pytest via uv. **track-web contains a Python subproject**: `scripts/pixellab`, with `npm run pl` and `npm run assets`.
- **No lint is configured.** There is no single "check" command, so the definition of done is `npm test` plus `build:<affected apps>`, as the config.yaml `rules.tasks` says.
- The test count was not checked, and tests were not run.

**Ports and dev servers.**
- `TW/packages/config/dev-ports.json`:

  | App | Port |
  |---|---|
  | time | 6010 |
  | watch | 6015 |
  | proto | 6020 |
  | trips | 6025 |
  | play | 6030 |
  | games | 6035 |
  | admin | 6040 |
  | me | 6045 |
  | home | 6050 |
  | talks | 6055 |

  The server is on 3000.
- The developer keeps a server and a client running at all times.
- `TW/docs/dev-second-instance.md` is the recipe for a disposable instance:
  - `SQLITE_PATH=.agent-instance/agent.db` (gitignored).
  - `users:create` to make a login.
  - Server on `PORT=3100`.
  - Client on `VITE_DEV_PORT=6135 VITE_API_TARGET=http://localhost:3100`.
  - Rules: stop what you start; tell agent processes apart by their open log path (`lsof -p`). On 2026-08-21, six stray agent processes held 3000, 4300, 5177 and 6035.
- **Only `client-games/vite.config.ts` reads `VITE_DEV_PORT`/`VITE_API_TARGET`.** The other 9 clients need those two lines before they can be browser-verified in isolation.

**Deploy (never run).**
- `TW/deploy.sh` ssh-runs `server-deploy.sh` on EC2.
- `server-deploy.sh` does `git pull --ff-only` and then `scripts/build-deploy.sh`.
- **A push to `main` triggers the deploy** through the GitHub webhook (`src/routes/deploy`). A push to `dev` doesn't.
- Local `main` is 10 behind `dev`. `main` is advanced by PR merges (`Merge pull request #18 …`).
- Also present:
  - `TW/cs411_ec2.pem` (gitignored).
  - `.env` (gitignored).
  - `exports/`, a git submodule pointing at `track-web-db` that is also gitignored. It is the target of `db:export-push`.
  - `data.db`, the developer's dev database.

**Activity by area** (commits: all time / since 08-01 / since 09-01 / last).

| Area | All | Since 08-01 | Since 09-01 | Last |
|---|---|---|---|---|
| `client-games` | 93 | 37 | 14 | 09-26 |
| `packages/dungeon-engine` | 11 | 11 | 0 | 08-23 |
| `scripts/pixellab` | 4 | 4 | 4 | 09-27 |
| `src` | 77 | 3 | 3 | 09-25 |

- Every other client, and `packages/{auth,ui,config}`: no commits since 08-08. That was a single sweep commit, "home dev links".
- By month: 241 in May, 195 in June, 98 in July, 67 in August, 27 in September.

### 4. harness today (H)

**Agent setup.**
- `H/CLAUDE.md` (about 300 lines):
  - A history note: the dungeon harness was backed out and rebuilt on the real engine.
  - The core invariant: **the engine referees every rule and the harness derives none**, plus the client hit-test vs legality rule.
  - The plan of record lives in `docs/dungeon-harness/plan-of-record.md`.
  - Intake: filing does not schedule.
  - Commands.
  - OpenSpec archive-on-the-human's-word.
  - **Never kill or restart the dev servers.** Developer ports are 4100/5175 (deck), 4200/5176 (introspect) and 4300/5177 (dungeon). Agents use 4400/5277 by convention. Stop what you start, and tell processes apart with `lsof`.
  - playwright-cli verification.
  - Why the servers use `tsx`.
  - Agent sandboxing for the pi `AgentSession`.
  - Deployment, and a "keep in sync" list.
- `H/.claude/skills/`: 10 OpenSpec skills, plus `check-doc-links` (with a script) and `land-the-work`.
- `H/.agents/skills/`: OpenSpec duplicates.
- No `.claude/settings.json`. **No agents dir.**
- `H/docs/workflow-instructions/`:
  - Added in the last commit, `b597498` on 09-05, "Workflow documentation".
  - It is the same driver/apply-agent/ticket guide that data-contracts vendors.
  - `diff -rq` shows `maintenance.md`, `ticket-conventions.md` and `scripts/check-tickets.py` differ.
  - Harness has no `docs/tickets/` directory, so the guide is vendored but not adopted here.
- `H/docs/workflow/driver-workflow.md` (430 lines): the live driver process, including §8 "Verify in the browser — the driver does this" and "Working across two repos" (engine first, host second; one OpenSpec change per repo; the proposals name each other).
- `H/docs/workflow/research/01–14`: the research bridle's design came from. 09 covers orchestration and paired worktrees; 12 covers topology and per-component rules.

**Tracking.**
- OpenSpec: 25 specs and 44 archived changes, all 08-15 to 08-23.
- 2 stale active changes: `add-ui-layout-recording` and `restore-live-state-on-replay-exit`, both 08-20.
- Prose intake and backlog files: `H/docs/dungeon-harness/intake/{bugs,usability,questions,research}.md` and `backlog.md`, with 4, 9, 3, 6 and 4 `##` items.
- The plan of record §2 says "**Next: nothing is scheduled**". The largest open correctness item (intake bugs §1, damage) is **engine-side**, so in track-web.

**Commands.** `npm run dev:<x>-server` and `dev:<x>-client`, `npm run build`, `npm run typecheck`, `npm test` (vitest). No lint.

**Ports.**
- Client ports are hardcoded constants in `client-*/vite.config.ts` (for example `client-dungeon`: `DEV_PORT = 5177`, `BACKEND_PORT = 4300`), with no env override.
- The servers read `PORT`.

**Deploy (never run).**
- `H/server-deploy.sh`, `ecosystem.config.cjs` (PM2) and `Caddyfile` are for running by hand over SSH on the NUC. There's no webhook, and the harness must stay off the public internet.

**Activity.** 210 commits: 209 in August and 1 in September. Branches: `dev` (current, 2 ahead of `main`), `main`, and `dungeon-harness-rebuild`, which is fully merged. The tree is clean.

### 5. The cross-repo coupling, concretely

- **What's shared.** Only `@repo/dungeon-engine`, i.e. `TW/packages/dungeon-engine`, which holds the rules: types, turn, actions, sequencer and so on.
- **Who consumes it:**
  - `H/dungeon-harness-server/package.json`: `"@repo/dungeon-engine": "file:../../track-web/packages/dungeon-engine"`
  - `H/client-dungeon/package.json`: the same `file:` path.
  - `H/client-dungeon/vite.config.ts`: `fs.allow` includes `path.resolve(import.meta.dirname, '../../track-web/packages/dungeon-engine')`.
  - Inside track-web, 24 files in `client-games/src` import it, all under Dungeon Tactics.
- **How it's linked.** npm turns the `file:` dependency into a relative symlink, so it depends on sibling depth. The deck and introspect harnesses don't use the engine.
- **How often both repos change together:**
  - Engine commits: 11, all between 08-18 and 08-23. On every one of those 6 days, harness also had dungeon commits (31 in total between 08-16 and 08-23).
  - Since 08-23: 0 engine commits, and 0 harness code commits.
  - About 1.7% of track-web's history (11 of about 658 commits) touches the engine.
  - So co-change is heavy **when dungeon work is active** and absent otherwise.
- **The rules that exist today:**
  - Engine first, host second, in planning, implementation and archive.
  - One OpenSpec change per repo, and the two proposals name each other.
  - Don't archive the producer before the consumer is verified.
  - The engine stays Node-safe.
  - Sources: `TW/CLAUDE.md` and `H/docs/workflow/driver-workflow.md`.
- **Which track-web parts are independent of harness:**
  - Clients: `client-time`, `-watch`, `-trips`, `-play`, `-me`, `-admin`, `-home`, `-talks`, `-proto`.
  - All of `client-games` except `src/games/dungeon-tactics-solo/`: Ball Merge, Orbital Dodger, Space Golf, the studio shell.
  - `src/` (except the `game_dt_*` tables and routes that Dungeon Tactics uses; how harness uses them was not checked).
  - `packages/{auth,ui,config}` and `scripts/pixellab`.

### 6. Classification

Key: **Base** = bridle L1. **TS** = TypeScript pack (L2). **Web** = a web-ui pack, if one is split out. **Proj** = the project's `.bridle/`, per repo. **Comp** = an L4 component or nested CLAUDE.md. **Super** = superseded by bridle. **OS** = depends on OpenSpec (needs P3).

| Item (source) | Class | Note |
|---|---|---|
| Never kill or restart the developer's servers; never squat a developer port; stop what you start; tell agent processes apart via `lsof` (both CLAUDE.mds, `TW/docs/dev-second-instance.md`) | Base (rule) + bridle feature | This is the P5 port registry: it records the pid per task and excludes reserved ports by config. It's the same rule in both repos, and other projects will need it too. |
| Disposable second instance for browser verification (`dev-second-instance.md`) | Web + Proj | The pattern is generic (own port, own database, own user). The three commands are Proj. The ports should come from bridle, not the 3100/6135 convention. |
| playwright-cli skill; screenshots to `/tmp/track-verify/` | Web | The skill is vendored in TW. The screenshot path should become per worktree or per task. |
| `VITE_DEV_PORT`/`VITE_API_TARGET` in vite configs | Proj (code change) | Only `client-games` has them today. harness's clients have none. |
| Push to `main` deploys; work on `dev`; `git push origin dev` allowed | Proj, **locked** | Needs bridle's integration branch to be configurable in the roles and skills (§8). |
| Phaser externalized; t4g.micro build limits | Comp (client-games, client-talks) | |
| "Keep in sync" 12-file list for adding a client | Proj | A checklist rule, or a check. |
| `client-proto` write-once, no `@repo/*` imports (`client-proto/CLAUDE.md`) | Comp | Already a nested CLAUDE.md. Spike 06 (bridle `docs/spikes/06-path-scoped-rules-findings.md`) confirms these load **when the cwd is at or below them**. Workers start at the worktree root; whether they load when an agent reads or edits files there was not checked. |
| dungeon-engine is Node-safe (no fetch, window or Phaser), owns gameplay decisions, has consumers | Comp (dungeon-engine) + cross-project edge | `consumers = ["harness"]`, as `docs/design/workflow-layers.md` already sketches. |
| harness "engine referees every rule", hit-test vs legality | Proj (harness), `must` | |
| `config.yaml` rules: build affected clients, update openapi.yaml / llm-context.md / TABLE_NAMES, kb pointers | Proj | Move them into `.bridle/rules/` so they don't depend on `openspec instructions`. |
| "Add **App**: <name>" to each spec | OS → Proj interim | The App tag maps specs to components. P3 import could use it. |
| Archive only on the human's word; same-spec pile-up (both CLAUDE.mds) | Super | Replaced by the gates plus the impact registry (`docs/design/gates.md`, `impact-and-conflicts.md`). Keep the human accept gate at first. |
| OpenSpec skills (TW: 9 + 6 in `.agents`; H: 10 + dupes) | OS | Replaced by `bridle-plan`/`-worker`/`-review`. |
| Engine first, host second; one change per repo; the proposals name each other (`H/docs/workflow/driver-workflow.md`) | Base (cross-project) | Bridle's cross-project `blocks` edges (`docs/questions/open/cross-project-specs-yghs.md`). |
| Driver / apply-agent / verifier roles (`H/docs/workflow/*`, `workflow-instructions/`) | Super | Same as data-contracts. |
| Intake / backlog / plan-of-record prose (`H/docs/dungeon-harness/`), per-app `planning.md`, `todo.md` (TW) | Super (tracking) / Proj (orientation) | Import the open items as tasks. Keep plan-of-record §0 as orientation. |
| `check-doc-links` skill (H) | Base candidate | The same question as data-contracts Q7. |
| `session-start.sh` for remote sessions (npm install, openspec) | Proj / Super | Bridle's worktree `setup` covers the install. OpenSpec goes away. |
| `scripts/pixellab` (uv, pytest) | Comp + Python pack | A second language inside a TS repo. |
| harness tsx-not-tsc, ESM-only, pi sandboxing | Proj (harness) | |
| harness research 01–14 | history | Bridle's research lineage. It isn't workflow to import. |

### 7. In-flight branches and worktrees

- **TW:**
  - `dev` is 3 commits ahead of `origin/dev` and not pushed.
  - Uncommitted pixellab work and an untracked OpenSpec change, `asset-game-init`.
  - `stash@{0}` on `main` (old).
  - Remote `claude/*` branches (8, from 2026-03 to 07-05), left over from Claude web sessions.
  - `git worktree list` shows only the main checkout.
- **PTW:** clean, `dev` equal to `origin/dev`. No worktrees.
- **H:** clean on `dev`. `dungeon-harness-rebuild` is fully merged into `dev` and could be deleted. No worktrees.
- **Active OpenSpec changes:** TW has 4 stale ones plus 1 new untracked one. H has 2 stale ones. None look in flight except `asset-game-init`, which is the human's current work.

### 8. What bridle must build or fix first

**Stage 1: track-web alone, no browser, `client-games` minus Dungeon Tactics.**
1. **Integration branch = `dev`, and never push `main`.**
   - The daemon already takes a configurable `base` (`crates/bridle-daemon/src/config.rs`, default `"HEAD"`).
   - But `.bridle/roles/manager.md` hardcodes `main` (`merge-base --is-ancestor main …`, `git push origin main`), and so does `workflow/base/skills/worker/SKILL.md`.
   - Make both use a project binding, and add a locked rule that nothing pushes `main`.
   - **S** (about half a day). **This is a blocker.**
2. **Configurable check command** instead of `just check`: here, `npm test && npm run build:<affected>`. This is already needed for data-contracts. **S.**
3. **Minimal TS pack** (`workflow/packs/typescript/`): npm workspaces, vitest, `tsc` builds, no lint. **S–M.**
4. **Worktree `setup` hook** (`npm install --prefer-offline`) and the worktree layout for this project.
   - Install time and disk use per worktree were **not measured**; that's spike uu5e, and it applies even without pairing.
   - Gitignored files won't be in worktrees: `.env`, `.mcp.json` (the PixelLab token), `data.db`. Whether tests need `.env` was not checked.
   - **S**, plus the spike.
5. **Protected paths**: keep workers off `packages/dungeon-engine/**` and `client-games/src/games/dungeon-tactics-solo/**` until stage 3. That can be a plan-review rule at first. **S.**
6. **Where the manager merges.** Bridle's manager merges and pushes from a checkout. In TW that checkout is the human's own daily working tree on `dev`, and it's dirty. The manager needs to merge somewhere else (its own worktree on `dev`), or the human has to stop committing to TW's `dev` directly. The ticket m2fq (`git merge main` under `merge.ff=only`) also applies. **M. Needs a decision.**

**Stage 2: browser verification.**
7. **Port registry, the lite part of P5.**
   - Allocate a server and client port pair per task.
   - Exclude 3000, 6010–6055 and harness's 4100–4300 and 5175–5177.
   - Record the pid, so "stop what you start" can be audited.
   - Tear down on `bridle rm`.
   - **M.**
8. **Web pack verification guide**: the second-instance recipe parameterised by the allocated ports, with screenshots per task. **S–M.**
9. **track-web code change** (a normal task): add the `VITE_DEV_PORT`/`VITE_API_TARGET` lines to the other clients as each one needs them. **S per client.**

**Stage 3: engine work and harness.**
10. **Settle which track-web checkout is canonical** (Q1 below) before anything else.
11. **Paired worktree layout** (`docs/design/worktrees-and-ports.md`): `linked` mode (track-web as a symlink) for harness-only tasks, `paired` mode for engine tasks. Spike uu5e and research 09 stage C (do one pair by hand) come first. **M–L.**
12. **Cross-project edges**: engine task `blocks` host task, merge in dependency order, and possibly the `before-merge` gate for harness (`docs/design/gates.md`). How a check runs across two project daemons is open (`docs/questions/open/cross-project-specs-yghs.md`, `operating-model` "Several projects"). **M–L.**
13. harness clients need env-aware ports (the same change as item 9). **S.**
14. Per-client rules (L4 components) in general: nested CLAUDE.md works now for cwd-scoped cases, and the P2 component layer does the rest. Not blocking for stage 1. **M, as part of P2.**

The sizes are rough judgements, not measurements. On bridle's build order, stage 1 needs only P0/P2-level pieces. Stage 2 pulls in part of P5. Stage 3 is P4 and P5.

### 9. Recommended onboarding order

1. **track-web, stage 1.**
   - Work on `client-games` non-dungeon games, `src/`, and `scripts/pixellab`.
   - One worker, with the human's plan and accept gates kept.
   - Why here: it's where the human is actually working (14 of the 27 September commits), it has vitest coverage, and it's the only client that's second-instance-ready.
   - A lower-risk alternative for the very first task is a dormant client with tests (`client-watch` or `client-trips`), which avoids colliding with the human's active pixellab and games work.
2. **track-web, stage 2**: browser-verified tasks using bridle-allocated ports. Extend the env-aware vite config to each client as it's touched.
3. **track-web, remaining clients**, with per-client rules as components. Import the per-app `planning.md` items as tasks.
4. **harness, linked mode**: harness-only tasks, with track-web symlinked. This is only worth doing when the human restarts harness work; nothing is scheduled now (plan-of-record §2).
5. **Engine and harness paired tasks.** The first real candidate is the engine-side damage bug (H intake bugs §1 and backlog §1, "enforce where damage is applied").

### 10. Open questions for the human

1. **Canonical track-web checkout.** TW is active, but harness is wired to PTW. Options:
   - (a) Make PTW the working checkout again and retire TW.
   - (b) Move harness next to TW, for example a new `/Volumes/Data/work/pi`-style folder that holds TW.
   - (c) Keep both, and accept that engine edits reach harness only via push and pull.
   Which one? This decides where bridle's track-web workspace (and `wt/`) lives.
2. Should bridle's track-web workers branch from and merge into **`dev`**, with only you ever pushing `main` (which deploys)? We recommend making "never push `main`" a locked rule.
3. You commit directly on TW's `dev`, often with a dirty tree. Would you accept the bridle manager merging into `dev` from its own worktree, while you keep committing in TW? Or should your own work also go through branches?
4. Is `client-games` (non-dungeon) a good first area, or would you rather start with a dormant client (watch or trips) so bridle doesn't compete with your current pixellab and games work?
5. Keep the human plan and accept gates for track-web at first, as today's "archive only on your word" does?
6. OpenSpec in track-web: use the same interim as data-contracts' option B (keep `openspec/specs/` with the `**App**:` tags, retire the CLI, skills and change directories)? And what should happen to the 4 stale active changes: drop them, or import them as tasks?
7. Browser verification: may bridle own a port range for agent instances (for example 3100–3199 and 6100–6199) and exclude your ports 3000, 6010–6055, 4100–4300 and 5175–5177 by config?
8. `.mcp.json` (the PixelLab token) and `.env` are gitignored, so worktrees won't have them. Should workers get them (copied or symlinked by the `setup` hook), or should pixellab tasks stay with you?
9. harness: is it paused (no commits since 09-05, "nothing is scheduled")? Is onboarding it now a goal, or only once dungeon work resumes?
10. The pi vault repo: should bridle ever write there (for example `PI/wt/`, as the paired layout proposes), or keep worktrees elsewhere and leave the vault untouched?
