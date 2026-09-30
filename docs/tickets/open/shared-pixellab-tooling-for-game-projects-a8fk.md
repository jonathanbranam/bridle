---
id: a8fk
title: Shared PixelLab tooling for the game projects
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [d9nu, ajqa, u8sm, 8xhh]
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

- Related surveys from the same request: [[onboarding-survey-file-db-d9nu|d9nu]], [[onboarding-survey-meta-notes-ajqa|ajqa]], [[onboarding-survey-track-web-and-harness-u8sm|u8sm]], [[onboarding-survey-otters-8xhh|8xhh]].
- The two surveys ran minutes apart while the human was working in `~/work/track-web`, so its HEAD and dirty state differ between them.
- The open questions at the end are for the human; none is answered yet.

## The survey

### Summary

There is **one** piece of shared PixelLab tooling, and it has **one copy**. It lives in **track-web** (`/Volumes/Data/work/track-web`) and nowhere else. It has four parts:

1. `scripts/pixellab/` is a Python package, `pixellab-tools` (uv, about 3k LOC, pytest suite). It installs two CLIs:
   - `pl`, a REST shim for the PixelLab API.
   - `assets`, a per-game asset pipeline: init / status / mark / ingest / adopt / review / pack / ship.
2. `docs/pixellab/` holds game-independent reference docs: `README.md`, `api.md` and `choosing-tools.md`.
3. A PixelLab **MCP server** entry in `.mcp.json`, which is gitignored.
4. An **asset workspace** outside git, in Dropbox. `GAME_ASSETS_DIR` points at it (`/Volumes/Data/Dropbox/games`). It holds one folder per game and a shared `pixellab-log.jsonl`.

The otter project and pi/harness contain **no tooling**. Otter art is managed *from* track-web's tooling, in the Dropbox workspace. Dungeon Tactics has PixelLab-style prompts but no pipeline wiring yet. No PixelLab skill, agent or MCP server exists in `~/.claude`.

### Where each piece lives

| Piece | Path | In git? |
|---|---|---|
| CLI package | `/Volumes/Data/work/track-web/scripts/pixellab/` (`src/pixellab_tools/{pl,assets_cli,api,config,ingest,adopt,review,pack,ship,manifest,layout,naming,log,args}.py`, `review_template.html`, `tests/`, `uv.lock`) | yes |
| npm entry points | `track-web/package.json`: `"pl"`, `"assets"`, `"test:pixellab"` (all `uv run --project scripts/pixellab …`) | yes |
| Docs | `track-web/docs/pixellab/{README,api,choosing-tools}.md`; also mentioned in `track-web/CLAUDE.md` lines 38–41 and `docs/CLAUDE.md` §`pixellab/` | yes |
| Specs | `track-web/openspec/specs/pixellab-cli/spec.md`, `openspec/specs/game-asset-pipeline/spec.md` (17 requirements total); archived change `openspec/changes/archive/2026-09-27-game-asset-pipeline/` | yes |
| MCP server | `track-web/.mcp.json` defines server `pixellab`, `type: http`, `url: https://api.pixellab.ai/mcp`, with an `Authorization: Bearer` header holding the token | **no** (gitignored) |
| Claude permissions | `track-web/.claude/settings.local.json` allows `mcp__pixellab__create_character`, `mcp__pixellab__get_character`, and `WebFetch` for `api.pixellab.ai`, `docs.pixellab.ai`, `www.pixellab.ai` | no (local) |
| Secrets / config | `track-web/.env` contains `PIXELLAB_SECRET`, `GAME_ASSETS_DIR` (optionally `PIXELLAB_API_BASE`); documented in `.env.example` lines 18–19 | no |
| Asset workspace | `/Volumes/Data/Dropbox/games/{mimlings,otter-game,otter_game}/` (each has `manifest.yaml`, `inbox/ work/ reference/ review/ dist/`), plus `pixellab-log.jsonl` | Dropbox, not git |
| Per-game prompts | `track-web/docs/games/mimlings/mochi-bunny-pixellab-prompts.md`; `track-web/docs/talks/ai-eng-rpg/prompt-log.md` + `assets.md`; `track-web/docs/games/dungeon-tactics/pc-art/prompts.md` (prompt text only, no PixelLab/tooling mention); `/Volumes/Data/work/otters/pixellab-notes.md` (24 lines of otter run-animation prompt attempts, outside any repo) | mixed |

### The two track-web checkouts

