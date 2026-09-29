# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `workflow/base/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 23:55 UTC (7:55 PM),
at the handover from the seventh orchestrator session.

## First, for the incoming orchestrator

- **Two projects are live.** bridle (`~/.bridle-orchestrator.token`) and meta-notes
  (`--project meta-notes`, `BRIDLE_TOKEN=$(cat ~/.bridle-orchestrator-meta-notes.token)`).
  Both daemons were restarted by the human at 23:45 UTC onto the installed build
  **ce4e25d** (CI green). `main` has moved since only with p4ks (abacf6d, resume into a fresh
  session) and docs; install it at the next maintenance window.
- **meta-notes** (the human's priority): the daemon runs from
  `/Volumes/Data/work/meta-notes-workspace/meta-notes`, which must stay on `bridle-adopt`
  (the trial branch; `main` is never touched, `existing-projects` rule). Its manager is
  `manager` (autostart, no PM); `max_workers = 1`. It has no CI: the worker's check
  (`./run_tests.sh && pipenv run pytest test/unit/`) is the gate. In flight: **mn-efc9**
  (OOO event not "mine"; worker `ooo-mine`), then **mn-bf7a** (daily-plan fills meetings
  first). Both briefs are in the task bodies, from the human's write-ups. My watcher doesn't
  cover this daemon: check `bridle agents --project meta-notes` and
  `git -C <clone> log bridle-adopt` on heartbeats, and read its manager's messages to you
  and to `human`. The project layer still carries its own role prompts and rules; w2rp (base
  roles now project-neutral) and the new python/vim packs mean they can be trimmed later.
- **The human's goal, which orders the queue** (2026-09-28): "Bridle should be working
  well enough and useful enough that we can do productive work on my other projects."
  Also, "keep our eyes on the prize". It's at the top of the product manager's role
  prompt. Project order: **meta-notes, then track-web** (most of their games; a big
  onboarding; y3sd's components parts 1-3 are merged), **then data-contracts** (lower).
  Side tickets (y496, ksn2, kpgy) go to the backlog.
- **Verify merges by CI only** (the human, 2026-09-28): no local `just check` on `main`.
  The watcher wakes on a failed GitHub Actions run on main (interim until the daemon's
  own CI watch, c8qw, merged in ce4e25d, has proved itself; then drop that part of the
  watcher). The role says so.
- **Open decision for the human:** kv7d. Should thresholds they set win over Claude Code's
  `allowed_warning`? Recommended yes; it isn't answered yet. Their `burst` override was
  defeated by it on 2026-09-28.
- **Watcher:** `scripts/orchestrator-watch.sh <seq>`; start from seq ~26218. It takes
  `FIVE_HOUR_WAKE=0.97` to raise the five_hour wake while an override is in force.
  Keep it running through budget pauses; don't stop it to silence "all idle" wakes.
  A background loop logs connectivity and battery once a minute to
  `/Volumes/Data/work/bridle/.bridle/connectivity.log` (started at 22:04 UTC; it dies
  with this session, which is useful: its last line marks when a session stopped).
- **Incidents** go in `docs/context/incidents.md` (the human, 2026-09-28). One so far: Remote
  Control and this session were lost from 19:23 to 21:27 UTC with the machine awake. The
  hotspot trial on the drive home held.
- **Renew:** pm-1 is at ~122K. Renew it when idle and above ~140K (not during a hold, r3nh).
  manager-2 was renewed at 23:47 UTC.
- **The standing rule on the human's projects** (`existing-projects`, 63rv): trials on
  `bridle-adopt`; `main`/`dev` never touched until the human approves.
- **Agents reach you directly** (a7h3), including the advisor. The human's inbox is only
  for what they must act on (kp3f).
- **The queue** (j479): pm-1 owns it; manager-2 is mechanical. pm-1 still has to drop
  merged tasks by hand (no done state yet, br-789a).
- **Local permissions** (`.claude/settings.local.json`, untracked): the watcher script,
  and reads under `/Volumes/Data/work/data-contracts-workspace/`. The auto-mode classifier
  sometimes refuses a routine read for a minute; ask the human to say "carry on".
- **Managers may `bridle rm` finished workers themselves**; only the orchestrator's auto
  mode can't.

### The data-contracts survey answers (the human, 2026-09-28)

1. The adopt branch: deleted, local and origin (tip 4200ad6). Done.
2. `.claude/settings.json`: removed on purpose; don't re-add.
3. OpenSpec option B: keep `openspec/specs/` and the pipeline; retire the
   CLI, the 10 skills and `changes/`.
4. No human plan/land gates for bridle's work (the trial branch is the
   review point).
5. `docs/tickets/` frozen; DC ticket 3fm6 deletes it once bridle is adopted.
   Workflow tickets not migrated (dropped: DC b923c7e). Library tickets
   become tasks.
6. `plan-of-record.md`: keep §0 and §3, retire §1 and §2.
7. DC keeps `check-tickets.py` for now; where docs live is open (hv8e).
8. `workflow` = absolute path to bridle's clone (git url isn't resolved yet).

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

