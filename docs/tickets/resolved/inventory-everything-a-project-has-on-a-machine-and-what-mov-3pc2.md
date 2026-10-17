---
id: br-3pc2
title: "Inventory: everything a project has on a machine, and what moves on a transfer"
kind: research
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kt25, br-53gy, br-rjd5]
parent: br-kx5u
resolved: 2026-10-17T02:08:19Z
---

The human, 2026-10-11 ~8:47 PM ET (to advisor product-manager), verbatim: "I will test with a smaller proejct firs, but this needs to work; also, i'm concerned about everything that is not moving oer". Part of epic br-kx5u (set up a new machine, transfer a project).

## The ask

A read-only inventory, from the code and docs (not memory), of everything a bridle project has on the machine that serves it, and for each item: where it lives, whether it moves on a transfer today, and how it should (carry, recreate, or name as left behind).

Start from the seven places in br-rjd5 and the gaps the PdM found tonight (br-53gy thread): `<workspace>/.bridle/bridle.db` (principals and every token, messages and inboxes, outbox, schedules, agents, sessions, costs), the state branch, the integration branch and unpushed commits, worktrees and uncommitted work, Claude Code session transcripts (resume), `~/.bridle/config.toml` (`[projects]`, `[machines]`, `[machine]`, mail, gateway), `credentials.toml` (role, `<role>.<machine>`, `[peer]`), `.bridle/tokens/human`, `~/.bridle/daemons/<p>.json`, the launchd plist or systemd unit, the mail bridge's project list, the gateway, `owner.toml` and the pre-push hook, logs, benchmarks, and anything else found. Also: what is special about moving bridle itself (the machine's orchestrator runs in it).

Deliverable: a table in this ticket (item, location, moves today?, proposal), plus a short list of what kt25 and 53gy must add. No code changes.

## Findings (2026-10-11, subagent of advisor product-manager, read-only code check)

Note: written before the human decided (same evening, on br-kx5u) that the database is NOT copied: the DB-copy fix-ups below apply only if that changes.

### br-3pc2: everything a project has on a machine, and what moves on a transfer