- **The active checkout is `/Volumes/Data/work/track-web`**: branch `dev` at `ccbbf99` (2026-09-28), clean.
- `/Volumes/Data/work/pi/track-web` is branch `dev` at `f3f25cc` (2026-09-27). It is a strict ancestor, **4 commits behind**. All 4 commits touch `scripts/pixellab`/`docs/pixellab` (14 files, +1180/−95): review viewer, statuses, the `assets init` rule, and a v3 docs fix.
- So the copies differ, but only because pi's is stale. It is the same repo and not a fork. The differing files are `assets_cli.py`, `ingest.py`, `manifest.py`, `review.py`, `review_template.html` and all three docs.
- The pi checkout has **no `.mcp.json`**, and its `.env` has **no `PIXELLAB_SECRET`/`GAME_ASSETS_DIR`**. The tooling can only actually run from `/Volumes/Data/work/track-web`.

### How it's invoked

- **CLI:** run `npm run pl -- <cmd>` / `npm run assets -- <cmd>` from the track-web root.
  - Needs `uv` and Python ≥3.11. `.venv` is not on PATH.
  - `pl balance`, `pl get` and `pl post --dry-run` are free.
  - `pl post … --wait` spends generations.
  - Each call is appended to `GAME_ASSETS_DIR/pixellab-log.jsonl`.
- **Env:**
  - `PIXELLAB_SECRET` is required for API calls.
  - `GAME_ASSETS_DIR` is required for the asset commands.
  - `PIXELLAB_API_BASE` is optional.
  - The shell environment wins over `.env`. `.env` is read from the *repo root*.
- **MCP:** Claude Code loads it through project `.mcp.json` (HTTP, bearer header). Two tools are pre-allowed: `create_character` and `get_character`.
- **Skills:** none. There is no PixelLab skill in track-web `.claude/skills/` (those are OpenSpec and playwright-cli), and none in `~/.claude/skills` (only a `synced/` dir, no PixelLab match). `~/.claude.json` has no global MCP servers; the only per-project server is `drawio` for `ai-dev/harness`.
- **Hard-coded to track-web:**
  - `config.find_repo_root()` walks up to the first dir that has both `package.json` and `openspec/`. The error text says "track-web repo root".
  - `ship` resolves a relative `config.ship.dest` against that root. It also accepts absolute paths and `${VAR}`. For example, mimlings uses `dest: client-games/public/mimlings`.

### Which projects use it

- **Mimlings** (`track-web/docs/games/mimlings/`, ships into `client-games/public/mimlings`):
  - Active. 387 manifest lines, subjects `berry`, `meadow`, `mochi-bunny`.
  - No shipped PNGs found in `client-games/public` yet (only icons).
- **Otter game** (`/Volumes/Data/work/otters/otter-life`):
  - The Dropbox `otter_game` has otter, esther, fish, human and background art, sourced from `pixellab-web` downloads. It has **no ship dest configured**.
  - A second, empty `otter-game` folder exists: a case/separator duplicate. Commit `ccbbf99` now refuses to create near-duplicates like this, so this is leftover.
  - otter-life itself has no references to PixelLab or the tooling. Its `public/assets` holds only `bg.png` and `logo.png`.
  - track-web's README says it serves "the otter game while it lives elsewhere".
- **Dungeon Tactics** (`track-web/docs/games/dungeon-tactics/`; runtime in `/Volumes/Data/work/pi/harness`: `client-dungeon`, `dungeon-harness-server`):
  - Only character prompts, dated 2026-07-06. They read like PixelLab prompts, but nothing says so.
  - No asset-workspace folder. pi/harness has zero PixelLab or sprite-pipeline references.
- **ADM talk RPG** (`track-web/docs/talks/ai-eng-rpg/`): used PixelLab through the MCP/web earlier (prompt log). This predates the pipeline.
- **Not found:** `/Volumes/Data/work/workflow-tools` does not exist. `/Volumes/Data/work/ai-dev` (only `drawio-test`) and `/Volumes/Data/work/dev-notes` have nothing related.

### Things that matter for bridle workers

