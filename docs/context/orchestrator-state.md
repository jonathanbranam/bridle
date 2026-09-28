# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `workflow/base/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 17:00 UTC,
during the sixth orchestrator session.

## First, for the incoming orchestrator

- **`main` is c533cb0, verified** (two local runs, 428/428); the daemon
  runs it since 17:53 UTC.
  (Superseded:) The installed binary was built from 8516bf8. The daemon started
  16:22 UTC on f401a7f (j479, a7h3, 9c63, persist-spawn-overrides all live);
  only the renewal first-message fix (br-ab66, 8516bf8) waits for the next
  restart. Start the watcher from seq ~22386.
- **Agents now reach you directly** (a7h3): `bridle send external:orchestrator`.
  The watcher wakes on your inbox; read with `bridle inbox`, then
  `bridle inbox --mark-read` or it fires again. The human's inbox is for what
  they must act on only (kp3f).
- **The queue is live** (j479): `bridle queue`. pm-1 owns it (tiers of
  equally ranked tasks, on the state branch); manager-2 takes from the top
  tier by load and never re-prioritises. Priorities go to pm-1, who records
  them; not by message alone.
- **In flight: the data-contracts onboarding.** The human answered the
  survey's 8 questions (2026-09-28, sixth session):
  1. `adopt-branch-per-change-workflow`: deleted, local and origin (tip
     4200ad6), worktree and orphaned `.claude/worktrees/` removed. Done.
  2. `.claude/settings.json`: the human removed it on purpose; don't re-add.
  3. OpenSpec: option B. Keep `openspec/specs/` and the spec->feature
     pipeline; retire the CLI, the 10 skills and `changes/`.
  4. **No gates**: bridle's normal merge model applies to data-contracts.
  5. `docs/tickets/` frozen; DC ticket 3fm6 deletes it once bridle is
     adopted. Workflow tickets are not migrated (vf32 sjkw 6tps mv9p d35t
     dropped, 992c landed: DC b923c7e, pushed). Library tickets become tasks.
  6. `plan-of-record.md`: keep §0 and §3, retire §1 and §2 for the queue.
  7. Link checking: DC keeps `check-tickets.py` for now. The human is unsure
     which docs live in bridle vs markdown: ticket hv8e (br-8c6e).
  8. `workflow`: absolute path to bridle's clone for now; a git url yields an
     empty base today (nothing clones it; `rules.rs` discover_layers).
  Bridle side queued by pm-1 (tier 2): br-1e0c roles into workflow/base (in
  progress), br-4221 check-command binding, br-7678 python pack (after
  br-4221), br-bc21 harvest base rules.
  **HOLD all changes in data-contracts** (the human's standing rule,
  2026-09-28: no changes to their existing projects without their review and
  approval; `workflow/base/rules/existing-projects.md`, ticket 63rv). Every
  onboarding is a trial on its own integration branch; `main`/`dev` are never
  touched until the human approves adoption. Needs rxe8 (br-29f9) first.
  Before the rule reached us, the orchestrator had already pushed ticket moves
  to DC `main` (b923c7e) and deleted the adopt branch (both on the human's
  explicit answers); the human was told and said keep it. The trial branch
  is `bridle-adopt` (63rv), made by the orchestrator from DC `main` (incl.
  b923c7e) once br-29f9 lands, and pushed.
- **`merge.ff = only` fix merged** (m2fq, br-544b, b809c4e): workers use
  `git merge --no-ff main`; live for new workers after the next restart.
- **Local permissions** (`.claude/settings.local.json`, untracked): the
  watcher script and reads under data-contracts-workspace are allowed.
- **Maintenance window, 2026-09-28 17:53 UTC:** `cargo clean` (8.2 GiB),
  installed from c533cb0, the human restarted the daemon; pm-1 and
  python-pack resumed. Still waiting: renew pm-1 once the hold lifts (not
  during one: r3nh). manager-2 removed the finished workers itself (the
  manager may `bridle rm`; only the orchestrator's auto mode can't).
  `j2vq-orchestrator-perms` is parked (keep).
- **Tests got ~10x faster after `cargo clean`** (400 s -> 35 s); see f75x.

## The data-contracts onboarding (plan agreed with the human)