Read-only inventory, verified against the code at main 1a8d93ec (2026-10-11) and against dalek's
live state (`~/.bridle`, `/Volumes/Data/work/*/.bridle`, `sqlite3 -readonly`). Paths below use
`<ws>` for the project workspace (default: the repo's parent, `crates/bridle-daemon/src/paths.rs:18`),
`<repo>` for the main checkout, `~/.bridle` for `$BRIDLE_HOME` (`crates/bridle-api/src/discovery.rs:84`).
P is the project being moved, OLD is dalek (macOS), NEW is remus (WSL2 Ubuntu).

"Moves today" means: carried by git today (the integration branch, `bridle/state`, or another
pushed branch), or not at all. Nothing else in bridle copies anything between machines today;
the only tool is the 7-step manual runbook in `docs/context/nuc-host.md:90-120`.

Proposal key: **CARRY** (copy or push it, say how), **RECREATE** (make it fresh on NEW, say with
what), **LEFT** (name it as left behind before the move, say the impact). **NEW-CMD** marks an
item for which no bridle command exists yet (kt25/53gy/2wnn/eurm must add it).
**PATH!** marks content with an absolute path, hostname, machine name or port baked in.

### Contents

- A. In git (moves today)
- B. The workspace (`<ws>`): files, worktrees, branches
- C. The database, table by table
- D. Machine-wide `~/.bridle`
- E. Service units
- F. Claude Code's own state
- G. Other machines and other projects that refer to P
- H. Machine prerequisites (recreate on NEW, not per project)
- Section 1: copying `bridle.db` after a clean stop: what breaks
- Section 2: what is special about moving `bridle` itself
- Section 3: uncommitted work, unpushed branches, paused sessions
- Section 4: what 53gy and kt25 as written still miss
- Section 5: items that do not move and have no proposal yet; highest-risk gaps

Item count: 106 rows (A 11, B 27, C 22, D 25, E 4, F 4, G 13), plus the H checklist;
some rows cover a family of files.

---

### A. In git: moves today

| # | Item | Location | Written by | Moves today? | Proposal |
|---|------|----------|------------|--------------|----------|
| A1 | Integration branch (`main`) | origin | autopush every 30 s, owner only (`crates/bridle-daemon/src/autopush.rs:18`); `bridle push` | Yes, but only what was pushed. Shutdown does NOT push it (`crates/bridle-daemon/src/lib.rs:1219-1231` pushes only `bridle/state`) | CARRY: the stop step must push it and confirm `local == origin` (53gy). Take-over already refuses unless local is an ancestor of origin (`state_branch.rs` `sync_integration_for_take_over`) |
| A2 | `bridle/state` branch: `tracking/<id>.md`, `tickets/<id>.md` threads, `edges.toml`, `claims.toml`, `queue.toml`, `events/YYYY-MM.jsonl`, `handovers/<id>.md` | origin, worktree at `<ws>/.bridle/state` (`paths.rs:81`) | batched flush every 30 s + best-effort push, 10 s cap on shutdown (`lib.rs:1229`, `docs/design/storage.md:250+`) | Yes, if the last push succeeded. Push on shutdown is best effort | CARRY: confirmed push (53gy). Remote watchers `remote:<who>@<home>` live in task frontmatter here, so they move |
| A3 | `owner.toml` (`host`, `since`) on `bridle/state` | `<ws>/.bridle/state/owner.toml` | `StateBranch::claim_owner` (`state_branch.rs:428`), host from the `hostname` command (`lib.rs:331`), not `[machine] name` | Yes | CARRY via take-over. **PATH!** host is `dalek.local` (hostname), while `[machine] name` is `dalek`: two names for one machine. On WSL2 the hostname is the Windows PC name unless `/etc/wsl.conf` sets it; set it to `remus` before the take-over or the owner record, the pre-push check and the mail bridge's owner check disagree with `[projects]` |
| A4 | `bridle/benchmarks` branch | origin | `scripts/bench/passive-sample.py:31,313` | Yes | none needed |
| A5 | Committed project config `<repo>/.bridle/config.toml` | repo | human / `bridle daemon init` (`crates/bridle/src/init.rs:93`) | Yes | CARRY, but **PATH!** bridle-ui, track-web and data-contracts commit dalek paths: `workflow = "/Volumes/Data/work/bridle/bridle/workflow"` and absolute `system_prompt` paths (e.g. bridle-ui `.bridle/config.toml:9,46,57`; track-web also its own `/Volumes/Data/work/track-web-workspace/track-web/.bridle/roles/*.md`). A machine `workflow` overrides the project's (`config.rs:1862`), but an absolute `system_prompt` is not overridden and a missing file only logs a WARN and runs the role with the preamble only (`config.rs:3227`). Fix before the move: make them repo-relative or move `workflow` to each machine's `~/.bridle/config.toml`. bridle's own config is clean (relative paths) |
| A6 | `.bridle/rules/`, `.bridle/roles/`, `.bridle/hooks/`, `.bridle/handover/<role>.md`, `.bridle/components/`, vendored `.bridle/workflow/` | repo | human / `bridle init` | Yes | none |
| A7 | `.bridle/cost-baseline.json` | repo | `crates/bridle-daemon/src/cost_audit.rs:90` | Yes (committed in bridle) | none |
| A8 | `.claude/settings.json` (committed hooks) | repo | human / `bridle sync` (`sync.rs:502`) | Yes | none; no absolute paths in any of the four dalek projects |
| A9 | `.bridle/migrations.toml` and `.bridle/migrations.log` | repo | `crates/bridle/src/migrate.rs:854-858` | Design says "committed with the project" (`docs/design/migrations.md:27`) but they are UNTRACKED in bridle-ui, track-web and data-contracts | CARRY: commit them before the move. If lost, migrations re-run; they must be idempotent (`migrations.md:20`), so low risk |
| A10 | `.bridle/review-documents.txt` (documents the review watcher follows) | repo | `crates/bridle/src/review.rs`, read each tick by `doc_watch.rs:21-25` | No: untracked in bridle | CARRY: commit it, or copy. LEFT impact: the human's in-review documents stop waking their agent on NEW |
| A11 | Docs under `docs/tickets/` edited but not committed in the main checkout | repo | humans, advisors | No (bridle-ui has 7 modified ticket files right now) | CARRY: commit and push before stop; the stop command must list a dirty main checkout (Section 3) |

### B. The workspace (`<ws>`): files, worktrees, branches

| # | Item | Location | Written by | Moves today? | Proposal |
|---|------|----------|------------|--------------|----------|
| B1 | The database | `<ws>/.bridle/bridle.db` + `-wal` + `-shm` | `Store::open` (`lib.rs:547`), schema `store.rs:1124-1528` | No | CARRY (PdM recommendation) after a clean stop, as one consistent copy: `sqlite3 bridle.db ".backup x.db"` or checkpoint first; never copy the `.db` alone while `-wal` holds pages. See Section 1 for what must be fixed up |
| B2 | Human token file | `<ws>/.bridle/tokens/human` (0600 in 0700 dir) | `ensure_human_token` (`lib.rs:1314`) | No | CARRY with B1 (it must match `principals.human`). If B1 moves without it, the daemon revokes and re-mints (`lib.rs:1324-1328`): safe, but every copy of the old token elsewhere (gateway reads it for local daemons) stops working |
| B3 | Per-agent token | `<ws>/.bridle/agents/<id>/token` | supervisor spawn | No | CARRY with B1: resume reads it and fails without it (`supervisor.rs:2595-2599`, `2801`) |
| B4 | Per-agent stream-json record | `<ws>/.bridle/agents/<id>/transcript.jsonl` (bridle has 634 agent dirs) | `paths.rs:65` | No | CARRY with B1 (history for `agent logs`), or LEFT (history only; not needed to resume) |
| B5 | Per-agent rendered prompt | `<ws>/.bridle/agents/<id>/system-prompt.md` | rewritten on every resume (`supervisor.rs` resume_inner) | No | RECREATE (automatic) |
| B6 | `bridle/state` worktree | `<ws>/.bridle/state/` | `StateBranch::open` | Via git (A2) | RECREATE from origin on NEW (serve fetches it, `lib.rs:558`). Do not copy (git worktree metadata is path-bound) |
| B7 | `daemon.json` | `<ws>/.bridle/daemon.json` | `lib.rs:684`; removed on clean stop (`lib.rs:1232`) | No | RECREATE (automatic). **PATH!** workspace, repo, url, pid |
| B8 | Daemon log | `<ws>/.bridle/daemon.log` | `serve --detach`, unit `StandardOutput` | No | LEFT (logs) or copy for history |
| B9 | Interactive-session registry | `<ws>/.bridle/sessions.json` | `sessions.rs` (pids, tmux panes, `machine: dalek.local`, claude session ids) | No | Do NOT copy; RECREATE as sessions register on NEW. Entries are dropped when their pid is not alive locally (`sessions.rs:215-231`) |
| B10 | Mail bridge sent-state | `<ws>/.bridle/mail/{digest,questions,senders}` | `bridle-mail/src/outbound.rs:112-194` | No | CARRY with B1 (open mailed questions and known senders), plus D17 `mail.key` |
| B11 | Mail attachments | `<ws>/.bridle/inbox/mail/<ses-id>/` | `bridle-mail/src/bridge.rs:447`, dir set at `crates/bridle/src/commands/orchestrator.rs:293` | No | CARRY (messages in B1 point at these files by path: **PATH!**) or LEFT with impact: old mail messages lose their attachments |
| B12 | Governor probe log | `<ws>/.bridle/governor-probe.jsonl` | `governor.rs:343` | No | LEFT (diagnostics) |
| B13 | Rollback binary | `<ws>/.bridle/bridle.prev` | `rollback.rs:21-35`, `release.rs` | No | Do NOT copy: a macOS binary; a rollback on Linux would exec it |
| B14 | Upgrade marker / notice | `<ws>/.bridle/upgrade-pending.json`, `upgrade-rolled-back.txt` | `rollback.rs:25-30,41-58` | No | Do NOT copy. If copied mid-upgrade, the first start on NEW treats it as a pending upgrade and can roll back to B13 (`rollback.rs:49-55`). Stop must refuse while an upgrade is pending |
| B15 | Upgrade build tree | `<ws>/.bridle/upgrade-src` (worktree), `upgrade-target` (cargo target), `upgrade-release` | `upgrade.rs:206,287`, `release.rs:298` | No | LEFT (rebuilt; platform-specific). Remove on OLD after the move (`git worktree prune`) |
| B16 | Agent worktrees | `<ws>/wt/<agent>/` (or `[worktrees] layout` root/paired elsewhere) | `worktree.rs` | Only what was committed AND pushed (nothing is, see B18) | Committed work: CARRY by pushing the branch. Uncommitted: LEFT unless committed first (Section 3). Do not copy the directories (git worktree links are absolute: **PATH!**) |
| B17 | Integrator worktree | `<ws>/integration/` on `integrate/<task>` | first `bridle land` | No | RECREATE (next land). Stop must refuse during an in-flight land |
| B18 | Worker branches `bridle/<name>` | local refs in `<repo>/.git` | worker agents | NO: never pushed. bridle has 55 local `bridle/*` branches, 36 with commits not on main; origin has only `bridle/state` and `bridle/benchmarks` | NEW-CMD: push the ones that matter (claimed or built tasks, live agents) to origin, or name them LEFT. A `built` ticket's frontmatter names a branch+commit that exists only on OLD, so landing it on NEW fails |
| B19 | Dirty main checkout | `<repo>` | humans, advisors, aide | No | Commit/push or LEFT; take-over already refuses to fast-forward a dirty checkout on NEW, but nothing checks OLD |
| B20 | `.claude/settings.local.json` (orchestrator's local permissions) | `<repo>/.claude/` (gitignored) | human | No. **PATH!** bridle's contains `/Volumes`/`/Users` paths | CARRY by hand with path edits, or RECREATE |
| B21 | Generated skills/agents/hook sidecar | `<repo>/.claude/skills/bridle-*/`, `.claude/agents/*.md`, `.claude/.bridle-sync-hooks.json` (gitignored) | `sync.rs:251,360,502` | No | RECREATE (`bridle sync`, also at serve) |
| B22 | Owner-only pre-push hook | `<repo>/.git/hooks/pre-push` | `serve` calls `install_owner_hook` (`crates/bridle/src/serve.rs:19-30`, `tools_only.rs:297-318`) | No | RECREATE (automatic at serve on NEW). On dalek bridle's pre-push is the user's git template hook, so the owner hook is NOT installed there (`tools_only.rs:305-312` refuses a foreign hook) |
| B23 | Stop-check marker | `<worktree>/.git/bridle-check-passed` | `crates/bridle/src/stop_check.rs:127-136` | No | LEFT (transient) |
| B24 | Last full test count | `<ws>/last-full-test-count` | `integrator.rs:19,334` | No | LEFT (re-written by the next land) |
| B25 | Build artifacts, dependencies | `target/`, bridle-ui `node_modules/`, `dist/` | cargo, npm | No | RECREATE (`cargo build`, `npm ci`) |
| B26 | Git clone-local config | `<repo>/.git/config` (remote url, `branch.*`, any user/email override) | git | No | RECREATE by the clone; check remote URL scheme (ssh vs https) works with NEW's credentials |
| B27 | Other repos in the workspace dir | `/Volumes/Data/work/bridle/{OpenSpec,beads,beads_rust,swarm-forge,workflow,workflow-instructions,workflow-tools}`, `connectivity.log` | not bridle code (no writer in `crates/`) | No | Human decides: LEFT or clone separately. Listed so nothing is a surprise |

### C. The database, table by table (`<ws>/.bridle/bridle.db`)

Schema version: `PRAGMA user_version`, 26 migrations (`store.rs:1528`). bridle's DB is at 26
(binary 0.6.2); bridle-ui, track-web, data-contracts are at 25 (0.6.1; still the `tasks`
table name). If the DB is not carried, everything below except what the state branch rebuilds
(`bridle daemon rebuild`, `storage.md` "Rebuild") is lost.

| # | Table | What it holds | Carry or not; what is machine-bound | Without carrying B1 |
|---|-------|---------------|-------------------------------------|---------------------|
| C1 | `principals` | every token hash: `human`, externals (`orchestrator`, `aide`, `advisor`, `mail`, `statusline`), visitors `<role>@<machine>` with `home`, peers `peer:<machine>`, 615 agent principals in bridle | Carries cleanly, but the names are RELATIVE TO THE SERVING MACHINE: bare `orchestrator`/`aide`/`advisor`/`human` mean "the one on this machine", `x@remus` means "remus's x, visiting". After a move to remus, the bare principals are dalek's people and `x@remus` are now local. See Section 1 | Every token on every machine for P is dead; re-mint all (`token pair`, `token create --peer`) |
| C2 | `agents` | rows with `cwd`, `worktree`, `branch`, `session_id`, `pid`, `pid_start`, `state` | **PATH!** `cwd`, `worktree` absolute; resume spawns claude in `agent.cwd` (`supervisor.rs:2641`, `2837`) and `--resume <session_id>` | Agent history gone; names free |
| C3 | `turns` | per-turn usage and cost | Carry | usage history lost |
| C4 | `events` | the event log (30-day prune) | Carry; data has paths (informational) | history lost |
| C5 | `messages` | every inbox: human (117 pending in bridle), externals (84 pending), agents, task threads | Carry. Bare `human` inbox is the NEW machine's human after the move (Section 1) | **unread inboxes lost** (state branch has only task-thread entries) |
| C6 | `outbox` | queued mail to other daemons (one queue per destination project) | Carry; destination resolved per attempt from `[projects]`/registry (`outbox.rs:183-196`), so it re-routes; needs NEW's `[peer]` tokens | queued mail lost |
| C7 | `forwarded_in` | dedup of mail accepted from peers, keyed `(origin_machine, origin_daemon, origin_id)` | Carry. If NOT carried and P later sends again from the SAME machine name with outbox ids restarting at o-0001, receivers silently drop them as duplicates (a fresh DB on a project that moves back to a machine it served before) | dedup lost (a resend could duplicate) |
| C8 | `schedules` | scheduled messages (once/cron, `tz` stored per row) | Carry; targets are principal names (bare `orchestrator` changes meaning) | **schedules lost** |
| C9 | `tickets` (v26) / `tasks` (v25) | ticket index | Carry, or rebuild from A2 | `rebuild` restores |
| C10 | `edges` | ticket edges | Carry, or rebuild (also reconciled from ticket files) | `rebuild` restores |
| C11 | `open_questions` | open questions | Carry, or rebuild (message ids become `m-rebuilt-*`) | `rebuild` restores |
| C12 | `claims` | who holds what | Carry; claimants are agents that will not be running on NEW: release or reassign | `rebuild` restores claims to agents that do not exist |
| C13 | `conflicts` | impact-check conflicts | Carry | lost (SQLite only, `storage.md`) |
| C14 | `ports` | `bridle port alloc` grants with pids | Do not carry rows: clear (runtime state) | n/a |
| C15 | `handovers` | orchestrator/aide/advisor handover notes | Carry, or rebuild from `handovers/` on A2 | `rebuild` restores |
| C16 | `rate_limits`, `rate_limit_history` | account-wide readings | Carry or not; account-wide, every daemon keeps its own | refilled |
| C17 | `interactive_usage` | statusline snapshots | Carry or not | history lost |
| C18 | `meta.claude_version` | Claude Code version seen | Carry; a change is noticed at start | refilled |
| C19 | `meta.orchestrator_wake_cursor` | event seq of the last delivered wake | Carry (it is an events seq, consistent only with C4) | fresh |
| C20 | `meta.upgrade.built` | the commit the last self-upgrade built | **Machine-bound**: describes OLD's installed binary. On NEW, `running_commit` and "nothing newer" use it (`upgrade.rs:180,331`, `restart.rs:30`), so NEW skips upgrades up to that commit while running some other build | Clear it on the move (or set it to NEW's installed commit) |
| C21 | `meta.restart.resume` | agents to resume after a restart in place | Must be empty after a clean stop; clear it | n/a |
| C22 | `turns`/`messages` mail and visitor ids in bodies | text | informational | n/a |

### D. Machine-wide `~/.bridle` (per user, per machine)

| # | Item | Location | Written by | Moves today? | Proposal |
|---|------|----------|------------|--------------|----------|
| D1 | `[machine] name` | `~/.bridle/config.toml` | hand | n/a (per machine) | RECREATE on NEW (`remus`). Used for `[projects]` lookups (`lib.rs:341-344`), peer names, visitor suffixes |
| D2 | `[machine] tools_only` | same | hand | No | NEW-CMD: after the move add OLD's clone of P (keep it as a tools-only clone, so OLD can never serve it; `crates/bridle/src/tools_only.rs:1-45`). **PATH!** absolute clone paths |
| D3 | `[machines]` (name to host) | same | hand | No | 2wnn sync; NEW needs every machine, every machine needs `remus` |
| D4 | `[projects]` P = {machine, port} | same, on EVERY machine | hand | No | 2wnn: update on every machine. **PATH!** port: the daemon binds `127.0.0.1:<port>` + Tailscale IP only when `[projects]` puts P on this machine (`lib.rs:637-660`), so NEW's entry must say `remus` BEFORE the first serve, and the port must be free on NEW |
| D5 | `[gateway]` bind / public_url / login | same | `bridle gateway` setup | No | Machine-wide, unaffected by moving P, but: the daemon's ticket links use `[gateway] public_url` from the repo or machine config (`config.rs:2675,2702-2708`); NEW has none, so P's messages lose UI links unless NEW's config gets dalek's public_url. **PATH!** bind is dalek's Tailscale IP |
| D6 | `[mail]` (bucket, region, allow, `projects`, `to`, `digest_at`) | same | hand | No | RECREATE on NEW (copy the section). `projects[0]` is the bridge that answers mail to unknown names (`bridle-mail/src/config.rs` `answers_unknown`): today `bridle` |
| D7 | `[[focus]]`, `focus_override_delay_minutes` | same | hand | No | Copy (per machine, local clock) |
| D8 | `[budget]`, `[[budget.schedule]]`, `[review]`, `[daemon] self_upgrade/release_repo/self_upgrade_min_interval`, `workflow`, `workflow_url`, `[auto_mode] environment` | same | hand | No | Copy what applies. Machine `workflow` overrides project `workflow` (`config.rs:1862`): useful to repoint dalek-path configs on NEW |
| D9 | `credentials.toml` plain tables `[orchestrator]`, `[aide]`, `[advisor]`, `[mail]`, `[statusline]` keyed P | `~/.bridle/credentials.toml` (0600) | `token create`, `token pair` (`discovery.rs:393-415`) | No | On OLD these become wrong: a plain key means "P is on this machine". Move them to `[<role>.remus] P` on OLD (same token keeps working if B1 is carried, but then OLD's roles act AS THE BARE PRINCIPALS on NEW, Section 1). Recommended: re-mint (eurm `pair --rotate` with homes) |
| D10 | `[<role>.<machine>] P`, `[human.<machine>] P` on other machines | other machines' credentials | `token pair` | No | Re-key `<machine>` from dalek to remus on every holder; the tokens are valid only if B1 is carried |
| D11 | `[peer] P` (on every machine that sends to P) and NEW's `[peer] <dest>` for every destination P sends to | credentials | `token create --peer` (pasted), `token pair --tokens peer` | No | `[peer] P` on dalek stays valid if B1 is carried (`peer:dalek` exists in P's DB). NEW needs `peer:remus` minted on P (for remus's own projects) and on every destination of P's outbox (`outbox.rs:198-240`) |
| D12 | `~/.bridle/daemons/<P>.json` registry | `~/.bridle/daemons/` | `write_registry` (`discovery.rs`, `lib.rs:688`); removed on clean stop (`lib.rs:1231`) | No | RECREATE (automatic). **PATH!** workspace, repo, loopback url, pid. Task prefix must be unique among NEW's registry (`discovery.rs` `DuplicatePrefix`) |
| D13 | `statusline.token` | `~/.bridle/statusline.token` | human, from `token create statusline` | No | Statusline counts use it against the daemon found by cwd/env (`crates/bridle/src/commands/orchestrator.rs:239-263`); it is bridle's `external:statusline`. If bridle moves, keep it working only if B1 is carried; NEW needs its own |
| D14 | `context/<session_id>` | `~/.bridle/context/` | `bridle statusline` (`statusline.rs:105-130`) | No | LEFT (ephemeral; read by the local daemon only, `sessions.rs:249`, `orchestrator.rs:549`) |
| D15 | Orchestrator files | `~/.bridle/orchestrator.pid`, `orchestrator.session`, `orchestrator.exits` | `crates/bridle/src/session.rs:302,325`; read by the supervising daemon (`orchestrator.rs:231,530,595`) | No | Machine-wide, one orchestrator per machine. **PATH!** `orchestrator.session` holds `~/.claude/projects/-Volumes-Data-work-bridle-bridle/<id>.jsonl`. See Section 2 |
| D16 | Session restart logs | `~/.bridle/restart-<identity>.log` | `session.rs:810` | No | LEFT |
| D17 | Mail reply-token key | `~/.bridle/mail.key` (0600) | `bridle-mail/src/token.rs:23-40` (per machine) | No | CARRY to NEW if any mailed question is still open: reply addresses are HMAC'd with this key, so replies to mail sent from OLD fail verification on NEW with a fresh key |
| D18 | Mail bridge log | `~/.bridle/mail-<P>.log` | `mail_install.rs:87` | No | LEFT |
| D19 | Gateway pid, log, collected interactions | `~/.bridle/gateway.pid`, `gateway.log`, `gateway-interactions.jsonl` | `bridle-gateway/src/lib.rs:131`, `crates/bridle/src/gateway.rs:160,370`, `bridle-gateway/src/collect.rs:324` | n/a | Machine-wide; stays with the gateway machine |
| D20 | Human prompt log | `~/.bridle/prompts.jsonl` (records machine, project) | `crates/bridle/src/focus.rs:205-224`; served by `server.rs:2437-2450` | No | LEFT on OLD: the interaction history of P's past sessions stays on OLD; the gateway polls each daemon, and P's daemon on NEW serves NEW's log. Name it |
| D21 | Focus state | `~/.bridle/focus-nudge`, `focus-nudge.d/<session>`, `focus-override.toml`, `load-note.stamp` | `focus.rs:18,92`, `bridle-daemon/src/focus.rs:12,41`, `load.rs:25` | No | LEFT (per machine) |
| D22 | Installed web UI | `~/.bridle/ui/` (`index.html`, `assets/`, `api-version`) | bridle-ui `npm run install-ui` (`bridle-ui/scripts/install-ui.mjs:9`), or a release upgrade (`release.rs:238`, `server.rs:4638`) | n/a | Served by the gateway (`bridle-gateway/src/config.rs:190`). Not a git clone. See Section 2 (bridle-ui move) |
| D23 | Managed workflow checkout | `~/.bridle/workflow/vX.Y.Z/` | `workflow_checkout.rs:15,50` (release daemons only) | No | RECREATE automatically on a release daemon |
| D24 | Test lock | `~/.bridle/test.lock` (symlink with pid and worktree path) | `scripts/test-lock.sh` | No | LEFT (transient) |
| D25 | Not read by code (legacy or personal) | `~/.bridle/handover/`, `aide-drafts/`, `pdm-roadmap.py`, `config.toml.bak-*` | none in `crates/` | No | Human decides. `pdm-roadmap.py` is a working tool of the PdM advisor that lives outside any repo: if the PdM moves with bridle, copy it or commit it |

### E. Service units

| # | Item | Location | Written by | Moves today? | Proposal |
|---|------|----------|------------|--------------|----------|
| E1 | Daemon unit | macOS `~/Library/LaunchAgents/dev.bridle.<P>.plist` (`launchd.rs:19,48`); Linux `~/.config/systemd/user/bridle-<P>.service` (`systemd.rs:21-22,130-160`) | `bridle daemon launchd install` / `systemd install`; writes only, never loads | No | OLD: uninstall AND unload. Important: the plist has `KeepAlive SuccessfulExit=false` (`launchd.rs:90-94`), so a plist left loaded on dalek restarts `serve` after every exit, and an owner refusal is exit 78 (`serve.rs:374-382`): launchd loops (about every 10 s), each start fetching origin. systemd has `RestartPreventExitStatus=78`, launchd has no equivalent. NEW: `bridle daemon systemd install --project P --projects-dir <P's workspace>` and enable. **Risk:** without `--project`, `systemd install` writes every project with `--workspace <projects-dir>` (`systemd.rs:183-191`), so several projects would share one `<projects-dir>/.bridle` (one `bridle.db`). **PATH!** exe, repo, workspace, PATH, HOME baked in |
| E2 | Mail bridge unit | `dev.bridle.mail.<P>.plist` / `bridle-mail-<P>.service` (`mail_install.rs:27-47`) | `bridle mail install` | No | OLD: uninstall + unload (it would keep polling S3 and refuse every mail as not-owner, `bridle-mail/src/local.rs:43-51`). NEW: needs `[mail]`, a `[mail] P` token (`mail_install.rs:57-68`), AWS credentials, then install + enable + linger |
| E3 | Gateway unit | `dev.bridle.gateway.plist` / `bridle-gateway.service` (`gateway.rs:528-548`) | `bridle gateway install` | n/a | Machine-wide; unaffected unless the gateway itself moves |
| E4 | Linux service prerequisites | `/etc/wsl.conf` `systemd=true`, `loginctl enable-linger`, a Task Scheduler task to start WSL (`docs/context/windows-wsl2-host.md:80-130`) | human | n/a | RECREATE once per machine (`bridle daemon doctor` warns, `docs/design/cli.md:660`) |

### F. Claude Code's own state

| # | Item | Location | Moves today? | Proposal |
|---|------|----------|--------------|----------|
| F1 | Session transcripts (the conversation `--resume` needs) | `~/.claude/projects/<cwd with / and . as ->/<session>.jsonl` (`docs/spikes/07-orchestrator-supervision-findings.md:28`). dalek has 605 dirs for bridle worktrees alone (`-Volumes-Data-work-bridle-wt-*`), 929 MB in total | No. **PATH!** keyed by absolute cwd; a different path on NEW (`/home/...`) is a different key | LEFT by default: resuming an agent on NEW is not supported (C2 cwd, F1 key, and old absolute paths throughout the conversation). Copying the jsonl into NEW's encoded dir is possible in principle but unverified: needs a spike before anyone relies on it. Prefer: hand over (agents: `renew`/a handoff message; sessions: handover notes, which travel in C15/A2) |
| F2 | Interactive sessions' transcripts (orchestrator, aide, advisors) | same, under `-Volumes-Data-work-bridle-bridle` | No | LEFT; the handover note is the carrier (A2 `handovers/`) |
| F3 | Claude login | `~/.claude/.credentials.json`, or the macOS keychain item (`usage_http.rs:90-110`) | No | RECREATE: `claude` login on NEW (usage polling and every agent need it) |
| F4 | Per-path trust and user settings | `~/.claude.json` (per-project trust, MCP), `~/.claude/settings.json` | No | RECREATE (first interactive session in the new path asks for trust) |

### G. Other machines and other projects that refer to P

| # | Reference | Where | Moves? | Proposal |
|---|-----------|-------|--------|----------|
| G1 | `[projects] P` on every machine | each `~/.bridle/config.toml` | No | 2wnn |
| G2 | Role tokens for P held elsewhere (`[<role>.dalek] P`, `[human.dalek] P`) | other machines' credentials | No | re-key to `.remus` (D10) |
| G3 | `[peer] P` held by every sending machine | credentials | No | stays valid with B1 (D11) |
| G4 | P's tokens for its destinations (`[peer] <dest>` on the serving machine) | the serving machine's credentials | No | NEW needs them, minted as `peer:remus` on each destination (D11) |
| G5 | Visitors on OTHER daemons whose `home` is P (`principals.home`) | other projects' DBs | n/a | Homes are project names, so forwards follow P to NEW automatically (`outbox.rs:183`). But a visitor `orchestrator@dalek` with home `bridle` would, after bridle moves, deliver dalek's orchestrator's mail to bridle on remus (Section 2). Today bridle's own visitors all have `home` NULL (eurm) |
| G6 | Outbox entries addressed to P on other daemons | their `outbox` | resolved per attempt | re-route automatically once G1 and G3 are right |
| G7 | `forwarded_in` rows on other daemons with `origin_daemon = P`, `origin_machine = dalek` | their DBs | n/a | With B1 carried, NEW forwards as `origin_machine = remus` and P's outbox ids continue: no collision. Entries queued at the stop that a receiver had already accepted (delivery unconfirmed) are re-sent under the new origin and duplicated once: low risk, name it |
| G8 | Remote watchers `remote:<who>@P` in other projects' task files | their `bridle/state` | n/a | follow G1/G3 automatically |
| G9 | `[mail] projects` list (all machines) and which machine runs P's bridge | config | No | the list is names only; the bridge moves with P (E2). `projects[0]` (bridle) answers unknown names |
| G10 | Gateway's view of P | dalek's gateway reads `[projects]`, `[machines]`, and `[human.<machine>] P` (`bridle-gateway/src/discovery.rs:104-160`, `actions.rs:128`) | No | after G1, dalek's gateway needs `[human.remus] P` (a `human@dalek` visitor on P). Its stale local registry entry for P is overridden by `[projects]` |
| G11 | Other projects' committed `workflow`/`system_prompt` paths into bridle's clone (`/Volumes/Data/work/bridle/bridle/workflow`) | bridle-ui, track-web, data-contracts `.bridle/config.toml` | Yes (git) but path-bound | See A5 and Section 2 |
| G12 | Shell and launcher defaults: `BRIDLE_PROJECT` set by `bridle session` (`session.rs` env), launchers, tmux pane tags `@bridle` (`session.rs:255-265`) | sessions | n/a | a session for P on OLD now talks to NEW's daemon; supervision does not follow (Section 2) |
| G13 | `[machine] tools_only` lists on any machine naming P's clones | config | No | D2 |

### H. Machine prerequisites on NEW (recreate once per machine; any missing one breaks P)

git with push credentials for P's origin (ssh key or `gh auth`), `gh auth` (CI watcher `[ci] github`,
self-upgrade lookups, `upgrade.rs`), `claude` installed and logged in (F3), the toolchain P's
`[integration] check` runs (bridle: rust stable, `just`, `cargo-nextest`; bridle-ui: node/npm),
`tmux` (orchestrator supervision, `orchestrator.rs:34-40`), Tailscale inside WSL with MagicDNS
name `remus`, hostname set to match (A3), the clone on ext4 not `/mnt/c` (doctor warns), the
machine timezone set to US Eastern (the mail digest at 06:30 runs "on the bridge machine's clock",
`bridle-mail/src/config.rs:30`; focus hours and budget schedules use local time), AWS credentials
in the default chain for the mail bridge (`bridle-mail/src/store.rs` `S3Store::connect`), a
`bridle` binary of the SAME OR NEWER version than OLD's (Section 1, schema).

---

### Section 1. Copying `bridle.db` to NEW after a clean stop: what breaks

What a clean stop leaves (`lib.rs:1195-1232`): agents stopped by `stop_all` (state `stopped`,
reason daemon shutdown), state branch flushed and pushed best effort (10 s), registry entry and
`daemon.json` removed. NOT done at stop: the integration branch push, draining the outbox,
releasing claims, clearing `ports`.

What then happens on NEW's first `serve` with the copied DB (order in `lib.rs:520-690`):

1. **Schema version skew.** `Store::open` migrates BEFORE anything else (`lib.rs:547`).
   `migrate` (`store.rs:1583-1590`) runs `MIGRATIONS.iter().skip(user_version)`: a DB NEWER than
   the binary is not refused, it is silently accepted, and queries then fail at random (a v26 DB
   has `tickets`, a v25 binary queries `tasks`). This is live on dalek today (bridle at 26,
   the other three at 25). An OLDER DB is migrated forward, one-way: OLD can then not open it if
   the move is rolled back. Proposal: the move checks `user_version` against NEW's binary and
   refuses if NEW is older; separately, `migrate` should refuse a DB newer than it knows (small
   bug fix, file a ticket).
2. **Human token.** Fine if `tokens/human` is copied with it; otherwise re-minted (B2).
3. **`reconcile`** (`lib.rs:1339-1400`): any agent still recorded running is marked `lost` and its
   written/held messages go back to pending. The pid check needs pid AND `ps lstart` to match, so
   a stale pid cannot hit a live process on NEW. Harmless after a clean stop.
4. **Owner.** `fetch_and_check_owner` with NEW's `hostname` (`lib.rs:553-578`): refuses until
   take-over (by design).
5. **Agents and paths.** `agents.cwd`/`worktree` point at `/Volumes/Data/work/<ws>/wt/<name>`.
   Resume uses them verbatim (`supervisor.rs:2641`), so `bridle agent resume` on NEW spawns claude
   in a directory that does not exist; and even with the path recreated, `--resume <session_id>`
   looks under F1's cwd-keyed dir. Every agent row is effectively not resumable on NEW. Proposal:
   on carry, mark them (state `stopped`, exit reason `moved from dalek`) and list them; `agent rm`
   would try to remove a worktree path that does not exist (check it tolerates that).
   `autostart` roles (`lib.rs` `run_autostart_and_resume`) are found by role and re-spawned
   fresh, as they should be.
6. **Principals change meaning.** The DB's bare `human`, `orchestrator`, `aide`, `advisor`,
   `mail`, `statusline` were OLD's people. On NEW, NEW's people expect the bare names, and OLD's
   people should be `<role>@dalek` visitors with a home on dalek. Copying the DB as-is means: the
   same token now gives dalek's orchestrator the identity of remus's local orchestrator, both
   machines' orchestrators share one inbox and one wake queue, and remus's human reads the 117
   pending messages left for dalek's human. Proposal (kt25): after the copy, rotate the bare role
   principals (revoke, re-mint for NEW's holders), mint `<role>@dalek` visitors with homes for
   OLD (eurm), and decide where the pending human inbox goes (it is the human either way, so
   keep it under bare `human` and tell the human the inbox for P is now read on NEW, or on the
   gateway through `[human.remus]`).
7. **Peers.** `peer:dalek` and `peer:nuc` stay valid: dalek's and nuc's `[peer] P` keep working.
   `peer:remus` does not exist: remus's own other projects cannot send to P until minted.
8. **`forwarded_in`, outbox, schedules, messages, events, handovers, turns**: carry correctly;
   outbox re-routes per attempt (C6). Schedules whose target is a bare role now reach NEW's role.
9. **`meta.upgrade.built`** (C20) describes OLD's binary: clear it. **`meta.restart.resume`**
   should be empty: check it.
10. **`ports`** rows reference dead pids: clear.
11. **`claims`** (15 claimed in bridle right now) name agents that will not run on NEW: release
    them at the stop (the lease would release them eventually, `claim_lease_after`).
12. **Files the DB points at**: agent tokens (B3), mail attachments (B11, absolute paths in
    message bodies), transcripts (B4). Copy `<ws>/.bridle/agents/` and `<ws>/.bridle/inbox/` with
    the DB, or accept the listed losses.
13. **Project name** defaults to the clone's folder name (`lib.rs:538-543`); units pass
    `--project`. If NEW's clone folder is named differently and serve runs without `--project`,
    it becomes a different project with the same DB. The move must pass `--project P`.

Copy method: stop, then on OLD `sqlite3 bridle.db "PRAGMA wal_checkpoint(TRUNCATE)"` (or
`.backup`), copy `bridle.db`, `tokens/`, `agents/`, `mail/`, `inbox/`; do NOT copy `state/`,
`daemon.json`, `sessions.json`, `bridle.prev`, `upgrade-*`.

### Section 2. What is special about moving `bridle` itself

1. **The machine's orchestrator lives in bridle.** bridle's committed config has
   `[orchestrator] enabled = true` (`.bridle/config.toml:87-93`), so bridle's daemon is the one that
   supervises dalek's orchestrator pane: it reads `~/.bridle/orchestrator.pid` and `.session`
   (machine-wide files, `orchestrator.rs:231,530`), checks liveness with local `ps` and types into
   local tmux panes. The orchestrator session runs with `BRIDLE_PROJECT=bridle` (`session.rs:302-307`),
   and its wakes, handover notes and inbox are in bridle's DB. After bridle moves to remus:
   remus's bridle daemon supervises remus's orchestrator; dalek's orchestrator is unsupervised and
   its home project is gone. Dalek needs: another dalek project with `[orchestrator] enabled`
   (project config is committed and shared by every machine, so this is a per-project choice,
   not per-machine: a gap), or a machine-level way to say which local project hosts the
   orchestrator. Nothing today. Proposal: decide this before bridle moves (the per-machine
   orchestrator epic was the plan; kt25's sequencing note says it was overridden).
2. **Supervision of interactive sessions is local-only.** The session registry drops any session
   whose pid is not alive on the daemon's own machine (`sessions.rs:215-231`) and reads context
   files from the daemon's own `~/.bridle/context`. So bridle's aide, advisors and the PdM must run
   on remus once bridle is there, or they become invisible (and the mail bridge's "is the aide
   running" check then routes mail to the orchestrator, `bridle-mail/src/local.rs:53-57`).
3. **Other dalek projects send to bridle.** They go through their own daemons' outboxes with
   `[peer] bridle` (valid if the DB is carried) and resolve remus from `[projects]` per attempt.
   CLI calls with `--project bridle` from dalek (inbox, ticket commands) use
   `[<role>.remus] bridle`, which do not exist yet. Agents on dalek projects that file tickets on
   bridle (visitor submit) need visitor tokens for remus.
4. **The workflow source.** bridle-ui, track-web and data-contracts point their committed
   `workflow` and `system_prompt` at `/Volumes/Data/work/bridle/bridle/workflow` (A5). After bridle
   moves, dalek's clone of bridle stops being updated by landings (they happen on remus). It must
   stay (as a tools-only clone, D2) and be kept current (`git pull` on a schedule), or those
   projects switch to `self_upgrade = "release"` with the managed workflow checkout (D23). Deleting
   dalek's clone silently degrades every dalek project's roles to preamble-only prompts
   (`config.rs:3227`). If bridle-ui moves to remus first, remus needs a bridle clone or a release
   workflow checkout too, and bridle-ui's absolute paths must be fixed first.
5. **Self-upgrade and the installed binary.** Only bridle's daemon has `self_upgrade = true`; it
   builds with `cargo install --path crates/bridle` into the machine's single binary and signs it
   with "bridle local signing" (`upgrade.rs:100-110,295-301`). That is how dalek's binary (used by
   every dalek daemon and the gateway, which re-execs on change) stays current. After the move,
   remus upgrades itself; dalek gets no upgrades unless: the human runs `just install` from the
   tools-only clone, or dalek sets `[daemon] self_upgrade = "release"` in `~/.bridle/config.toml`
   (releases exist: v0.6.2 is current). The release path does not sign (`release.rs` has no
   signing call), so dalek's firewall prompt returns after each release upgrade (p88z). Also C20.
6. **`~/.bridle/ui` on dalek.** It is installed from dalek's bridle-ui checkout by
   `npm run install-ui`, by hand (`bridle-ui/scripts/install-ui.mjs:9`; ticket nvea would automate
   it). When bridle-ui moves, dalek's UI stops updating (unless release upgrades ship `ui/`,
   `release.rs:238`, release mode only). Keep a tools-only bridle-ui clone on dalek and run
   install-ui after landings, or switch dalek to release mode, or move the gateway to remus.
   The gateway warns on a UI/API version mismatch (`api-version`, `bridle-gateway/src/config.rs`).
7. **Release cutting, CI, signing.** `release.yml` and `ci.yml` are GitHub-side and move with git.
   `just sign-setup` and the `.cargo/config.toml` ad-hoc flag are macOS-only: nothing to carry to
   Linux.
8. **Mail.** bridle is `[mail] projects[0]`, so its bridge answers unknown-name mail; only bridle
   has a `[mail]` token on dalek today. Moving bridle moves the mail bridge to remus (E2, D6,
   D17, AWS credentials, timezone).
9. **Statusline.** dalek's `statusline.token` is bridle's `external:statusline` (D13): still valid
   on remus if the DB is carried, but the counts become remote calls from every statusline.
10. **Gateway.** The gateway stays on dalek (`[gateway] bind` is dalek's Tailscale IP); it reaches
    bridle on remus via `[projects]` with `[human.remus] bridle`.

### Section 3. Uncommitted work, unpushed branches, paused sessions

Live state on dalek, bridle only, at writing:
- Dirty worktrees: `wt/self-upgrade` (14 files), `wt/workflow-checkout` (14), `wt/w2wnn` (10),
  `wt/warm-build` (3), `wt/page-title-rule2` (2), `wt/weurm` (2), `wt/page-title-rule` (1), and the
  main checkout (2 untracked under `.bridle/`).
- 55 local `bridle/*` branches, 36 with commits not on `main`, none on origin.
- 15 claimed tickets; agents: 4 working, 2 idle, 1 lost, 19 stopped.
- bridle-ui main checkout: 7 modified ticket files.

Today none of it moves, and nothing lists it. The runbook says only "Uncommitted worktree changes
don't move" (`nuc-host.md:95-96`). Proposal for the stop step (53gy) or move (kt25), before
anything stops:
1. List: every worktree with `git status --porcelain` non-empty (agent and integration worktrees,
   main checkout), every local branch ahead of origin (`bridle/*`, `integrate/*`, the
   integration branch), every claimed ticket and its agent, every running or resumable agent,
   every interactive session registered for P, an in-flight land or upgrade (B14, B17), queued
   outbox entries, and active schedules.
2. Offer per item: commit-and-push (WIP commit on the agent's branch, pushed as
   `bridle/<name>`), or name LEFT with a confirm. Built tickets (`built` with `branch`+`commit`)
   must have their branch pushed or they cannot land on NEW.
3. Agents: do not promise resume across machines (Section 1 point 5, F1). Ask each running agent
   for a handoff (the renew/handoff text) before the stop, so NEW can start a fresh agent from it;
   release its claim.
4. Interactive sessions (orchestrator, aide, advisors, PdM): ask for handover notes
   (`bridle session restart --handover` already does this; the notes travel in `handovers/` on the
   state branch), then end them on OLD and start them on NEW.

### Section 4. What 53gy and kt25 as written still miss

53gy (stop, one-shot take-over, start) should add:
- The stop must confirm BOTH pushes (state branch and integration branch; shutdown pushes only the
  state branch today, best effort, `lib.rs:1229`), and fail rather than warn.
- The pre-stop listing of Section 3 (dirty worktrees, unpushed branches, claims, agents, sessions,
  in-flight land/upgrade, queued outbox, schedules), with confirm or `-y`.
- "Stop the persistent daemon" must uninstall/unload or at least disable the unit on OLD: a loaded
  launchd plist loops on exit 78 (E1). Same for the mail unit (E2).
- Take-over host identity: use one name. `owner.toml` uses `hostname` (`dalek.local`), `[projects]`
  uses `[machine] name` (`dalek`). On WSL2 the hostname is the Windows name by default. Either set
  the hostname in the runbook or record `[machine] name` in `owner.toml`.
- The take-over should check `[projects]` P says this machine (D4) before claiming, else the
  daemon serves loopback only on the default port.
- Linux start: `systemd install` must be run with `--project` (E1 shared-workspace risk).
- Refuse to start a carried DB with a newer schema than the binary (Section 1 point 1).

kt25 (the full move) should add, beyond its own list:
- The DB carry procedure and fix-ups of Section 1: checkpoint/backup, copy `tokens/`, `agents/`,
  `mail/`, `inbox/`; never `state/`, `bridle.prev`, `upgrade-*`, `sessions.json`; mark agent rows
  not resumable; clear `ports`, `meta.upgrade.built`, `meta.restart.resume`; release claims.
- Principal re-labelling (Section 1 point 6): rotate the bare role principals for NEW's holders,
  mint `<role>@OLD` visitors with homes, re-key OLD's credentials from plain tables to `.NEW`,
  `peer:NEW` on P and on every destination of P, `[human.NEW] P` for OLD's gateway. eurm's open
  point (which project is a visitor's home) decides the dalek orchestrator's home after bridle
  moves.
- Committed absolute paths in project configs (A5) checked and refused or rewritten before the
  move (other projects' `workflow`/`system_prompt`).
- Machine files to carry: `mail.key` (D17), `[mail]`, `[gateway] public_url`, `[[focus]]`,
  `[budget]` sections, `pdm-roadmap.py` if the PdM moves, `.claude/settings.local.json` (with path
  edits), uncommitted repo files (`review-documents.txt`, `migrations.toml`/`.log`).
- Machine prerequisites check on NEW (H): binary version, `claude` login, `gh auth`, git push,
  toolchain for the integration check, tmux, Tailscale name, hostname, timezone, AWS for mail,
  WSL systemd and linger.
- What happens to OLD afterwards: add P's clone to `[machine] tools_only` (D2), remove the
  registry entry and units, prune the upgrade worktree; for bridle and bridle-ui specifically,
  Section 2 points 1, 4, 5 and 6 (orchestrator home and supervision, workflow source, binary
  upgrades, `~/.bridle/ui`).
- Interactive sessions must move with P (supervision is local-only, Section 2 point 2).
- Lists that do not move and need naming at confirm time: Claude transcripts (F1/F2), the prompt
  log (D20), daemon/governor/mail logs, `last-full-test-count`, focus state.
- A rollback path: after the move NEW's DB may be migrated to a newer schema; moving back needs
  a binary at least that new on OLD, and `forwarded_in` dedup (C7) argues for always carrying
  the DB rather than starting fresh.

### Section 5. Items with no proposal or command today, and the highest risks

Do not move today AND no bridle command or ticket covers them yet (kt25 names some only in
general terms):
- `bridle.db` with its inboxes, outbox, schedules, conflicts, agents history (B1, C5, C6, C8, C13):
  kt25 names "carry the database" but no procedure or fix-ups.
- Principal meaning after a move (C1, Section 1.6): not in any ticket.
- Unpushed worker branches and built tickets whose branch is only on OLD (B18).
- Uncommitted worktree work and the dirty main checkout (B16, B19, A11): kt25 only asks the question.
- `mail.key` (D17), mail sent-state and attachments (B10, B11).
- Committed absolute paths in project configs (A5) and their silent preamble-only fallback.
- Schema-version skew with no refusal (Section 1.1).
- `meta.upgrade.built` carried to a different binary (C20).
- `owner.toml` hostname vs `[machine] name` (A3) on WSL2.
- The launchd exit-78 restart loop on OLD (E1).
- `systemd install` without `--project` sharing one workspace (E1).
- The orchestrator's home and supervision on OLD after bridle moves (Section 2.1); interactive
  session supervision being local-only (Section 2.2).
- dalek's binary upgrades and `~/.bridle/ui` updates after bridle / bridle-ui move (Section 2.5, 2.6).
- dalek's projects' workflow source after bridle moves (Section 2.4).
- `.claude/settings.local.json`, `review-documents.txt`, uncommitted `migrations.toml`,
  `pdm-roadmap.py` (B20, A10, A9, D25).
- Claude Code session transcripts for resume (F1): left behind; needs an explicit "agents will
  not resume" confirm, or a spike if resume is wanted.

Highest risk (silent failure or data loss):
1. Committed absolute `system_prompt` paths: on remus, workers and managers in bridle-ui,
   track-web, data-contracts run without their role prompts, with only a WARN in the log.
2. DB carried to an older binary: accepted silently, then breaks (no `user_version` guard).
3. Principals carried unchanged: dalek's and remus's orchestrators (and humans) share one identity
   and inbox on the moved project.
4. Unpushed `bridle/*` branches and uncommitted worktrees: lost with no listing; built tickets
   cannot land.
5. Integration branch commits made in the last 30 s before the stop: not pushed by shutdown.
6. Moving bridle leaves dalek without orchestrator supervision, binary upgrades, UI updates and a
   maintained workflow source, with nothing telling the human.
7. A launchd plist left loaded on dalek: restart loop against origin.