1. **The MCP server won't load.** Bridle spawns claude with `--strict-mcp-config` (`docs/design/agent-host/agents.md`), so project `.mcp.json` is ignored. `mcp__pixellab__*` tools are unavailable to workers unless bridle passes an `--mcp-config`, and the token in `.mcp.json` is a header value, not an env var.
2. **Worktrees break `.env` discovery.** A worker in a track-web worktree has no `.env`, because it's gitignored. `pl`/`assets` then fail with "PIXELLAB_SECRET is not set". The fix is to pass `--env PIXELLAB_SECRET=… --env GAME_ASSETS_DIR=…` on spawn, which works today via `bridle spawn --env`. `find_repo_root` will find the worktree root, which is correct for `ship`.
3. **The asset workspace is shared mutable state outside every worktree.** Concurrent workers writing the same `manifest.yaml` or `pixellab-log.jsonl` in Dropbox is a conflict risk. Nothing isolates it.
4. **Tools a worker needs:**
   - `Bash` for `npm run pl/assets`, `uv` and `npm`.
   - Network egress to `api.pixellab.ai`.
   - Optionally `WebFetch(domain:docs.pixellab.ai)`.
   - The PixelLab MCP tools, only if bridle supplies an MCP config.
5. **Spending money.** Real generations cost credits. `pl post --dry-run` and `pl balance` are the safe probes. There is no spend cap in the tool itself (not checked in depth). Bridle's `--max-budget-usd` covers Claude tokens only.

### Recommendation

- **Keep the code as a standalone tool, not in bridle and not a bridle pack.**
  - `pixellab-tools` is a real Python package with tests and specs. It is already game-independent in design; only `find_repo_root` and the `.env` location tie it to track-web.
  - The right first step is to extract it to its own repo (or keep it in track-web but make root/env discovery generic), installable with `uv tool install`, reading config from env or `~/.config`.
  - That is a track-web change; it's a separate question from bridle.
- **Add a small `workflow/packs/pixel-art` (or `game`) pack in bridle holding guidance only.** A `game` pack is already listed as an L2 example in `workflow-layers.md`. The pack would contain:
  - A rule/guide on cost discipline: dry-run first, check balance, log every generation, and never commit raw assets (only packed `dist` sheets go into the game repo).
  - The asset-workspace convention (`GAME_ASSETS_DIR`, manifest statuses, review before pack/ship).
  - A `skills/pixellab/SKILL.md` that says when to run which `pl`/`assets` command. It should point at the tool's own docs rather than copying `api.md`, which would go stale.
  - Then track-web, otters and harness opt in with `packs = ["pixel-art"]`, and per-game art direction stays in each project's L3/docs, as it does today.
- **Runtime needs are a bridle-side gap.** A worker needs, per spawn:
  - `--env PIXELLAB_SECRET`, `--env GAME_ASSETS_DIR` (the secret must come from somewhere other than the task text).
  - `--allow-tool` for `Bash(npm run pl:*)`, `Bash(npm run assets:*)` or equivalents.
  - Network access.
  - For MCP use, a bridle-supplied `--mcp-config` that takes the bearer token from env.
  - Doing this per spawn is error-prone. A per-pack or per-role "requires env/tools" declaration would be the durable fix (not designed anywhere yet). File it as a question/ticket.
- **Don't leave it per project.** Otters and Dungeon Tactics would have to copy it. Today they reach the tool only by running track-web's checkout against the Dropbox folders.

### Open questions for the human

1. Should `pixellab-tools` leave track-web, for example as its own repo or `uv tool`? Or should otters and harness keep calling it from the track-web checkout?
2. Which otter folder in `/Volumes/Data/Dropbox/games` is canonical, `otter_game` (populated) or `otter-game` (empty)? And where should otter sheets ship: `otter-life/public/...` via an absolute or `${VAR}` dest?
3. Is Dungeon Tactics art meant to go through this pipeline? If so, is its asset folder under `GAME_ASSETS_DIR` too, and is the ship dest `pi/harness/client-dungeon/public`?
4. Should bridle workers be allowed to spend PixelLab credits at all, or only dry-run, ingest, review and pack while the human generates? If they may spend, what cap, and where is it enforced?
5. Should workers get the PixelLab MCP server (which would need bridle to supply `--mcp-config` and a token source) or only the REST CLI?
6. Where should bridle get `PIXELLAB_SECRET` for a spawn: the human's `--env`, a daemon-side secret store, or a pack-declared requirement?
7. Is `/Volumes/Data/work/pi/track-web` (4 commits behind) still used for anything, or can it be treated as stale?
8. Pack name and scope: a narrow `pixel-art`/`pixellab`, or a broader `game` pack that also covers simulation/rendering separation and CLI-driven game testing (otters)?

Not checked: PixelLab plan or spend limits, the full contents of the specs, and whether otter-life's webpack build expects any sheet layout.
