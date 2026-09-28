# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `workflow/base/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 18:00 UTC,
at the handover from the sixth orchestrator session.

## First, for the incoming orchestrator

- **meta-notes is live** (2026-09-28, 22:52 UTC): its daemon runs from
  `/Volumes/Data/work/meta-notes-workspace/meta-notes` on `bridle-adopt` (never `main`);
  its manager is `manager` (autostart). Reach it with `--project meta-notes` and
  `BRIDLE_TOKEN=$(cat ~/.bridle-orchestrator-meta-notes.token)`. It has no CI, so the worker's
  check is the gate. The human gives it tasks directly.
- **The human's goal, which orders the queue** (2026-09-28, via the advisor): "Bridle should be
  working well enough and useful enough that we can do productive work on my other
  projects." They worry software factories end up working on themselves. Also in the
  product manager's role prompt. meta-notes onboards first, then track-web (most of the
  human's games live there; "kind of a beast to onboard"), then data-contracts (lower; "I will
  have some things to do soon"). The human, 2026-09-28.
- **Do these first, before the budget hold lifts (five_hour resets 19:20
  UTC, 3:20 PM):** the sixth session couldn't send them while the human
  cycled the daemon for a new budget setting.
  1. `bridle send pm-1`: drop br-d063 (vxp6 spike, merged ea9ee52), br-544b
     (m2fq, merged b809c4e) and br-1e0c (roles, merged d9a770a). The 17:53
     restart released their claims and they show "planned startable" in
     tiers 1-2; with no done state (br-789a), manager-2 would re-spawn them.
     Also: queue the advisor's new ticket 6t29 (named budget presets via
     `bridle budget override`, max_workers per period) in tier 4 with br-7ab7.
  2. `bridle send manager-2`: don't start those three; for br-7678 spawn a
     fresh worker that continues from `bridle/python-pack` (482825b; handoff
     note on the task). **Never resume `python-pack`**: every resume dies on
     its first turn (p4ks, br-26ae).
  3. `bridle inbox --mark-read` (the advisor's 6t29 note is unread).
  4. After the hold lifts: renew pm-1 (idle at ~146K). Not during a hold:
     a refused renew leaves the agent stopped (r3nh, br-187b). If the daemon
     cycle left pm-1 or manager-2 stopped, `bridle resume` them.
- **`main` is 0ca0ffd plus doc and watcher commits; code verified at c533cb0**
  (two local runs, 428/428). Installed binary: c533cb0. The daemon was cycled
  by the human after 17:55 UTC for a budget setting. Start the watcher from
  seq ~23921.
- **Budget holds are the maintenance window** (the human, 2026-09-28; the
  role's new section; ticket m7wn). The watcher now wakes when a hold starts
  (it remembers the reported state in `~/.bridle-orchestrator-hold-state`),
  and ignores `budget_paused` exits. Keep the list below current.
- **Waiting for the next maintenance window:** nothing yet beyond the pm-1
  renewal.
- **The standing rule on the human's projects** (`existing-projects`, 63rv):
  no change to any of their projects without their review and approval.
  Onboardings are trials on a `bridle-adopt` branch the orchestrator creates
  from the project's `main` (or `dev`) and pushes; `main`/`dev` never touched.
  br-29f9 (the `[branches]` setting) is merged, so data-contracts can start:
  create `bridle-adopt` in DC (from `main`, which includes b923c7e; the
  human said keep it), then survey step 3 on that branch, once the Python
  pack (br-7678) lands.
- **Onboarding order:** data-contracts, then meta-notes (Python pack with
  pipenv, the Vim/vader pack br-55e2), then track-web (needs y3sd's
  component design, br-a702). Other surveys filed by the advisor: d9nu
  file-db, 8xhh otters, a8fk PixelLab tooling; not scheduled.
- **Agents reach you directly** (a7h3), including the advisor (reminded
  twice this session). The human's inbox is only for what they must act on
  (kp3f); only the recipient can mark a message read (cu5m, queued br-00eb).
- **The queue** (j479): pm-1 owns it; manager-2 is mechanical. Merged tasks
  must be dropped by pm-1 (no done state yet, br-789a).
- **Local permissions** (`.claude/settings.local.json`, untracked): the
  watcher script, and reads under `/Volumes/Data/work/data-contracts-workspace/`.
- **Managers may `bridle rm` finished workers themselves**; only the
  orchestrator's auto mode can't. Don't ask the human to.

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

- **`pm-1`** (product manager, Sonnet) and **`manager-2`** (development
  manager, Sonnet). Renew by hand when idle above ~140K.
- **Workers:** none live. `python-pack` is dead (p4ks; don't resume; remove
  once its branch is taken over). `j2vq-orchestrator-perms` is parked.
- **The advisor** (`external:advisor`): the human's chat and ticket session.

## Queue (`bridle queue`, at 17:55 UTC)

Tier 1: (only the stale merged d063, 544b; drop). Tier 2: br-7678 python
pack, br-bc21 harvest base rules, br-55e2 vim pack, br-00eb read/mark one
message (plus stale 1e0c). Tier 3: br-a702 components design (y3sd).
Tier 4: f75x, statusline tokens, TUI inbox full message, br-7ab7 (c424
budget display). Tier 5: TUI inbox scroll, send quoting, m3wq.

Parked: `bridle/j2vq-orchestrator-perms`, `bridle/mcp-1` (u6wk).

## The human's decisions (2026-09-27 and 28)

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

## Tickets filed this session (sixth)

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
