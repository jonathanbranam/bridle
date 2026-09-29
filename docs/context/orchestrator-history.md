# Orchestrator history

Past sessions' state notes, moved out of `orchestrator-state.md` (2026-09-29) so the
orchestrator's startup prompt stays small (ticket ct8m). Not loaded at startup; read on demand.
The human's decisions that still stand are also in `workflow/base/roles/orchestrator.md`.

## Ninth session (2026-09-29 02:50-10:45 UTC); read this first

- **Handover at a full system reboot** (the human, ~11:00 UTC: "can we do a full system
  reboot instead? I think it needs it"). Every agent was idle, nothing queued. After the reboot:
  start the three daemons (or move them to launchd per `docs/context/launchd-restart-plan.md`),
  then run `scripts/claude-orchestrator`. The managers and pm-1 resume by `resume_on_restart`;
  renew manager-2 (~125K) and pm-1 (~120K) right after. `max-workers 3` is lost; the human
  re-sets it if wanted. No workers are running, so none to resume.
- **Open for the human:** try meta-notes mn-fbc0 and track-web's five Space golf tasks;
  review the data-contracts trial (`.bridle/ADOPT-REVIEW.md`, four questions); answer
  rs7p (email); the NUC comes online today (test load there).
- **Next work once running:** the queue is empty. P3-P5 are done; P6 (migrating the human's
  projects onto bridle specs) needs the human's approval per project. Ask the human; meanwhile
  pm-1 can triage the open tickets.

- **Overnight plan** (the human: "what is our next work on the roadmap? ... We still have 40%
  weekly budget to burn"): pm-1 feeds manager-2 P3 (spec parser, check, id, export) and P1's
  `bridle wait` (br-4444), then k3wp (br-7d81, br-0589), h5qd. Take/give parked (build-order
  refreshed, 86bdc71). The human set `max-workers 3` live (lost on restart).
- **Merged and CI-green this session:** statusline flake (792629c), sq4m squash landing
  (3d8f986), task search, b5br claim release, n8tj `send --task`, c9zm (bb6bbf9, plus a fmt fix
  for a red main), qr8z/br-936d `bridle launchd` (069fae2), t6kq credentials (2e39995),
  `bridle-spec` parser, `spec check`, build-order, `spec export`.
- **Overnight (to 10:45 UTC), all merged and CI-green:** P3 done (spec parser/check/id/export/import/coverage,
  goals, arch, explore, spec docs, Python and vitest adapters), P4 done (impact registry/check, conflict
  threads, trace links/suspect, rebase notice, spec-changed notice, reevaluate, arch/goals propose,
  arch-guard), P5 done (ports, merge probe, worktree layout, paired worktrees, `bridle land` + fix; the
  manager prompt uses `land`), `bridle wait`, k3wp, h5qd, wake-manager, PM autostart, doctor, init, doc
  audits, 22 stale tickets resolved. Five reds, all test-only (fmt twice, git identity twice, macOS port
  race); each fixed at the cause. Watch: workers must run full `just check` after their last commit;
  Haiku workers often print their report instead of sending it.
- **Installed: 2780c95** (10:45 UTC), includes launchd/land/doctor/init. Daemons still run cc9b612. **At the next restart** the
  manager's squash/commit permissions and squash landing take effect; until then managers land
  with `--no-ff`. The human plans to move the daemons to launchd in the morning
  (`docs/context/launchd-restart-plan.md`; track-web first). Workers come back `lost`: resume them.
- **track-web:** all five Space golf tasks integrated on `bridle-adopt` (tw-80fc, 54dc, 090c,
  da80, f1c0). Gap: the human's new tasks in a project with no PM reached nobody until I told
  the manager; and it didn't `task done` until told. Idle, waiting on tasks.
- **meta-notes:** mn-fbc0 (check-in skill + sleep-and-return CLI) v1 landed on `bridle-adopt`
  with defaults for the human to review in the morning. Idle.
- **data-contracts step 3 done** (a subagent): `bridle-adopt` 74f2aea + d73ce54, pushed; `main`
  untouched (b923c7e); clone back on `main`, clean. `.bridle/ADOPT-REVIEW.md` has four questions
  for the human (library tickets → tasks; `.claude/settings.json` that sync writes as `{}`;
  acceptance-verifier; Python 3.12). Then the human starts its daemon and adds a
  `data-contracts` entry under `[orchestrator]` in `~/.bridle/credentials.toml`. To ticket: `bridle sync` writes an empty
  `.claude/settings.json`.
- **NUC:** SSD installed, Ubuntu on it, online "tomorrow" (2026-09-29). Test load there with a
  timed `just check` and a real worker task; revisit b7cz (daemons-only) if builds are fast now.
- **Email/texting:** researched, ticket rs7p; waits on the human's answers.
- **The human's Claude Code settings:** sent options (classic renderer, reduced motion) to try.

## First, for the incoming orchestrator (eighth session's handover)

- **Three projects are live**, each with its own daemon; tokens are in
  `~/.bridle/credentials.toml` (`BRIDLE_AS=orchestrator`, set by `scripts/claude-orchestrator`):
  - bridle.
  - meta-notes: `--project meta-notes`. Clone
    `/Volumes/Data/work/meta-notes-workspace/meta-notes`, on `bridle-adopt`.
  - **track-web (new, 2026-09-29):** `--project track-web`. Clone
    `/Volumes/Data/work/track-web-workspace/track-web`, on `bridle-adopt` (cut from
    `origin/dev`, no upstream; 7c9a6cb + 80d9d20, pushed). Scope: **client-games only, minus
    Dungeon Tactics** (the human: "start with one client ... TBD onboarding other components
    until that is vetted"); `.bridle/rules/scope.md`, `specs.md`, `dev-servers.md`; the
    typescript pack; check `npx vitest run client-games && npm run build:games` (272 tests,
    no `.env` needed); `[worktrees] setup = npm install` (~2 min); one worker; the manager can
    push only `bridle-adopt`. Its manager is up and idle: **waiting on the human's first
    task.** Recommended (a) Orbital Dodger proximity-scaled star bonus; (b) Ball Merge
    pop-and-clear; (c) Orbital Dodger leaderboard only under the Default config (all from
    `docs/games/planning.md`). Not answered: survey Q6 (OpenSpec interim assumed like
    meta-notes: edit `openspec/specs/` directly, no CLI) and Q8 (`.env`/`.mcp.json` into
    worktrees; `[worktrees] copy` exists now, unused).
  - None of the three is covered by the watcher except bridle: on heartbeats check
    `bridle agents --project <p>` and the managers' messages.
- **Installed build: cc9b612** (02:50 UTC), signed at link time by cs7x (`codesign -dv` shows
  `adhoc`), starts cleanly, no new syspolicyd crash. The human restarts all three daemons onto
  it at this handover. After it: resume `lost` workers (`squash-land`, `statusline-flake`)
  and tell them the daemon restarted. The new rules (`talk-on-the-task`, tr7k's landing
  record) take effect with it.
- **meta-notes:** out of tasks; its manager is idle. Five tasks merged and integrated with
  summaries and commits (traceability backfilled): mn-efc9, mn-bf7a, mn-d160, mn-b6c5
  (Time Block highlights), mn-dcc1 (time-of-day highlights removed; the human: "don't
  really care about either"). The human owes more tasks.
- **The human's goal orders the queue** (2026-09-28): "Bridle should be working well enough
  and useful enough that we can do productive work on my other projects." Order: meta-notes,
  track-web, then data-contracts.
- **Tokens:** done (t6kq). The human made `~/.bridle/credentials.toml` on 2026-09-29 with
  `[orchestrator]` and `[advisor]` entries for all three projects; the old `~/.bridle-*.token`
  files are no longer read.
- **New principles from the human this session** (all ticketed, most queued):
  - **Traceability** (tr7k, sq4m): from a task id, the brief, implementation summary, branch
    and merge commit; one squash commit per task on the integration branch. The advisor must
    be able to answer "what was done for X" precisely. tr7k merged (2349fc6); sq4m running
    (`squash-land`).
  - **Talk on the task, like JIRA** (rule `talk-on-the-task`, 958e43a; tooling n8tj): task
    discussion goes on the task thread; messages only notify. All managers told.
  - **Cleanup is mechanical** (k3wp): `task done` removes the task's agents, worktree and
    branch; one agent per branch.
  - **Questions the orchestrator answers for the human should close for them** (h5qd).
  - **Watch your own context** (c9zm): the watcher should wake the orchestrator at ~140K.
    Until then, check it yourself on every heartbeat and hand over before ~150K. This session
    didn't, and the human called the handover.
- **main:** green except ad1b485's Linux run, a flake (statusline_test race); worker
  `statusline-flake` is fixing it. manager-2 holds merges until green.
- **Verify merges by CI only** (no local `just check` on `main`).
- **kv7d answered and merged** (538ca98): the human's thresholds win over `allowed_warning`.
- **Watcher:** `scripts/orchestrator-watch.sh <seq>`; start from seq ~28650.
- **Renew:** manager-2 renewed 02:38 UTC. pm-1 at ~88K.
- **Cleanup:** manager-2 was asked to `bridle rm python-pack-2 --delete-branch` and
  `bridle rm j2vq-orchestrator-perms` (keep its parked branch); check it happened. Send
  cleanup to a manager, not the human.
- **Incidents** (`docs/context/incidents.md`): two ~40-minute hangs of every new program
  (syspolicyd crashing on unsigned binaries on this Intel Mac); fixed by cs7x.
- **The standing rule on the human's projects** (`existing-projects`): trials on
  `bridle-adopt`; `main`/`dev` never touched until the human approves.
- **Agents reach you directly**; the human's inbox is only for what they must act on.
- **Local permissions** (`.claude/settings.local.json`, untracked): the watcher script, and
  reads under `/Volumes/Data/work/data-contracts-workspace/`.
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

## Who's running (02:40 UTC)

- **bridle:** `pm-1` (product manager), `manager-2` (development manager, renewed 02:38).
  Workers `squash-land` (br-0b22, sq4m) and `statusline-flake` (the CI flake).
- **meta-notes:** `manager`, idle, no tasks.
- **track-web:** `manager`, idle, waiting on the first task.
- **The advisor** (`external:advisor`): the human's chat and ticket session.

## Queue (`bridle queue`, 02:40 UTC)

Stale merged entries still listed (pm-1 must `task done` them: br-1ac1, br-d744, br-9cf3,
br-b30d; br-29f9's stuck claim is b5br/br-ce1a). Open: br-ce1a (b5br), br-0b22 (sq4m,
running), br-f86f (task search), br-9474 (n8tj), br-7d81 and br-0589 (k3wp), br-b29c (h5qd).
To add: c9zm (the watcher and the orchestrator's context).

Parked: `bridle/j2vq-orchestrator-perms`, `bridle/mcp-1` (u6wk).

## The human's decisions (2026-09-27 to 29)

- 2026-09-29, eighth session:
  - Onboard track-web now, client-games only; other components TBD once it's vetted.
  - kv7d: "my settings override Claude warnings".
  - Traceability: tasks carry brief, summary, branch and merge commit; one squash commit per
    task (tr7k, sq4m). Talk about a task on the task, like JIRA comments (talk-on-the-task,
    n8tj). Cleanup consistent and mechanical (k3wp). The orchestrator's own context is
    watched (c9zm).
  - meta-notes: drop the time-of-day highlights; the Time Block cell highlights stay.
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

## Tickets filed in the eighth session

Orchestrator: n4vk, sq4m, tr7k, h5qd, cs7x, k3wp, n8tj, c9zm; rule `talk-on-the-task`;
two incidents. The advisor: b5br and others.

## Done on 2026-09-29, eighth session (merged, CI green, pushed)

r3nh (renew vs hold), br-42dd (worktree setup), br-d16e (typescript pack), statusline
tokens docs, n4vk (TUI shows new agents), br-0e14 (TUI open a message), m3wq (disk
monitor), br-3822 (`--body-file`/`--text-file`), kv7d, br-789a (task done state), br-0685
(task size), br-3309 (`[worktrees] copy`), br-e7f4 (CHANGELOG union merge), br-d744, br-b30d,
the TUI highlight regression fix, cs7x (ad-hoc signing), the created_at tiebreak (f1ky), tr7k.
meta-notes: five tasks. track-web onboarded. Installs: three, plus the human's restart of all
three daemons at 01:26 UTC.

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