Repo: `/Volumes/Data/work/data-contracts-workspace/data-contracts`, sibling
worktrees like bridle's. It has its own CLAUDE.md (183 lines), 10 OpenSpec
skills, 2 agents, a `docs/tickets/` pipeline, an untracked
`.claude/settings.json` and an in-flight branch
`adopt-branch-per-change-workflow` (7 ahead). Leave those two alone until the
human says what they are.

1. Survey (read-only): classify its rules/skills/agents into base, a Python
   pack, the project layer, superseded by bridle, or OpenSpec-dependent; list
   its workflow tickets (some belong in bridle, the human says).
2. Bridle side (bridle's workers): move roles and role prompts into
   `workflow/base` (today they are per repo in `.bridle/roles/`), and build the
   Python pack.
3. data-contracts side (the orchestrator, on a branch; the human reviews):
   `.bridle/config.toml` (project, workflow path, packs = ["python"],
   max_workers 1), a project layer, `bridle sync`, commit.
4. The human starts a second daemon there and makes the orchestrator a token
   for it.
5. A first small real task with one worker.

The human: "we've dropped openspec". OpenSpec's replacement is P3 (bridle
specs and a Python adapter replacing `spec-to-feature.py`); onboarding needs
only an interim answer. Recommendations given: OpenSpec stays for specs until
P3; new work goes in bridle's queue; old `docs/tickets/` stays as history.

## Who's running

- **`pm-1`** (product manager, Sonnet) and **`manager-2`** (development
  manager, Sonnet). Renew them by hand when idle above ~140K (the config
  still renews managers only at 200K).
- **Workers:** `spike-path-rules` (vxp6, Haiku) and `smaller-debug-builds`
  (nbkj).
- **The advisor** (`external:advisor`, `scripts/claude-advisor`,
  `workflow/base/roles/advisor.md`): the human's chat and ticket session.
  Read-only; files tickets; messages you directly.

## Queue (`bridle queue`)

Tier 1: vxp6 spike. Tier 2: nbkj (smaller debug builds), f75x (clean stale
build output), statusline token count, TUI inbox full message. Tier 3: TUI
inbox scroll, send-body quoting bug, m3wq (disk monitoring). Onboarding
tasks join when the survey's done. P3 is held until onboarding.

Parked: `bridle/j2vq-orchestrator-perms` (the human's decision: scope bridle
permissions to the orchestrator via `scripts/claude-orchestrator` or
settings.local.json, never the repo-wide settings), `bridle/mcp-1` (u6wk).

## The human's decisions (2026-09-27 and 28)

- 2026-09-28, this session:
  - After P1: P2, the workflow; one repo, the workflow in `workflow/` inside
    bridle (no separate repos). Shared workflow updated automatically; a
    project sees a changelog and can override (quoted in workflow-layers.md).
  - After P2's core: onboard a second real project (data-contracts), not P3.
  - KISS for per-task tools, models and secrets; "I trust Claude agents".
  - The human's inbox: only questions, blockers, decisions (kp3f).
    Changes go in `CHANGELOG.md` (Unreleased; the orchestrator moves them
    under the version at release).
  - The task queue is a PM-owned tiered record (j479); dependencies only for
    real dependencies; the manager is mechanical.
  - The orchestrator is the "voice of bridle"; a mechanical in-bridle admin
    role is to be designed. Keep `docs/context/role-notes.md` (every
    session and at handover).
  - No short alias (sqt6); the project will be renamed later (geem;
    catalogue in `docs/context/agent-harness-name-catalogue.md`; "bridle" is
    taken by neiii/bridle).
  - Laptop sleep: just let it be interrupted (prvy; `caffeinate -s` doesn't
    help on battery).
- Permissions: "I trust Claude agents so I don't think we need to go
  overboard in restricting their access too much."
- Earlier: P0 task records on a state branch (c7eb); questions inline
  (c5a8); rules KISS, YAGNI, cost-of-not-doing (now in
  `workflow/base/rules/`); the merger pushes `main`; SemVer releases by the
  orchestrator; contexts well under 200K; split managers (tx3f); flaky tests
  fixed properly (f1ky); times to the human in US Eastern.

## Things to know

- **The daemon reads `.bridle/config.toml` and role prompts only at
  startup.** Rebuild with `cargo install --path crates/bridle`; only the
  human restarts the daemon. After a restart, resume stopped workers and
  tell them.
- **Never start the watcher with `&`** in a Bash call; use
  `run_in_background`. It happened twice this session.