- **bridle:** `pm-1` (product manager, Sonnet) and `manager-2` (development manager,
  Sonnet, renewed 23:47 UTC). Worker `tui-open-msg` (br-0e14) at handover. Stopped, to
  remove: `python-pack-2` (merged), `j2vq-orchestrator-perms` (parked).
- **meta-notes:** `manager`; worker `ooo-mine` (mn-efc9).
- **The advisor** (`external:advisor`): the human's chat and ticket session.

## Queue (`bridle queue`, at 23:55 UTC)

Tier 1: br-37b2 (p4ks; merged abacf6d, so drop it). Tier 2: br-1185 (r3nh). Tier 3: br-7bf8
(TUI inbox scroll). Tier 4: br-0e14 (running), br-d5c8 (statusline tokens). Tier 5:
br-3822, br-f6ad (m3wq). pm-1 should refill it by the goal: whatever meta-notes needs next,
then track-web's onboarding (u8sm survey; components done), and kv7d once answered.

Parked: `bridle/j2vq-orchestrator-perms`, `bridle/mcp-1` (u6wk).

## The human's decisions (2026-09-27 and 28)

- 2026-09-28, seventh session:
  - meta-notes onboards now, on the fast path (project-layer rules, no packs), in a fresh
    clone; OpenSpec option B; the worker bumps the version, the manager tags (ajqa).
  - Project order: meta-notes, track-web, data-contracts. The goal quoted above.
  - No local `just check` on main; CI is the verification. Bridle watches CI itself
    (c8qw), with no agent and no webhook. The NUC suits light daemons, not bridle's builds (b7cz).
  - Record connection losses in `docs/context/incidents.md`.
  - The roles a project needs start by themselves (qun8).
- 2026-09-28, sixth session:
  - The data-contracts answers above; the `existing-projects` rule and
    `bridle-adopt` trials (63rv); budget holds are the maintenance window
    (m7wn); workers use `git merge --no-ff main` (m2fq); CLI times in local
    time plus the applied budget and schedule (c424); meta-notes onboards
    after data-contracts; track-web needs y3sd first.
- 2026-09-28, fifth session:
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
- **Verifying:** by GitHub Actions only, no local runs (the human, 2026-09-28; the role's
  "Verify every merge by its CI run"). `gh run watch <id> --exit-status` until c8qw lands.
- **Test daemons are isolated from `~/.bridle`** (fix-test-home-leak); the
  human's `~/.bridle/config.toml` has a budget schedule.
- **Haiku workers sometimes print their final `bridle send` instead of
  running it**; if a worker is idle with a clean tree, read its log.
- **Auto mode's classifier sometimes errors** for a minute or two; it hits
  the orchestrator and the advisor, not bridle's agents.

## Tickets filed this session (seventh)

Orchestrator: kv7d, h8tq, g3ck, w2rp, k7nr, c8qw, b7cz; the incident log. The advisor:
y2eb, zm95, qun8, yurx, y496, ksn2, kpgy, ma8e.

## Done on 2026-09-28, seventh session (merged, CI green, pushed)

g3ck (CI red: tests on `main` explicitly, global git config isolated), y2eb and k7nr
(`max_workers` enforced and live; managers always resume after a pause), br-7678 python
pack, br-bc21 base rules, br-00eb `inbox show/read`, br-a702 and components parts 2-3,
br-e8b3 `just clean-stale`, w2rp (project-neutral base roles), c8qw (daemon watches CI),
b7cz (warm worktree `target/`, `check_worker`), zm95 (shutdown with open streams), qun8
and the autostart-by-role fix (a duplicate manager caught before restart), c424/h8tq
(`bridle budget` display), 6t29 (budget presets), the vim pack, p4ks (resume into a fresh
session; not installed yet). The meta-notes trial (`bridle-adopt`, 053089e + af8b6c0).
Two restarts by the human (21:49 and 23:45 UTC).

## Tickets filed in the sixth session

Orchestrator: m2fq, hv8e, r3nh, m7wn, p4ks; DC 3fm6; cu5m and c424
extended. The advisor: surveys d9nu, ajqa, u8sm, 8xhh, a8fk; rxe8, 63rv,
y3sd, cu5m, c424, 6t29.

## Done on 2026-09-28, sixth session (merged, verified, pushed)

vxp6 spike (path-scoped rules), m2fq (`--no-ff`), br-1e0c (roles into
`workflow/base/roles/`), br-4221 (check command per project), br-29f9
(`[branches]` integration/release setting; `state` reserved), br-5042
(line-tables-only debug), the `existing-projects` rule, the watcher's hold
wake. The orchestrator merged br-29f9 and br-5042 itself during the hold at
the human's request. Maintenance: `cargo clean`, install, restart.

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