- **Verifying:** `just check` twice with the 5-minute load under ~16. If a
  merge lands mid-run, the first run can build a mix of commits; re-run.
- **Test daemons are isolated from `~/.bridle`** (fix-test-home-leak); the
  human's `~/.bridle/config.toml` has a budget schedule.
- **Haiku workers sometimes print their final `bridle send` instead of
  running it**; if a worker is idle with a clean tree, read its log.
- **Auto mode's classifier sometimes errors** for a minute or two; it hits
  the orchestrator and the advisor, not bridle's agents.

## Tickets filed this session

geem (name research), prvy (laptop sleep; observed), k8dw, 2ty9, 9c63,
role notes; the advisor filed xpuc, 8ups, fgu6, j479, kp3f, sed3, nbkj,
f75x, m3wq.

## Done on 2026-09-28, fifth session (merged, verified, pushed)

P2-1 (workflow/base in-repo), P2-2 (layer resolution, `rules
explain/diff`), P2-3 (`bridle sync`), the manager and worker skills, k8dw
(`--allow-tool`), 2ty9 (`--env`), persisting both across renew/resume,
9c63 (read-only access without a token), a7h3 (messages to external
principals), j479 (the queue), kp3f (inbox rule, CHANGELOG.md), the
renewal first-message fix, the test home-dir leak and the flaky renew
test, and the advisor role and script.

## Done on 2026-09-28, fourth session (merged, verified green locally, pushed)

- P0-5 (`bridle rebuild`), P0-6a (the gate, signed off by the orchestrator)
  and P0-6b (68 tasks imported, idempotent; README states the post-cutover
  rule). P0 is complete.
- htp6b (automatic context renewal, plus two renew races and the message
  delivery fifo race), 78sp (built-in tools denied by default; `Agent`
  allowed again, da5fe47), and the Linux CI fixes listed above.
- Rules and tickets: `human-timezone.md` (bare Eastern times to the human),
  x7gp, n9qh, n6gy reopened.

## Done on 2026-09-28, third session (merged, verified green, pushed)

- htp6a (`bridle renew`), kc4v (accurate context via `get_context_usage`),
  P0-3a/b (questions on tasks: `ask`, `answer`, inbox), P0-4a/b (claims and
  leases), the f1ky hang-guard timeouts and the promptness-test fix, s8kn
  (status line).
- Tickets: kc4v, d4mz (step 1 done), mt7r, w4tb, s8kn, r7cs; f1ky updated.
- Rules `yagni.md` and `cost-of-not-doing.md`; the merge-cleanup step; 27
  merged branches deleted.

## Done on 2026-09-27, second session (merged, verified green, pushed)

- `bridle cost audit`, `bridle usage --by role|model|agent`, the governor's
  per-role model step-down (and its precedence fix), held messages delivered
  when the governor recovers, P0-1 (task records, state branch), P0-2 (edges,
  `dep`, `ready`), context measuring, and a timestamp-precision fix.
- Tickets: tskm, f1ky, tx3f, 78sp, a7h3, 4eep, htp6. Resolved c7eb and c5a8.

## Done on 2026-09-27, first session (all merged into `main`, all verified green)

- **Fixes and small gaps:**
  - the v1 gaps: SSE reconnect, `events` defaults, `logs` tool-input
    rendering, `token list/revoke`, agent counts in `daemons`, spawn
    readiness, agent identity in the system prompt;
  - the e2e tests' env leak;
  - the shutdown panic, resuming the manager after a clean shutdown, the
    shutdown notice, and the test-daemon leak;
  - junk windows from `get_usage`;
  - the containment sweep's per-pid `ps` scans, which made stops take minutes;
  - three test races that only surfaced under load.
- **Budget governor**, core and wind-down (usage and budget design).
- **`bridle statusline`** and the ledger for sessions bridle doesn't host
  (P0a).
- **`bridle-tui`**: agents list, event tail, per-agent logs, inbox and
  reply.
- **Spikes:** u7pw (get_usage) and 03 / a7w8 (permission prompts need an MCP
  tool).
- **Tickets** for the run's findings: tk3m, q7fx, n6gy, w8bz, zk4p, j2vq,
  v4nk, r5hc, m9wt.
- **Merge policy** and the $50 spend caps.

## Findings not yet ticketed

- **`bridle status` lists `nimbus_quill 0%`,** an unnamed `get_usage` entry
  that carries a utilization. It's harmless, but decide whether status should
  show only named windows.
